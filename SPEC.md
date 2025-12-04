# Anna Specification v0.0.1

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
2. Probe hardware (CPU, RAM, GPU) and run throughput benchmark
3. Select and pull appropriate model based on hardware
4. Maintain installation ledger at `/var/lib/anna/ledger.json`
5. Provide RPC interface for annactl
6. Run update check every 600 seconds
7. Auto-fix: repair permissions, restart services, re-pull models if needed

**Ledger** tracks:
- Packages installed by Anna
- Models pulled
- Files created
- Configuration changes made

**RPC Methods** (JSON-RPC 2.0):
- `status` - Returns daemon state, model info, ledger summary
- `request` - Send a natural language request to the LLM pipeline
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
- `annactl uninstall` - Trigger safe uninstall
- `annactl reset` - Reset learned data
- `annactl -V` / `annactl --version` - Show version

No other commands or flags are permitted.

**Behavior**:
- If annad is unreachable, display error and suggest re-running installer
- If problems detected, automatically trigger autofix via annad

### LLM Pipeline (v0.0.1)

For v0.0.1, the pipeline is simplified:
- Single model selected based on hardware
- Requests are processed with system prompt for safe, read-only operations
- Pipeline asks clarifying questions when request is ambiguous
- No write operations to user system in v0.0.1

Future versions will implement full translator/dispatcher/specialist/approver roles.

## Installation

**Single command**: `curl -sSL https://anna.example.com/install.sh | sudo bash`

**Installer performs**:
1. Install annad and annactl binaries to `/usr/local/bin/`
2. Create systemd service file
3. Create required directories with correct permissions
4. Add user to `anna` group
5. Enable and start annad service
6. Wait for annad to complete initialization (Ollama + model)

**Requirements**:
- Linux with systemd
- curl, bash
- 8GB+ RAM recommended
- Internet connection for initial setup

## Uninstallation

`annactl uninstall` triggers:
1. annad reads ledger
2. Removes only what Anna installed (packages, models, files)
3. Stops and disables annad service
4. Removes annad and annactl binaries
5. Removes Anna directories

If annad is dead, user must re-run installer to get a working annad for uninstall,
or manually remove files.

## File Layout

```
/usr/local/bin/annad          # Daemon binary
/usr/local/bin/annactl        # CLI binary
/run/anna/anna.sock           # Unix socket (runtime)
/var/lib/anna/                # State directory
/var/lib/anna/ledger.json     # Installation ledger
/var/lib/anna/config.json     # Runtime configuration
/etc/systemd/system/annad.service  # Systemd unit
```

## Constraints

1. **400-line limit**: No source file may exceed 400 lines
2. **CLI surface locked**: Only the commands listed above are allowed
3. **LLM mandatory**: Anna without a working LLM is considered broken
4. **Ledger discipline**: Every system change must be recorded in ledger
5. **No invented facts**: Documentation must reflect actual behavior

## Version

- Version: 0.0.1
- Status: Initial release
- Model selection: Automatic based on hardware probe
