import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface TorrentInfo {
  name: string;
  total_size: number;
  piece_count: number;
  info_hash: string;
  is_multi_file: boolean;
  announce: string | null;
  comment: string | null;
  created_by: string | null;
}

interface PeerInfo {
  ip: string;
  port: number;
}

interface AnnounceResult {
  interval: number;
  peers: PeerInfo[];
}

interface ConnectionResult {
  peer_id: string;
  ip: string;
  port: number;
}

interface ProgressPayload {
  downloaded: number;
  total: number;
  piece_index: number;
}

function App() {
  const [torrentInfo, setTorrentInfo] = useState<TorrentInfo | null>(null);
  const [torrentPath, setTorrentPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [peers, setPeers] = useState<PeerInfo[] | null>(null);
  const [announcing, setAnnouncing] = useState(false);
  const [connection, setConnection] = useState<ConnectionResult | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState<ProgressPayload | null>(null);

  useEffect(() => {
    const unlistenProgress = listen<ProgressPayload>("download-progress", (event) => {
      setProgress(event.payload);
    });

    const unlistenComplete = listen("download-complete", () => {
      setDownloading(false);
    });
    
    return () => {
      unlistenProgress.then((f) => f());
      unlistenComplete.then((f) => f());
    };
  }, []);

  async function selectTorrentFile() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Torrent", extensions: ["torrent"] }],
    });

    if (!path) return;

    setTorrentPath(path);

    try {
      const info = await invoke<TorrentInfo>("parse_torrent_file", { path });
      setTorrentInfo(info);
      setError(null);
    } catch (err) {
      setError(String(err));
      setTorrentInfo(null);
    }
  }

  async function announceToTracker() {
    if (!torrentInfo) return;
    if (!torrentInfo.announce) {
      setError("this torrent has no tracker url");
      return;
    }

    try {
      setAnnouncing(true);
      const result = await invoke<AnnounceResult>("announce_to_tracker", {
        trackerUrl: torrentInfo.announce,
        infoHash: torrentInfo.info_hash,
        totalSize: torrentInfo.total_size,
      });
      setPeers(result.peers);
      setError(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setAnnouncing(false);
    }
  }

  async function connectToPeer(ip: string, port: number) {
    if (!torrentInfo) return;

    try {
      const result = await invoke<ConnectionResult>("connect_to_peer_command", {
        ip,
        port,
        infoHash: torrentInfo.info_hash,
      });
      setConnection(result);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }

  async function startDownload() {
    if (!torrentInfo || !torrentPath) return;

    const outputDir = await open({
      directory: true,
      multiple: false,
    });

    if (!outputDir) return;

    try {
      setDownloading(true);
      setProgress(null);
      await invoke("start_download", {
        torrentPath,
        outputDir,
      });
    } catch (err) {
      setError(String(err));
    } finally {
      setDownloading(false);
    }
  }

  return (
    <main className="container">
      <h1>emrent</h1>
      <button onClick={selectTorrentFile}>Open Torrent File</button>

      {error && <p style={{ color: "red" }}>{error}</p>}

      {torrentInfo && (
        <div>
          <p><strong>Name:</strong> {torrentInfo.name}</p>
          <p><strong>Size:</strong> {torrentInfo.total_size} bytes</p>
          <p><strong>Pieces:</strong> {torrentInfo.piece_count}</p>
          <p><strong>Info Hash:</strong> {torrentInfo.info_hash}</p>
          <p><strong>Multi-file:</strong> {torrentInfo.is_multi_file ? "Yes" : "No"}</p>
          <p><strong>Tracker:</strong> {torrentInfo.announce ?? "None"}</p>
          <p><strong>Comment:</strong> {torrentInfo.comment ?? "None"}</p>
          <p><strong>Created by:</strong> {torrentInfo.created_by ?? "None"}</p>
        </div>
      )}

      {torrentInfo && (
        <button onClick={() => announceToTracker()} disabled={announcing}>
          {announcing ? "Announcing..." : "Find Peers"}
        </button>
      )}

      {torrentInfo && (
        <button onClick={() => startDownload()} disabled={downloading}>
          {downloading ? "Downloading..." : "Download"}
        </button>
      )}

      {progress && (
        <div>
          <p>
            <strong>Progress:</strong> {progress.downloaded} / {progress.total} pieces
          </p>
          <progress value={progress.downloaded} max={progress.total} />
        </div>
      )}

      {peers && (
        <div>
          <p><strong>Peers found: {peers.length}</strong></p>
          {peers.map((peer, index) => (
            <div key={index}>
              <span>{peer.ip}:{peer.port}</span>
              <button onClick={() => connectToPeer(peer.ip, peer.port)}>
                Connect
              </button>
            </div>
          ))}
        </div>
      )}

      {connection && (
        <div>
          <p><strong>Connected to:</strong> {connection.ip}:{connection.port}</p>
          <p><strong>Peer ID:</strong> {connection.peer_id}</p>
        </div>
      )}
    </main>
  );
}

export default App;