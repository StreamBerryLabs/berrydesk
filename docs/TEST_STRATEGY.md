# Test strategy

- Unit: URL parsing, protocol detection, pipeline state machine, ring buffer.
- Integration: fake feeds (RTSP/MJPEG with loss/delay/auth failure).
- UI e2e: Tauri webdriver / Playwright.
- Manual checklists: OS drivers and target apps.
- Soak: 72h memory/CPU tracking.
- Security: dependency audit, secret scanning, control socket permissions.
