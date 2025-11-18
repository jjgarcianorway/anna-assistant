# Anna Assistant Architecture

**Version:** 5.7.0-beta.69
**Last Updated:** 2025-11-18

## High-Level Overview

Anna is a three-layer Arch Linux system assistant designed for reliability, safety, and professional system administration.

```
┌─────────────────────────────────────────┐
│          annactl (User Layer)           │
│  CLI/TUI · Confirmations · Display      │
└────────────────┬────────────────────────┘
                 │ IPC (Unix socket)
┌────────────────┴────────────────────────┐
│         annad (System Layer)            │
│  Root daemon · Telemetry · Execution    │
└────────────────┬────────────────────────┘
                 │ LLM queries
┌────────────────┴────────────────────────┐
│      Anna LLM Layer (Brain)             │
│  Ollama/local · Context-aware prompts   │
└─────────────────────────────────────────┘
```

## Core Components

### 1. annactl (CLI Client)

**Location:** `crates/annactl/`

**Responsibilities:**
- **User Interface**: TUI with colors, progress bars, confirmations
- **Language Support**: Multi-language display (current: English)
- **Model Management**: Select and install LLM models based on hardware
- **REPL Mode**: Interactive conversational interface
- **Intent Routing**: Natural language → Intent enum → handlers
- **Display Formatting**: Present structured LLM outputs to users

**Key Modules:**
- `runtime_prompt.rs` - Build comprehensive LLM prompts with Historian data
- `model_catalog.rs` - Hardware-aware model selection (llama, qwen, etc.)
- `model_setup_wizard.rs` - First-run LLM installation
- `startup_summary.rs` - Display 30-day Historian trends on startup
- `llm_integration.rs` - Query LLM with full system context
- `repl.rs` - Conversational REPL loop
- `intent_router.rs` - Parse natural language to Intents

**Dependencies:**
- `anna_common` (shared types, Historian, LLM config)
- `ratatui` (TUI rendering)
- `owo-colors` (terminal colors)
- `clap` (CLI argument parsing)

### 2. annad (System Daemon)

**Location:** `crates/annad/`

**Responsibilities:**
- **Root Execution**: Run privileged commands safely
- **Telemetry Collection**: Gather real system metrics via `sysinfo`
- **Historian Integration**: Record long-term performance trends
- **IPC Server**: Respond to annactl requests via Unix socket
- **Fact Generation**: Build comprehensive SystemFacts snapshots
- **Service Monitoring**: Track systemd services, boot performance

**Key Modules:**
- `rpc_server.rs` - IPC handler for all client requests
- `telemetry.rs` - Collect real-time system metrics
- `historian_integration.rs` - Record telemetry to Historian database
- `process_stats.rs` - CPU/memory process statistics
- `fact_builder.rs` - Generate SystemFacts from multiple sources

**IPC Methods** (`anna_common/src/ipc.rs`):
- `GetFacts` - Fetch current system facts
- `GetHistorianSummary` - Get 30-day trends from Historian
- `GetAdvice` - Request system recommendations
- `ApplyAction` - Execute approved commands (Phase 2, not yet enabled)
- `Ping` / `Status` - Health checks

**Dependencies:**
- `anna_common` (shared types, Historian engine)
- `sysinfo` (real system metrics)
- `tokio` (async runtime)
- `axum` (HTTP server for IPC)

### 3. anna_common (Shared Library)

**Location:** `crates/anna_common/`

**Responsibilities:**
- **Historian Engine**: Long-term memory and trend analysis (SQLite)
- **Type Definitions**: SystemFacts, Advice, IPC messages
- **LLM Configuration**: Model profiles, backend settings
- **Context Database**: ContextDb for persistent state
- **Display Utilities**: UI helpers for consistent formatting

**Key Modules:**
- `historian.rs` - Historian engine with `SystemSummary` generation
- `types.rs` - Core data structures (SystemFacts, Advice, etc.)
- `ipc.rs` - IPC protocol definitions (Request, Response, Method)
- `llm.rs` - LLM configuration and model profiles
- `context/db.rs` - ContextDb SQLite wrapper
- `display.rs` - UI formatting utilities

**Historian Database Schema** (`crates/anna_common/src/historian.rs`):
```sql
-- Boot sessions with health scores
boot_sessions (boot_id, ts_start, boot_health_score, ...)

-- CPU usage windows (5-minute samples)
cpu_windows (window_start, avg_util_per_core, top_processes, ...)

-- Memory pressure events
memory_windows (window_start, avg_mem_mb, top_processes, ...)

-- Disk capacity snapshots (daily)
disk_snapshots (ts, mountpoint, total_gb, used_gb, ...)

-- Service reliability tracking
service_windows (service_name, crashes_count, restarts_count, ...)

-- Log error signatures
log_signatures (signature_hash, count, last_seen, ...)
```

