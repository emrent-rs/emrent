#![allow(unused)]

use rand::Rng;

const CLIENT_ID: &str = "-EM0100-";

pub type PeerId = [u8; 20];

pub fn generate_peer_id() -> PeerId {
    let mut peer_id = [0u8; 20];
    let prefix = CLIENT_ID.as_bytes();

    peer_id[..prefix.len()].copy_from_slice(prefix);

    rand::rng().fill(&mut peer_id[prefix.len()..]);

    peer_id
}
