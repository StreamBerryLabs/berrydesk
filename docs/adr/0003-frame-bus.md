# ADR 0003: Frame bus transport

## Status

Accepted

## Context

Decoded frames must reach OS-specific drivers in separate processes.

## Decision

Shared-memory ring buffer (single producer, multi consumer) with per-consumer read
cursors. Format documented in ARCHITECTURE.md. No network, no sockets for frames.

## Consequences

Low latency and low overhead. Requires careful synchronization; platform-specific
shared memory APIs wrapped by `shared_memory` crate with POSIX shm fallback.
