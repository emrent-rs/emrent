# emrent

A modern, cross-platform BitTorrent client built in Rust with a Tauri + React frontend.


---

## What is emrent?

emrent is a fully featured BitTorrent client that aims to be fast, lightweight, and approachable. It is built on Rust for the backend — giving it low memory usage, no garbage collector pauses, and safe concurrency — and Tauri + React for the frontend, delivering a native desktop experience on Windows, macOS, and Linux from a single codebase.

The project is being built progressively:
- Phase 1 uses well-established public crates to get a working application shipped
- Later phases replace those crates one by one with purpose-built `emrent-*` implementations

---

## Features (Planned)

- Parse `.torrent` files and magnet links
- Connect to HTTP and UDP trackers
- Peer discovery via DHT (Distributed Hash Table)
- Multi-peer concurrent downloading
- Piece verification via SHA-1
- Seeding / uploading to the swarm
- Clean native desktop UI built with React
- Cross-platform: Linux, macOS, Windows

---

## Project Structure

emrent is organized as a Cargo workspace. Each concern is its own crate:

```
emrent/
├── apps/
│   └── desktop/              # Tauri desktop application
│       ├── src/              # React frontend
│       └── src-tauri/        # Rust backend (Tauri entry point)
├── crates/
│   ├── emrent-bencode/       # Bencode encoder/decoder
│   ├── emrent-metainfo/      # .torrent file and magnet link parsing
│   ├── emrent-tracker/       # HTTP and UDP tracker communication
│   ├── emrent-dht/           # Kademlia DHT implementation
│   ├── emrent-peer/          # Peer wire protocol
│   └── emrent-pieces/        # Piece selection, verification, and disk I/O
├── patches/                  # Patched transitive dependencies
├── Cargo.toml                # Workspace root
├── LICENSE-MIT
└── LICENSE-APACHE
```

The `crates/` directory is reserved for pure Rust libraries. The `apps/` directory holds deliverable applications. A `web/` frontend may be added to `apps/` in the future.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| Async runtime | Tokio |
| Desktop framework | Tauri v2 |
| Frontend framework | React + TypeScript |
| Serialization | Serde + serde_bencode |
| Networking | Reqwest, Tokio TCP |
| Hashing | SHA-1 |
| Error handling | anyhow + thiserror |

---

## Prerequisites

### All platforms
- [Rust](https://rustup.rs) (latest stable or nightly)
- [Node.js](https://nodejs.org) v18 or later
- npm v9 or later

### Linux only

Tauri requires the following system libraries on Linux:

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libcairo2-dev \
  libgdk-pixbuf2.0-dev \
  libpango1.0-dev \
  libatk1.0-dev \
  libglib2.0-dev
```

### macOS only
- Xcode Command Line Tools: `xcode-select --install`

### Windows only
- [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11)

---

## Getting Started

### Clone the repo

```bash
git clone https://github.com/emrent-rs/emrent.git
cd emrent
```

### Install frontend dependencies

```bash
cd apps/desktop
npm install
```

### Run in development mode

```bash
npm run tauri dev
```

### Build for production

```bash
npm run tauri build
```

---

## Development Roadmap

| Phase | Description | Status |
|---|---|---|
| 1 | Torrent file parsing and info hash computation | 🚧 In progress |
| 2 | Tracker communication (HTTP + UDP) | ⏳ Planned |
| 3 | Peer handshake and wire protocol | ⏳ Planned |
| 4 | Piece downloading and SHA-1 verification | ⏳ Planned |
| 5 | Piece selection strategy and peer management | ⏳ Planned |
| 6 | Seeding and uploading | ⏳ Planned |
| 7 | DHT (Distributed Hash Table) | ⏳ Planned |
| 8 | Magnet link support | ⏳ Planned |
| 9 | Replace public crates with emrent-* implementations | ⏳ Planned |

---

## Workspace Crates

As the project matures, the following crates will be extracted as standalone, publishable libraries:

| Crate | Description |
|---|---|
| `emrent-bencode` | Bencode encoder and decoder |
| `emrent-metainfo` | Torrent metadata parsing |
| `emrent-tracker` | Tracker client (HTTP + UDP) |
| `emrent-dht` | Kademlia DHT node |
| `emrent-peer` | Peer wire protocol implementation |
| `emrent-pieces` | Piece management and verification |

---

## Contributing

emrent is currently in early development. Contributions, ideas, and feedback are welcome. Please open an issue before submitting a pull request so we can discuss the change first.

---

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.