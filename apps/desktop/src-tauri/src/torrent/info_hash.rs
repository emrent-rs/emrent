use anyhow::{anyhow, Result};
use sha1::{Digest, Sha1};

#[allow(unused)]
pub type InfoHash = [u8; 20];

#[allow(unused)]
pub fn compute_info_hash(torrent_bytes: &[u8]) -> Result<InfoHash> {
    let torrent_value: serde_bencode::value::Value = serde_bencode::from_bytes(torrent_bytes)?;

    if let serde_bencode::value::Value::Dict(dict) = torrent_value {
        let info_value = dict.get(b"info" as &[u8])
            .ok_or_else(|| anyhow!("missing info dictionary"))?;

        let info_bytes = serde_bencode::to_bytes(info_value)?;

        let mut hasher = Sha1::new();
        hasher.update(info_bytes);
        let info_hash = hasher.finalize();
        
        Ok(info_hash.into())
    } else {
        anyhow::bail!("torrent file is not a bencode dictionary")
    }
}
