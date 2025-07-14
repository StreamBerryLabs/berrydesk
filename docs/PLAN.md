# BerryDesk Plan

## Mission

BerryDesk lets a user paste any video feed URL (or pick a file) and turns it into a real
system-level virtual camera named "BerryDesk Camera", usable by Zoom, Teams, Google Meet,
OBS, browsers, FaceTime and any app that lists cameras. Scope is only this: no recording,
motion detection, multi-camera grids, PTZ, accounts, cloud, or mobile monitoring. Audio /
virtual microphone is a later option, not v1.

## Supported inputs

RTSP/RTSPS, HTTP MJPEG, HTTP snapshot polling, HLS, RTMP, SRT, WebRTC (WHEP), local
video/image files (loop), built-in test pattern, and an optional clearly-labeled resolver
plugin for page URLs. Protocol auto-detected from the URL scheme; codec and resolution
probed on add; credentials kept in the OS keychain.

## Platforms

Windows 10/11, macOS 12.3+ (Intel + Apple Silicon), Linux (x86_64, arm64), plus a browser
extension fallback. Android/iOS cannot provide a system-wide virtual camera and are not
supported in v1; stated plainly in docs and UI.

## Architecture

Rust workspace: berry-core (ingest, decode, normalize, reconnect, fallback frame, shared
memory ring frame bus), berryd (background service, local-only control socket), berry-cli
(`berry add|list|start|stop|doctor|install-driver`), berry-vcam (virtual camera
abstraction). Thin per-OS drivers read frames from the bus: platform/windows-vcam (MF +
DirectShow), platform/macos-vcam (Swift Camera Extension), platform/linux-vcam
(v4l2loopback writer). Desktop app: Tauri 2 + React + TypeScript. Browser extension (MV3)
after desktop works. FFmpeg dynamically linked. Default license Apache-2.0.

## Working method

Step A inspect. Step B planning docs. Step C scaffold. Step D fake feed rig. Step E phases
0-5. Step F per-task code/tests/docs/commit. Step G phase status reports.

## Quality bar

Added latency < 300 ms on LAN; < 5% CPU for 1080p H.264 with hardware decode; flat memory
over 72 h. Auto-reconnect; survive codec/resolution changes; never crash host app using
the camera. Unit/integration/UI-e2e tests + manual OS driver checklists. No plaintext
credentials; local-only control socket; signed releases; SBOM; dependency audits; no
telemetry by default. Add a feed and see it as a camera in < 2 min.

## Rules

Check dependency licenses; avoid GPL contamination unless ADR approves. Record assumptions
in ADRs. Ask only when blocked on something only the user can supply. Document platform
limits honestly. No secrets in repo. No schedules/deadlines. Build full production app,
not prototype.
