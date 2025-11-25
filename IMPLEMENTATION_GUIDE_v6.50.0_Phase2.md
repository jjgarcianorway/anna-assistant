# v6.50.0 Phase 2 Implementation Guide

## Status: Phase 1 Complete ✓

**Phase 1 Foundation** (COMPLETED):
- ✓ Execution safety infrastructure (`execution_safety.rs`)
- ✓ Extended ActionEpisode with PostValidation and ExecutionStatus
- ✓ Updated episode storage with new schema
- ✓ Risk classification integrated into planner
- ✓ 23 tests written and passing
- ✓ All 700 tests pass
- ✓ Documentation updated (CHANGELOG, README)

**Phase 2 Integration** (IN PROGRESS):
This guide provides step-by-step instructions for completing the integration.

---

## Architecture Overview

```
User Query
    ↓
Planner (generates CommandPlan)
    ↓
compute_plan_summary(is_interactive) → PlanSummary
    ↓
[PHASE 2] Display plan + confirmation prompt
    ↓
[If user confirms "y"]
    ↓
[PHASE 2] Executor (execute commands, record ActionRecords)
    ↓
[PHASE 2] Store ActionEpisode in database
    ↓
[PHASE 2] Post-Validation LLM call → PostValidation
    ↓
[PHASE 2] Update episode with validation results
    ↓
Display results + satisfaction score to user
```

---

## Task 1: Confirmation Layer (REPL Mode)

**Location**: Likely in `crates/annactl/src/` (REPL/interactive handler)

**Current Behavior**: Plans are generated but not executed

**New Behavior**:
1. After planner generates CommandPlan
2. Compute PlanSummary: `let summary = plan.compute_plan_summary(true);`
3. Display to user:
```rust
use anna_common::execution_safety::ExecutionMode;

// Display plan summary
println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
println!("PLAN SUMMARY: {}", summary.description);
println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

println!("Risk Level: {}", summary.risk_description());
println!("Commands: {}", summary.command_count);
println!("Domains: {:?}", summary.domains);
if summary.will_create_backups {
    println!("✓ Backups will be created before changes");
}
println!();

// Show commands
println!("Commands to execute:");
for cmd in &plan.commands {
    let full_cmd = format!("{} {}", cmd.command, cmd.args.join(" "));
    println!("  {} - {}", full_cmd, cmd.purpose);
}
println!();

// Check execution mode
match summary.execution_mode {
    ExecutionMode::PlanOnly => {
        println!("⚠️  This plan is HIGH RISK and will not be executed automatically.");
        println!("   Review the plan above. If you need to proceed, execute manually.\n");
        return Ok(());  // Do not execute
    }
    ExecutionMode::ConfirmRequired => {
        // Prompt for confirmation
        print!("{} ", summary.confirmation_prompt());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes") {
            println!("\nCancelled. No changes were made.\n");
            return Ok(());
        }

        println!("\nExecuting plan...\n");
        // Proceed to execution
    }
    ExecutionMode::Automatic => {
        // Future: automatic execution
        println!("Executing automatically (future feature)...\n");
    }
}
```

---

## Task 2: Executor Integration

**Location**: `crates/anna_common/src/executor_core.rs`

**Current Behavior**: Commands are executed, output is captured

**New Behavior**: Record each command as ActionRecord

```rust
use anna_common::action_episodes::{ActionRecord, ActionKind, EpisodeBuilder};

// Before executing plan, create episode builder
let mut episode_builder = EpisodeBuilder::new(&user_question);

// For each command in plan
for (idx, planned_cmd) in plan.commands.iter().enumerate() {
    let full_command = format!("{} {}", planned_cmd.command, planned_cmd.args.join(" "));

    // Determine ActionKind
    let kind = if full_command.contains("pacman -S") || full_command.contains("yay -S") {
        ActionKind::InstallPackages
    } else if full_command.contains("pacman -R") || full_command.contains("yay -R") {
        ActionKind::RemovePackages
    } else if full_command.contains("systemctl enable") {
        ActionKind::EnableServices
    } else if full_command.contains("systemctl disable") {
        ActionKind::DisableServices
    } else if full_command.contains("systemctl start") {
        ActionKind::StartServices
    } else if full_command.contains("systemctl stop") {
        ActionKind::StopServices
    } else if planned_cmd.writes_files {
        // Determine if edit or create based on file existence
        ActionKind::EditFile  // or CreateFile
    } else {
        ActionKind::RunCommand
    };

    // Create backup if needed
    let mut backup_paths = Vec::new();
    if planned_cmd.writes_files && kind == ActionKind::EditFile {
        // TODO: Create backup before modifying
        // backup_paths.push(create_backup(&file_path)?);
    }

    // Execute command
    let result = execute_command(&planned_cmd.command, &planned_cmd.args)?;

    // Record action
    let action = ActionRecord {
        id: 0,  // Will be set by builder
        kind,
        command: full_command,
        cwd: std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
        files_touched: vec![],  // TODO: detect from command
        backup_paths,
        notes: Some(planned_cmd.purpose.clone()),
    };

    episode_builder.add_action(action);
}
```

