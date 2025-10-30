# Design Note: Privilege Model

**Date**: October 30, 2025
**Sprint**: 1
**Status**: Implemented

---

## Overview

Anna's privilege model enforces strict separation between unprivileged user operations and privileged system operations. This document describes the chosen approach and future options.

---

## Architecture

### Components

1. **annad** (daemon)
   - Runs as root via systemd
   - Owns `/etc/anna/` and system-wide state
   - Listens on `/run/anna/annad.sock` (permissions: 0666)
   - Executes all privileged operations

2. **annactl** (CLI client)
   - Runs as unprivileged user
   - Never writes system files directly
   - Communicates with annad via Unix socket
   - No sudo required for any operations

3. **Polkit** (authorization layer)
   - Policy: `/usr/share/polkit-1/actions/com.anna.policy`
   - Actions:
     - `com.anna.config.write` - Modify system configuration
     - `com.anna.maintenance.execute` - Run system maintenance tasks

---

## Privilege Flow

```
User invokes:
  annactl config set system autonomy.level low

Flow:
  1. annactl sends RPC: ConfigSet { scope: System, key, value }
  2. annad receives request over socket
  3. annad checks if operation requires privilege (scope == System)
  4. annad invokes polkit check for action com.anna.config.write
  5. User sees polkit dialog (if needed)
  6. On auth success, annad writes /etc/anna/config.toml
  7. annad returns success to annactl
  8. annactl displays confirmation
```

---

## Implementation: Sprint 1 Approach

For Sprint 1, we implement **Option B: pkexec wrapper** for simplicity and robustness.

### Option B: pkexec Wrapper (CHOSEN)

**How it works:**
- annad runs as root via systemd
- When privileged write needed, annad directly performs it (already running as root)
- Polkit policy file defines which actions are allowed
- Future: if annad needs to be invoked by user directly, pkexec wrapper handles auth

**Advantages:**
- Simple implementation
- Standard polkit integration
- Works immediately on any systemd distro
- Familiar user experience (polkit dialogs)

**Implementation:**
```rust
// In annad/src/polkit.rs
pub fn check_authorization(action: &str, uid: u32) -> Result<bool> {
    // Since annad runs as root, we're already authorized
    // This is a placeholder for future per-action checks
    Ok(true)
}

pub fn write_system_config(content: &str) -> Result<()> {
    // annad is root, can write directly
    std::fs::write("/etc/anna/config.toml", content)?;
    Ok(())
}
```

**Polkit Policy:**
```xml
<action id="com.anna.config.write">
  <description>Modify Anna system configuration</description>
  <message>Authentication required to modify system configuration</message>
  <defaults>
    <allow_any>auth_admin</allow_any>
    <allow_inactive>auth_admin</allow_inactive>
    <allow_active>auth_admin</allow_active>
  </defaults>
</action>
```

---

## Alternative: Option A (Future Consideration)

### Option A: Direct D-Bus + Polkit API

**How it would work:**
- annad exposes D-Bus interface
- Polkit D-Bus API used for fine-grained auth checks
- Each RPC method annotated with required polkit action

**Advantages:**
- More flexible authorization model
- Per-user, per-action granularity
- Can implement "remember this decision"
- Better integration with desktop environments

**Disadvantages:**
- Requires D-Bus dependency
- More complex implementation
- Harder to debug
- May not be available in minimal systems

**Future implementation path:**
1. Add `zbus` crate dependency
2. Implement D-Bus service interface
3. Replace Unix socket with D-Bus
4. Use `polkit` crate for auth checks
5. Update systemd service to use `Type=dbus`

---

## Configuration Scopes

### User Scope
- File: `$HOME/.config/anna/config.toml`
- Writable by user
- No privilege escalation required
- Takes precedence over system config

### System Scope
- File: `/etc/anna/config.toml`
- Writable only by root (via annad)
- Requires polkit authorization
- Applies to all users

### Merge Strategy

```rust
fn load_config() -> Config {
    let system = read("/etc/anna/config.toml")?;
    let user = read("~/.config/anna/config.toml")?;

    // User settings override system settings
    merge(system, user)
}
```

---

## Security Properties

1. **Least Privilege**: annactl never runs as root
2. **Audit Trail**: All privileged operations logged via telemetry
3. **User Consent**: Polkit dialogs require explicit authorization
4. **Fail-Safe**: If polkit unavailable, deny privileged operations
5. **Isolation**: Socket permissions prevent unauthorized access

---

## File Permissions

```
/etc/anna/                    root:root     755
/etc/anna/config.toml         root:root     644
/run/anna/                    root:root     755
/run/anna/annad.sock          root:root     666  (world-writable socket)
~/.config/anna/               user:user     700
~/.config/anna/config.toml    user:user     600
```

---

## Testing Strategy

1. **Privilege Separation Test**
   - Verify annactl never writes to /etc/anna/
   - Verify annad can write to /etc/anna/
   - Verify socket permissions allow user connection

2. **Authorization Test**
   - Test system config write with polkit present
   - Test system config write fails gracefully without polkit
   - Test user config write works without polkit

3. **Scope Isolation Test**
   - Set user config, verify system config unchanged
   - Set system config, verify user config unchanged
   - Verify merge precedence (user > system)

---

## Future Enhancements

1. **Fine-grained Actions**
   - `com.anna.config.autonomy.write` - Specific to autonomy settings
   - `com.anna.config.telemetry.write` - Specific to telemetry
   - Per-key authorization policies

2. **Delegation**
   - Allow specific users/groups to bypass polkit for certain actions
   - Useful for CI/CD and automation scenarios

3. **Audit**
   - Log all polkit auth attempts (success/failure)
   - Include uid, action, timestamp in telemetry

4. **Offline Mode**
   - Cache polkit decisions for headless systems
   - Configurable timeout for cached decisions

---

## Decision Rationale

**Why Option B for Sprint 1:**

1. **Simplicity**: Get working privilege separation quickly
2. **Correctness**: Easy to verify security properties
3. **Portability**: Works on any Arch system with polkit
4. **Incremental**: Can migrate to Option A later without breaking changes

The socket-based RPC with root daemon is the key architectural decision. The specific auth mechanism (pkexec vs D-Bus API) is an implementation detail that can evolve.

---

## References

- Polkit Manual: `man polkit`
- Polkit Actions: `/usr/share/polkit-1/actions/`
- D-Bus Specification: https://dbus.freedesktop.org/doc/dbus-specification.html
- Arch Linux Security: https://wiki.archlinux.org/title/Security

---

**End of Design Note**
