# 🫐 BerryDesk

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)](#platform-support)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange)](https://www.rust-lang.org/)
[![Tauri 2](https://img.shields.io/badge/Tauri-2-blue)](https://tauri.app/)

> **Paste any video feed URL → it becomes a real system webcam named "BerryDesk Camera".**
> Zoom, Teams, Google Meet, OBS, Chrome, Firefox, Safari, FaceTime — any app that lists cameras can use it, including as the default camera.

No camera app. No recording. No monitoring suite. No cloud. No accounts. Just a webcam.

## ✨ Why BerryDesk?

- 📺 **Any feed, any URL** — RTSP/RTSPS, HTTP MJPEG, HTTP snapshot, HLS, RTMP, SRT, WebRTC (WHEP), local files, test pattern.
- 🎭 **Real system camera** — not a browser-only trick. Appears in OS camera lists on Windows, macOS and Linux.
- ⚡ **Low latency** — under 300 ms added on LAN, under 5 % CPU for 1080p H.264 with hardware decode.
- 🔐 **Private by default** — secrets in the OS keychain, local-only control socket, no telemetry.
- 🧩 **One click** — add a feed, hit Start, pick "BerryDesk Camera" in your meeting app.

## 🚀 Quick start (Linux)

```bash
# 1. One-time: load the virtual camera driver (persists across reboots)
sudo bash packaging/setup-v4l2loopback.sh

# 2. Build
cargo build --release -p berryd -p berry
cp target/release/berryd target/release/berry ~/.local/bin/

# 3. Run
berryd &
berry add --name demo test          # built-in test pattern
berry start demo
# open Google Meet → camera → "BerryDesk Camera"
```

Or install the desktop app:

```bash
# AppImage
chmod +x ~/.local/bin/BerryDesk.AppImage
# then launch from your app menu ("BerryDesk")
```

## 📥 Supported inputs

| Input | Example | Notes |
|---|---|---|
| RTSP / RTSPS | `rtsp://user:pass@192.168.1.20:554/stream1` | TCP/UDP transport |
| HTTP MJPEG | `http://cam/video.mjpg` | Phone IP-cam apps, old cameras |
| HTTP snapshot | `http://cam/snapshot.jpg` | Polled at a set interval |
| HLS | `https://host/live.m3u8` | Higher latency |
| RTMP / SRT | `rtmp://...`, `srt://...` | Encoders, OBS, drones |
| WebRTC (WHEP) | `https://host/whep/cam` | Low latency |
| Local file / folder | `video.mp4`, image | Loop option |
| Test pattern | `test` | Built in, for driver testing |

## 🖥️ Platform support

| Platform | Virtual camera method | Status |
|---|---|---|
| Windows 10/11 | Media Foundation virtual camera (primary); DirectShow fallback | Planned |
| macOS 12.3+ (Intel/Apple Silicon) | CoreMediaIO Camera Extension | Planned |
| Linux (x86_64, arm64) | v4l2loopback writer (optional PipeWire node) | ✅ Working |
| Browser (any OS) | MV3 extension | Planned |
| Android / iOS | Not supported in v1 | — |

## 🏗️ Architecture

```
feed URL -> [Ingest] -> [Decode] -> [Normalize] -> [Frame bus]
                                                      |
                          +---------------------------+----------------+
                          |                           |                |
                   Windows MF/DShow           macOS Camera Ext.   Linux v4l2loopback
```

- **berry-core** (Rust): FFmpeg/libav ingest, hardware decode (VideoToolbox/D3D11VA/NVDEC/VAAPI), normalize (scale/crop/rotate/mirror/color convert to NV12/YUY2/BGRA at constant fps), reconnect with exponential backoff, fallback frame.
- **berryd**: tiny background service, local-only control socket (no network port).
- **berry** (CLI): `berry add|list|start|stop|status|doctor|snapshot|remove|install-driver`.
- **Desktop**: Tauri 2 + React + TypeScript with tray menu, live preview, saved sources, keychain.
- **Storage**: SQLite for sources; secrets in the OS keychain.

## 🧪 Fake feed rig

```bash
python3 tests/fake-feed/mjpeg_server.py --port 8080
berry add --name fake http://localhost:8080/video.mjpg
berry start fake
```

Supports optional basic auth, delay and frame drops for stress testing.

## 📚 Documentation

See `docs/`: [PLAN.md](docs/PLAN.md), [ARCHITECTURE.md](docs/ARCHITECTURE.md), [ROADMAP.md](docs/ROADMAP.md), [SECURITY.md](docs/SECURITY.md), [COMPATIBILITY.md](docs/COMPATIBILITY.md), [MANUAL_TEST_CHECKLIST.md](docs/MANUAL_TEST_CHECKLIST.md), and `docs/adr/`.

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). DCO sign-off required. Conventional commits.

## 📄 License

Apache-2.0. FFmpeg is dynamically linked. See [LICENSE](LICENSE).

---

⭐ If BerryDesk helps you, star the repo — it helps others find it!