# Anna Specification v0.0.7

This document is the authoritative specification for Anna. All implementation
must conform to this spec. If code and spec conflict, update spec first, then code.

## Overview

Anna is a local AI assistant for Linux systems. It consists of two components:
- **annad**: A root-level systemd service that manages system state, Ollama, and models
- **annactl**: A user-facing CLI that communicates with annad over a Unix socket

## Architecture

```
┌─────────────┐     Unix Socket     ┌─────────────┐
│   annactl   │ ◄─────────────────► │    annad    │
│  (user CLI) │    JSON-RPC 2.0     │  (root svc) │
└─────────────┘                     └─────────────┘
                                          │
                                          ▼
                                   ┌─────────────┐
                                   │   Ollama    │
                                   │  (managed)  │
                                   └─────────────┘
```

## Component Specifications

### annad (Daemon)

**Runs as**: root (systemd service)
**Socket**: `/run/anna/anna.sock`
**State directory**: `/var/lib/anna/`
**Log**: systemd journal

**Responsibilities**:
1. Install and manage Ollama
2. Probe hardware (CPU, RAM, GPU) and select appropriate model
3. Pull and manage models based on hardware capabilities
4. Maintain installation ledger at `/var/lib/anna/ledger.json`
5. Provide RPC interface for annactl
6. Run update check every 60 seconds, auto-update if enabled
7. Self-healing: repair permissions, restart services, re-pull models
8. Run system probes for grounded LLM responses
9. Execute service desk pipeline for all requests

**Ledger** tracks:
- Packages installed by Anna
- Models pulled
- Files created
- Configuration changes made

**RPC Methods** (JSON-RPC 2.0):
- `status` - Returns daemon state, hardware info, model info, update status
- `request` - Send a natural language request (service desk pipeline)
- `probe` - Run a read-only system probe (top_memory, top_cpu, disk_usage, etc.)
- `reset` - Wipe learned data and post-install ledger entries
- `uninstall` - Execute safe uninstall using ledger
- `autofix` - Trigger self-repair routines

### annactl (CLI)

**Runs as**: Current user
**Connects to**: `/run/anna/anna.sock`

**Commands** (locked CLI surface - no additions allowed):
- `annactl <request>` - Send request to Anna
- `annactl` (no args) - Enter REPL mode
- `annactl status` - Show system status
- `annactl reset` - Reset learned data
- `annactl uninstall` - Trigger safe uninstall
- `annactl -V` / `annactl --version` - Show version

No other commands or flags are permitted.

**REPL exit commands**: `exit`, `quit`, `bye`, `q`, `:q`, `:wq`

**Behavior**:
- If annad is unreachable, display error and suggest re-running installer
- If problems detected, automatically trigger autofix via annad

## Service Desk Architecture (v0.0.7)

Anna implements a service desk with internal roles (not CLI commands):

### Internal Roles

1. **Translator**: Converts user text to structured intent
   - Classifies query into specialist domain
   - Detects ambiguity and need for clarification

2. **Dispatcher**: Routes to appropriate specialist
   - Determines required probes based on domain and query
   - Selects specialist for the domain

3. **Specialist**: Domain expert with deep knowledge
   - **System**: CPU, memory, processes, services, systemd
   - **Network**: Interfaces, routing, DNS, ports, connectivity
   - **Storage**: Disks, partitions, mounts, filesystems
   - **Security**: Permissions, firewalls, audit, ssh
   - **Packages**: Package managers, installation, updates

4. **Supervisor**: Quality control
   - Estimates reliability score (0-100)
   - Validates response is grounded in probe data

### Probe Allowlist (Read-Only)

Only these commands are allowed for probes:
- `ps aux --sort=-%mem` - Top processes by memory
- `ps aux --sort=-%cpu` - Top processes by CPU
- `lscpu` - CPU information
- `free -h` - Memory status
- `df -h` - Disk usage
- `lsblk` - Block devices
- `ip addr show` - Network interfaces
- `ip route` - Routing table
- `ss -tulpn` - Listening ports
- `systemctl --failed` - Failed services
- `journalctl -p warning..alert -n 200 --no-pager` - Recent warnings

Any command not in this list is DENIED.

### Response Format

Every response includes:
- `answer`: The LLM's response text
- `reliability_score`: 0-100 confidence rating
- `domain`: Which specialist handled it (system/network/storage/security/packages)
- `probes_used`: List of probes that were run
- `needs_clarification`: Whether more info is needed
- `clarification_question`: Question to ask if clarification needed

### Unified Output

One-shot and REPL modes use identical formatting:
```
anna v0.0.7 (dispatch)
──────────────────────────────────────
[you]
<user query>

[anna] <domain> specialist  reliability: <score>%
<response>

probes: <list of probes used>
──────────────────────────────────────
```

### Clarification Rules

Clarification is requested when:
- Query has 2 or fewer words (except "cpu" or "memory")
- Query is just "help" or "help me"

## LLM Pipeline

**Grounding Policy** (MANDATORY):
1. Every LLM request includes a RuntimeContext with:
   - Exact version number
   - Hardware snapshot (CPU model, cores, RAM, GPU, VRAM)
   - Capability flags (what Anna can/cannot do)
   - Probe results (if relevant to the query)

2. The system prompt enforces:
   - Never invent facts not in the context
   - Answer hardware questions directly from snapshot
   - Auto-run probes for process/memory/disk queries
   - Never suggest manual commands when data is available
   - Never claim capabilities not in the flags

**Model Selection** (based on hardware):
- 12GB+ VRAM: qwen2.5:14b
- 8GB VRAM: llama3.1:8b
- 6GB VRAM: qwen2.5:7b
- 4GB VRAM: llama3.2:3b
- No GPU, 32GB+ RAM: llama3.1:8b
- No GPU, 16GB+ RAM: llama3.2:3b
- No GPU, 8GB+ RAM: llama3.2:1b
- Limited: qwen2.5:0.5b

## Installation

**Single command**:
```bash
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

**Installer performs**:
1. Download and verify binaries (SHA256)
2. Install to `/usr/local/bin/`
3. Create `anna` group and add user
4. Create directories with correct permissions
5. Install and start systemd service
6. Wait for annad to complete initialization

**Requirements**:
- Linux with systemd
- curl, bash, sha256sum
- 8GB+ RAM recommended

## File Layout

```
/usr/local/bin/annad          # Daemon binary
/usr/local/bin/annactl        # CLI binary
/run/anna/anna.sock           # Unix socket (runtime)
/var/lib/anna/                # State directory
/var/lib/anna/ledger.json     # Installation ledger
/var/lib/anna/models/         # Ollama models
/etc/anna/config.toml         # Configuration
/etc/systemd/system/annad.service  # Systemd unit
```

## Constraints

1. **400-line limit**: No source file may exceed 400 lines
2. **CLI surface locked**: Only the commands listed above are allowed
3. **LLM mandatory**: Anna without a working LLM is considered broken
4. **Ledger discipline**: Every system change must be recorded in ledger
5. **Grounding mandatory**: All LLM responses must be grounded in runtime context
6. **No invented facts**: Anna must never claim capabilities or state facts not in context
7. **Probe allowlist**: Only read-only commands in the allowlist may be executed

## Version

- Version: 0.0.7
- Status: Service desk with unified output and reliability scores
