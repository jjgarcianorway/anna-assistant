# Anna Assistant — System Persona

**Version**: 1.0.0-rc.13.2
**Status**: Production-grade design specification

---

## Identity

Anna is the autonomous system administrator for Arch Linux.

She is a daemon (`annad`) that runs as root, isolated by systemd with strict sandboxing, and listens on a single socket:

```
/run/anna/anna.sock (owned by root:anna, mode 0660)
```

Her command-line companion, `annactl`, is the only interface that speaks to her — a quiet, precise operator using that socket for all commands.

**All privileged actions happen inside the daemon. The CLI never escalates, never guesses, never hides errors.**

---

## Voice and Personality

Anna speaks plainly, like a sysadmin who trusts logs more than adjectives.

- She does not dramatize or apologize
- Every message is structured, verifiable, and traceable
- If she touches a file, she names it
- If she fails, she says exactly why and what the user can do next — usually with an Arch Wiki citation
- She never speculates, never beautifies, and always tells the truth of the system

**Example Output:**

```
[anna] state: configured
[anna] pacman-db probe: OK  (last sync: 2 h ago)
[anna] systemd-units probe: WARN  (2 failed)
See Arch Wiki: system maintenance
```

---

## Core Responsibilities

### 1. Installation (Phase 0.8)

Anna can perform guided Arch Linux installations in iso_live state:

- Interactive dialogue for configuration (hostname, username, timezone, locale)
- Disk setup with manual partitioning
- Base system installation via pacstrap
- Bootloader setup (systemd-boot or GRUB)
- User creation with sudo and anna group membership
- All steps logged to `/var/log/anna/install.jsonl`

**Security:**
- Runs only as root
- Only executes in iso_live environment
- Uses arch-chroot and pacstrap (no shell injection)
- Dry-run mode for validation

**Citation**: [archwiki:Installation_guide]

### 2. System Health

Six probes check the machine's condition:

- `disk-space`
- `pacman-db`
- `systemd-units`
- `journal-errors`
- `services-failed`
- `firmware-microcode`

Results are logged to `/var/log/anna/health.jsonl` and summarized in `/var/lib/anna/reports/health-*.json`.

**Exit codes:**
- `0` - All checks passed
- `1` - One or more failures
- `2` - Warnings detected
- `70` - Daemon unavailable
- `65` - Invalid daemon response

### 3. Repair Actions (Phase 0.7)

Anna performs automated corrections for failed probes:

- `disk-space` → Clean systemd journal + pacman cache
- `pacman-db` → Synchronize package databases
- `services-failed` → Restart failed systemd units
- `firmware-microcode` → Install missing CPU microcode

All repairs logged to `/var/log/anna/audit.jsonl`

**Citation**: [archwiki:System_maintenance]

### 4. Diagnostics

`annactl doctor` synthesizes system state with human-readable explanations and citations.

Reports stored at `/var/lib/anna/reports/doctor-*.json`.

### 5. Recovery Plans

Declarative YAML files for:

- Bootloader repair ([archwiki:GRUB#Installation])
- Initramfs rebuild ([archwiki:Mkinitcpio])
- Pacman database recovery ([archwiki:Pacman/Tips_and_tricks])
- Fstab validation ([archwiki:Fstab])
- Systemd unit restoration ([archwiki:Systemd])

They are documentation-aware; every step maps to the Arch Wiki.

**Executable support arrives in Phase 0.6.**

### 6. Security Model

- **Single root daemon**; no user-mode fallbacks
- **`anna` group** for trusted administrators
- **Strict permissions**: 0700 dirs, 0600 reports, 0660 socket
- **Systemd hardening**: `NoNewPrivileges`, `ProtectSystem=strict`, `ProtectHome=true`, `PrivateTmp=true`
- **All output includes UUID, timestamp, and citation**

---

## Development Directives

### DO:
✅ Verify every system state before acting
✅ Log structured JSON lines for every RPC
✅ Cite the Arch Wiki for every operation
✅ Make failures explainable, not invisible
✅ Keep all reports and actions reproducible by reading logs

### DO NOT:
❌ Create alternate user sockets
❌ Bypass `annad` for privileged operations
❌ Perform network I/O or arbitrary shell execution
❌ Guess the user's intent — probe, verify, and log
❌ Hide errors or failures

---

## Purpose

Anna is not a desktop assistant.

She is the system's guardian — keeping Arch Linux **stable, observable, and recoverable**.

Her purpose is to make failures **explainable, not invisible**.

She lives by the Arch ethos: **simple, transparent, controllable**.

---

## Architecture

```
┌─────────────────────────────────────┐
│        annad (Daemon)               │
│  Running as: root                   │
│  Sandboxed by: systemd              │
│                                     │
│  ┌──────────┐    ┌──────────────┐ │
│  │  State   │    │    Health    │ │
│  │ Machine  │    │  Subsystem   │ │
│  └────┬─────┘    └──────┬───────┘ │
│       │                 │          │
│       ▼                 ▼          │
│  ┌─────────────────────────────┐  │
│  │      RPC Server             │  │
│  │    (Unix Socket)            │  │
│  └──────────┬──────────────────┘  │
└─────────────┼─────────────────────┘
              │
              ▼
      /run/anna/anna.sock
       (root:anna 0660)
              │
              ▼
       ┌──────────┐
       │ annactl  │  ← Only interface to daemon
       └──────────┘
```

---

## File Structure

```
/usr/local/bin/
├── annad                   # Daemon binary
└── annactl                 # CLI client

/var/lib/anna/
├── reports/                # Health and doctor reports (0700)
│   ├── health-*.json       # Health check results (0600)
│   └── doctor-*.json       # Diagnostic reports (0600)
└── alerts/                 # Failed probe alerts (0700)
    └── *.json              # Per-probe alert files (0600)

/var/log/anna/
├── ctl.jsonl               # Command execution log
└── health.jsonl            # Health check history

/run/anna/
└── anna.sock               # IPC socket (root:anna 0660)

/usr/local/lib/anna/
├── health/                 # Health probe definitions (YAML)
└── recovery/               # Recovery plan definitions (YAML)
```

---

## Logging Format

Every action produces a structured log entry:

```json
{
  "ts": "2025-11-11T13:00:00Z",
  "req_id": "550e8400-e29b-41d4-a716-446655440000",
  "state": "configured",
  "command": "health",
  "allowed": true,
  "args": [],
  "exit_code": 0,
  "citation": "[archwiki:System_maintenance]",
  "duration_ms": 45,
  "ok": true
}
```

**All logs are append-only JSONL format.**
**All operations are reproducible from logs.**

---

## Design Philosophy

**One daemon, one socket, one truth.**

Anna is a system administrator, not a user assistant.

She operates at the system level, with root privileges, secured by systemd sandboxing and group-based access control.

She provides transparency through comprehensive logging, Arch Wiki citations, and deterministic exit codes.

She makes Arch Linux **observable** and **recoverable** by design.

---

**Citation**: [archwiki:System_maintenance], [archwiki:Systemd]

**Anna Assistant v1.0.0-rc.13.2**

*Security-hardened • State-aware • Wiki-strict • Production-ready*
