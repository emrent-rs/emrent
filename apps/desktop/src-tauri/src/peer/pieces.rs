#![allow(unused)]

use super::session::PeerSession;
use anyhow::{anyhow, Result};
use sha1::{Digest, Sha1};

const BLOCK_SIZE: u32 = 16384;

pub struct PieceDownloader {
    pub index: u32,
    pub length: u32,
    pub hash: [u8; 20],
}

impl PieceDownloader {
    pub fn new(index: u32, length: u32, hash: [u8; 20]) -> Self {
        Self {
            index,
            length,
            hash,
        }
    }

    pub async fn download(&self, session: &mut PeerSession) -> Result<Vec<u8>> {
        let mut data = vec![0u8; self.length as usize];
        let mut offset = 0u32;

        while offset < self.length {
            let block_size = BLOCK_SIZE.min(self.length - offset);

            let block = session
                .request_block(self.index, offset, block_size)
                .await?;

            if block.len() != block_size as usize {
                return Err(anyhow!(
                    "block size mismatch: expected {} got {}",
                    block_size,
                    block.len()
                ));
            }

            data[offset as usize..offset as usize + block.len()].copy_from_slice(&block);
            offset += block_size;
        }

        self.verify(&data)?;

        Ok(data)
    }

    fn verify(&self, data: &[u8]) -> Result<()> {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let hash: [u8; 20] = hasher.finalize().into();

        if hash != self.hash {
            return Err(anyhow!(
                "piece {} hash mismatch - data is corrupt",
                self.index
            ));
        }
        Ok(())
    }
}
