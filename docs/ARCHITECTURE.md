# BerryDesk Architecture

## Data flow

```
feed URL -> [Ingest] -> [Decode] -> [Normalize] -> [Frame bus: shared memory ring]
                                                          |
                          +-------------------------------+----------------+
                          |               |                                |
                   Windows MF/DShow   macOS Camera Ext.               Linux v4l2loopback
                   virtual camera     (Swift)                          (+ PipeWire)
```

## Components

- **berry-core** (Rust): ingest (FFmpeg/libav or GStreamer), hardware decode
  (VideoToolbox, D3D11VA/NVDEC, VAAPI), normalize (scale, crop, rotate, mirror, color
  convert to NV12/YUY2/BGRA at constant frame rate), reconnect with exponential backoff,
  fallback frame, shared-memory frame bus.
- **berryd**: tiny background service owning the active pipeline and a local-only control
  socket (named pipe / Unix socket, no network port).
- **berry** (CLI): `berry add <url>`, `berry list`, `berry start <name>`, `berry stop`,
  `berry doctor`, `berry install-driver`.
- **berry-vcam**: virtual camera abstraction trait.
- **Per-OS drivers**: thin; only read frames from the bus.
- **Desktop UI**: Tauri 2 + React + TypeScript.
- **Browser extension**: MV3, optional, after desktop works.
- **Storage**: SQLite for sources; secrets in OS keychain.

## Threading model

- Ingest thread (or async task) reads packets.
- Decoder runs on a dedicated thread (FFmpeg) or via hardware decode context.
- Normalizer runs inline after decode.
- Frame bus writer pushes normalized frames into a shared-memory ring buffer.
- Driver processes (separate, OS-specific) read from the ring buffer and present to the OS.

## IPC

Control socket is local-only and permission-restricted (Unix socket mode 0600 / named pipe
ACL). Protocol: newline-delimited JSON requests/responses. Commands: add, list, start,
stop, status, doctor. No network port is ever opened.

## Frame format

Ring buffer header (little-endian, 64 bytes):

- magic: u32 = 0x42444652 ("BDFR")
- version: u32
- width: u32, height: u32
- format: u32 (0=NV12, 1=YUY2, 2=BGRA)
- fps_num: u32, fps_den: u32
- stride: u32
- capacity: u32 (number of slots)
- write_idx: atomic u64
- read_idx: atomic u64 (per consumer; multiple consumers tracked separately)
- slot_size: u64

Each slot: timestamp_us: u64 + frame data (slot_size bytes). Ring is single-producer,
multi-consumer with per-consumer read cursors.

## Security

No plaintext credentials in files or logs. Control socket local only and permission
restricted. Signed releases, SBOM, dependency audit, no telemetry by default.
