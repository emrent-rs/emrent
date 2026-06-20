#![allow(unused)]

use crate::torrent::info_hash::InfoHash;
use crate::torrent::peer_id::PeerId;
use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const PROTOCOL: &str = "Bittorrent protocol";
const RESERVED: [u8; 8] = [0u8; 8];

#[derive(Debug)]
pub struct Handshake {
    pub info_hash: InfoHash,
    pub peer_id: PeerId,
}

impl Handshake {
    pub fn new(info_hash: InfoHash, peer_id: PeerId) -> Self {
        Self { info_hash, peer_id }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(68);
        bytes.push(PROTOCOL.len() as u8);
        bytes.extend_from_slice(PROTOCOL.as_bytes());
        bytes.extend_from_slice(&RESERVED);
        bytes.extend_from_slice(&self.info_hash);
        bytes.extend_from_slice(&self.peer_id);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 68 {
            return Err(anyhow!("invalid handshake length: {}", bytes.len()));
        }

        let pstrlen = bytes[0] as usize;
        if pstrlen != 19 {
            return Err(anyhow!("invalid protocol length: {}", pstrlen));
        }

        let pstr = std::str::from_utf8(&bytes[1..20])?;
        if pstr != PROTOCOL {
            return Err(anyhow!("invalid protocol: {}", pstr));
        }

        let mut info_hash = [0u8; 20];
        info_hash.copy_from_slice(&bytes[28..48]);

        let mut peer_id = [0u8; 20];
        peer_id.copy_from_slice(&bytes[48..68]);

        Ok(Self { info_hash, peer_id })
    }
}

pub async fn perform_handshake(
    stream: &mut TcpStream,
    info_hash: InfoHash,
    peer_id: PeerId,
) -> Result<Handshake> {
    let handshake = Handshake::new(info_hash, peer_id);
    let bytes = handshake.to_bytes();

    // println!("handshake length: {}", bytes.len());
    // println!("handshake bytes: {:?}", bytes);
    stream.write_all(&handshake.to_bytes()).await?;

    let mut response = [0u8; 68];
    stream.read_exact(&mut response).await?;

    let peer_handshake = Handshake::from_bytes(&response)?;

    if peer_handshake.info_hash != info_hash {
        return Err(anyhow!("info has mismatch - disconnecting"));
    }

    Ok(peer_handshake)
}
