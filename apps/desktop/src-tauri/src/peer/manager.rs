#![allow(unused)]

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter};
use crate::torrent::metainfo::Torrent;
use crate::torrent::info_hash::InfoHash;
use crate::torrent::peer_id::PeerId;
use crate::tracker::response::Peer;
use super::session::PeerSession;
use super::pieces::PieceDownloader;
use super::disk::DiskWriter;

#[derive(Clone, serde::Serialize)]
pub struct ProgressPayload {
    pub downloaded: u32,
    pub total: u32,
    pub piece_index: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum PieceState {
    Downloading,
    Pending,
    Done,
}

pub struct DownloadManager {
    torrent: Torrent,
    info_hash: InfoHash,
    peer_id: PeerId,
    output_dir: PathBuf,
    piece_states: Arc<Mutex<Vec<PieceState>>>,
    piece_rarities: Arc<Mutex<HashMap<u32, u32>>>,
}

impl DownloadManager {
    pub fn new(
        torrent: Torrent,
        info_hash: InfoHash,
        peer_id: PeerId,
        output_dir: PathBuf,
    ) -> Self {
        let piece_count = torrent.piece_count();
        let piece_states = vec![PieceState::Pending; piece_count];
        
        Self {
            torrent,
            info_hash,
            peer_id,
            output_dir,
            piece_states: Arc::new(Mutex::new(piece_states)),
            piece_rarities: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(
        &self,
        peers: Vec<Peer>,
        app_handle: AppHandle,
    ) -> Result <()> {
        let mut handles = Vec::new();

        for peer in peers {
            let info_hash = self.info_hash;
            let peer_id = self.peer_id;
            let torrent = self.torrent.info.clone();
            let output_dir = self.output_dir.clone();
            let piece_states = Arc::clone(&self.piece_states);
            let piece_rarities = Arc::clone(&self.piece_rarities);
            let app_handle = app_handle.clone();
            let total = self.torrent.piece_count() as u32;

            let handle = tokio::spawn(async move {
                let stream = match tokio::net::TcpStream::connect(
                    format!("{}:{}", peer.ip, peer.port)
                ).await {
                    Ok(stream) => stream,
                    Err(_) => return,
                };

                let mut session = match PeerSession::new(
                    stream,
                    info_hash,
                    peer_id,
                ).await {
                    Ok(session) => session,
                    Err(_) => return,
                };

                // update rarities from this peer's bitfield
                {
                    let mut rarities = piece_rarities.lock().await;
                    for i in 0..total {
                        let i = i as u32;
                        if session.has_piece(i) {
                            *rarities.entry(i).or_insert(0) += 1;
                        }
                    }
                }

                if let Err(_) = session.send_interested().await {
                    return;
                }

                if let Err(_) = session.wait_for_unchoke().await {
                    return;
                }

                let disk = DiskWriter::new(output_dir, torrent.clone());

                loop {
                    let piece_index = {
                        let mut states = piece_states.lock().await;
                        let rarities = piece_rarities.lock().await;

                        let next = Self::select_piece(
                            &states,
                            &rarities,
                            &session,
                            total,
                        );

                        match next {
                            Some(index) => {
                                states[index as usize] = PieceState::Downloading;
                                index
                            }
                            None => break,
                        }
                    };

                    let piece_length = torrent.piece_length as u32;
                    let total_size = torrent.length.unwrap_or(0) as u32;

                    let length = if piece_index == total - 1 {
                        total_size - (piece_index * piece_length)
                    } else {
                        piece_length
                    };

                    let hash_start = piece_index as usize * 20;
                    let mut hash = [0u8; 20];
                    hash.copy_from_slice(&torrent.pieces[hash_start..hash_start + 20]);

                    let downloader = PieceDownloader::new(piece_index, length, hash);

                    match downloader.download(&mut session).await {
                        Ok(data) => {
                            if let Ok(_) = disk.write_piece(piece_index, &data).await {
                                let mut states = piece_states.lock().await;
                                states[piece_index as usize] = PieceState::Done;

                                let downloaded = states
                                    .iter()
                                    .filter(|s| **s == PieceState::Done)
                                    .count() as u32;

                                let _ = app_handle.emit("download-progress", ProgressPayload {
                                    downloaded,
                                    total,
                                    piece_index,
                                });
                            }
                        }
                        Err(_) => {
                            let mut states = piece_states.lock().await;
                            states[piece_index as usize] = PieceState::Pending;
                        }
                    }
                }
                
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        let _ = app_handle.emit("download-complete", ());
        
        Ok(())
    }

    fn select_piece(
        states: &[PieceState],
        rarities: &HashMap<u32, u32>,
        session: &PeerSession,
        total: u32,
    ) -> Option<u32> {
        let mut candidates: Vec<(u32, u32)> = (0..total)
            .filter(|&i| {
                states[i as usize] == PieceState::Pending && session.has_piece(i)
            })
            .map(|i| {
                let rarity = rarities.get(&i).copied().unwrap_or(0);
                (i, rarity)
            })
            .collect();

        candidates.sort_by_key(|&(_, rarity)| rarity);

        candidates.first().map(|&(index, _)| index)
    }
}