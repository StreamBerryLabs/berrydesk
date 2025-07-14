# Security

- No plaintext credentials in files or logs; secrets in OS keychain.
- Control socket local only, permission-restricted (0600 / restrictive ACL).
- Signed releases, SBOM, dependency audit (cargo-audit, npm audit).
- No telemetry by default.
- Input validation on all control socket commands.
- FFmpeg runs with least privilege; no shell interpolation of URLs.
