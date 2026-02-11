# Anna Architecture

v0.3.155 - Production Ready

Anna is an AI assistant for Arch Linux that combines instant pattern matching with intelligent reasoning for system administration tasks.

## Query Pipeline

```
User Question
    ↓
Pattern Library (28 patterns) → Instant answer (<1ms) if match
    ↓ no match
Ralph Loop (Multi-Agent Orchestration)
    ↓
├─ Domain Routing → 8 domains, 16 specialists
├─ Wiki RAG → Arch Wiki embeddings (768-dim)
├─ System Investigation → Real command execution
├─ Telemetry Analysis → 30-day historical data
└─ Action Plans → Confirmation + rollback
    ↓
LLM (Ollama - qwen2.5)
    ↓
Answer (grounded, evidence-based)
```

## Pattern Library

**Instant Answers:**
- 28 predefined patterns for common scenarios
- Keyword-based matching (0.85-0.95 confidence)
- Response time: <1ms (no LLM overhead)
- Covers: errors, configuration, common Linux tasks

**Categories:**
- Error patterns (12): pacman-lock, disk-full, wifi, audio, etc.
- Config patterns (3): vim-syntax, vim-line-numbers, bash-alias
- Task patterns (13): git-init, ssh-keygen, systemctl, pacman, etc.

**Fallback:**
- Non-matching queries → Ralph loop with LLM

## Ralph Loop (Intelligent Reasoning)

**Iterative Investigation:**
```
1. LLM: "What commands should I run?"
2. Execute commands, capture output
3. LLM: "Is output sufficient?" (YES/NO)
4. If NO: Loop back (up to N iterations)
5. If YES: Generate final answer
```

**Evidence Tracking:**
- Probe results (command + exit code + output)
- Wiki citations (Arch Wiki RAG)
- Man page references
- All answers grounded in actual data

## Multi-Agent System

**8 Domains:**
- Desktop, Network, Storage, Security
- Services, Performance, Hardware, Packages

**16 Specialists (Junior/Senior per domain):**
- Junior: Basic queries, quick diagnostics
- Senior: Complex debugging, multi-step fixes

**Routing:**
- Keyword-based domain detection
- Escalation to seniors for complex issues
- Parallel investigation for multi-domain queries

## Wiki RAG

**Arch Wiki Knowledge Base:**
- Embeddings: nomic-embed-text (768-dim)
- Semantic search + keyword fallback
- Citations in responses
- Local vector database

## Telemetry

**30-Day Historical Data:**
- Hardware snapshots (CPU, RAM, disks, network)
- Package history (installs, updates, removals)
- Performance samples (boot time, resource usage, 100 samples)
- Anomaly baselines (CPU, RAM, disk, network)

**Predictive ML:**
- Disk full prediction (linear regression)
- Boot time degradation alerts
- Memory leak detection
- Capacity planning forecasts

## Action Execution

**Template Plans:**
- GDM auto-login
- Disable lid close suspend
- Display resolution settings

**Lifecycle:**
1. Template match
2. Preflight check (idempotency)
3. State capture (backup to `/var/lib/anna/rollback/`)
4. User confirmation
5. Execute steps (with pkexec)
6. Per-step verification
7. Final verification
8. Rollback on failure
9. Cleanup on success

## Self-Healing

**Auto-Recovery Subsystems:**
- Daemon: Socket connection, auto-restart
- Ollama: Service start with pkexec
- Permissions: Auto-add user to anna group
- Models: Retry with exponential backoff
- Wiki: Retry initialization

**Recovery Hierarchy:**
1. Retry silently
2. Use pkexec for privilege escalation
3. Report failure (no manual commands)

## Multi-Interface

**CLI (annactl):**
- Natural language queries
- Interactive session mode
- Real-time streaming responses
- Status monitoring

**Telegram Bot (Optional):**
- Same backend as CLI
- Remote system administration
- Natural language queries
- Session isolation per user

## Module Structure

```
crates/
├── anna-shared/           # Shared types and utilities
│   ├── patterns/          # Pattern library (28 patterns)
│   ├── agent/             # Multi-agent system
│   ├── prediction/        # Predictive ML
│   ├── knowledge/         # Wiki RAG, man pages
│   └── ...
├── annad/                 # Daemon
│   ├── orchestrator/      # Ralph loop
│   ├── specialists/       # 16 domain specialists
│   ├── model_router/      # LLM model selection
│   └── server/            # HTTP server, streaming
└── annactl/               # CLI client
    ├── main.rs            # CLI interface
    └── ...
```

## System Paths

| Purpose | Path |
|---------|------|
| Config  | `/etc/anna/` |
| State   | `/var/lib/anna/` |
| Runtime | `/run/anna/` |
| Socket  | `/run/anna/anna.sock` |

**Permissions:**
- Directories: 750
- Files: 640
- Socket: 660
- Group: anna

## Data Storage

**State Files:**
- `/var/lib/anna/telemetry/` - Historical snapshots
- `/var/lib/anna/rollback/` - Backup states
- `/var/lib/anna/memory/` - Agent learning
- `/var/lib/anna/wiki/` - RAG embeddings

**Configuration:**
- `/etc/anna/config.toml` - Main config
- `/etc/anna/telegram.env` - Telegram bot credentials (optional)

## Privacy

**100% Local:**
- No cloud APIs
- No external telemetry
- All processing on-device
- Ollama for LLM inference

**Data Retention:**
- Telemetry: 30-day rolling window
- Rollback states: 7 days
- Logs: systemd journald (configurable)

## Version History

**v0.3.155** (2026-02-07):
- Pattern library: 28 patterns
- Multi-agent system: 8 domains, 16 specialists
- Telemetry: 30-day historical analysis
- Predictive ML: trend analysis, forecasting
- Production ready: 100% test success rate

**Earlier phases:**
- v0.3.50: Action execution, template plans
- v0.3.36: Self-healing, auto-recovery
- Phase 47: Capability-gated command policy
- Phase 46: Runtime trust disclosure
- Phase 45: Trust surface review
