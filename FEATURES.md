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

## Not Yet Implemented

### Future
- User confirmation dialog for system changes
- Multi-file change transactions
- Package installation recipes
- Service configuration recipes
- Recipe replay from persisted patterns
