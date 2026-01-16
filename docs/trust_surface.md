# Anna Trust Surface (Deterministic)

Ledger Version: 1.0

## NO_destructive_commands
- Description: Anna cannot run destructive commands
- Category: Execution
- Execution: None
- Args: FORBIDDEN - rm, dd, mkfs, fdisk rejected by HumanExecutionAdapter
- Persistence: None
- Module: FORBIDDEN

## NO_network_requests
- Description: Anna cannot make arbitrary network requests
- Category: Network
- Execution: None
- Args: FORBIDDEN - no wget, curl, nc, etc. in execution path
- Persistence: None
- Module: FORBIDDEN

## NO_package_installation
- Description: Anna cannot install packages automatically
- Category: PackageManagement
- Execution: None
- Args: FORBIDDEN - pacman, yay, apt etc. not in allowlist
- Persistence: None
- Module: FORBIDDEN

## NO_sudo_execution
- Description: Anna cannot execute sudo commands automatically
- Category: Execution
- Execution: None
- Args: FORBIDDEN - sudo, pkexec, doas rejected by HumanExecutionAdapter
- Persistence: None
- Module: FORBIDDEN

## assisted_operation_proposal
- Description: Propose fixes as AssistedOperation structures
- Category: Proposal
- Execution: ManualOnly
- Args: No binaries - proposals are data structures
- Persistence: None
- Module: annad::assisted_ops::types

## audit_logging
- Description: Write execution attempt audit records
- Category: FilesystemWrite
- Execution: None
- Args: Rust std::fs only, paths under /var/lib/anna/execution_attempts/
- Persistence: AuditOnly
- Module: anna_shared::human_execution

## automatic_safe_execution
- Description: Auto-execute safe commands after explicit confirmation
- Category: Execution
- Execution: HumanConfirmedSafeAutomatic
- Binaries: cat, echo, iw, lsmod, lspci
- Args: Same restrictions as human_mediated_execution
- Confirmation: "I understand this will execute automatically."
- Persistence: AuditOnly
- Module: annactl::repair

## config_file_read
- Description: Read configuration files for diagnosis
- Category: FilesystemRead
- Execution: None
- Binaries: cat
- Args: Specific paths only: /etc/modprobe.d/*.conf
- Persistence: None
- Module: annad::assisted_ops::detection

## execution_request_creation
- Description: Create ExecutionRequest for human review
- Category: Proposal
- Execution: ManualOnly
- Args: No binaries - requests are data structures
- Persistence: AuditOnly
- Module: anna_shared::execution_request

## human_mediated_execution
- Description: Execute commands via HumanExecutionAdapter
- Category: Execution
- Execution: HumanConfirmedSafeAutomatic
- Binaries: cat, echo, iw, lsmod, lspci
- Args: No sudo, no pipes, no redirects, no command substitution
- Confirmation: "I understand this will not execute automatically."
- Persistence: AuditOnly
- Module: anna_shared::human_execution

## state_persistence
- Description: Write Anna's own state files
- Category: FilesystemWrite
- Execution: None
- Args: Rust std::fs only, paths under /var/lib/anna/
- Persistence: StateChanging
- Module: anna_shared::paths, anna_shared::safe_ops

## system_state_diagnosis
- Description: Read system state via diagnostic commands
- Category: Diagnosis
- Execution: None
- Binaries: df, free, hostname, lscpu, lsmod, lspci, lsusb, uname
- Args: Read-only flags only (-m, -mm, -k, -g, -h, -r)
- Persistence: None
- Module: anna_shared::profile, anna_shared::monitor

## wifi_diagnosis
- Description: Diagnose WiFi issues by reading wireless state
- Category: Diagnosis
- Execution: None
- Binaries: iw, lsmod, lspci
- Args: Read-only: 'iw <dev> link', 'lsmod', 'lspci -k'
- Persistence: None
- Module: annad::assisted_ops::detection

---

## Summary

**Total capabilities**: 14
**Execution capabilities**: 2
**Forbidden capabilities**: 4
**Unique allowed binaries**: 11

### What Anna Can Read
- System state via lspci, lsusb, lsmod, lscpu, free, df, uname, hostname
- WiFi state via iw, lsmod, lspci
- Configuration files via cat (restricted paths)

### What Anna Can Suggest
- AssistedOperation proposals (data structures, no execution)
- ExecutionRequest records (audit only)

### What Anna Can Execute
- Commands via HumanExecutionAdapter with exact confirmation
- Allowlist: iw, lsmod, lspci, cat, echo
- Restrictions: No sudo, no pipes, no redirects, no command substitution

### What Anna Will NEVER Do
- Make arbitrary network requests
- Install packages automatically
- Execute sudo/pkexec/doas commands
- Run destructive commands (rm, dd, mkfs, fdisk)

---

*This document is generated from the capability ledger.*
*To regenerate: `cargo test --package anna-shared test_deterministic_surface_is_stable`*
*Changes require LEDGER_VERSION bump in capabilities.rs*
