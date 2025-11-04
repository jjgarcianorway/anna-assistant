# Anna Synchronization Architecture

> **Version**: v1.4.0 "Synchronization & Web GUI"
>
> **Status**: Design Complete, Implementation In Progress
>
> **Security Model**: Local-first, consent-driven, auditable

---

## Overview

Anna v1.4 introduces a **real-time synchronization layer** that keeps the daemon, TUI, and optional GUI in perfect harmony. Any configuration change triggers immediate updates across all active interfaces, with full audit trails and snapshot creation.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Configuration Sources                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ~/.config/anna/anna.yaml          ~/.config/anna/priorities.yaml      │
│  ~/.config/anna/profiles/*.yaml                                         │
│                                                                         │
└────────────────────────────┬────────────────────────────────────────────┘
                             │
                             │ File System Events (inotify/notify)
                             ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      Config API Layer (anna_common)                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  • Unified read/write/watch API                                         │
│  • Schema validation                                                    │
│  • Atomic file operations                                               │
│  • Change detection & diffing                                           │
│  • Event emission                                                       │
│                                                                         │
└────┬────────────────────────┬────────────────────────┬─────────────────┘
     │                        │                        │
     │ ConfigUpdateEvent      │ ConfigUpdateEvent      │ ConfigUpdateEvent
     ▼                        ▼                        ▼
┌─────────────┐      ┌─────────────┐        ┌─────────────────────┐
│  annad      │      │ annactl     │        │  GUI (Tauri)        │
│  (daemon)   │      │ config (TUI)│        │  (optional)         │
├─────────────┤      ├─────────────┤        ├─────────────────────┤
│             │      │             │        │                     │
│ • Reload    │      │ • Live      │        │ • Live preview      │
│   config    │      │   refresh   │        │ • Theme rendering   │
│ • Apply     │      │ • Visual    │        │ • Remote editing    │
│   changes   │      │   feedback  │        │                     │
│ • Broadcast │      │ • No restart│        │                     │
│             │      │             │        │                     │
└─────────────┘      └─────────────┘        └─────────────────────┘
     │                        │                        │
     └────────────────────────┴────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Audit Log       │
                    │  (audit.jsonl)   │
                    ├──────────────────┤
                    │ • config_updated │
                    │ • timestamp      │
                    │ • actor          │
                    │ • changes        │
                    │ • snapshot_token │
                    └──────────────────┘
```

---

## Message Protocol

### ConfigUpdateEvent

```json
{
  "event": "config_updated",
  "timestamp": "2025-11-03T18:47:23Z",
  "source": "tui|gui|daemon|remote",
  "actor": "user|system",
  "changes": {
    "master_config": {
      "profile": {"old": "beautiful", "new": "workstation"},
      "autonomy": {"old": "advice_only", "new": "auto_low_risk"}
    },
    "priorities": {
      "performance": {"old": "balanced", "new": "maximum"}
    }
  },
  "snapshot_token": "snap_k9j2_20251103_184723",
  "checksum": "sha256:a1b2c3d4..."
}
```

### ConfigReloadRequest

```json
{
  "event": "config_reload",
  "timestamp": "2025-11-03T18:47:24Z",
  "source": "daemon",
  "reason": "external_change|user_apply|schedule"
}
```

### ConfigValidationError

```json
{
  "event": "config_validation_error",
  "timestamp": "2025-11-03T18:47:25Z",
  "errors": [
    {
      "file": "anna.yaml",
      "line": 12,
      "field": "autonomy",
      "message": "Invalid value 'auto_high_risk': expected 'advice_only' or 'auto_low_risk'"
    }
  ]
}
```

---

## Synchronization Flow

### 1. User Edits Config in TUI

```
User modifies priority slider in TUI
  ↓
TUI marks state as dirty
  ↓
User navigates to Review section → presses Enter
  ↓
TUI calls config_api::save_with_sync()
  ↓
  ├─→ Create snapshot (snap_xxx)
  ├─→ Write anna.yaml atomically
  ├─→ Write priorities.yaml atomically
  ├─→ Log to audit.jsonl (config_updated event)
  └─→ Emit ConfigUpdateEvent
        ↓
        ├─→ Daemon receives event → reloads config
        ├─→ GUI (if running) receives event → refreshes UI
        └─→ Other TUI instances refresh (if any)
```

### 2. External File Change

```
User edits ~/.config/anna/anna.yaml directly (vim, etc.)
  ↓
File watcher detects inotify event
  ↓
Config API validates changes
  ↓
  ├─→ Valid: Emit ConfigUpdateEvent
  │           ↓
  │           └─→ All clients refresh
  │
  └─→ Invalid: Emit ConfigValidationError
              ↓
              └─→ GUI/TUI show error notification
                  (do not reload)
```

### 3. Daemon-Initiated Change

```
Daemon applies autonomous action (auto_low_risk mode)
  ↓
Daemon modifies config via config_api
  ↓
  ├─→ Create snapshot
  ├─→ Update config files
  ├─→ Log to audit (actor: "system")
  └─→ Emit ConfigUpdateEvent
        ↓
        └─→ GUI/TUI refresh with "Applied by Anna" notification
```

---

## File Watching Strategy

### notify Crate Configuration

```rust
use notify::{Watcher, RecursiveMode, Event};

let watcher = RecommendedWatcher::new(
    move |res: Result<Event, _>| {
        if let Ok(event) = res {
            match event.kind {
                EventKind::Modify(_) => {
                    // File modified, trigger reload
                    handle_config_change(&event.paths);
                }
                EventKind::Create(_) => {
                    // New profile created
                    handle_profile_added(&event.paths);
                }
                _ => {}
            }
        }
    },
    Config::default(),
)?;

watcher.watch(
    Path::new("~/.config/anna"),
    RecursiveMode::Recursive
)?;
```

### Debouncing

- **Problem**: Text editors write multiple times (save, backup, swap)
- **Solution**: 200ms debounce window
  - Collect all events in 200ms window
  - Only trigger reload after silence
  - Prevents reload storms

### Conflict Resolution

**Scenario**: TUI and GUI both edit config simultaneously

```rust
// Optimistic locking with timestamps
fn save_config(config: &MasterConfig) -> Result<()> {
    let current_checksum = calculate_checksum()?;

    if current_checksum != config.last_checksum {
        // Conflict detected
        return Err(ConfigConflict {
            current: load_master_config()?,
            proposed: config.clone(),
            resolution: ConflictResolution::MergeOrAbort,
        });
    }

    // Atomic write
    atomic_write(config)?;
    Ok(())
}
```

---

## Security Model

### Local-First

- **All synchronization is local** by default
- No network traffic unless explicitly requested
- No telemetry, no cloud sync
- File permissions: `0600` (user-only)

### Remote Configuration (SSH)

```bash
# Fetch remote config
annactl config --remote user@host

# Behind the scenes:
# 1. SSH to host
# 2. rsync ~/.config/anna/ to /tmp/anna-remote-xxx/
# 3. Launch TUI with temp config
# 4. On save: rsync back + snapshot on remote
# 5. Clean up temp files
```

**Security Guarantees**:
- Uses user's existing SSH keys (no new credentials)
- Transient only (no persistent connection)
- All changes logged on both local and remote
- Snapshot created before remote apply
- User confirmation required for each push

### Audit Trail

Every sync event logged to `~/.local/state/anna/audit.jsonl`:

```json
{
  "timestamp": "2025-11-03T18:47:23Z",
  "event": "config_updated",
  "actor": "user",
  "source": "tui",
  "changes_summary": "profile: beautiful → workstation",
  "snapshot_token": "snap_k9j2",
  "verification": {
    "checksum_before": "sha256:aaa...",
    "checksum_after": "sha256:bbb...",
    "files_modified": ["anna.yaml", "priorities.yaml"]
  }
}
```

---

## API Reference

### anna_common::config_api

```rust
pub mod config_api {
    /// Watch for config file changes
    pub fn watch_config_dir(
        callback: impl Fn(ConfigUpdateEvent) + Send + 'static
    ) -> Result<ConfigWatcher>;

    /// Save config with synchronization
    pub fn save_with_sync(
        master: &MasterConfig,
        priorities: &PrioritiesConfig,
    ) -> Result<SyncResult>;

    /// Load config with validation
    pub fn load_validated() -> Result<ValidatedConfig>;

    /// Calculate config checksum
    pub fn calculate_checksum(paths: &[PathBuf]) -> Result<String>;

    /// Create snapshot before changes
    pub fn create_snapshot(label: &str) -> Result<SnapshotToken>;

    /// Emit sync event
    pub fn emit_event(event: ConfigUpdateEvent) -> Result<()>;
}
```

### Event Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigEvent {
    Updated(ConfigUpdateEvent),
    ValidationError(ValidationError),
    ReloadRequest(ReloadRequest),
    SnapshotCreated(SnapshotInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdateEvent {
    pub timestamp: DateTime<Utc>,
    pub source: EventSource,
    pub actor: Actor,
    pub changes: ChangeSet,
    pub snapshot_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSource {
    Tui,
    Gui,
    Daemon,
    Remote { host: String },
    External, // Manual file edit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Actor {
    User,
    System,
    Scheduler,
}
```

---

## Performance Targets

| Operation                  | Target    | Current  |
|----------------------------|-----------|----------|
| File change detection      | <10ms     | TBD      |
| Event emission             | <5ms      | TBD      |
| TUI refresh                | <16ms     | TBD      |
| Config reload (daemon)     | <50ms     | TBD      |
| GUI update                 | <16ms     | TBD      |
| SSH config fetch           | <2s       | TBD      |
| Snapshot creation          | <100ms    | TBD      |

---

## Error Handling

### File System Errors

- **Permission denied**: Show helpful error with fix command
- **Disk full**: Warn user, prevent partial writes
- **Config locked**: Detect lock file, wait or abort

### Validation Errors

- **Schema mismatch**: Show diff, offer migration
- **Invalid value**: Highlight field, show valid options
- **Missing required**: Show defaults, allow apply with defaults

### Network Errors (Remote)

- **SSH timeout**: Retry with backoff
- **Connection refused**: Check SSH config, firewall
- **Auth failed**: Guide user to SSH key setup

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_config_change_detection() {
    let (tx, rx) = channel();
    let watcher = watch_config_dir(move |event| {
        tx.send(event).unwrap();
    }).unwrap();

    // Modify config
    write_test_config("test_profile");

    // Assert event received
    let event = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(event.source, EventSource::External);
}

#[test]
fn test_atomic_write() {
    let config = MasterConfig::default();

    // Simulate crash mid-write
    let result = atomic_write_with_interrupt(&config);

    // Assert: original file unchanged
    let loaded = load_master_config().unwrap();
    assert_eq!(loaded.version, 1);
}
```

### Integration Tests

```rust
#[test]
fn test_tui_daemon_sync() {
    // Start daemon
    let daemon = spawn_test_daemon();

    // Launch TUI, make change
    let tui = spawn_test_tui();
    tui.change_profile("workstation");
    tui.apply();

    // Assert daemon reloaded
    wait_for_condition(|| {
        daemon.current_profile() == "workstation"
    }, Duration::from_secs(2));
}
```

### End-to-End Tests

```bash
#!/bin/bash
# E2E: Remote configuration sync

# Setup
ssh test-host "annactl config --export /tmp/remote-config.yaml"

# Fetch
annactl config --remote test-host --import /tmp/local-config.yaml

# Modify locally
# (automated TUI interaction)

# Push back
annactl config --remote test-host --apply

# Verify
ssh test-host "cat ~/.config/anna/anna.yaml" | diff - /tmp/local-config.yaml
```

---

## Rollout Plan

### Phase 3.1: Sync Infrastructure (Current)

- [x] Design synchronization architecture
- [ ] Implement config_api module
- [ ] Add file watching with notify
- [ ] Daemon sync broadcaster
- [ ] TUI live refresh
- [ ] Profile import/export wizard
- [ ] Remote SSH configuration

### Phase 3.2: GUI Shell

- [ ] Tauri application scaffold
- [ ] Share config_api backend
- [ ] Implement 7 sections (mirroring TUI)
- [ ] Live theme preview
- [ ] Aesthetic settings visualization
- [ ] Cross-platform builds (Linux, macOS, Windows)

### Phase 3.3: Advanced Features

- [ ] Multi-host configuration management
- [ ] Configuration profiles marketplace (opt-in)
- [ ] Collaborative editing (shared profiles)
- [ ] Visual diff viewer
- [ ] Rollback UI (time-travel)

---

## Privacy & Ethics

### Data Sovereignty

- **All data stays local** unless user explicitly requests remote sync
- No telemetry, no analytics, no cloud dependencies
- User owns all configuration files

### Transparency

- Every sync event logged with full context
- User can audit all changes
- Clear indication of actor (user vs system)

### Consent

- Remote configuration requires explicit `--remote` flag
- SSH key usage requires existing trust relationship
- No automatic syncing without user initiation

---

## Conclusion

Anna's synchronization architecture is designed for **beauty, security, and user sovereignty**. By keeping all data local-first and requiring explicit consent for remote operations, we maintain Anna's core principle: **autonomy with consent**.

The synchronization layer enables seamless configuration management across multiple interfaces while preserving full audit trails and snapshot-based rollback capabilities.

---

🌸 **Anna v1.4 "Synchronization & Web GUI"** - *Harmony across interfaces, sovereignty over data.*
