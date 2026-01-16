# Anna Phase Registry

This document is the authoritative record of capability-affecting phases.

## Constitutional Rule

**Capability expansion requires a new phase declaration.**

Any change to the following files that adds execution power MUST be accompanied by a new phase entry in this document:

- `crates/anna-shared/src/capabilities.rs` (capability ledger)
- `crates/anna-shared/src/command_policy.rs` (authorization engine)
- `crates/anna-shared/src/human_execution.rs` (execution adapter)

Changes that do NOT require a new phase:
- Bug fixes that do not expand power
- Documentation improvements
- Test hardening
- Refactoring that preserves behavior

CI will fail any PR that modifies capability files without a corresponding phase entry.

## Phase Registry

### Phase 39: Assisted Operations Layer
- Introduced AssistedOperation structure
- Classified commands as Safe or Manual
- No execution capability

### Phase 40: Execution Request
- Added ExecutionRequest for human-issued execution
- Introduced confirmation strings
- Execution still impossible without adapter

### Phase 42: Human-Mediated Execution Adapter
- First execution path in the system
- Binary allowlist: iw, lsmod, lspci, cat, echo
- Audit trail for all executions

### Phase 43: End-to-End Assisted Ops to Human Execution
- Wired WiFi diagnosis to execution adapter
- Safe commands execute after confirmation
- Manual commands shown for copy/paste

### Phase 45: Trust Surface Review + Capability Ledger
- Codified capability ledger in capabilities.rs
- 14 capabilities defined
- Guardrail tests for ledger consistency
- CI enforcement for version bumps

### Phase 46: Capability Declaration & Runtime Trust Disclosure
- Added declaration.rs for runtime disclosure
- `annactl capabilities` command
- Declaration derives from ledger, cannot diverge

### Phase 47: Capability-Gated Command Policy Engine
- Single authorization path via command_policy.rs
- All execution must go through authorize_command()
- 37 guardrail tests for policy enforcement
- Hard bans on dangerous patterns

### Phase 48: Constitutional Freeze (Current)
- Stabilization mode begins
- No new capabilities, commands, or binaries
- This document created as phase registry
- CI enforcement of constitutional rule

## How to Propose a New Phase

If you need to expand Anna's capabilities, you MUST:

1. Create a proposal explaining:
   - What capability is being added
   - Why it cannot be achieved with existing capabilities
   - What new attack surface this creates
   - How it will be tested and disclosed

2. Add a new phase entry to this document BEFORE modifying capability files

3. Update VERSION and CHANGELOG

4. Ensure all tests pass, including new tests for the capability

5. Verify `annactl capabilities` reflects the change

The friction is intentional. Power expansion should be rare, visible, and deliberate.

## Frozen State (Phase 48+)

As of Phase 48, the following are frozen:

**Allowed Binaries (11):**
- Execution: iw, lsmod, lspci, cat, echo
- Diagnosis: lsusb, lscpu, free, df, uname, hostname

**Execution Capabilities (2):**
- human_mediated_execution
- automatic_safe_execution

**Forbidden Capabilities (4):**
- NO_network_requests
- NO_package_installation
- NO_sudo_execution
- NO_destructive_commands

Any expansion beyond this list requires Phase 51+.
