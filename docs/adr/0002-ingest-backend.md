# ADR 0002: Ingest backend

## Status

Accepted

## Context

Need to ingest RTSP, MJPEG, HLS, RTMP, SRT, WHEP, files, snapshots.

## Decision

Use FFmpeg (libav) via the `ffmpeg-next` Rust crate for ingest and decode, with hardware
decode where available (VideoToolbox, D3D11VA/NVDEC, VAAPI). GStreamer remains an option
behind a feature flag for platforms where FFmpeg is unavailable.

## Consequences

Single codebase for all protocols; well-tested. Dynamic linking keeps license clean.
Hardware decode availability checked at runtime with software fallback.
