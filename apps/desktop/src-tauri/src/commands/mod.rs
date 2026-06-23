#![allow(unused)]

use crate::peer::connection::connect_to_peer;
use crate::torrent::info_hash::{compute_info_hash, InfoHash};
use crate::torrent::metainfo::Torrent;
use crate::torrent::peer_id::generate_peer_id;
use crate::tracker::client::announce;
use crate::tracker::response::Peer;
use crate::tracker::AnnounceRequest;
use crate::peer::manager::DownloadManager;
use std::path::PathBuf;
use serde::Serialize;
use tauri::AppHandle;

#[derive(Debug, Serialize)]
pub struct TorrentInfo {
    pub name: String,
    pub total_size: u64,
    pub piece_count: usize,
    pub info_hash: String,
    pub is_multi_file: bool,
    pub announce: Option<String>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
}

#[tauri::command]
pub fn parse_torrent_file(path: String) -> Result<TorrentInfo, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;

    let torrent = serde_bencode::from_bytes::<Torrent>(&bytes).map_err(|e| e.to_string())?;

    let hash = compute_info_hash(&bytes).map_err(|e| e.to_string())?;

    let info_hash = hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok(TorrentInfo {
        name: torrent.info.name.clone(),
        total_size: torrent.total_size(),
        piece_count: torrent.piece_count(),
        info_hash,
        is_multi_file: torrent.is_multi_file(),
        announce: torrent.announce,
        comment: torrent.comment,
        created_by: torrent.created_by,
    })
}

#[derive(Debug, Serialize)]
pub struct PeerInfo {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Serialize)]
pub struct AnnounceResult {
    pub interval: u64,
    pub peers: Vec<PeerInfo>,
}

#[tauri::command]
pub async fn announce_to_tracker(
    tracker_url: String,
    info_hash: String,
    total_size: u64,
) -> Result<AnnounceResult, String> {
    let mut hash = [0u8; 20];
    hex::decode_to_slice(&info_hash, &mut hash).map_err(|e| e.to_string())?;

    let peer_id = generate_peer_id();

    let request = AnnounceRequest::new(hash, peer_id, total_size);

    let response = announce(&tracker_url, &request)
        .await
        .map_err(|e| e.to_string())?;

    let peers = response
        .peers
        .into_iter()
        .map(|p| PeerInfo {
            ip: p.ip.to_string(),
            port: p.port,
        })
        .collect();

    Ok(AnnounceResult {
        interval: response.interval,
        peers,
    })
}

#[derive(Debug, Serialize)]
pub struct ConnectionResult {
    pub peer_id: String,
    pub ip: String,
    pub port: u16,
}

#[tauri::command]
pub async fn connect_to_peer_command(
    ip: String,
    port: u16,
    info_hash: String,
) -> Result<ConnectionResult, String> {
    let mut hash = [0u8; 20];
    hex::decode_to_slice(&info_hash, &mut hash).map_err(|e| e.to_string())?;

    let peer_id = generate_peer_id();

    let connection = connect_to_peer(&ip, port, hash, peer_id)
        .await
        .map_err(|e| e.to_string())?;

    let peer_id_hex = connection
        .handshake
        .peer_id
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok(ConnectionResult {
        peer_id: peer_id_hex,
        ip,
        port,
    })
}


#[tauri::command]
pub async fn start_download(
    app_handle: AppHandle,
    torrent_path: String,
    output_dir: String,
) -> Result<(), String> {
    let bytes = std::fs::read(&torrent_path)
        .map_err(|e| e.to_string())?;

    let torrent: Torrent = serde_bencode::from_bytes(&bytes)
        .map_err(|e| e.to_string())?;

    let info_hash = compute_info_hash(&bytes)
        .map_err(|e| e.to_string())?;

    let peer_id = generate_peer_id();

    let tracker_url = torrent.announce
        .as_ref()
        .ok_or("torrent has no tracker url".to_string())?;

    let total_size = match &torrent.info.files {
        None => torrent.info.length.unwrap_or(0),
        Some(files) => files.iter().map(|f| f.length).sum(),
    };

    let request = AnnounceRequest::new(info_hash, peer_id, total_size);

    let response = announce(tracker_url, &request)
        .await
        .map_err(|err| err.to_string())?;

    let manager = DownloadManager::new(
        torrent,
        info_hash,
        peer_id,
        PathBuf::from(output_dir),
    );

    manager.start(response.peers, app_handle)
        .await
    .map_err(|e| e.to_string())?;
    
    
    Ok(())
}