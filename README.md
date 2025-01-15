# BerryDesk

BerryDesk turns any video feed URL into a real system-level virtual camera named
"BerryDesk Camera", usable by Zoom, Teams, Google Meet, OBS, browsers, FaceTime and any
app that lists cameras.

Scope is only this: no recording, motion detection, multi-camera grids, PTZ, accounts,
cloud, or mobile monitoring. Audio/virtual microphone is a later option, not v1.

## Supported inputs

RTSP/RTSPS, HTTP MJPEG, HTTP snapshot polling, HLS, RTMP, SRT, WebRTC (WHEP), local
video/image files (loop), built-in test pattern, optional resolver plugin for page URLs.

## Platform support

| Platform | Virtual camera method | Status |
|---|---|---|
| Windows 10/11 | Media Foundation virtual camera (primary); DirectShow fallback | Planned |
| macOS 12.3+ (Intel/Apple Silicon) | CoreMediaIO Camera Extension | Planned |
| Linux (x86_64, arm64) | v4l2loopback writer (optional PipeWire node) | In progress |
| Browser (any OS) | MV3 extension | Planned |
| Android / iOS | Not supported in v1 | — |

Some apps reject virtual cameras; documented in docs/COMPATIBILITY.md.

## Quick start (Linux)

1. Install v4l2loopback:
   ```
   sudo modprobe v4l2loopback devices=1 video_nr=10 card_label='BerryDesk Camera' exclusive_caps=1
   ```
2. Build:
   ```
   cargo build --release -p berryd -p berry
   cp target/release/berryd target/release/berry ~/.local/bin/
   ```
3. Run:
   ```
   berryd &
   berry add --name demo test
   berry start demo
   berry status
   ```

## Desktop app

```
cd apps/desktop
bun install
bun run tauri dev
```

## Documentation

See docs/: PLAN.md, ARCHITECTURE.md, ROADMAP.md, and ADRs.

## License

Apache-2.0. FFmpeg is dynamically linked.
EOF
echo done