# Anna Specification

This document is law. Every statement is testable. CI enforces it.

## What Anna Is

Anna is a local Linux assistant that answers system questions using real data.

When you ask "how much disk space do I have?", Anna:
1. Runs diagnostic commands (e.g., `df -h`)
2. Parses the output
3. Synthesizes a natural language answer grounded in that output

Anna does not guess. Anna does not hallucinate. If Anna cannot determine the answer from probe output, Anna says so.

## What Anna Is Not

- A general-purpose chatbot
- A web search engine
- A code generator
- A remote management tool
- A security scanner

## Problems Anna Solves

1. "What is the state of my system?" - Disk, RAM, CPU, services, packages
2. "Why is X happening?" - Diagnostics via targeted probes
3. "How do I do Y?" - Procedural guidance from Arch Wiki, man pages, --help
4. "Fix X for me" - Configuration changes with backup and rollback

## Problems Anna Refuses

- Questions requiring internet search
- Questions about other machines
- Questions outside Linux system administration
- Requests requiring speculation or prediction

## Guarantees

| Guarantee | Enforcement |
|-----------|-------------|
| Every factual claim is backed by probe output or trusted docs | ClaimGate verification |
| No answer contains invented data | Grounding check before emit |
| User can see what commands were run | Evidence line in output |
| Configuration changes are backed up | Backup created before modification |
| All state in /etc/anna, /var/lib/anna, /run/anna | Acceptance gate |
| Daemon updates itself from GitHub releases | Auto-update mechanism |
| Uninstaller removes all Anna artifacts | Uninstall verification |

## Failure Modes

| Failure | Behavior |
|---------|----------|
| Probe command fails | Report failure, do not guess |
| LLM returns ungrounded claim | Block or mark as unverified |
| Cannot determine answer | Say "I don't know" explicitly |
| Ollama unavailable | Report in status, degrade gracefully |
| Update check fails | Continue running, log failure |

### Extended Failure Mode Contracts (Phase 3)

| Failure | Detection | Required Behavior | Forbidden |
|---------|-----------|-------------------|-----------|
| Conflicting probes | Probes disagree on same state | Set `conflicts_detected=true`, BLOCK single-truth assertion | Asserting one truth when probes conflict |
| Socket EACCES | Client receives permission denied | Print message containing "permission" and suggest anna group | Generic "cannot connect" error |
| Empty probe output (success) | Exit code 0 but stdout empty | Set `output_empty=true`, mark in evidence as `[empty]` | Factual assertion from empty output |
| Backup failure | `backup_file()` returns Err | Abort change, propagate error | Proceeding to write after backup fails |

Each row is testable via the field or output pattern specified.

## Exposure Model (v0.3.45)

Anna enforces strict information boundaries through exposure levels.

### Mental Model Contract

**What Anna IS:**
- A software tool that executes commands and processes text
- A local assistant running on the user's machine
- A deterministic system following programmed rules

**What Anna is NOT:**
- Not conscious, aware, or sentient
- Not an entity with desires, feelings, or intentions
- Not an authority figure or decision-maker

**What "Internal Dialogue" represents:**
- Processing stages shown in human-readable format
- Routing decisions displayed as conversation for clarity
- Debug information formatted for readability
- NOT actual communication between conscious entities

### Exposure Levels

| Level    | Dialogue | Metadata | Timing | Debug |
|----------|----------|----------|--------|-------|
| Silent   | No       | No       | No     | No    |
| Summary  | No       | Summary  | No     | No    |
| Dialogue | Yes      | Summary  | Yes    | No    |
| Debug    | Yes      | Full     | Yes    | Yes   |

Levels are strictly ordered: Silent < Summary < Dialogue < Debug.
No implicit escalation. No partial overlap.

### Consent Requirements

- Internal dialogue never appears by surprise
- First enablement requires explicit acknowledgement
- No alerts or error-style language during enablement
- Consent state persists across sessions

### Replay Redaction

- Replays obey the exposure level at record time
- Cannot elevate above recorded level via replay
- Debug information only visible if recorded at Debug level

### Forbidden Dialogue Patterns

The sanitization layer rejects:
- Urgency language (critical, urgent, immediately)
- Authority language (must, required, mandatory)
- Consciousness attribution (thinks, decides, wants)
- Alarm language (danger, warning!, panic)

This is testable: any user-visible dialogue containing forbidden patterns fails validation.

### Specialist Execution and Exposure Guarantees (v0.3.46)

All specialist activity flows through the ExposureGate before user display.

**Central Filtering Contract:**
1. ExposureGate is the ONLY point where dialogue visibility is decided
2. No specialist may bypass exposure filtering
3. No conditional logic inside specialists themselves
4. Ralph loop dialogue obeys the same filtering rules

