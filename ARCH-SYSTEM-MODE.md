# Anna System Mode Architecture

## Overview

Anna v0.6.5 introduces **system mode** as the default installation, transitioning from per-user to system-wide operation. This document explains the architecture, design rationale, and migration path.

## Key Principles

1. **System-wide installation by default**: `annad` runs as a root system service
2. **Per-user data isolation**: Each user has isolated profile, data, and preferences under `/var/lib/anna/users/<uid>/`
3. **Least-privilege UX**: `annactl` runs unprivileged; privileged operations flow through `annad` with policy
4. **Read operations never require sudo**: Status, quickscan, advice list, and persona queries work without elevation
5. **Stable advice numbering and de-duplication**: Advice entries maintain consistent IDs and auto-clear when resolved

## Installation Modes

### System Mode (Default, v0.6.5+)

**Binary locations:**
- `annad`: `/usr/local/sbin/annad` (root:root, 0755)
- `annactl`: `/usr/local/bin/annactl` (root:root, 0755)

**Directory structure:**
```
/etc/anna/                          # System-wide configuration
/var/lib/anna/                      # System data root
/var/lib/anna/users/<uid>/          # Per-user data
  ├── profiles/                     # User profiles
  ├── reports/                      # Quickscan reports
  ├── signals/                      # Signal processing data
  ├── advice/                       # Generated advice (0770 root:anna)
  └── persona/                      # Persona state
/run/anna/                          # Runtime directory (created by systemd)
  └── annad.sock                    # Unix domain socket (0660 root:anna)
```

**Group membership:**
- System group `anna` is created during installation
- Users added to `anna` group can access read-only operations without sudo
- Per-user directories are created on-demand by `annad` with 0770 mode

**Service:**
- Systemd unit: `/etc/systemd/system/annad.service`
- Type: simple
- User: root
- Group: anna
- RuntimeDirectory: anna (0770)
- Socket: `/run/anna/annad.sock` (0660 root:anna)

### User Mode (Dev/Testing only)

**Binary locations:**
- `annad`: `~/.local/bin/annad`
- `annactl`: `~/.local/bin/annactl`

**Directory structure:**
```
~/.anna/
  ├── config/                       # User-specific config
  ├── data/                         # User data root
  │   ├── profiles/
  │   ├── reports/
  │   ├── signals/
  │   ├── advice/
  │   └── persona/
  └── run/
      └── annad.sock                # User-mode socket
```

**Activation:**
- Set `ANNA_MODE=user` environment variable
- Systemd user service: `~/.config/systemd/user/annad.service`

## RPC Protocol

### Transport

Anna uses a **Unix domain socket** for communication between `annactl` and `annad`. The protocol is length-prefixed JSON for simplicity and debuggability.

**Message format:**
1. 4-byte length prefix (big-endian u32)
2. JSON payload

**Socket path:**
- System mode: `/run/anna/annad.sock`
- User mode: `~/.anna/run/annad.sock`

### Request Types (S1)

All requests include the caller's UID for per-user path resolution.

#### StatusRequest
```json
{"type": "Status", "uid": 1000}
```

Returns:
- Install mode (system/user)
- Socket path
- Per-user data directories
- Service state
- Last quickscan timestamp
- Advice count

#### QuickscanRequest
```json
{"type": "Quickscan", "uid": 1000}
```

Returns the latest quickscan report for the user.

#### AdviceListRequest
```json
{"type": "AdviceList", "uid": 1000}
```

Returns all advice entries for the user.

#### PersonaRequest
```json
{"type": "Persona", "uid": 1000, "op": "Show"}
```

Returns persona information (stub in S1).

### Why RPC?

1. **No sudo required for reads**: Socket permissions (0660 root:anna) allow group members to read without elevation
2. **Per-user isolation**: Daemon resolves paths based on caller UID, preventing cross-user data access
3. **Policy enforcement point**: Future write operations (S2) will be gated through `annad` with policy checks
4. **Clean separation**: annactl becomes a thin client; all logic lives in the daemon

## Migration from 0.6.4

The installer automatically handles migration:

