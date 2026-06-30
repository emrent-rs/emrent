use anyhow::Result;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::torrent::info_hash::InfoHash;
use crate::torrent::peer_id::PeerId;
use crate::torrent::metainfo::Info;
use super::handshake::perform_handshake;
use super::upload::UploadSession;
use super::manager::PieceState;

pub struct PeerListener {
    info_hash: InfoHash,
    peer_id: PeerId,
    info: Info,
    piece_states: Arc<Mutex<Vec<PieceState>>>,
}

impl PeerListener {
    pub fn new(
        info_hash: InfoHash,
        peer_id: PeerId,
        info: Info,
        piece_states: Arc<Mutex<Vec<PieceState>>>,
    ) -> Self {
        Self {
            info_hash,
            peer_id,
            info,
            piece_states,
        }
    }

    pub async fn listen(&self, port: u16) -> Result<()> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr).await?;

        println!("listening for peers on {}", addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;

            println!("incoming connection from {}", peer_addr);

            let info_hash = self.info_hash;
            let peer_id = self.peer_id;
            let info = self.info.clone();
            let piece_states = Arc::clone(&self.piece_states);

            tokio::spawn(async move {
                let mut stream = stream;

                let handshake = match perform_handshake(
                    &mut stream,
                    info_hash,
                    peer_id,
                ).await {
                    Ok(h) => h,
                    Err(e) => {
                        println!("handshake failed with {}: {}", peer_addr, e);
                        return;
                    }
                };

                if handshake.info_hash != info_hash {
                    println!("info hash mismatch from {}", peer_addr);
                    return;
                }

                let mut upload_session = UploadSession::new(
                    stream,
                    info,
                    piece_states,
                );

                if let Err(e) = upload_session.run().await {
                    println!("upload session error with {}: {}", peer_addr, e);
                }
            });
        }
    }
}
