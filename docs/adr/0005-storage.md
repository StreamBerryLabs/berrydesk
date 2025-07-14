# ADR 0005: Storage and secrets

## Status

Accepted

## Context

Sources need persistence; credentials must not be plaintext.

## Decision

SQLite for source records. Secrets in OS keychain via `keyring` crate. .env.example only.

## Consequences

No plaintext credentials in files or logs. Keychain availability varies by OS; documented
fallback noted in COMPATIBILITY.md.
