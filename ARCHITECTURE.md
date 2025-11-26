# Anna Assistant Architecture v6.57.0

## Core Philosophy

Anna is a conversational sysadmin assistant that understands your questions, examines your system using real telemetry, plans appropriate commands, executes them safely, and explains the results with visible "thinking traces".

**Key Principle**: One unified pipeline, not a collection of special-case handlers.

## v6.57.0: Single Pipeline Architecture

**CRITICAL CHANGE**: This release eliminates ALL legacy handlers, hardcoded recipes, and shortcut paths. Every query flows through ONE unified pipeline:

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Question                            │
└────────────────────────────┬────────────────────────────────────┘
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

## Removed in v6.57.0

The following legacy systems were **deleted** (scorched earth cleanup):

### Files Deleted
- `crates/annactl/src/legacy_recipes/` - 82 hardcoded recipe files
- `crates/annactl/src/deterministic_answers.rs` - 841 lines of bypass logic
- `crates/annactl/src/sysadmin_answers.rs` - 2464 lines of hardcoded templates
- `crates/annactl/src/query_handler.rs` - Legacy recipe-based handler
- `crates/annactl/src/recipe_formatter.rs` - Legacy recipe formatting
- `crates/annactl/src/plan_command.rs` - Legacy orchestrator-based planning
- `crates/annactl/src/selftest_command.rs` - Depended on orchestrator
- `crates/annactl/src/json_types.rs` - Depended on caretaker_brain
- `crates/anna_common/src/command_recipe.rs` - Recipe system
- `crates/anna_common/src/recipe_executor.rs` - Recipe execution
- `crates/anna_common/src/recipe_planner.rs` - Recipe planning
- `crates/anna_common/src/recipe_validator.rs` - Recipe validation
- `crates/anna_common/src/template_library.rs` - 2589 lines of templates
- `crates/anna_common/src/orchestrator/` - Legacy planner directory
- `crates/anna_common/src/caretaker_brain.rs` - Legacy brain rules
- `crates/anna_common/src/wiki_answer_engine.rs` - Wiki-based shortcuts
- `crates/anna_common/src/answer_formatter.rs` - Legacy formatting
- `crates/anna_common/src/executor.rs` - Legacy executor
- `crates/anna_common/src/selftest.rs` - Legacy selftest
- `crates/anna_common/src/context/noise_control.rs` - Legacy context
- `crates/annad/src/intel/sysadmin_brain.rs` - Hardcoded diagnostic rules

### Why Deleted?
- **Hardcoded answers**: Pattern-matching handlers that bypassed the LLM
- **Recipe system**: Static templates that couldn't adapt to system state
- **Legacy brains**: Rule-based analyzers that duplicated planner logic
- **Shortcut paths**: Any code that avoided the unified pipeline

## Core Modules (v6.57.0)

### 1. Planner Core (`anna_common/src/planner_core.rs`)

**Responsibility**: Interpret user intent and generate system-specific command plans.

**Key Types**:
```rust
pub struct Intent {
    pub goal: GoalType,         // Inspect, Diagnose, List, Check
    pub domain: DomainType,     // Packages, Hardware, GUI, etc.
    pub constraints: Vec<Constraint>,
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

### 2. Executor Core (`anna_common/src/executor_core.rs`)

**Responsibility**: Execute command plans safely with tool validation.

**Safety Features**:
- Tool inventory detection
- Command safety validation
- Fallback execution
- Timeout protection
- Output capture

### 3. Interpreter Core (`anna_common/src/interpreter_core.rs`)

**Responsibility**: Analyze command outputs and generate answers.

**Features**:
- LLM-driven result interpretation
- Fallback interpreters for offline scenarios
- Honest error handling

### 4. Trace Renderer (`anna_common/src/trace_renderer.rs`)

**Responsibility**: Generate visible "thinking traces" showing Anna's process.

### 5. Query Handlers (Simplified in v6.57.0)

- `planner_query_handler.rs` - THE unified pipeline entry point
- `llm_query_handler.rs` - CLI one-shot queries (delegates to planner)
- `unified_query_handler.rs` - Backwards compatibility (delegates to planner)

## Design Principles

### 1. **Single Pipeline**
All queries flow through: `Planner → Executor → Interpreter → Trace`

NO exceptions. NO shortcuts. NO special-case handlers.

### 2. **LLM-Driven**
The LLM plans commands based on real telemetry. No static templates.

### 3. **Telemetry-First**
All planning uses real system state from daemon telemetry.

### 4. **Safety First**
- Read-only commands execute automatically
- Risky commands require explicit approval
- Safety validation before execution

### 5. **Visible Thinking**
Users see: intent, commands, outputs, reasoning.

### 6. **Honest Errors**
No hallucination - all data from real commands.

## Key Files

```
crates/
├── anna_common/
│   └── src/
│       ├── planner_core.rs        # Intent + command planning
│       ├── executor_core.rs       # Safe execution
│       ├── interpreter_core.rs    # Result interpretation
│       ├── trace_renderer.rs      # Thinking traces
│       ├── de_wm_detector.rs      # DE/WM detection
│       └── system_info.rs         # System telemetry
└── annactl/
    └── src/
        ├── planner_query_handler.rs   # THE unified pipeline
        ├── llm_query_handler.rs       # CLI one-shot (simplified)
        └── unified_query_handler.rs   # Compatibility layer (simplified)
```

## Version History

**v6.57.0** (Current):
- SCORCHED EARTH: Deleted all legacy handlers and recipes
- Single pipeline enforcement: ALL queries → planner_core
- Removed ~15,000+ lines of legacy code
- Simplified llm_query_handler.rs from 1281 to 110 lines
- Simplified unified_query_handler.rs from 3481 to 120 lines

**v6.54.0-v6.56.0**:
- Config-driven LLM model selection
- Identity, persistence, multi-user awareness
- User policies and guardrails

**v6.41.0-v6.53.0**:
- Implemented Planner → Executor → Interpreter core
- Added visible thinking traces
- Episodic action log and semantic rollback foundation
- See CHANGELOG.md for detailed history
