import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
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

function App() {
  const [torrentInfo, setTorrentInfo] = useState<TorrentInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function selectTorrentFile() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Torrent", extensions: ["torrent"] }],
    });

    if (!path) return;

    try {
      const info = await invoke<TorrentInfo>("parse_torrent_file", { path });
      setTorrentInfo(info);
      setError(null);
    } catch (err) {
      setError(String(err));
      setTorrentInfo(null);
    }
  }

  async function testInvoke() {
		  try {
				  const result = await invoke("greet", { name: "test" });
				  console.log("IPC works:", result);
		  } catch (err) {
				  console.log("IPC failed:", err);
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
    </main>
  );
}

export default App;
