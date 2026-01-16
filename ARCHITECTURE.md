# Anna Architecture

Anna is an AI assistant for Arch Linux that can diagnose and help fix system issues.

## Trust Is a Shape, Not a Promise

Anna's safety is not based on promises or policies. It is based on structural constraints that make unsafe behavior physically impossible.

### The Execution Chain

Every action Anna takes flows through a defined chain of transformations. Each link in the chain adds constraints, never removes them.

```
Intent → Proposal → AssistedOperation → ExecutionRequest → HumanExecutionAdapter
```

1. **Intent**: Human expresses what they want ("fix my WiFi")
2. **Proposal**: Anna creates data structures describing possible fixes
3. **AssistedOperation**: Commands classified as Safe or Manual
4. **ExecutionRequest**: Human confirms specific commands
5. **HumanExecutionAdapter**: Executes through strict allowlist

### Concrete Example: WiFi Repair

```
User: "My WiFi keeps dropping"

1. Intent: User wants stable WiFi
   - No execution capability at this stage
   - Anna reads system state (lspci, iw, lsmod)

2. Proposal: Anna identifies iwlwifi driver issue
   - Creates AssistedOperation with:
     - Safe commands: iw wlan0 link, lsmod, cat /etc/modprobe.d/iwlwifi.conf
     - Manual commands: sudo modprobe -r iwlwifi && sudo modprobe iwlwifi

3. ExecutionRequest: User confirms safe commands
   - Must type exactly: "I understand this will execute automatically."
   - Request persisted to /var/lib/anna/execution_requests/

4. HumanExecutionAdapter: Executes safe commands
   - Binary must be in allowlist: [iw, lsmod, lspci, cat, echo]
   - No sudo, no pipes, no redirects
   - Full audit trail recorded

5. Manual commands: Shown to user for copy/paste
   - Anna cannot execute sudo commands
   - User runs them in their own terminal
```

### Why This Architecture?

1. **No implicit execution**: Commands cannot be executed without explicit human action
2. **Structural boundaries**: The code physically cannot bypass constraints
3. **Audit trail**: Every execution attempt is recorded
4. **Versioned contract**: Capability changes require version bumps

### Capability Ledger

Anna's capabilities are defined in `crates/anna-shared/src/capabilities.rs`. This is the single source of truth for what Anna can and cannot do.

**Execution Capabilities** (require human confirmation):
- `human_mediated_execution`: Human provides command at runtime
- `automatic_safe_execution`: Safe commands after explicit confirmation

**Diagnosis Capabilities** (read-only, no confirmation):
- `system_state_diagnosis`: lspci, lsmod, free, df, etc.
- `wifi_diagnosis`: iw, lsmod, lspci for WiFi state
- `config_file_read`: cat for specific config paths

**Forbidden Capabilities** (structurally impossible):
- `NO_network_requests`: wget, curl, etc. not in allowlist
- `NO_package_installation`: pacman, apt, etc. not in allowlist
- `NO_sudo_execution`: sudo rejected by adapter
- `NO_destructive_commands`: rm, dd, etc. rejected by adapter

### Binary Allowlist

The HumanExecutionAdapter has an explicit allowlist:
```
iw, lsmod, lspci, cat, echo
```

Everything else is rejected. The allowlist is enforced at runtime and verified by tests.

### Guardrails

Changes to the capability system trigger CI checks:
1. Capability ledger changes require VERSION bump
2. Binary allowlist changes require ledger update
3. Confirmation string changes require test updates
4. Forbidden binaries are checked on every build

### Trust Surface Report

A complete trust surface report is maintained at `docs/trust_surface.md`. This document is:
- Generated from the capability ledger
- Deterministic and diffable
- Updated with each capability change

### Runtime Trust Disclosure (Phase 46)

Anna declares her capabilities before acting. This is not politeness - it prevents silent power growth.

**Why Anna Declares Herself**

Every request is answered against a published truth, not improvisation. The declaration:
- Is derived from `capabilities.rs` - cannot diverge from actual behavior
- Is deterministic - same ledger produces same declaration
- Is human-readable - no jargon, plain language

**How to View the Declaration**

```bash
annactl capabilities             # Full declaration
annactl capabilities --onboarding   # Compact summary
annactl capabilities --deterministic   # Diffable format
```

