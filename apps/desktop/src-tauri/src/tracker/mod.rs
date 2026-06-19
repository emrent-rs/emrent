#![allow(unused)]

pub mod announce;
pub mod response;
pub mod client;

use crate::torrent::info_hash::InfoHash;
use crate::torrent::peer_id::PeerId;

#[derive(Debug)]
pub enum AnnounceEvent {
    Started, Stopped, Completed, Empty,
}

impl std::fmt::Display for AnnounceEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnnounceEvent::Started => write!(f, "started"),
            AnnounceEvent::Stopped => write!(f, "stopped"),
            AnnounceEvent::Completed => write!(f, "completed"),
            AnnounceEvent::Empty => write!(f, "empty"),
        }
    }
}

#[derive(Debug)]
pub struct AnnounceRequest {
    pub info_hash: InfoHash,
    pub peer_id: PeerId,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub compact: bool,
    pub event: AnnounceEvent,
}

impl AnnounceRequest {
    pub fn new(
        info_hash: InfoHash, peer_id: PeerId, left: u64,
    ) -> Self {
        Self {
            info_hash,
            peer_id,
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left,
            compact: true,
            event: AnnounceEvent::Started
        }
    }
}