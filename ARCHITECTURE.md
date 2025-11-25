# Anna Assistant Architecture v6.41.0

## Core Philosophy

Anna is a conversational sysadmin assistant that understands your questions, examines your system using real telemetry, plans appropriate commands, executes them safely, and explains the results with visible "thinking traces".

**Key Principle**: One unified brain, not a collection of special-case handlers.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Question                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
                    ┌────────────────┐
                    │ Intent Router  │  Classify: Inspect/Diagnose/List/Check
                    └────────┬───────┘  Domain: Packages/Hardware/GUI/etc
                             │
                             ▼
                  ┌─────────────────────┐
                  │  Planner Core       │
                  │  (LLM-backed)       │
                  │  - Reads telemetry  │
                  │  - Detects tools    │
                  │  - Plans commands   │
                  └──────────┬──────────┘
                             │
                             ▼
              ┌──────────────────────────┐
              │  Executor Core           │
              │  - Safety checks         │
              │  - Tool validation       │
              │  - Command execution     │
              │  - Result capture        │
              └────────────┬─────────────┘
                           │
                           ▼
            ┌─────────────────────────────┐
            │  Interpreter Core           │
            │  (LLM-backed)               │
            │  - Analyzes outputs         │
            │  - Validates success        │
            │  - Computes confidence      │
            └────────────┬────────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │  Trace Renderer              │
          │  - Shows thinking process    │
          │  - Commands + outputs        │
          │  - Reasoning summary         │
          └────────────┬─────────────────┘
                       │
                       ▼
              ┌────────────────┐
              │  Final Answer  │
              │  + Source      │
              └────────────────┘
```

## Core Modules (v6.41.0)

### 1. Planner Core (`anna_common/src/planner_core.rs`)

**Responsibility**: Interpret user intent and generate system-specific command plans.

**Key Types**:
```rust
pub struct Intent {
    pub goal: GoalType,        // Inspect, Diagnose, List, Check
    pub domain: DomainType,     // Packages, Hardware, GUI, etc.
    pub constraints: Vec<Constraint>,  // Path, Count, Feature, Category
    pub query: String,
}

pub struct CommandPlan {
    pub commands: Vec<PlannedCommand>,
    pub safety_level: SafetyLevel,  // ReadOnly, MinimalWrite, Risky
    pub fallbacks: Vec<PlannedCommand>,
    pub expected_output: String,
    pub reasoning: String,
}
```

**Features**:
- Pattern-based intent classification (no hardcoded question matching)
- Constraint extraction (paths, counts, features, categories)
- LLM prompt generation for dynamic planning
- Fallback planning for offline/no-LLM scenarios

**Example**: "do I have games installed?"
- Intent: `{goal: Inspect, domain: Packages, constraints: [Category("games")]}`
- Plan: `pacman -Qq | grep -Ei '(steam|game|lutris|heroic|wine|proton)'`

### 2. Executor Core (`anna_common/src/executor_core.rs`)

**Responsibility**: Execute command plans safely with tool validation.

**Key Types**:
```rust
pub struct ExecutionResult {
    pub plan: CommandPlan,
    pub command_results: Vec<CommandResult>,
    pub success: bool,
    pub execution_time_ms: u64,
}

pub struct CommandResult {
    pub command: String,
    pub full_command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub time_ms: u64,
}
```

**Safety Features**:
- Tool inventory detection (checks what's actually installed)
- Command safety validation (blocks rm, dd, package removal)
- Fallback execution when primary tools missing
- Timeout protection
- Output capture (stdout/stderr/exit codes)

**Tool Inventory**:
- Package managers: pacman, yay, paru, apt, dnf, flatpak, snap
- System tools: grep, awk, sed, du, df, find, ps, systemctl, lscpu, lspci

### 3. Interpreter Core (`anna_common/src/interpreter_core.rs`)

**Responsibility**: Analyze command outputs and generate answers.

**Key Types**:
```rust
pub struct InterpretedAnswer {
    pub answer: String,
    pub details: Option<String>,
    pub confidence: ConfidenceLevel,  // High, Medium, Low
    pub reasoning: String,
    pub source: String,
}
```

**Features**:
- LLM prompt generation for result interpretation
- Fallback interpreters (domain-specific parsers):
  - Package results: filters and counts packages
  - Hardware results: parses CPU flags, groups SSE/AVX features
  - GUI results: uses de_wm_detector for accurate DE/WM detection
- Honest error handling (explains what went wrong)

### 4. Trace Renderer (`anna_common/src/trace_renderer.rs`)

**Responsibility**: Generate visible "thinking traces" showing Anna's process.

**Example Trace**:
```
🧠 Anna thinking trace

Intent:
  - Goal: Inspect
  - Domain: Packages
  - Constraints: Category("games")

Commands executed:
  [CMD] sh -c pacman -Qq | grep -Ei '(steam|game...)' ✓

Key outputs:
  sh: gamemode, steam, wine-staging

