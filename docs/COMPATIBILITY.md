# Compatibility

| Platform | Virtual camera method | Status | Notes |
|---|---|---|---|
| Windows 10/11 | Media Foundation virtual camera (primary); DirectShow fallback | Planned | Signed COM DLL required |
| macOS 12.3+ (Intel/ARM) | CoreMediaIO Camera Extension | Planned | Apple Developer account, entitlements, notarization, user approval |
| Linux x86_64/arm64 | v4l2loopback writer (optional PipeWire node) | In progress | One-time root/DKMS install |
| Browser (any OS) | MV3 extension overriding getUserMedia/enumerateDevices | Planned | Browser only, not system-wide |
| Android | None system-wide without root | Not supported v1 | Stated in docs/UI |
| iOS | Impossible | Not supported | Stated in docs/UI |

Some apps reject virtual cameras (anti-cheat, some conferencing clients); documented.
H.265: always output decoded raw frames so apps never see source codec.
