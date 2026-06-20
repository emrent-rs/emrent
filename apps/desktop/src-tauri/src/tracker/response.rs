use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::net::Ipv4Addr;

#[derive(Debug, Deserialize)]
struct RawAnnounceResponse {
    pub interval: u64,
    pub peers: serde_bytes::ByteBuf,
}

#[derive(Debug)]
pub struct Peer {
    pub ip: Ipv4Addr,
    pub port: u16,
}

#[derive(Debug)]
pub struct AnnounceResponse {
    pub interval: u64,
    pub peers: Vec<Peer>,
}

impl AnnounceResponse {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let raw: RawAnnounceResponse = serde_bencode::from_bytes(bytes)?;

        if !raw.peers.len().is_multiple_of(6) {
            return Err(anyhow!("invalid peers length: {} bytes", raw.peers.len()));
        }

        let peers = raw
            .peers
            .chunks(6)
            .map(|chunk| {
                let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
                let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                Peer { ip, port }
            })
            .collect();

        Ok(AnnounceResponse {
            interval: raw.interval,
            peers,
        })
    }
}
