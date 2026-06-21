use super::announce::build_announce_url;
use super::response::AnnounceResponse;
use super::AnnounceRequest;
use anyhow::Result;

pub async fn announce(tracker_url: &str, request: &AnnounceRequest) -> Result<AnnounceResponse> {
    let url = build_announce_url(tracker_url, request);

    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "tracker returned status: {}",
            response.status()
        ));
    }

    let bytes = response.bytes().await?;

    AnnounceResponse::from_bytes(&bytes)
}
