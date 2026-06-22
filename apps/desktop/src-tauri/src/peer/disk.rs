#![allow(unused)]

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use crate::torrent::metainfo::{Info, File as TorrentFile};

pub struct DiskWriter {
    output_dir: PathBuf,
    info: Info,
}

impl DiskWriter {
    pub fn new(output_dir: PathBuf, info: Info) -> Self {
        Self { output_dir, info }
    }

    pub async fn write_piece(&self, piece_index: u32, data: &[u8]) -> Result<()> {
        let piece_offset = piece_index as u64 * self.info.piece_length;

        match &self.info.files {
            None => self.write_single_file(piece_offset, data).await,
            Some(files) => self.write_multi_file(piece_offset, data, files).await,
        }
    }

    async fn write_single_file(&self, offset: u64, data: &[u8]) -> Result<()> {
        let path = self.output_dir.join(&self.info.name);
        self.ensure_dir(&path).await?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .await?;

        tokio::io::AsyncSeekExt::seek(&mut file, std::io::SeekFrom::Start(offset)).await?;

        file.write_all(data).await?;

        Ok(())
    }

    async fn write_multi_file(&self, piece_offset: u64, data: &[u8], files: &[TorrentFile]) -> Result<()> {
        let mut file_start = 0u64;
        let mut data_offset = 0usize;

        for torrent_file in files {
            let file_end = file_start + torrent_file.length;

            let piece_end = piece_offset + data.len() as u64;

            if piece_offset >= file_end || piece_end <= file_start {
                file_start = file_end;
                continue;
            }

            let write_start = piece_offset.max(file_start);
            let write_end = piece_end.min(file_end);

            let file_offset = write_start - file_start;
            let data_len = (write_end - write_start) as usize;

            let path = self.output_dir
                .join(&self.info.name)
                .join(torrent_file.path.join(std::path::MAIN_SEPARATOR.to_string().as_str()));

            self.ensure_dir(&path).await?;

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&path)
                .await?;

            tokio::io::AsyncSeekExt::seek(
                &mut file,
                std::io::SeekFrom::Start(file_offset),
            ).await?;

            file.write_all(&data[data_offset..data_offset + data_len]).await?;

            data_offset += data_len;
            file_start = file_end;
        }

        Ok(())
    }
    
    async fn ensure_dir(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }
}