---

## Task 3: Episode Storage

**Location**: After executor completes

```rust
use anna_common::episode_storage::EpisodeStorage;
use anna_common::action_episodes::{ExecutionStatus, infer_tags_from_plan_and_answer};

// After all commands execute
let tags = infer_tags_from_plan_and_answer(
    &user_question,
    &plan.reasoning,
    &interpreter_answer.summary
);

let mut episode = episode_builder
    .with_final_answer_summary(&interpreter_answer.summary)
    .with_tags(tags)
    .build();

// Update execution status
episode.execution_status = if all_commands_succeeded {
    ExecutionStatus::Executed
} else {
    ExecutionStatus::PartiallyExecuted
};

// Store episode
let db_path = /* path to episode database */;
let storage = EpisodeStorage::new(db_path)?;
let episode_id = storage.store_action_episode(&episode)?;

println!("✓ Episode {} recorded", episode_id);
```

---

## Task 4: Post-Validation LLM Call

**Location**: After episode is stored

**Purpose**: Query LLM to assess if the execution satisfied the user's request

```rust
use anna_common::llm_client::LlmClient;
use anna_common::action_episodes::PostValidation;

// Build validation prompt
let validation_prompt = format!(
    r#"You executed a plan to address the user's request. Assess the results.

USER REQUEST: {}

PLAN EXECUTED:
{}

COMMANDS RUN:
{}

OUTPUT:
{}

Respond with JSON:
{{
  "satisfaction_score": 0.95,  // 0.0-1.0, how well request was satisfied
  "summary": "Successfully configured vim with 4-space tabs. Config is active.",
  "residual_concerns": ["Minor: old backup file left in ~"],
  "suggested_checks": ["vim --version", "cat ~/.vimrc"]
}}
"#,
    user_question,
    plan.reasoning,
    commands_executed_text,
    execution_output
);

// Query LLM with JSON schema
let schema = serde_json::json!({
    "type": "object",
    "properties": {
        "satisfaction_score": {"type": "number", "minimum": 0.0, "maximum": 1.0},
        "summary": {"type": "string"},
        "residual_concerns": {"type": "array", "items": {"type": "string"}},
        "suggested_checks": {"type": "array", "items": {"type": "string"}, "maxItems": 3}
    },
    "required": ["satisfaction_score", "summary", "residual_concerns", "suggested_checks"]
});

let llm_response = llm_client.query_with_schema(&validation_prompt, &schema).await?;
let post_validation: PostValidation = serde_json::from_str(&llm_response)?;

// Update episode with validation
episode.post_validation = Some(post_validation.clone());
storage.store_action_episode(&episode)?;  // Update in database

// Display to user
println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
println!("POST-EXECUTION ASSESSMENT");
println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
println!("Satisfaction: {:.0}%", post_validation.satisfaction_score * 100.0);
println!("{}\n", post_validation.summary);

if !post_validation.residual_concerns.is_empty() {
    println!("⚠️  Concerns:");
    for concern in &post_validation.residual_concerns {
        println!("   • {}", concern);
    }
    println!();
}

if !post_validation.suggested_checks.is_empty() {
    println!("Suggested checks:");
    for check in &post_validation.suggested_checks {
        println!("   $ {}", check);
    }
    println!();
}
```

---

## Task 5: Rollback Execution

**Location**: New command handler or extension to existing rollback query

**Integration**: Use existing episode search (v6.49.0) + rollback_engine

