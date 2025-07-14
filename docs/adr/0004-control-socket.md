# ADR 0004: Control socket

## Status

Accepted

## Context

Desktop UI and CLI need to talk to berryd.

## Decision

Local-only control socket: Unix domain socket on Linux/macOS (mode 0600), named pipe on
Windows with restrictive ACL. Newline-delimited JSON. No TCP port.

## Consequences

No network exposure. Permission-restricted. berryd must run as the same user.