**Dialogue Classification:**
| Classification | Min Level | Purpose |
|---------------|-----------|---------|
| Informational | Summary   | Status updates, completion notifications |
| Procedural    | Dialogue  | Step-by-step actions, assignments |
| Diagnostic    | Debug     | Raw output, prompts, technical details |

**Classification by StepType:**
- Always shown: FinalAnswer, UserQuestion, ClarificationQuestion, SystemAlert
- Informational: TicketCreated, InvestigationComplete, IntentResult, WikiResults
- Procedural: TeamAssignment, SpecialistWorking, InvestigationProbe, CommandExec
- Diagnostic: CommandOutput, InvestigationResult, ValidationPrompt

**Invariants:**
1. At Silent level, zero dialogue lines emitted
2. At Summary level, only Informational classification emitted
3. At Dialogue level, Informational and Procedural emitted
4. At Debug level, all classifications emitted
5. Forbidden patterns blocked at all levels

This is testable: run the same query at each exposure level and verify dialogue count matches classification rules.

## System Paths

| Purpose | Path |
|---------|------|
| Config | /etc/anna/ |
| State | /var/lib/anna/ |
| Runtime | /run/anna/ |
| Socket | /run/anna/anna.sock |

No user home directory writes. No exceptions.

## Permissions

| Type | Mode |
|------|------|
| Directories | 750 |
| Files | 640 |
| Socket | 660 |

## Unverified Claim Definition

A claim is **unverified** if any of the following apply:

1. The claim asserts system state (service running, file exists, port open) but no probe was run to verify it
2. The claim asserts system state but the relevant probe failed (exit code != 0)
3. The claim asserts system state but probe output contradicts the claim
4. The claim requires documentation (how-to, syntax, behavior) but no trusted doc was cited

**Unverified claims must be BLOCKED, not emitted.** The response must either:
- Omit the unverified factual sentence entirely, OR
- Replace it with an explicit uncertainty statement: "I cannot verify [X] because [reason]"

This is testable: any response containing a factual pattern (see ClaimGate) without corresponding evidence in the Evidence line is a test failure.

## Hard Invariants

These are testable. CI must enforce them.

1. Anna must never emit a factual claim without evidence
2. Anna must never write to user home directories
3. Anna must never require manual commands for install/update/uninstall
4. All state must reside in system paths only
5. Socket permissions must be 0660 (root:anna)
6. Directory permissions must be 750
7. Auto-update must succeed when new release exists
8. Uninstall must remove all Anna-created files
9. Status command must report daemon health, version, update state
10. Probe failures must not produce invented answers
11. Configuration changes must create backups
12. Deployment must be via auto-update or installer script only
13. Build must succeed with zero errors
14. All acceptance gates must pass before release

## Production Hardening Gates (v0.3.46)

E2E smoke tests validate the complete trust chain before release.

**Pre-flight Gate Tests** (`crates/annad/tests/e2e_exposure_smoke.rs`):

| Test | Purpose |
|------|---------|
| `test_silent_level_blocks_all_dialogue` | Silent emits zero dialogue |
| `test_summary_level_allows_only_informational` | Summary emits only status updates |
| `test_dialogue_level_allows_procedural` | Dialogue emits step-by-step actions |
| `test_debug_level_allows_all` | Debug emits all classifications |
| `test_forbidden_patterns_blocked_at_all_levels` | Sanitization enforced everywhere |
| `test_replay_cannot_elevate_exposure` | Cannot see more than recorded |
| `test_replay_silent_produces_no_dialogue` | Silent replay = zero lines |
| `test_error_recovery_no_forbidden_commands` | No "sudo" in recovery messages |
| `test_exposure_gate_performance_budget` | Filter < 1ms per line |

**Ollama Skip Detection:**
- Tests check `http://127.0.0.1:11434/api/tags` with 2s timeout
- If unavailable: skip with `SKIPPED: Ollama not available`
- Not a test failure - graceful degradation

**Performance Budget:**
- ExposureGate.filter() must complete in < 1ms per call
- Justification: 50 lines * 1ms = 50ms max overhead (acceptable)
- Actual: ~5-10us per call on modern hardware

**Running the Gate:**
```bash
cargo test --package annad --test e2e_exposure_smoke
# With Ollama tests:
cargo test --package annad --test e2e_exposure_smoke -- --ignored
```

## Definition of Done

Anna is "done" when:
1. All guarantees are enforceable via automated tests
2. No manual intervention required for install, update, or uninstall
3. Zero home directory writes
4. Zero invented facts in production output
