# Threat Model

## Assets

| Asset | Sensitivity | Location |
|-------|-------------|----------|
| GPG release signing key | Critical | Offline, operator-held |
| Telegram bot token | High | `/etc/anna/config.toml` (mode 640) |
| Configuration | Medium | `/etc/anna/config.toml` |
| Session memory / conversation history | Medium | `/var/lib/anna/memory.json` |
| Executor audit log | Medium | `/var/lib/anna/executor_audit.jsonl` |
| Node identity key | Low-Medium | `/var/lib/anna/node_key` (mode 600) |
| Installed binaries | High | `/usr/local/bin/` |

---

## Trust Boundaries

```
Internet
  │
  ▼
Telegram API  ──────────►  annad (anna service user, non-root)
                               │ Unix socket (SO_PEERCRED gated)
                               ▼
                          anna-executor (root, PrivateNetwork=true)
                               │
                               ▼
                          systemctl / paccache / journalctl / find
```

GitHub release pipeline is a separate trust boundary:
- Releases are signed by the operator GPG key (offline)
- `annad` verifies GPG signature before trusting checksums
- Checksums gate binary installation

---

## Adversaries

| Adversary | Capability |
|-----------|------------|
| Network MITM | Can intercept HTTPS but not forge GPG signatures |
| Compromised GitHub account | Can publish releases but not forge signatures (key is offline) |
| Local unprivileged user | Cannot connect to executor socket (SO_PEERCRED rejects non-anna UID) |
| Local anna service user | Can send RPC to executor; limited by enum protocol + allowlist + policy |
| LLM prompt injection | Telegram message crafted to make Anna execute unintended actions |
| Compromised LLM (Ollama) | Can return any tool call, but bounded by executor enum and allowlist |

---

## Attack Vectors and Mitigations

### 1. Malicious release (supply chain)
**Vector:** Attacker publishes a release to GitHub with a backdoored binary.
**Mitigation:** GPG signature required over SHA256SUMS; key is offline.
**Residual:** If the operator's signing machine is compromised.

### 2. HTTPS MITM on update download
**Vector:** Attacker intercepts the GitHub download and serves a different binary.
**Mitigation:** SHA256SUMS is verified against the GPG-signed manifest. Tampered binary fails checksum.
**Residual:** None; checksum is over the binary, not the transport.

### 3. Downgrade attack
**Vector:** Attacker tries to roll back to a version with a known vulnerability.
**Mitigation:** `is_newer_version` rejects remote ≤ current. `Pinned` channel blocks all movement.
**Residual:** Attacker with write access to `/etc/anna/config.toml` could change pinned version.

### 4. Executor privilege escalation via RPC
**Vector:** Attacker sends a crafted message to `/run/anna/anna-executor.sock`.
**Mitigation:** Socket is `root:anna, 0660`. SO_PEERCRED rejects any UID other than anna or root.
**Residual:** None from network. Local root can trivially bypass (root is always trusted).

### 5. Arbitrary executor action via LLM injection
**Vector:** Telegram message convinces LLM to call an executor action not in the enum.
**Mitigation:** Executor protocol is an enum — non-enum requests are rejected as malformed JSON.
Service restart is restricted to a static allowlist. Policy layer can further restrict.
**Residual:** LLM could call a legitimate enum action (e.g. CleanTmpFiles) inappropriately.
Policy file can restrict individual actions to zero.

### 6. Telegram token theft
**Vector:** Attacker reads `/etc/anna/config.toml`.
**Mitigation:** File is mode 640, owner root:anna. Unprivileged users cannot read it.
**Residual:** If attacker achieves anna-group membership or root access.

### 7. Audit log tampering
**Vector:** Attacker deletes or modifies `/var/lib/anna/executor_audit.jsonl`.
**Mitigation:** anna-executor runs with `ProtectSystem=strict`; audit log is in `ReadWritePaths`.
The file is append-only by convention but not enforced by the kernel (no `O_APPEND` locking).
**Residual:** Root can always modify the log.

---

## Residual Risks (Explicit)

- **No remote attestation.** The machine cannot prove to a remote party that it is running unmodified binaries.
- **No Telegram-level authentication.** Any Telegram user who can message the bot can interact with Anna. Bot privacy must be configured at the Telegram level.
- **LLM is untrusted.** The LLM (Ollama) runs locally but its output drives action selection. A manipulated model or jailbroken prompt can abuse legitimate actions.
- **No secrets management.** The Telegram token is stored in plaintext on disk. A future improvement would use a secrets manager (e.g. systemd credentials, kernel keyring).
- **Node key is not yet used for signing.** `/var/lib/anna/node_key` establishes identity for future fleet features but does not currently sign anything.