1. **Detection**: Checks for user-mode binaries at `~/.local/bin/`
2. **User service cleanup**: Disables `systemctl --user` service if present
3. **Binary shadowing**: System binaries in `/usr/local/` override any user copies
4. **Data preservation**: Existing user data remains untouched
5. **Group addition**: Installer adds the invoking user to the `anna` group

**Post-install verification:**
```bash
annactl status         # Should show mode=system
annactl doctor perms   # Verifies socket and directory permissions
```

## Security Model

### S1: Read-only RPC

- **annad runs as root** to manage per-user directories
- **annactl runs unprivileged** and connects via socket
- **Socket is 0660 root:anna**, allowing group members to connect
- **All read operations flow through RPC**, no direct file access by annactl
- **Per-user directories are 0770 root:anna**, writable by daemon only

### S2 (Current): Policy-gated writes

- `annactl apply` sends write requests to `annad` via RPC
- Daemon validates requests against per-user policy at `/etc/anna/policy.d/<uid>.toml`
- Low-risk actions proceed automatically; high-risk require confirmation
- Dangerous operations (filesystem, package removal) always require explicit approval
- Notifications sent to user on apply completion
- Successful applies auto-clear advice to prevent duplicate execution

## Path Resolution

The `AnnaPaths` struct centralizes path logic:

```rust
pub fn detect_for_uid(uid: u32) -> Self
```

**Logic:**
1. Check `ANNA_MODE` environment variable (user/system override)
2. If system paths exist (`/var/lib/anna`, `/etc/anna`), use system mode
3. If socket exists (`/run/anna/annad.sock`), use system mode
4. Otherwise, fall back to system mode (S1 default)

**Per-user paths in system mode:**
- `/var/lib/anna/users/<uid>/reports`
- `/var/lib/anna/users/<uid>/advice`
- `/var/lib/anna/users/<uid>/persona`
- `/var/lib/anna/users/<uid>/signals`
- `/var/lib/anna/users/<uid>/profiles`

## Directory Permissions

### System Mode

| Path | Owner | Group | Mode | Description |
|------|-------|-------|------|-------------|
| `/etc/anna` | root | root | 0755 | System config (read by all) |
| `/var/lib/anna` | root | root | 0755 | System data root |
| `/var/lib/anna/users` | root | root | 0755 | User data root |
| `/var/lib/anna/users/<uid>` | root | anna | 0770 | Per-user data (daemon-writable) |
| `/run/anna` | root | root | 0755 | Runtime directory |
| `/run/anna/annad.sock` | root | anna | 0660 | Socket (anna group can connect) |

### User Mode

| Path | Owner | Group | Mode | Description |
|------|-------|-------|------|-------------|
| `~/.anna/config` | user | user | 0755 | User config |
| `~/.anna/data` | user | user | 0755 | User data root |
| `~/.anna/run` | user | user | 0755 | Runtime directory |
| `~/.anna/run/annad.sock` | user | user | 0660 | Socket |

## Doctor Command

`annactl doctor perms` verifies the installation:

**System mode checks:**
- `/etc/anna` and `/var/lib/anna` exist with correct ownership
- `/run/anna/annad.sock` exists and is accessible
- Current user is in the `anna` group
- Per-user subtree `/var/lib/anna/users/<uid>` has correct permissions

**User mode checks:**
- `~/.anna/data` and `~/.anna/config` exist
- Directories are owned by current user
- Modes are at least 0700 (user rwx)

## Advice Lifecycle

