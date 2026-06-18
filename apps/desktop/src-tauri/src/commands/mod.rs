#![allow(unused)]

use serde::Serialize;
use crate::torrent::metainfo::Torrent;
use crate::torrent::info_hash::{compute_info_hash, InfoHash};

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

    let torrent = serde_bencode::from_bytes::<Torrent>(&bytes)
        .map_err(|e| e.to_string())?;

    let hash = compute_info_hash(&bytes)
        .map_err(|e| e.to_string())?;

    let info_hash = hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok(TorrentInfo {
        name: torrent.info.name.clone(),
        total_size: torrent.total_size(),
        piece_count: torrent.piece_count(),
        info_hash: info_hash,
        is_multi_file: torrent.is_multi_file(),
        announce: torrent.announce,
        comment: torrent.comment,
        created_by: torrent.created_by,
    })
}


