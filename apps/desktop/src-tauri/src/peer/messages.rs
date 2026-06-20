#![allow(unused)]

use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub enum Message {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Vec<u8>),
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        block: Vec<u8>,
    },
    KeepAlive,
}

impl Message {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Message::KeepAlive => vec![0, 0, 0, 0],

            Message::Choke => vec![0, 0, 0, 1, 0],
            Message::Unchoke => vec![0, 0, 0, 1, 1],
            Message::Interested => vec![0, 0, 0, 1, 2],
            Message::NotInterested => vec![0, 0, 0, 1, 3],

            Message::Have(index) => {
                let mut bytes = vec![0, 0, 0, 5, 4];
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes
            }

            Message::Bitfield(bitfield) => {
                let len = (1 + bitfield.len()) as u32;
                let mut bytes = len.to_be_bytes().to_vec();
                bytes.push(5);
                bytes.extend_from_slice(bitfield);
                bytes
            }
            
            Message::Request { index, begin, length } => {
                let mut bytes = vec![0, 0, 0, 13, 6];
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(&length.to_be_bytes());
                bytes
            }

            Message::Piece { index, begin, block } => {
                let len = (9 + block.len()) as u32;
                let mut bytes = len.to_be_bytes().to_vec();
                bytes.push(7);
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(block);
                bytes
            }
        }
    }

    pub async fn read_from(stream: &mut TcpStream) -> Result<Self> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf);

        if len == 0 {
            return Ok(Message::KeepAlive);
        }

        let mut id_buf = [0u8; 1];
        stream.read_exact(&mut id_buf).await?;
        let id = id_buf[0];

        let payload_len = (len - 1) as usize;
        let mut payload = vec![0u8; payload_len];
        if payload_len > 0 {
            stream.read_exact(&mut payload).await?;
        }

        match id {
            0 => Ok(Message::Choke),
            1 => Ok(Message::Unchoke),
            2 => Ok(Message::Interested),
            3 => Ok(Message::NotInterested),

            4 => {
                if payload.len() != 4 {
                    return Err(anyhow!("invalid have payload length"));
                }
                let index = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                Ok(Message::Have(index))
            }

            5 => Ok(Message::Bitfield(payload)),

            6 => {
                if payload.len() != 12 {
                    return Err(anyhow!("invalid request payload length"));
                }

                let index = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                let begin = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
                let length = u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]);
                Ok(Message::Request { index, begin, length })
            }

            7 => {
                if payload.len() < 8 {
                    return Err(anyhow!("invalid piece payload length"));
                }

                let index = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                let begin = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
                let block = payload[8..].to_vec();
                Ok(Message::Piece { index, begin, block })
            }

            _ => Err(anyhow!("unknown message id: {}", id)),
        }
    }

    pub async fn send_to(&self, stream: &mut TcpStream) -> Result<()> {
        stream.write_all(&self.to_bytes()).await?;
        Ok(())
    }
}
