# ADR 0006: Virtual camera strategy per OS

## Status

Accepted

## Context

Each OS exposes virtual cameras differently.

## Decision

- Windows: Media Foundation virtual camera (MFCreateVirtualCamera) primary; DirectShow
  filter fallback for Windows 10 / older apps. Signed COM media source DLL required.
- macOS: CoreMediaIO Camera Extension (system extension). No deprecated DAL plug-ins.
- Linux: v4l2loopback writer (optional PipeWire virtual node).
- Browser: MV3 extension overriding getUserMedia/enumerateDevices (browser only).

## Consequences

Requires code signing and notarization. Some apps reject virtual cameras; documented.
Android/iOS not supported in v1.
