# Anna Features

## Implemented and Tested

### Core Pipeline
- **Grounded Responses**: Answers from actual system data, never invents facts
- **Auto-probes**: Automatically runs system queries for memory/CPU/disk questions
- **Deterministic Routing**: Queries routed to appropriate handlers based on intent
- **Hardware-aware**: Selects optimal model based on CPU, RAM, and GPU

### Team-Scoped Specialists (v0.0.26)
- **8 Domain Teams**: Desktop, Storage, Network, Performance, Services, Security, Hardware, General
- **3 Roles per Team**: Translator, Junior Reviewer, Senior Reviewer
- **SpecialistsRegistry**: 24 profiles with configurable model, rounds, and thresholds

### Deterministic Review Gate (v0.0.26)
- **Hybrid Review**: Deterministic logic first, LLM only when unclear
- **Gate Rules**:
  - Invention detected → Escalate to Senior
  - No claims with evidence required → Revise (TooVague)
  - Low grounding → Revise (MissingEvidence)
  - High score (≥80) → Accept
  - Medium score (50-79) → LLM review required
- **Confidence Scoring**: 0.0-1.0 confidence on each decision

### Team-Specific Review Prompts (v0.0.26)
- **Storage Team**: Verifies df, lsblk, mount output
- **Network Team**: Verifies ip, ss, nmcli output
- **Performance Team**: Verifies free, top, vmstat output
- **Services Team**: Verifies systemctl status
- **Security Team**: Flags risky operations
- **Hardware Team**: Verifies lscpu, lspci, lsusb output
- **Desktop Team**: Verifies DE environment variables

### Execution Traces (v0.0.23)
- **SpecialistOutcome**: Ok, Timeout, BudgetExceeded, Skipped, Error
- **FallbackUsed**: None, Deterministic, Timeout
- **ReviewerOutcome**: DeterministicAccept, DeterministicReject, JuniorOk, etc.
- **ProbeStats**: Planned, succeeded, failed, timed_out counts

### Reliability Scoring (v0.0.23)
- **Explicit Thresholds**: Named constants for all scoring rules
- **ProbeHealth**: AllSucceeded, SomeSucceeded, AllFailed, NoneNeeded, NotApplicable
- **Confidence Bands**: Low (<0.70), Medium (0.70-0.85), High (>0.85)

### Operations
- **Auto-update**: Checks for updates every 60 seconds
- **Self-healing**: Auto-repairs Ollama and model issues
- **Request Timeout**: Configurable global timeout (default 20s)
- **Latency Stats**: Per-stage avg and p95 tracking

### Recipe Learning Loop (v0.0.27)
- **RecipeKind**: Query, ConfigEnsureLine, ConfigEditLineAppend
- **RecipeTarget**: App identifier + config path template
- **RecipeAction**: EnsureLine, AppendLine, None
- **RollbackInfo**: Backup path, description, tested flag
- **Persistence**: ~/.anna/recipes/ with deterministic naming
- **Learning Trigger**: Only when Verified + score >= 80

### Safe Change Engine (v0.0.27)
- **ChangePlan**: Description, target, backup, operation, risk, is_noop
- **ChangeResult**: Applied, was_noop, backup_path, diagnostics
- **Operations**: EnsureLine (idempotent), AppendLine
- **Risk Levels**: Low, Medium, High
- **Backup Strategy**: Automatic before modification, deterministic naming
- **Rollback**: Full restore from backup

### Config Intent Detection (v0.0.27)
- **Vim Patterns**: syntax on, line numbers, autoindent, mouse, tabs
- **ConfigTarget**: vim, nano, bash with path templates
- **Integration**: Bridges query classification to change engine

### Statistics Command (v0.0.27)
- **Global Stats**: Total requests, success rate, avg reliability
- **Per-Team Stats**: Total, success, failed, avg rounds, avg score
- **CLI**: `annactl stats` command

### Team-Specialized Execution (v0.0.28)
- **Prompt Accessors**: SpecialistProfile.prompt() returns team-specific prompt
- **Registry Methods**: junior_prompt, senior_prompt, junior_model, senior_model
- **Escalation Threshold**: Configurable per-team escalation threshold

### Helpers Management (v0.0.28)
- **HelperPackage**: id, name, version, install_source, available, binary_path
- **InstallSource**: Anna, User, Bundled, Unknown
- **HelpersRegistry**: Track installed helpers with source attribution
- **Persistence**: ~/.anna/helpers.json
- **Known Helpers**: ollama (required)

### True Reset (v0.0.28)
- **Full State Wipe**: Clears ledger, recipes, helpers store
- **Enhanced Feedback**: Shows what will be cleared before confirmation
- **Fresh Install State**: Returns Anna to initial configuration

### IT Department Dialog (v0.0.28)
- **Context-Aware Greeting**: Based on query domain
- **Confidence Statements**: Reliability as IT confidence note
- **Domain Labels**: IT department style context names
- **Clean Output**: Polished non-debug user-facing format

### Service Desk Theatre Renderer (v0.0.63)
- **Narrative Flow**: Normal mode shows "Checking X..." before answers
- **Evidence Source**: Footer shows verification source when grounded
- **Clarification Options**: Numbered list display for multiple choices
- **No Probe Leak**: Raw probe output never leaks in normal mode
- **New Transcript Events**: EvidenceSummary, DeterministicPath, ProposedAction, ActionConfirmationRequest
- **Debug Enhancements**: Risk-colored actions, rollback indicators

### ConfigureEditor Grounding (v0.0.62)
- **Proper Probe Accounting**: Valid evidence count from ToolExists probes
- **Execution Trace**: All ConfigureEditor paths include accurate probe stats
- **Grounding Signals**: answer_grounded, probe_coverage based on valid evidence

### HardwareAudio Parser (v0.0.61)
- **Content-Based Detection**: Detects audio devices by output content, not command
- **pactl Detection**: Detects cards by "Card #" blocks in output
- **Evidence Merge**: Combines lspci + pactl evidence with deduplication

### Unified Versioning (v0.0.69)
- **Single Source**: Workspace Cargo.toml version as only source of truth
- **Compile-time Resolution**: `env!("CARGO_PKG_VERSION")` for VERSION constant
- **Version Tests**: Consistency tests in version_consistency.rs
- **Status Display**: Shows installed, available, check_pace, next_check

### REPL "Since Last Time" Summary (v0.0.69)
- **Snapshot Comparison**: Shows changes since last session
- **Delta Tracking**: Failed services, disk changes, memory changes
- **Clean Format**: `[warn]`, `[crit]`, `[fail]`, `[ok]` prefixes (no emojis)
- **Persistence**: Snapshots saved to ~/.anna/snapshots/last.json

### Version Unification (v0.0.70)
- **Single Source of Truth**: Workspace Cargo.toml version is authoritative
- **Compile-Time Resolution**: All binaries use env!("CARGO_PKG_VERSION")
- **Dynamic Installer**: install.sh fetches version from GitHub releases API
- **Status Output Contract**: Shows installed, available, last_check, next_check, auto_update
- **No Downgrade Guarantee**: Auto-update uses semantic version comparison
- **Version Tests**: Consistency tests validate all sources match

## Not Yet Implemented

### Future
- User confirmation dialog for system changes
- Multi-file change transactions
- Package installation recipes
- Service configuration recipes
- Recipe replay from persisted patterns
