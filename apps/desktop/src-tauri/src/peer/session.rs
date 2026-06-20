#![allow(unused)]

use super::handshake::perform_handshake;
use super::messages::Message;
use crate::torrent::info_hash::InfoHash;
use crate::torrent::peer_id::PeerId;
use anyhow::{anyhow, Result};
use tokio::net::TcpStream;

pub struct PeerSession {
    stream: TcpStream,
    pub peer_id: PeerId,
    pub bitfield: Vec<u8>,
    pub choked: bool,
    pub interested: bool,
}

impl PeerSession {
    pub async fn new(mut stream: TcpStream, info_hash: InfoHash, peer_id: PeerId) -> Result<Self> {
        let handshake = perform_handshake(&mut stream, info_hash, peer_id).await?;

        let mut session = Self {
            stream,
            peer_id: handshake.peer_id,
            bitfield: Vec::new(),
            choked: true,
            interested: false,
        };

        session.receive_bitfield().await?;

        Ok(session)
    }

    async fn receive_bitfield(&mut self) -> Result<()> {
        let message = Message::read_from(&mut self.stream).await?;

        match message {
            Message::Bitfield(bitfield) => {
                self.bitfield = bitfield;
                Ok(())
            }
            _ => Err(anyhow!("expected bitfield message, got something else")),
        }
    }

    pub async fn send_interested(&mut self) -> Result<()> {
        Message::Interested.send_to(&mut self.stream).await?;
        self.interested = true;
        Ok(())
    }

    pub async fn wait_for_unchoke(&mut self) -> Result<()> {
        loop {
            let message = Message::read_from(&mut self.stream).await?;
            match message {
                Message::Unchoke => {
                    self.choked = false;
                    return Ok(());
                }
                Message::Have(index) => {
                    self.update_bitfield(index);
                }
                Message::KeepAlive => {}
                _ => {}
            }
        }
    }

    fn update_bitfield(&mut self, piece_index: u32) {
        let byte_index = piece_index as usize / 8;
        let bit_index = 7 - (piece_index as usize % 8);
        if byte_index < self.bitfield.len() {
            self.bitfield[byte_index] |= 1 << bit_index;
        }
    }

    pub fn has_piece(&self, piece_index: u32) -> bool {
        let byte_index = piece_index as usize / 8;
        let bit_index = 7 - (piece_index as usize & 8);
        if byte_index >= self.bitfield.len() {
            return false;
        }
        self.bitfield[byte_index] & (1 << bit_index) != 0
    }

    pub async fn request_block(&mut self, index: u32, begin: u32, length: u32) -> Result<Vec<u8>> {
        if self.choked {
            return Err(anyhow!("Peer has choked us"));
        }

        Message::Request {
            index,
            begin,
            length,
        }
        .send_to(&mut self.stream)
        .await?;

        loop {
            let message = Message::read_from(&mut self.stream).await?;
            match message {
                Message::Piece {
                    index: i,
                    begin: b,
                    block,
                } if i == index && b == begin => {
                    return Ok(block);
                }
                Message::Choke => {
                    self.choked = true;
                    return Err(anyhow!("peer choked us mid-download"));
                }
                Message::KeepAlive => {}
                _ => {}
            }
        }
    }
}
