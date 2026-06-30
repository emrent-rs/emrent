use anyhow::Result;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::path::PathBuf;
use crate::torrent::metainfo::Info;
use super::message::Message;
use super::manager::PieceState;

pub struct UploadSession {
    stream: TcpStream,
    info: Info,
    piece_states: Arc<Mutex<Vec<PieceState>>>,
}

impl UploadSession {
    pub fn new(
        stream: TcpStream,
        info: Info,
        piece_states: Arc<Mutex<Vec<PieceState>>>,
    ) -> Self {
        Self {
            stream,
            info,
            piece_states,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.send_bitfield().await?;

        loop {
            let message = Message::read_from(&mut self.stream).await?;

            match message {
                Message::Interested => {
                    Message::Unchoke.send_to(&mut self.stream).await?;
                }
                Message::NotInterested => {
                    Message::Choke.send_to(&mut self.stream).await?;
                }
                Message::Request { index, begin, length } => {
                    self.handle_request(index, begin, length).await?;
                }
                Message::KeepAlive => {}
                _ => {}
            }
        }
    }

    async fn send_bitfield(&mut self) -> Result<()> {
        let states = self.piece_states.lock().await;
        let piece_count = states.len();
        let byte_count = (piece_count + 7) / 8;
        let mut bitfield = vec![0u8; byte_count];

        for (i, state) in states.iter().enumerate() {
            if *state == PieceState::Done {
                let byte_index = i / 8;
                let bit_index = 7 - (i % 8);
                bitfield[byte_index] |= 1 << bit_index;
            }
        }

        Message::Bitfield(bitfield)
            .send_to(&mut self.stream)
            .await?;

        Ok(())
    }

    async fn handle_request(
        &mut self,
        index: u32,
        begin: u32,
        length: u32,
    ) -> Result<()> {
        let has_piece = {
            let states = self.piece_states.lock().await;
            states.get(index as usize)
                .map(|s| *s == PieceState::Done)
                .unwrap_or(false)
        };

        if !has_piece {
            return Ok(());
        }

        let block = self.read_block_from_disk(index, begin, length).await?;

        Message::Piece {
            index,
            begin,
            block,
        }
        .send_to(&mut self.stream)
        .await?;

        Ok(())
    }

    async fn read_block_from_disk(
        &self,
        index: u32,
        begin: u32,
        length: u32,
    ) -> Result<Vec<u8>> {
        use tokio::io::AsyncReadExt;
        use tokio::io::AsyncSeekExt;
        use tokio::fs::File;

        let offset = index as u64 * self.info.piece_length + begin as u64;

        let path = match &self.info.files {
            None => PathBuf::from(&self.info.name),
            Some(_) => PathBuf::from(&self.info.name),
        };

        let mut file = File::open(&path).await?;

        file.seek(std::io::SeekFrom::Start(offset)).await?;

        let mut block = vec![0u8; length as usize];
        file.read_exact(&mut block).await?;

        Ok(block)
    }
}
