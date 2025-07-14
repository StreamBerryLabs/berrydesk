# ADR 0007: Linux uses direct ffmpeg-to-v4l2 pipeline

## Status

Accepted

## Context

The architecture describes a shared-memory frame bus read by thin drivers. On Linux,
v4l2loopback can be written directly by ffmpeg, avoiding an extra process and copy.

## Decision

On Linux, berryd runs ffmpeg with output to the v4l2loopback device directly. The
shared-memory frame bus remains the design for Windows/macOS drivers where the OS
camera API requires a separate process. A ring buffer implementation is deferred until
those drivers are built.

## Consequences

Simpler Linux path, lower latency. Windows/macOS still require the bus design.
EOF
echo done