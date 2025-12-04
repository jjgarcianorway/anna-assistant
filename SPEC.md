# Anna Specification v0.0.6

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

**Ledger** tracks:
- Packages installed by Anna
- Models pulled
- Files created
- Configuration changes made

**RPC Methods** (JSON-RPC 2.0):
- `status` - Returns daemon state, hardware info, model info, update status
- `request` - Send a natural language request (with grounded context)
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

### LLM Pipeline (v0.0.6)

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

**Probe Types**:
- `top_memory` - Top 10 processes by memory usage
- `top_cpu` - Top 10 processes by CPU usage
- `disk_usage` - Filesystem usage
- `network_interfaces` - Network interface info

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

## Version

- Version: 0.0.6
- Status: Grounded assistant with auto-update
