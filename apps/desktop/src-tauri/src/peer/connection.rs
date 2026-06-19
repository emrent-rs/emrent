#![allow(unused)]

use anyhow::Result;
use tokio::net::TcpStream;
use std::time::Duration;
use crate::torrent::info_hash::InfoHash;
use crate::torrent::peer_id::PeerId;
use super::handshake::{perform_handshake, Handshake};

pub struct PeerConnection {
    pub stream: TcpStream,
    pub handshake: Handshake,
}

pub async fn connect_to_peer( ip: &str, port: u16, info_hash: InfoHash, peer_id: PeerId)
    -> Result<PeerConnection> {
        let addr = format!("{}:{}", ip, port);

        let stream = tokio::time::timeout(
            Duration::from_secs(5),
            TcpStream::connect(&addr),
        )
        .await
        .map_err(|_| anyhow::anyhow!("connection to {} timed out", addr))?
        .map_err(|e| anyhow::anyhow!("failed to connect to {}: {}", addr, e))?;

        let mut stream = stream;

        let handshake = tokio::time::timeout(
            Duration::from_secs(5),
            perform_handshake(&mut stream, info_hash, peer_id),
        )
        .await
        .map_err(|_| anyhow::anyhow!("handshake with {} timed out", addr))??;

        Ok(PeerConnection { stream, handshake })
    }