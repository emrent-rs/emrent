use super::AnnounceRequest;
use crate::torrent::info_hash::InfoHash;
use crate::torrent::peer_id::PeerId;

fn url_encode_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("%{:02x}", b)).collect()
}

pub fn build_announce_url(tracker_url: &str, request: &AnnounceRequest) -> String {
    let info_hash = url_encode_bytes(&request.info_hash);
    let peer_id = url_encode_bytes(&request.peer_id);

    format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}&event={}",
        tracker_url,
        info_hash,
        peer_id,
        request.port,
        request.uploaded,
        request.downloaded,
        request.left,
        if request.compact { 1 } else { 0 },
        request.event,
    )
}