1. **Generation**: `annad` creates advice JSON files in `/var/lib/anna/users/<uid>/advice/`
2. **Numbering**: Entries are numbered sequentially (#1, #2, #3...) in `annactl advice list` output
3. **De-duplication**: Same advice (by kind + normalized reason) updates in place, keeping most recent
4. **Auto-clear**: When `apply` succeeds (S2), the advice file is deleted to prevent spam
5. **RPC read**: `annactl advice list` fetches via RPC, no direct file access

## Policy Model and Privilege Escalation (S2)

As of v0.6.6, Anna implements **policy-gated apply** for controlled privilege escalation. All write operations (applying advice) flow through `annad` with policy validation.

### Policy Levels

Each user has a policy file at `/etc/anna/policy.d/<uid>.toml` that defines their automation level:

| Level | Name | Description |
|-------|------|-------------|
| 0 | Manual | User must manually review and execute all commands |
| 1 | SafeMaintenance | Auto-apply routine maintenance (updates, cleanup) |
| 2 | SafeModerate | Auto-apply moderate changes (config tweaks, service restarts) |
| 3 | FullAutonomy | Auto-apply all non-dangerous actions without confirmation |

### Policy File Structure

```toml
[policy]
level = 1  # SafeMaintenance

[approval]
confirm_dangerous = true  # Always confirm dangerous operations
notify_on_apply = true    # Send notification after applying
```

### Dangerous Actions Safeguards

Anna maintains a table of dangerous operations that always require explicit confirmation, regardless of policy level:

- Filesystem operations: `mkfs`, `fdisk`, `dd if=`, `parted`, `gdisk`, `sgdisk`
- Package removal: `pacman -R`, `apt remove`, `dnf remove`
- Btrfs operations: `btrfs balance`, `mkswap`, `swapon`, `swapoff`
- System-wide destructive commands: `rm -rf /`

### Apply RPC Flow

1. **User invokes**: `annactl advice apply <id>`
2. **RPC request**: annactl sends `ApplyRequest` to annad with UID and advice ID
3. **Policy check**: annad loads user's policy and evaluates action
4. **Decision**:
   - **Allowed**: Execute commands, capture output, send notification
   - **RequiresConfirmation**: Return to user with reason, require `--confirm` flag
   - **RequiresElevation**: Deny with explanation (e.g., policy level too low)
5. **Auto-clear**: On success, advice file is deleted to prevent re-application
6. **Notification**: Desktop notification sent to user (with log fallback)

### Notification System

Notifications are delivered via multiple channels:

1. **Desktop notification**: `notify-send` via sudo -u #<uid>
2. **Log fallback**: Append to `/var/lib/anna/users/<uid>/notifications.log`
3. **Broadcast**: `wall` for system-wide messages (admin actions only)

### DoctorPerms RPC

`annactl doctor perms` now uses RPC to verify:

- Socket existence and accessibility
- User group membership (anna group)
- Per-user directory permissions
- Policy file presence at `/etc/anna/policy.d/<uid>.toml`

Returns structured issues with severity levels:
- **Error**: Critical problems blocking normal operation
- **Warning**: Non-critical issues that may affect functionality
- **Info**: Informational notices

## Status Command Output

Example system mode output:

```
Anna status
━━━━━━━━━━━━━━
Install mode:    system
Socket path:     /run/anna/annad.sock
User data dir:   /var/lib/anna/users/1000
System config:   /etc/anna
Service state:   active
Last quickscan:  2025-10-24T10:30:00Z
Advice count:    3
```

## Future Enhancements (S3+)

1. **Audit log**: All write operations logged to `/var/log/anna/` with structured format
2. **Rate limiting**: Per-user quotas for advice application
3. **Persona-driven policy**: Different thresholds based on user persona
4. **Batch apply**: `annactl advice apply --all` with intelligent grouping
5. **Web UI**: Optional web interface for multi-user management
6. **Policy templates**: Pre-defined policy sets for common roles (developer, admin, user)

## Troubleshooting

### "Anna daemon socket not found"

- Check service: `systemctl status annad`
- Start service: `sudo systemctl start annad`
- Verify socket: `ls -l /run/anna/annad.sock`

### "Permission denied" on socket

- Check group membership: `groups`
- Add to group: `sudo usermod -aG anna $USER`
- Re-login to apply group changes

### "No quickscan report available"

- Run a scan: `sudo systemctl kill -s SIGUSR1 annad`
- Wait 5 seconds, then: `annactl quickscan`

### Advice count is 0

- Trigger advice generation by fixing issues found in quickscan
- Check `/var/lib/anna/users/<uid>/advice/` exists and is writable by daemon

## References

- Systemd service: `systemd/annad.service`
- Installer: `scripts/install_anna.sh`
- Uninstaller: `scripts/uninstall_anna.sh`
- RPC protocol: `cmd/anna-rpc/src/lib.rs`
- Path resolver: `cmd/annactl/src/paths.rs`, `cmd/annad/src/rpc.rs` (paths module)
