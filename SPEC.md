# Anna Specification v0.0.9

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
- `progress` - Get progress events for current/last request (for polling)
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

## Service Desk Architecture (v0.0.9)

Anna implements a service desk with internal roles (not CLI commands):

### Internal Roles

1. **Translator** (LLM-based): Converts user text to structured JSON ticket
   - Returns strict JSON with: intent, domain, entities, needs_probes, clarification_question, confidence
   - May only select known domains and probe IDs from allowlist
   - Falls back to keyword matching if LLM fails

2. **Dispatcher**: Routes to appropriate specialist
   - Runs probes requested in translator ticket
   - Caches probe results for 30 seconds (TTL)
   - Selects specialist for the domain

3. **Specialist**: Domain expert with deep knowledge
   - **System**: CPU, memory, processes, services, systemd
   - **Network**: Interfaces, routing, DNS, ports, connectivity
   - **Storage**: Disks, partitions, mounts, filesystems
   - **Security**: Permissions, firewalls, audit, ssh
   - **Packages**: Package managers, installation, updates

4. **Supervisor**: Quality control with deterministic scoring
   - Calculates reliability score from concrete signals (not vibes)
   - Builds evidence block showing data sources
   - Validates response is grounded in probe data

### Translator Ticket Format

```json
{
  "intent": "question|request|investigate",
  "domain": "system|network|storage|security|packages",
  "entities": ["process_name", "service_name", ...],
  "needs_probes": ["top_memory", "cpu_info", ...],
  "clarification_question": null | "What do you mean?",
  "confidence": 0.0-1.0
}
```

### Probe IDs (maps to allowlist commands)

| Probe ID | Command |
|----------|---------|
| top_memory | `ps aux --sort=-%mem` |
| top_cpu | `ps aux --sort=-%cpu` |
| cpu_info | `lscpu` |
| memory_info | `free -h` |
| disk_usage | `df -h` |
| block_devices | `lsblk` |
| network_addrs | `ip addr show` |
| network_routes | `ip route` |
| listening_ports | `ss -tulpn` |
| failed_services | `systemctl --failed` |
| system_logs | `journalctl -p warning..alert -n 200 --no-pager` |

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

### Structured Probe Results

Probe results include metadata for evidence tracking:
```json
{
  "command": "ps aux --sort=-%mem",
  "exit_code": 0,
  "stdout": "USER PID %MEM...",
  "stderr": "",
  "timing_ms": 45
}
```

### Deterministic Reliability Scoring

The reliability score is calculated from 5 boolean signals, each worth 20 points:

| Signal | Condition | Points |
|--------|-----------|--------|
| translator_confident | confidence >= 0.7 | 20 |
| probe_coverage | all requested probes succeeded | 20 |
| answer_grounded | answer references probe/hardware data | 20 |
| no_invention | no hedging words (probably, typically, etc.) | 20 |
| clarification_not_needed | no clarification question | 20 |

**Formula**: `score = sum(signal * 20 for signal in signals)`

This is deterministic: same inputs always produce the same score.

### Evidence Block

Every response includes an evidence block showing exactly what data was used:
```json
{
  "hardware_fields": ["cpu_model", "ram_gb", "version"],
  "probes_executed": [/* ProbeResult objects */],
  "translator_ticket": {/* TranslatorTicket object */},
  "last_error": null | "timeout at translator"
}
```

### Response Format

Every response includes:
- `answer`: The LLM's response text
- `reliability_score`: 0-100 deterministic score from signals
- `reliability_signals`: The 5 boolean signals used to calculate score
- `domain`: Which specialist handled it (system/network/storage/security/packages)
- `evidence`: Evidence block with hardware_fields, probes_executed, translator_ticket
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

Clarification is requested when (determined by LLM translator):
- Query is too vague to classify with confidence
- LLM translator sets clarification_question and low confidence
- Falls back to keyword rules if LLM fails:
  - Query has 2 or fewer words (except "cpu" or "memory")
  - Query is just "help" or "help me"

### Timeout Handling (v0.0.9)

All LLM calls and probes have hard timeouts to prevent indefinite hangs:

| Stage | Timeout | Description |
|-------|---------|-------------|
| Translator | 8s | LLM translation of user query |
| Probe (each) | 4s | Individual probe execution |
| Probes (total) | 10s | Total time for all probes |
| Specialist | 12s | LLM specialist response |
| Supervisor | 8s | Response validation |
| RPC call | 45s | Client-side total request timeout |

**Timeout Response Format**:
- `reliability_score` ≤ 20 (max for timeout)
- `needs_clarification` = true
- `clarification_question` = actionable message about the timeout
- `evidence.last_error` = "timeout at <stage>"

**Progress Streaming**:
- In debug mode (default ON), structured progress events are emitted
- Client polls `/progress` RPC method every 250ms
- Events include: starting, complete, timeout, error, probe_running, probe_complete

**Debug Mode**:
- Enabled by default (`debug_mode: true` in status)
- Shows stage transitions instead of static spinner
- Visible in `annactl status` output

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

- Version: 0.0.9
- Status: Timeout handling, progress streaming, hang prevention