### 4. Anna LLM Layer

**Backend:** Ollama (local) or OpenAI-compatible APIs (remote)

**Model Selection:**
- **Hardware detection** → Recommend best model for available RAM
- **Basic**: llama3.2:3b (4GB+ RAM)
- **Standard**: llama3.1:8b (8GB+ RAM)
- **Advanced**: qwen2.5:14b (16GB+ RAM)
- **Premium**: deepseek-r1:8b (future, 16GB+ RAM)

**Prompt Structure** (see `INTERNAL_PROMPT.md`):
```
[System Identity] + [Model Context] + [Historian Summary] +
[Current State] + [Personality] + [User Message] + [Instructions]
```

**Output Format:**
```
[ANNA_TUI_HEADER]       # Status, focus, model hint
[ANNA_SUMMARY]          # 2-4 line summary
[ANNA_ACTION_PLAN]      # Machine-readable steps (JSON)
[ANNA_HUMAN_OUTPUT]     # Markdown answer for user
[ANNA_PERSONALITY_VIEW] # Optional: trait display
[ANNA_ROADMAP_UPDATES]  # Optional: dev tasks
[ANNA_CHANGELOG_SUGGESTIONS] # Optional: release notes
```

## Data Flow

### Typical User Query Flow

```
1. User types: "Why is my boot slow?"
   ↓
2. annactl REPL (repl.rs)
   - Parse natural language
   - Route to Intent::SystemStatus
   ↓
3. annactl requests facts + Historian summary
   - IPC call: GetFacts → annad
   - IPC call: GetHistorianSummary → annad
   ↓
4. annad returns:
   - SystemFacts (current state)
   - SystemSummary (30-day trends)
   ↓
5. annactl builds LLM prompt (runtime_prompt.rs)
   - Include SystemFacts
   - Include SystemSummary
   - Include user message
   - Add all instructions (Phase 1, backup rules, etc.)
   ↓
6. annactl queries LLM (llm_integration.rs)
   - Send prompt to Ollama API
   - Receive structured response
   ↓
7. annactl parses and displays response
   - Extract [ANNA_HUMAN_OUTPUT]
   - Render markdown to terminal
   - Show backup/restore commands
```

### Telemetry Collection Flow

```
1. annad starts (main.rs)
   ↓
2. Initialize Historian (historian_integration.rs)
   - Open SQLite database
   - Create schema if needed
   ↓
3. Start telemetry loop (telemetry.rs)
   - Every 5 minutes: collect CPU/memory/disk/services
   - Use sysinfo crate for real metrics
   ↓
4. Record to Historian (historian_integration.rs)
   - Boot events → boot_sessions table
   - CPU samples → cpu_windows table
   - Memory snapshots → memory_windows table
   - Service status → service_windows table
   ↓
5. Generate trends (historian.rs)
   - Compute 30-day averages
   - Detect Up/Down/Flat trends
   - Calculate health scores
   ↓
6. Serve to clients via IPC
   - GetHistorianSummary → SystemSummary
```

## Operating Modes

### Phase 1: Answers Only (Current)

**Behavior:**
- Anna **does NOT execute commands**
- Anna **does NOT modify files**
- Anna **ONLY provides**:
  - Explanations
  - Step-by-step instructions
  - Exact commands for user to run
  - Backup and restore procedures

**Enforcement:**
- LLM prompt explicitly states Phase 1 mode
- All action plans have `requires_confirmation: false`
- User must manually run commands

### Phase 2: Execution Mode (Future)

**Planned Behavior:**
- Anna asks: "Do you want me to do it for you? [Y/N]"
- If confirmed:
  - annactl sends [ANNA_ACTION_PLAN] to annad
  - annad executes commands in order
  - annad returns stdout/stderr/exit codes
  - Anna narrates progress and results

**Not yet enabled** - requires additional safety mechanisms.

## Safety Mechanisms

### 1. Backup Rules

**Mandatory for all file modifications:**
```bash
# Before modifying ~/.vimrc
cp ~/.vimrc ~/.vimrc.ANNA_BACKUP.20251118-203512

# Modify
echo "syntax on" >> ~/.vimrc

# Restore if needed
cp ~/.vimrc.ANNA_BACKUP.20251118-203512 ~/.vimrc
```

### 2. Telemetry-First Approach

