# Security

## Privilege Model

Two processes; principle of least privilege.

| Process | User | Capabilities |
|---------|------|--------------|
| `annad` | `anna` (non-root) | No network write, no root, no raw sockets |
| `anna-executor` | `root` | No network (`PrivateNetwork=true`), `CapabilityBoundingSet=` empty, `NoNewPrivileges=true`, `MemoryDenyWriteExecute=true` |

Telegram → `annad` only. `annad` → `anna-executor` via Unix socket RPC only.
Telegram cannot trigger executor RPCs directly.

## Update Chain

1. GitHub release contains: `annad-linux-{arch}`, `anna-executor-linux-{arch}`, `annactl-linux-{arch}`, `SHA256SUMS`, `SHA256SUMS.asc`
2. `annad` downloads all assets, then verifies `SHA256SUMS.asc` with the embedded GPG key before reading checksums.
3. SHA-256 of each binary is verified against the signed `SHA256SUMS`.
4. Downloaded binaries must report the expected version string.
5. Downgrade is blocked: `is_newer_version` rejects remote ≤ current.
6. Rollback slot (`/var/lib/anna/rollback/`) is saved before any binary is replaced.
7. Post-restart: `annad --version` polled 5×2s; on failure, rollback slot is restored and both services restarted.

## Key Management

`ANNA_GPG_PUBLIC_KEYS` in `update.rs` is a slice of `(fingerprint, armored_key)` pairs.
`REVOKED_GPG_FINGERPRINTS` lists fingerprints whose signatures are rejected even if the key is still present.

Rotation procedure: add new key → release → remove old key in the next release.

## Network Isolation

`anna-executor` runs with `PrivateNetwork=true` and `RestrictAddressFamilies=AF_UNIX`.
It cannot open TCP/UDP sockets or make outbound connections.

`annad` has network access for GitHub release downloads and Telegram polling only.

## Audit Trail

Every `anna-executor` RPC is appended to `/var/lib/anna/executor_audit.jsonl`:

```json
{"ts":"2026-01-01T00:00:00Z","action":"RestartService:pipewire","outcome":"ok"}
```

## Concurrency

`annad` accepts at most 8 concurrent connections (semaphore). Each connection times out after 300 s.

## Multi-Machine Model

Each machine runs an independent anna instance. Updates come from GitHub releases only, verified by the embedded GPG key. No central controller exists. Telegram tokens are configured per machine via `/etc/anna/config.toml`.
