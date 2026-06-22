#![allow(unused)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Torrent {
    pub announce: Option<String>,

    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,

    pub info: Info,

    #[serde(rename = "created by")]
    pub created_by: Option<String>,

    #[serde(rename = "creation date")]
    pub creation_date: Option<i64>,

    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Info {
    pub name: String,

    #[serde(rename = "piece-length")]
    pub piece_length: u64,
    pub pieces: serde_bytes::ByteBuf,
    pub private: Option<u8>,
    pub length: Option<u64>,
    pub files: Option<Vec<File>>,
}

#[derive(Debug, Deserialize)]
pub struct File {
    pub length: u64,
    pub path: Vec<String>,
}

impl Torrent {
    /// Returns true if this is a multi-file torrent
    pub fn is_multi_file(&self) -> bool {
        self.info.files.is_some()
    }

    /// Return the total size of all files in bytes
    pub fn total_size(&self) -> u64 {
        match &self.info.files {
            Some(files) => files.iter().map(|file| file.length).sum(),
            None => self.info.length.unwrap_or(0),
        }
    }

    /// Return the number of pieces
    pub fn piece_count(&self) -> usize {
        self.info.pieces.len()
    }
}
