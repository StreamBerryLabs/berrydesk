# Roadmap

Ordered task list. Each phase fully finished and verified before the next. No schedules.

## Phase 0 - Foundation
- Scaffold monorepo, license, CI matrix, ADRs.
- Fake feed test rig (RTSP/MJPEG with packet loss, delay, codec switch, bad password).
- One test frame shown in Chrome on each OS.
Acceptance: CI green; fake feed serves; test frame visible.

## Phase 1 - Core + Linux
- RTSP + MJPEG ingest, decode, normalize, frame bus, reconnect, CLI, v4l2loopback end to end.
Acceptance: `berry start` exposes feed as `/dev/video*` usable by Chrome/Firefox/OBS.

## Phase 2 - Windows + macOS drivers
- MF + DirectShow, Camera Extension, installers, driver wizard, `berry doctor`.
- Test in Zoom, Teams, Meet, OBS, Chrome, Firefox, Safari, FaceTime.
Acceptance: manual checklist per OS/app passes.

## Phase 3 - More inputs + UI
- HLS, RTMP, SRT, WHEP, files, snapshots; Tauri UI with tray, preview, saved sources, keychain.
Acceptance: all inputs work; UI end-to-end.

## Phase 4 - Browser extension + polish
- MV3 extension, adjustments, fallback image, auto-update, docs.
Acceptance: extension works on Chrome/Edge/Firefox.

## Phase 5 - Release
- Signing/notarization, 72h soak test, security review, packaging, v1.0.
Acceptance: signed installers; soak test passes.