**Anna never guesses:**
- Hardware specs → Read from SystemFacts
- Service status → Read from systemd telemetry
- Performance metrics → Read from Historian

**If data is missing:**
```
"I do not have that information yet.
I will propose commands to retrieve it."
```

### 3. Arch Wiki Authority

**All non-trivial advice must cite sources:**
```markdown
This is based on [Arch Wiki: Systemd](https://wiki.archlinux.org/title/Systemd).
```

### 4. Zero Hallucination Policy

**Anna NEVER invents:**
- File paths
- Package names
- Service names
- Configuration values

## Personality System

**Configurable traits** (0-10 scale):
- `introvert_vs_extrovert: 3` → Reserved, speaks when it matters
- `calm_vs_excitable: 8` → Calm, reassuring tone
- `direct_vs_diplomatic: 7` → Clear and direct
- `minimalist_vs_verbose: 7` → Concise but complete
- `analytical_vs_intuitive: 8` → Structured, logical

**Implementation:**
- Traits stored in LLM prompt (`[ANNA_PERSONALITY]`)
- LLM adjusts tone and verbosity based on values
- User can modify: "Make Anna more direct" → Adjust `direct_vs_diplomatic`

## Build and Deployment

**Build:**
```bash
cargo build --release
```

**Install:**
```bash
# Copy binaries
sudo cp target/release/annad /usr/local/bin/
sudo cp target/release/annactl /usr/local/bin/

# Start daemon
sudo systemctl enable annad
sudo systemctl start annad

# Use client
annactl status
annactl  # Interactive REPL
```

**Configuration:**
- Daemon config: `/etc/anna/daemon.conf`
- User config: `~/.config/anna/`
- Context DB: `/var/lib/anna/context.db`
- Historian DB: `/var/lib/anna/historian.db`

## Dependencies

**Core:**
- Rust 2021 edition
- tokio (async runtime)
- sysinfo (system metrics)
- rusqlite (Historian/ContextDb)
- axum (IPC server)
- ratatui (TUI)

**LLM:**
- Ollama (local, recommended)
- Or any OpenAI-compatible API

**System:**
- Arch Linux
- systemd
- pacman/yay

## Security Architecture

**Added in Beta.66-68**: Production-ready security with injection-resistant execution pipeline.

### Command Injection Protection (Beta.66)

**Problem:** Shell command execution vulnerable to metacharacter injection.

**Solution:** Multi-layer security with `ACTION_PLAN` validation:

```rust
// OLD (UNSAFE - deprecated):
fn parse_command(cmd: &str) -> (String, Vec<String>) {
    cmd.split_whitespace()... // ❌ Broken with quotes/spaces
}

// NEW (SECURE):
pub struct ActionPlan {
    steps: Vec<ActionStep>,
}

pub struct ActionStep {
    commands: Vec<Vec<String>>,  // Structured, not shell strings
    backup: Option<String>,      // ANNA_BACKUP.YYYYMMDD-HHMMSS
    risk: ActionRisk,            // Low/Medium/High
    requires_confirmation: bool,
}
```

**Security Layers:**

1. **Structured Commands**: `Vec<Vec<String>>` prevents shell interpretation
2. **Mandatory Validation**: All plans validated before execution
3. **Backup Enforcement**: Critical files require `ANNA_BACKUP` naming
4. **Metacharacter Detection**: Rejects `;`, `&&`, `|`, backticks, `$()`
5. **Risk Classification**: High/Medium risk requires user confirmation
6. **Halt on Failure**: No cascading failures from broken commands

**Example:**
```rust
let plan = ActionPlan::new(vec![
    ActionStep {
        commands: vec![
            vec!["cp".to_string(), ".vimrc".to_string(),
                 ".vimrc.ANNA_BACKUP.20251118-150000".to_string()],
        ],
        risk: ActionRisk::Medium,
        requires_confirmation: true,
        ...
    }
]);

plan.validate()?;  // Checks backup naming, no metacharacters, etc.
execute_action_plan(&plan).await?;  // Safe execution
```

**Testing:** 6 security tests validate injection prevention, backup naming, risk enforcement.

### SafeCommand Builder

**Injection-resistant command construction:**

```rust
let cmd = SafeCommand::new("git")
    .arg("add")
    .arg("file with spaces.txt")  // Automatically safe
    .build()?;

// Generates: ["git", "add", "file with spaces.txt"]
// NOT: "git add file\ with\ spaces.txt"
```

**Benefits:**
- No shell escaping needed
- Spaces/quotes handled correctly by design
- Cannot inject metacharacters

