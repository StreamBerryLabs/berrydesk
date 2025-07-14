# ADR 0001: License

## Status

Accepted

## Context

Need an open-source license. FFmpeg is dynamically linked to avoid GPL contamination.

## Decision

Apache-2.0 for all first-party code. FFmpeg dynamically linked (LGPL-compatible usage).
 GPL contamination avoided unless an ADR approves.

## Consequences

Permissive reuse; patent grant included. Contributors must sign off (DCO).
