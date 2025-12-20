# BerryDesk

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

BerryDesk turns a video feed URL into a system-level virtual camera named "BerryDesk Camera". Zoom, Teams, Google Meet, OBS, Chrome, Firefox, and FaceTime can use it like any other webcam, including as the default camera.

Scope is deliberately narrow: no recording, motion detection, multi-camera grids, PTZ, accounts, or cloud. Audio and virtual microphone are planned for a later version.

## Supported inputs

RTSP/RTSPS, HTTP MJPEG, HTTP snapshot polling, HLS, RTMP, SRT, WebRTC (WHEP), local video/image files with loop, and a built-in test pattern. Protocol is auto-detected from the URL scheme; codec and resolution are probed when a source is added. Credentials are stored in the OS keychain.

## Quick start (Linux)

```bash
# One-time: load the virtual camera driver (persists across reboots)
sudo bash packaging/setup-v4l2loopback.sh

# Build
cargo build --release -p berryd -p berry
cp target/release/berryd target/release/berry ~/.local/bin/

# Run
berryd &
berry add --name demo test
berry start demo
# In Google Meet: camera -> "BerryDesk Camera"
```

Or use the desktop app (AppImage in `~/.local/bin/BerryDesk.AppImage`).

## Supported inputs (detail)

| Input | Example | Notes |
|---|---|---|
| RTSP / RTSPS | `rtsp://user:pass@192.168.1.20:554/stream1` | TCP or UDP transport |
| HTTP MJPEG | `http://cam/video.mjpg` | Phone IP-camera apps, older cameras |
| HTTP snapshot | `http://cam/snapshot.jpg` | Polled at a set interval |
| HLS | `https://host/live.m3u8` | Higher latency |
| RTMP / SRT | `rtmp://...`, `srt://...` | Encoders, OBS, drones |
| WebRTC (WHEP) | `https://host/whep/cam` | Low latency |
| Local file / folder | `video.mp4`, image | Loop option |
| Test pattern | `test` | Built in, for driver testing |

## Platform support

| Platform | Virtual camera method | Status |
|---|---|---|
| Linux (x86_64, arm64) | v4l2loopback writer, optional PipeWire node | Working |
| Windows 10/11 | Media Foundation virtual camera; DirectShow fallback | Planned |
| macOS 12.3+ (Intel, Apple Silicon) | CoreMediaIO Camera Extension | Planned |
| Browser (any OS) | MV3 extension overriding getUserMedia | Planned |
| Android / iOS | Not supported in v1 | — |

Some applications reject virtual cameras by policy (anti-cheat software, some conferencing clients). That is an OS or application limitation and is documented in `docs/COMPATIBILITY.md`.

## Architecture

```
feed URL -> [Ingest] -> [Decode] -> [Normalize] -> [Frame bus]
                                                      |
                          +---------------------------+----------------+
                          |                           |                |
                   Windows MF/DShow           macOS Camera Ext.   Linux v4l2loopback
```

- `berry-core` (Rust): FFmpeg/libav ingest, hardware decode (VideoToolbox, D3D11VA/NVDEC, VAAPI), normalize (scale, crop, rotate, mirror, color convert to NV12/YUY2/BGRA at constant frame rate), reconnect with exponential backoff, fallback frame.
- `berryd`: background service with a local-only control socket (Unix socket or named pipe, no network port).
- `berry` (CLI): `add`, `list`, `start`, `stop`, `status`, `doctor`, `snapshot`, `remove`, `install-driver`.
- Desktop: Tauri 2 + React + TypeScript, with tray menu, live preview, saved sources, and OS keychain integration.
- Storage: SQLite for sources; secrets in the OS keychain.

## Fake feed rig

```bash
python3 tests/fake-feed/mjpeg_server.py --port 8080
berry add --name fake http://localhost:8080/video.mjpg
berry start fake
```

The server supports optional basic auth, artificial delay, and frame drops for stress testing.

## Documentation

- [PLAN.md](docs/PLAN.md)
- [ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [ROADMAP.md](docs/ROADMAP.md)
- [SECURITY.md](docs/SECURITY.md)
- [COMPATIBILITY.md](docs/COMPATIBILITY.md)
- [MANUAL_TEST_CHECKLIST.md](docs/MANUAL_TEST_CHECKLIST.md)
- [ADRs](docs/adr/)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Commits require DCO sign-off. Use conventional commit messages.

## License

Apache-2.0. FFmpeg is dynamically linked. See [LICENSE](LICENSE).