Reasoning (LLM summary):
  Fallback interpretation without LLM

Execution time: 27ms
```

**Features**:
- Compact format for non-TTY mode
- TTY detection for automatic display
- Structured output (not raw LLM tokens)
- Execution timing

## Integration Points

### Query Handler (`annactl/src/planner_query_handler.rs`)

**Flow**:
1. Check if query matches pilot patterns
2. Get telemetry from daemon
3. Interpret intent → Plan commands → Execute → Interpret results → Render trace
4. Return formatted output

**Pilot Queries (v6.41.0)**:
- "do I have games installed?"
- "what DE/WM am I running?"
- "does my CPU have SSE/AVX?"
- "do I have any file manager installed?"

### Fallback Chain

```
User Query
    ↓
Planner Query Handler? (pilot queries)
    ↓ (if not matched)
Deterministic Answers? (system facts)
    ↓ (if not matched)
Intent Router (personality, status, etc.)
    ↓ (if not matched)
Full LLM Query (generic questions)
```

## Design Principles

### 1. **No Special-Case Logic**
Bad: `if query.contains("games") { check_for_games() }`
Good: `Intent → Plan → Execute → Interpret` (handles all queries generically)

### 2. **Telemetry-Driven**
- All planning uses real system state from Historian
- Tool inventory detects what's actually available
- DE/WM detector for accurate environment detection

### 3. **Safety First**
- Read-only commands execute automatically
- Risky commands require explicit approval
- Safety validation before execution

### 4. **Visible Thinking**
- Always show the thinking trace
- Users see: intent, commands, outputs, reasoning
- Builds trust and understanding

### 5. **Honest Errors**
- No hallucination - all data from real commands
- Clear explanations when things fail
- Suggest fixes or alternatives

## Module Sizes (v6.41.0)

- `planner_core.rs`: 382 lines ✓
- `executor_core.rs`: 244 lines ✓
- `interpreter_core.rs`: 371 lines ✓
- `trace_renderer.rs`: 147 lines ✓
- `planner_query_handler.rs`: 215 lines ✓

**Total new code**: ~1,359 lines across 5 focused modules

## Testing Strategy

### Unit Tests
- Intent interpretation (planner_core)
- Command safety validation (executor_core)
- Result parsing (interpreter_core)
- Trace rendering (trace_renderer)

### Integration Tests
- End-to-end pilot queries
- Fallback scenarios (missing tools)
- Error handling paths

### Manual Testing
All pilot queries verified working:
- ✓ Games detection
- ✓ DE/WM detection (using de_wm_detector)
- ✓ CPU features (SSE/AVX parsing)
- ✓ File manager detection

## Future Roadmap

### v6.42.0 - LLM Integration
- Real LLM-driven planning (not just fallback)
- Dynamic command generation based on telemetry
- Adaptive strategies per system configuration

### v6.43.0 - Interactive REPL
- Conversational shell mode
- Session context for follow-up questions
- "and free?" type queries

### v6.44.0 - Action Plans
- Commands with approval flow
- Rollback support for changes
- Safety guardrails for system modifications

### v6.45.0+ - Full Migration
- Migrate remaining special-case handlers to core
- Deprecate old intent routing
- Unified architecture for all queries

## Dependencies

**Core**:
- `serde` / `serde_json`: Data serialization
- `anyhow`: Error handling
- `atty`: TTY detection for trace rendering

**Execution**:
- `std::process::Command`: Shell command execution
- Tool detection via `which`

**Detection**:
- `de_wm_detector`: Desktop environment/window manager detection
- `system_info`: Deterministic system information (CPU, RAM, GPU, disk)

## Performance

**Typical Query Timing** (from traces):
- Intent interpretation: <1ms
- Command planning: <5ms (fallback), variable (LLM)
- Command execution: 10-50ms (depends on command)
- Result interpretation: <5ms (fallback), variable (LLM)
- Trace rendering: <1ms

**Total**: 20-100ms for pilot queries with fallback planning

## Key Files

```
crates/
├── anna_common/
│   └── src/
│       ├── planner_core.rs       # Intent + command planning
│       ├── executor_core.rs       # Safe execution
│       ├── interpreter_core.rs    # Result interpretation
│       ├── trace_renderer.rs      # Thinking traces
│       ├── de_wm_detector.rs      # DE/WM detection
│       └── system_info.rs         # System telemetry
└── annactl/
    └── src/
        ├── planner_query_handler.rs  # Integration layer
        ├── llm_query_handler.rs      # Main query dispatcher
        └── deterministic_answers.rs  # Fallback for system facts
```

## Version History

**v6.41.0** (Current):
- Implemented Planner → Executor → Interpreter core
- Added visible thinking traces
- Pilot queries working (games, DE/WM, CPU features, file managers)
- All tests passing (169 total)

**v6.40.0**:
- DE/WM detector with multi-layer detection
- System report v2 (deterministic, no LLM)

**v6.39.0 and earlier**:
- See CHANGELOG.md for detailed history