```rust
use anna_common::rollback_engine::generate_rollback_plan;

// User requests rollback: "undo vim changes" or "rollback episode 42"
let episodes = if let Some(episode_id) = specific_episode_id {
    vec![storage.load_action_episode(episode_id)?.unwrap()]
} else {
    // Search by topic
    storage.list_action_episodes_by_topic("vim", 5)?
};

// Let user select episode
let selected = select_episode_interactive(&episodes)?;

// Generate rollback plan
let rollback_plan = generate_rollback_plan(&selected)?;

if rollback_plan.rollback_actions.is_empty() {
    println!("⚠️  This episode cannot be rolled back automatically.");
    return Ok(());
}

println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
println!("ROLLBACK PLAN");
println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
println!("Will undo: {}", selected.user_question);
println!("Summary: {}\n", rollback_plan.summary);

println!("Rollback commands:");
for action in &rollback_plan.rollback_actions {
    println!("  {} - {}", action.command, action.reason);
}
println!();

// Convert rollback plan to CommandPlan
let command_plan = rollback_plan_to_command_plan(&rollback_plan);

// Compute plan summary (reuse confirmation flow!)
let summary = command_plan.compute_plan_summary(true);

// Show confirmation prompt (same as Task 1)
// ... confirmation logic ...

// If confirmed, execute rollback
// ... execution logic (same as Task 2) ...

// Create new episode for rollback
let mut rollback_episode = episode_builder.build();
rollback_episode.rolled_back_episode_id = Some(selected.episode_id);
rollback_episode.execution_status = ExecutionStatus::Executed;

storage.store_action_episode(&rollback_episode)?;

// Mark original episode as rolled back
selected.execution_status = ExecutionStatus::RolledBack;
storage.store_action_episode(&selected)?;

println!("✓ Rollback complete. Original episode {} marked as rolled back.", selected.episode_id);
```

---

## Task 6: Safety Constraints

**Already implemented in Phase 1!**

The safety constraints are enforced via:
1. `classify_command_risk()` - Pattern-based risk detection
2. `determine_execution_mode()` - Context-aware execution decisions
3. High-risk plans return `ExecutionMode::PlanOnly` (will not execute)

Additional constraints to consider:
- Never execute commands containing `rm -rf /`
- Never execute without backups for file modifications
- Never skip confirmation even for "Safe" commands in one-shot mode

---

## Testing Strategy

**Unit Tests** (add to existing test modules):
1. Confirmation flow with various risk levels
2. ActionRecord creation from PlannedCommand
3. PostValidation JSON schema parsing
4. Rollback episode linking

**Integration Tests** (add new test file):
1. Full flow: plan → confirm → execute → validate → store
2. High-risk rejection flow
3. User cancellation flow
4. Rollback execution flow
5. Episode linking validation

**Manual Testing**:
1. Test in REPL mode: "make vim use 4 spaces"
2. Verify confirmation prompt appears
3. Confirm and verify execution
4. Check episode is stored
5. Check post-validation appears
6. Test rollback: "undo vim changes"
7. Verify rollback episode links to original

---

## File Locations

**Files to modify**:
- `crates/annactl/src/unified_query_handler.rs` or similar (confirmation layer)
- `crates/anna_common/src/executor_core.rs` (action recording)
- `crates/anna_common/src/interpreter_core.rs` (post-validation)

**Files already ready**:
- ✓ `crates/anna_common/src/execution_safety.rs`
- ✓ `crates/anna_common/src/action_episodes.rs`
- ✓ `crates/anna_common/src/episode_storage.rs`
- ✓ `crates/anna_common/src/planner_core.rs`
- ✓ `crates/anna_common/src/rollback_engine.rs`

---

## Success Criteria

Phase 2 is complete when:
- [ ] User can confirm plans in REPL mode
- [ ] High-risk plans show "plan-only" message
- [ ] Executed commands are recorded as ActionEpisodes
- [ ] Post-validation runs and displays results
- [ ] Rollback commands can be executed with confirmation
- [ ] All new integration tests pass
- [ ] Manual testing scenarios complete successfully

---

## Notes

- Keep the implementation generic (no per-question hardcoding)
- Use existing infrastructure (planner, executor, interpreter, rollback_engine)
- Conservative approach: when in doubt, require confirmation
- Prioritize safety over convenience
- Full audit trail: every execution must create an episode

---

## Phase 1 Summary

**What's Already Done**:
- ✅ Risk classification infrastructure (8 tests)
- ✅ Episode tracking with validation fields (11 tests)
- ✅ Plan summary computation (4 tests)
- ✅ Database schema updates
- ✅ All foundations tested and documented

**What's Left**:
- Integration work (wiring components together)
- UI/UX implementation (confirmation prompts)
- Testing of integrated flows

The hard part (architecture and data models) is complete. The remaining work is primarily connecting the pieces together.
