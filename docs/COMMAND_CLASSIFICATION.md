# Command Classification System

**Phase 3.4: Adaptive CLI** - Command categorization for context-aware help

## Overview

Anna's commands are classified into three categories to enable adaptive UI based on user experience level and system context:

- **User-Safe**: Everyday commands safe for all users
- **Advanced**: System administration commands requiring knowledge
- **Internal**: Developer/diagnostic commands for experts

## Classification Schema

### 🟢 User-Safe Commands

Commands that are safe for daily use by any user, with minimal risk:

| Command | Description | Risk Level | Prerequisites |
|---------|-------------|------------|---------------|
| `help` | Show available commands | None | None |
| `status` | Show system status | None | Daemon running |
| `ping` | Test daemon connection | None | Daemon running |
| `health` | Check system health | None | Daemon running |
| `profile` | Show system profile | None | Daemon running |
| `metrics` | Display system metrics | None | Daemon running |
| `monitor status` | Check monitoring stack | None | Daemon running |
| `self-update --check` | Check for updates | None | Network access |
| `triage` | Show system recommendations | None | Daemon running |

**Characteristics:**
- Read-only operations
- No system modifications
- Safe to run repeatedly
- Helpful for learning
- Display information only

### 🟡 Advanced Commands

Commands requiring system administration knowledge:

| Command | Description | Risk Level | Prerequisites |
|---------|-------------|------------|---------------|
| `update` | Update system packages | Medium | Root access, understanding of package management |
| `install` | Install Arch Linux | High | ISO live environment, disk partitioning knowledge |
| `backup` | Create system backup | Medium | Root access, sufficient disk space |
| `doctor` | Run diagnostics & fixes | Medium | Root access, understanding of proposed fixes |
| `rollback` | Revert system changes | Medium | Root access, understanding of action history |
| `repair` | Repair failed probes | Medium | Root access, diagnostic skills |
| `audit` | Show audit logs | Low | Root access |
| `monitor install` | Install monitoring | Medium | Root access, 2-4GB RAM available |
| `rescue` | Rescue/recovery tools | High | Recovery environment knowledge |
| `collect-logs` | Diagnostic log collection | Low | Root access |
| `self-update --list` | List available versions | Low | Network access |

**Characteristics:**
- May modify system state
- Require root/sudo access
- Potentially destructive if misused
- Show warnings before execution
- Logged to audit trail

### 🔴 Internal Commands

Developer and diagnostic commands for system experts:

| Command | Description | Risk Level | Prerequisites |
|---------|-------------|------------|---------------|
| `sentinel` | Sentinel framework management | Low | Developer knowledge |
| `config` | Configuration management | Medium | Understanding of config schema |
| `conscience` | Conscience governance | Low | Phase 1.1 knowledge |
| `empathy` | Empathy kernel diagnostics | Low | Phase 1.2 knowledge |
| `collective` | Collective mind management | Low | Phase 1.3 knowledge |
| `mirror` | Mirror protocol diagnostics | Low | Phase 1.4 knowledge |
| `chronos` | Temporal analysis | Low | Phase 1.5 knowledge |
| `consensus` | Distributed consensus | Low | Phase 1.7 knowledge |

**Characteristics:**
- Experimental or development features
- May expose internal state
- Require understanding of Anna's architecture
- Not needed for normal operation
- May have incomplete implementations

## Adaptive Help System

### Context-Aware Display

The `annactl help` command adapts based on:

1. **User Experience Level** (future):
   - Beginner: Show only user-safe commands
   - Intermediate: Show user-safe + common advanced
   - Expert: Show all commands

2. **System Context**:
   - ISO Live: Emphasize `install`, `rescue`
   - Degraded: Emphasize `doctor`, `repair`, `rollback`
   - Healthy: Show standard maintenance commands
   - Constrained Resources: Hide `monitor install`

3. **Daemon State**:
   - Daemon Unavailable: Show only standalone commands
   - Daemon Running: Show full command set

### Command Visibility Rules

```
IF system_state == "iso_live" THEN
  SHOW: install, rescue, health
  HIDE: update, backup, monitor

ELSE IF system_state == "degraded" THEN
  HIGHLIGHT: doctor, repair, rollback, health
  SHOW: status, triage, collect-logs

ELSE IF is_constrained == true THEN
  HIDE: monitor install (full mode)
  WARN: update, install (show resource warning)

ELSE
  SHOW: all user-safe + advanced commands
  GRAY: internal commands (visible but de-emphasized)
```

## Implementation Plan

### Phase 3.5: Command Metadata

Add metadata to each command:

```rust
pub struct CommandMetadata {
    name: &'static str,
    category: CommandCategory,
    risk_level: RiskLevel,
    requires_root: bool,
    requires_daemon: bool,
    min_system_state: Vec<SystemState>,
    description_short: &'static str,
    description_long: &'static str,
}

pub enum CommandCategory {
    UserSafe,      // 🟢 Safe for all users
    Advanced,      // 🟡 Requires admin knowledge
    Internal,      // 🔴 Developer/diagnostic
}

pub enum RiskLevel {
    None,          // Read-only, no risk
    Low,           // Minimal impact
    Medium,        // Can modify system
    High,          // Potentially destructive
}
```

### Phase 3.6: Adaptive Help Command

Enhance `annactl help` to filter and organize by category:

```bash
# Default: Show user-safe + common advanced
$ annactl help

# Show only safe commands
$ annactl help --safe

# Show all commands including internal
$ annactl help --all

# Show commands available in current state
$ annactl help --available
```

### Phase 3.7: Command Hints

Provide contextual hints when users run commands:

```bash
$ sudo annactl update
💡 Tip: This is an advanced command. Run 'annactl help update' for details.
⚠️  Resource Constraint Warning: ...
```

## Future Enhancements

### User Profiles (Phase 4.0)

Store user experience level in persistent context:

```json
{
  "user_profile": {
    "experience_level": "beginner",
    "command_history": [...],
    "successful_commands": [...],
    "help_mode": "verbose"
  }
}
```

### Command Recommendations (Phase 4.1)

Suggest appropriate commands based on system state:

```bash
$ annactl status
System Status: DEGRADED

Recommended actions:
  1. annactl doctor          # Diagnose and fix issues
  2. annactl health          # Check probe status
  3. annactl repair --probe systemd-failed-units
```

### Learning Mode (Phase 4.2)

Track command usage and graduate users automatically:

```
After 10 successful advanced commands → Promote to intermediate
After 5 internal command uses → Promote to expert
```

## Security Considerations

1. **Privilege Escalation**: Advanced commands must validate sudo/root
2. **Audit Logging**: All advanced commands logged to audit trail
3. **Confirmation Dialogs**: High-risk commands require explicit confirmation
4. **Resource Checks**: Commands check system constraints before execution
5. **State Validation**: Commands verify system is in appropriate state

## Accessibility

- Clear command descriptions for all experience levels
- Examples provided for complex commands
- `--dry-run` available for all destructive operations
- Verbose mode explains what will happen
- Help text tailored to current system state

## Documentation

Each command category has dedicated documentation:

- **User Guide**: Focus on user-safe commands
- **Admin Guide**: Cover advanced commands
- **Developer Guide**: Document internal commands

---

**Status**: Phase 3.4 - Classification complete
**Next**: Phase 3.5 - Implement command metadata system
**Author**: Anna Adaptive Intelligence Team
**License**: Custom (see LICENSE file)

Citation: [ux-design:progressive-disclosure]