### Backup Safety (Beta.66-67)

**Mandatory backup naming convention:**
- Format: `{original}.ANNA_BACKUP.YYYYMMDD-HHMMSS`
- Example: `.vimrc.ANNA_BACKUP.20251118-143022`
- Validation: Regex `\\.ANNA_BACKUP\\.\\d{8}-\\d{6}$`
- Purpose: Easy identification, sortable, collision-free

**Enforcement:**
- ACTION_PLAN validation rejects incorrect naming
- QA scenarios test backup creation
- Restore hints provided automatically

### Execution Safety

**Halt on First Failure:**
```rust
for step in plan.steps {
    let result = execute_step(&step).await?;
    if !result.success() {
        eprintln!("❌ Failed - Halting execution");
        break;  // NO cascading damage
    }
}
```

**Risk-Based Confirmation:**
- **Low Risk**: Auto-execute (e.g., `ls`, `systemctl status`)
- **Medium Risk**: Require confirmation (e.g., file edits, service restarts)
- **High Risk**: Strong warning + confirmation (e.g., package removal, config overwrites)

### Deprecation Strategy

**Safe migration from unsafe code:**
```rust
#[deprecated(since = "5.7.0-beta.66",
             note = "Use SafeCommand::new() for injection-resistant execution")]
fn parse_command(cmd: &str) -> (String, Vec<String>) {
    eprintln!("⚠️  WARNING: Using deprecated unsafe parse_command()");
    // ... old implementation
}
```

- Existing code continues to work
- Warnings guide migration
- New code uses SafeCommand

### Security Test Coverage

**6 tests in `action_plan.rs`:**
- `test_valid_action_plan` - Valid plans accepted
- `test_reject_empty_commands` - Empty commands rejected
- `test_reject_shell_metacharacters` - Injection attempts blocked
- `test_reject_bad_backup_naming` - ANNA_BACKUP enforced
- `test_high_risk_requires_confirmation` - Risk validation
- `test_safe_command_builder` - Handles spaces/quotes correctly

**9 tests in `qa_scenarios.rs`:**
- Vim scenario: Backup creation, no duplicate blocks
- Hardware scenario: Anti-hallucination validation
- LLM upgrade scenario: Hardware-aware, backup-before-change

**Total:** 15 security-focused tests, all passing.

### Threat Model

**Prevented Attacks:**

1. **Shell Injection via LLM Output**
   - Attacker: Malicious prompt → LLM generates `rm -rf / && ...`
   - Defense: Structured commands, metacharacter validation

2. **File Overwrite Without Backup**
   - Attacker: Trick Anna to modify config without backup
   - Defense: Mandatory `ANNA_BACKUP` naming validation

3. **Cascading Failures**
   - Attacker: One broken command damages entire system
   - Defense: Halt on first failure

4. **Privilege Escalation**
   - Attacker: Use annad's root to run arbitrary commands
   - Defense: SafeCommand prevents injection, risk classification limits damage

**Residual Risks:**
- LLM could still generate *valid but harmful* commands (requires human confirmation)
- Social engineering (user approves bad ACTION_PLAN)
- Mitigation: Medium/High risk requires explicit confirmation

### Security Philosophy

**"Fort Knox Security"** (User Requirement):
1. **Defense in Depth**: Multiple validation layers
2. **Fail Secure**: Reject on any validation failure
3. **Audit Trail**: All actions logged (future: to Historian)
4. **Least Privilege**: Only execute approved commands
5. **Human in Loop**: High-risk operations require confirmation

**NOT Trusted:**
- LLM output (treat as untrusted user input)
- Shell interpretation (bypass with structured commands)
- File system state (validate before modifying)

**Trusted:**
- User confirmation (for Medium/High risk)
- Validated ACTION_PLAN structure
- Backup naming convention

## Future Enhancements

See `ROADMAP.md` for detailed plans:

- **Phase 2**: Execution mode with confirmations
- **Streaming responses**: Real-time LLM output
- **Multi-model support**: Switch models per query
- **Advanced Historian**: Predictive maintenance alerts
- **Web UI**: Browser-based interface alongside CLI

## Documentation

- `README.md` - User-facing overview
- `ARCHITECTURE.md` - This document
- `INTERNAL_PROMPT.md` - LLM prompt structure
- `CHANGELOG.md` - Version history
- `ROADMAP.md` - Future plans
- `CONTRIBUTING.md` - Development guide

## Version

Current: **5.7.0-beta.69** (Production Security + Benchmarking + Model Catalog)

For detailed changes, see `CHANGELOG.md`.
