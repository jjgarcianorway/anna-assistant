# Feature Inventory - 6.x Series

This document tracks the codebase cleanup for the 6.x series.

## Planning Systems

### ✅ KEEP - Active (6.3.0+)

**Orchestrator Module** (`crates/anna_common/src/orchestrator/`)
- `telemetry.rs` - TelemetrySummary schema for planner
- `knowledge.rs` - Arch Wiki consultation (DNS, service failure)
- `planner.rs` - Plan generation with safety guarantees
- Status: **ACTIVE** - This is the only planning path

**Plan CLI** (`crates/annactl/src/plan_command.rs`)
- Status: **ACTIVE** - User-facing entry point for planner
- Commands: `annactl plan`, `annactl plan --json`

**ACTS v1** (`crates/anna_common/tests/acts_v1.rs`)
- Status: **ACTIVE** - Enforces planner safety guarantees
- Tests: DNS fix, service failure, network-down safety

### ❌ DELETE - Legacy Recipe System

**Recipe Library** (`crates/annactl/src/recipes/`)
- `mod.rs` - Recipe registry (77+ hardcoded recipes)
- `network.rs` - Network troubleshooting recipes
- `services.rs` - Service management recipes
- `disk.rs` - Disk space recipes
- `packages.rs` - Package management recipes
- `wallpaper.rs` - Desktop customization recipes
- `bluetooth.rs` - Bluetooth recipes
- `sound.rs` - Audio recipes
- ... (all recipe files)
- **Reason**: Replaced by adaptive planner with Arch Wiki consultation
- **Action**: Mark as deprecated, remove from active imports

**Recipe Formatter** (`crates/annactl/src/recipe_formatter.rs`)
- Status: **DELETE**
- **Reason**: Only used by legacy recipe system

**Intent Router** (`crates/annactl/src/intent_router.rs`)
- Status: **DELETE** or **QUARANTINE**
- **Reason**: Routes to recipe library, which is being replaced
- **Action**: Remove recipe routing logic, keep only if used for other purposes

**Brain Command** (`crates/annactl/src/brain_command.rs`)
- Status: **REVIEW**
- **Reason**: May reference old recipe/brain architecture
- **Action**: Verify it doesn't depend on recipes, keep if standalone

### 🔍 REVIEW - Potential Dependencies

**Sysadmin Answers** (`crates/annactl/src/sysadmin_answers.rs`)
- Status: **REVIEW**
- **Reason**: May use recipes for remediation suggestions
- **Action**: Verify independence from recipe system

**Context Engine** (`crates/annactl/src/context_engine.rs`)
- Status: **REVIEW**
- **Reason**: May reference recipes for proactive suggestions
- **Action**: Migrate to orchestrator if needed

**Net Diagnostics** (`crates/annactl/src/net_diagnostics.rs`)
- Status: **REVIEW**
- **Reason**: Network diagnostic engine - may overlap with DNS planner
- **Action**: Verify it doesn't duplicate planner logic

## IPC Types

### ✅ KEEP - Active

**SuggestedFixData** (`crates/anna_common/src/ipc.rs`)
- Status: **ACTIVE**
- Used by: Orchestrator planner for IPC serialization
- Fields: description, steps, knowledge_sources

**ProactiveIssueSummaryData** (`crates/anna_common/src/ipc.rs`)
- Status: **ACTIVE**
- Contains: optional `suggested_fix` field (6.2.0+)

### ❌ DELETE - Legacy Brain Types

**BrainAnalysisData** (`crates/anna_common/src/ipc.rs`)
- Status: **REVIEW**
- **Reason**: Old "brain" diagnostic format
- **Action**: Verify if still used by status command, remove if obsolete

**RecipeData** (`crates/anna_common/src/ipc.rs`)
- Status: **DELETE**
- **Reason**: Legacy recipe serialization format
- **Action**: Remove after recipe system cleanup

## Testing

### ✅ KEEP - Active Tests

**ACTS v1** (`crates/anna_common/tests/acts_v1.rs`)
- 4 integration tests enforcing planner safety

**Plan CLI Tests** (`crates/annactl/tests/plan_cli_test.rs`)
- 7 tests proving CLI is thin wrapper over planner

**Orchestrator Unit Tests** (`crates/anna_common/src/orchestrator/*.rs`)
- All tests in telemetry.rs, knowledge.rs, planner.rs modules

### ❌ DELETE - Legacy Recipe Tests

Any test files in `crates/annactl/tests/` that test recipe functionality should be:
1. Reviewed for coverage of real scenarios
2. Converted to ACTS-style planner tests if valuable
3. Removed if redundant

## Action Plan

### Phase 1: Mark for Deletion (6.3.0)
- [x] Create this inventory
- [ ] Add deprecation warnings to recipe modules
- [ ] Update imports to prefer orchestrator over recipes

### Phase 2: Quarantine (6.3.1)
- [ ] Move `recipes/` directory to `legacy_recipes/`
- [ ] Move `recipe_formatter.rs` to `legacy_recipes/`
- [ ] Remove recipe imports from active codepaths

### Phase 3: Delete (6.4.0)
- [ ] Remove legacy_recipes directory entirely
- [ ] Remove RecipeData IPC types
- [ ] Clean up any remaining recipe references

## Guiding Principle

**6.x Rule**: One planning path only
- ✅ Telemetry → Wiki → Planner → Plan
- ❌ Intent → Recipe → ActionPlan

All planning must go through the orchestrator module with Arch Wiki sources.