**Isolation Guarantee**

The declaration module (`declaration.rs`) is architecturally isolated:
- Imports only from `capabilities.rs`
- Contains no execution code (no `Command::new`, no process spawning)
- Cannot request, trigger, or enable execution
- Tests verify this isolation on every build

This means the declaration layer physically cannot expand Anna's power - it can only describe existing power.

### Single Authorization Path (Phase 47)

All command execution flows through ONE authorization path. There are no case-by-case exceptions.

**The Command Policy Engine**

`command_policy.rs` is the ONLY way execution-capable code can approve an OS command:

```
CommandSpec (argv tokens) → authorize_command() → PolicyDecision
                                                      ↓
                                          Allowed { capability_id, safety }
                                          Denied { reason }
```

HumanExecutionAdapter MUST call `authorize_command()` before executing anything.

**Why Single Path Matters**

- Eliminates "WiFi special logic" and "YouTube special logic"
- Adding a new command requires updating the ledger + tests
- No backdoors - policy engine checks every execution
- Auditable at a single location

**Hard Bans (Always Denied)**

These patterns are rejected regardless of ledger content:
- Privilege escalation: sudo, su, pkexec, doas
- Shells: sh, bash, zsh, fish, dash, csh
- Destructive: rm, dd, mkfs, fdisk, shred
- Network: wget, curl, nc, ssh, scp
- Package managers: pacman, apt, dnf, yum
- Dangerous patterns: pipes (|), redirects (>, <), command substitution ($())

**Guardrail Tests**

37 tests enforce the single path:
- `guardrail_policy_authorizes_exactly_ledger_binaries`
- `guardrail_no_second_authorization_outside_policy`
- `guardrail_human_execution_uses_policy`
- `guardrail_hard_bans_comprehensive`

### Capability Change Checklist

To add a new allowed command:

1. **Update capability ledger** (`capabilities.rs`)
   - Add binary to appropriate capability's `allowed_binaries`

2. **Update VERSION** (Cargo.toml and VERSION file)
   - Bump version number

3. **Update CHANGELOG.md**
   - Document the new capability

4. **Run tests**
   - `cargo test --workspace`
   - All guardrail tests must pass

5. **Verify disclosure**
   - Run `annactl capabilities`
   - Confirm new capability appears

6. **Update trust surface** if needed
   - Regenerate `docs/trust_surface.md`

If any step fails, the new command cannot be authorized.

## Module Structure

```
crates/
├── anna-shared/           # Shared types and utilities
│   ├── capabilities.rs    # Capability ledger (Phase 45)
│   ├── command_policy.rs  # Single authorization path (Phase 47)
│   ├── declaration.rs     # Runtime trust disclosure (Phase 46)
│   ├── execution_request.rs # Human-issued requests (Phase 40)
│   ├── human_execution.rs # Execution adapter using policy (Phase 42, 47)
│   ├── action_plan.rs     # Plan types (no execution)
│   └── ...
├── annad/                 # Daemon
│   ├── assisted_ops/      # Diagnosis and proposals (Phase 39, 43)
│   │   ├── types.rs       # AssistedOperation types
│   │   ├── wifi_diagnosis.rs # WiFi diagnosis
│   │   ├── execution_bridge.rs # Proposal → Request
│   │   └── detection.rs   # System state reading
│   └── ...
└── annactl/               # CLI
    ├── main.rs            # CLI including capabilities command (Phase 46)
    ├── repair.rs          # Repair commands (Phase 43)
    └── ...
```

## System Paths

All state is stored in system directories, never in user home:

| Purpose | Path |
|---------|------|
| Config  | /etc/anna/ |
| State   | /var/lib/anna/ |
| Runtime | /run/anna/ |
| Socket  | /run/anna/anna.sock |

## Version History

- **Phase 47**: Capability-Gated Command Policy Engine (Single Path)
- **Phase 46**: Capability Declaration & Runtime Trust Disclosure
- **Phase 45**: Trust Surface Review + Capability Ledger
- **Phase 43**: End-to-End Assisted Ops → Human Execution
- **Phase 42**: Human-Mediated Execution Adapter
- **Phase 40**: Execution Request (human-issued)
- **Phase 39**: Assisted Operations Layer
