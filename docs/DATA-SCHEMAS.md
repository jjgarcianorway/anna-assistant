# Anna Data Schemas v1.2+

> **Purpose**: Canonical data structure definitions for Anna's local state management
>
> **Principle**: Everything is local, deterministic, and auditable

---

## File Locations

```
~/.local/state/anna/          # User-specific state
├── facts.json                # System facts (read-only observations)
├── advice_state.json         # Advice lifecycle states
├── audit.jsonl               # Event log (append-only)
├── rollback_tokens.jsonl     # Rollback capabilities
└── snapshots/                # File/state backups
    ├── files/
    │   └── <hash-path>/
    │       ├── <timestamp>.orig
    │       └── <timestamp>.meta.json
    └── packages/
        └── <timestamp>.json

/var/lib/anna/                # System-wide state
├── facts.json                # System-level facts
└── wiki_index/               # Offline Arch Wiki FTS index
```

---

## 1. Facts Schema (`facts.json`)

**Purpose**: Immutable system observations, refreshed periodically

```json
{
  "version": "1.2.0",
  "collected_at": 1705334567,
  "hostname": "archbox",

  "hardware": {
    "cpu": {
      "model": "AMD Ryzen 9 5950X",
      "cores": 16,
      "threads": 32,
      "vendor": "AuthenticAMD",
      "flags": ["aes", "avx2", "sse4_2"]
    },
    "memory": {
      "total_gb": 64,
      "available_gb": 48
    },
    "gpu": [
      {
        "vendor": "NVIDIA",
        "model": "RTX 3080",
        "driver": "nvidia",
        "pci_id": "10de:2206"
      }
    ],
    "storage": [
      {
        "device": "/dev/nvme0n1",
        "type": "nvme",
        "size_gb": 1000,
        "filesystem": "btrfs",
        "mount": "/"
      }
    ]
  },

  "system": {
    "kernel": "6.7.1-arch1-1",
    "boot_loader": "systemd-boot",
    "init": "systemd",
    "hostname": "archbox",
    "timezone": "America/New_York",
    "locale": "en_US.UTF-8",
    "uptime_seconds": 86400
  },

  "packages": {
    "total": 1234,
    "explicit": 456,
    "orphans": ["old-lib-1", "unused-dep"],
    "aur": 78,
    "groups": {
      "base": ["linux", "systemd", "pacman"],
      "base-devel": ["gcc", "make", "binutils"],
      "nvidia": ["nvidia", "nvidia-utils"],
      "vulkan": ["vulkan-icd-loader"],
      "editors": ["vim", "neovim"]
    },
    "last_update": 1705330000
  },

  "services": {
    "enabled": ["sshd", "NetworkManager", "systemd-timesyncd"],
    "active": ["sshd", "NetworkManager"],
    "failed": [],
    "masked": []
  },

  "configs": {
    "files": [
      {
        "path": "/etc/pacman.conf",
        "checksum": "abc123def456...",
        "size": 4096,
        "modified": 1705330000
      },
      {
        "path": "~/.vimrc",
        "checksum": "def456abc789...",
        "size": 512,
        "modified": 1705320000
      }
    ]
  },

  "behavior": {
    "shell": {
      "type": "zsh",
      "history_size": 5000,
      "top_commands": [
        {"cmd": "git", "count": 450},
        {"cmd": "cargo", "count": 320},
        {"cmd": "vim", "count": 280}
      ]
    },
    "editors": {
      "primary": "neovim",
      "frequency": "daily"
    },
    "workflow": {
      "type": "developer",
      "languages": ["rust", "python"],
      "confidence": 0.85
    }
  },

  "network": {
    "interfaces": ["wlan0", "lo"],
    "firewall": {
      "installed": true,
      "type": "ufw",
      "enabled": false
    }
  }
}
```

**Collection Cadence**:
- On daemon start
- Every 4 hours (with ±15min jitter)
- On package change events (pacman hook)
- On manual trigger (`annactl advisor --refresh`)

---

## 2. Advice State Schema (`advice_state.json`)

**Purpose**: Track lifecycle of each piece of advice

```json
{
  "version": "1.2.0",
  "updated_at": 1705334567,

  "advice_states": [
    {
      "advice_id": "vim-syntax-highlighting",
      "state": "open",
      "first_seen": 1705330000,
      "last_checked": 1705334567,
      "check_count": 12,
      "state_history": [
        {
          "state": "open",
          "timestamp": 1705330000,
          "reason": "condition detected"
        }
      ]
    },
    {
      "advice_id": "microcode-intel-missing",
      "state": "applied_auto",
      "first_seen": 1705320000,
      "last_checked": 1705334567,
      "applied_at": 1705334500,
      "applied_by": "auto",
      "rollback_token_id": "microcode-intel-missing_1705334500",
      "state_history": [
        {
          "state": "open",
          "timestamp": 1705320000,
          "reason": "condition detected"
        },
        {
          "state": "applied_auto",
          "timestamp": 1705334500,
          "reason": "auto-applied via apply --auto"
        }
      ]
    },
    {
      "advice_id": "orphan-packages",
      "state": "applied_user",
      "first_seen": 1705310000,
      "last_checked": 1705334567,
      "resolved_at": 1705325000,
      "state_history": [
        {
          "state": "open",
          "timestamp": 1705310000,
          "reason": "45 orphan packages detected"
        },
        {
          "state": "applied_user",
          "timestamp": 1705325000,
          "reason": "condition no longer true, no matching rollback token"
        }
      ]
    },
    {
      "advice_id": "firewall-missing",
      "state": "ignored",
      "first_seen": 1705300000,
      "last_checked": 1705334567,
      "ignored_at": 1705305000,
      "ignored_reason": "user manages firewall manually",
      "state_history": [
        {
          "state": "open",
          "timestamp": 1705300000,
          "reason": "no firewall detected"
        },
        {
          "state": "ignored",
          "timestamp": 1705305000,
          "reason": "user manually ignored"
        }
      ]
    },
    {
      "advice_id": "swappiness-high",
      "state": "snoozed",
      "first_seen": 1705290000,
      "last_checked": 1705334567,
      "snoozed_at": 1705334000,
      "snoozed_until": 1707926000,
      "snooze_duration": "30d",
      "state_history": [
        {
          "state": "open",
          "timestamp": 1705290000,
          "reason": "swappiness=60 on 64GB system"
        },
        {
          "state": "snoozed",
          "timestamp": 1705334000,
          "reason": "user snoozed for 30 days"
        }
      ]
    },
    {
      "advice_id": "journald-size-unlimited",
      "state": "reverted_detected",
      "first_seen": 1705280000,
      "last_checked": 1705334567,
      "applied_at": 1705334400,
      "reverted_at": 1705334550,
      "state_history": [
        {
          "state": "open",
          "timestamp": 1705280000,
          "reason": "SystemMaxUse not set"
        },
        {
          "state": "applied_auto",
          "timestamp": 1705334400,
          "reason": "auto-applied"
        },
        {
          "state": "rolled_back",
          "timestamp": 1705334500,
          "reason": "user rolled back via rollback --last"
        },
        {
          "state": "reverted_detected",
          "timestamp": 1705334550,
          "reason": "condition true again after rollback"
        }
      ]
    }
  ]
}
```

**State Transitions**:

```
open → applied_auto     (Anna applied low-risk fix)
     → applied_user     (User fixed manually)
     → ignored          (User explicitly ignored)
     → snoozed          (User snoozed)

applied_auto → rolled_back       (Anna rolled back)
             → reverted_detected  (Condition true again)

applied_user → reverted_detected  (User reverted their change)

rolled_back → open               (Re-evaluate shows condition still true)
            → reverted_detected  (Same as above, more explicit)

snoozed → open                   (Snooze expired, condition still true)
        → applied_user           (Fixed while snoozed)

ignored → (stays ignored unless user un-ignores)
```

---

## 3. Audit Log Schema (`audit.jsonl`)

**Purpose**: Append-only event log for complete audit trail

Each line is a JSON object (JSONL format):

```jsonl
{"version":"1.2.0","timestamp":1705334500,"event":"advice_shown","actor":"advisor","advice_id":"microcode-intel-missing","category":"system","risk":"low","wiki_ref":"https://wiki.archlinux.org/title/Microcode"}
{"version":"1.2.0","timestamp":1705334567,"event":"apply_started","actor":"auto","advice_id":"microcode-intel-missing","fix_cmd":"sudo pacman -S intel-ucode && sudo grub-mkconfig -o /boot/grub/grub.cfg"}
{"version":"1.2.0","timestamp":1705334575,"event":"snapshot_created","actor":"apply","advice_id":"microcode-intel-missing","snapshot_type":"package","snapshot_path":"~/.local/state/anna/snapshots/packages/1705334575.json"}
{"version":"1.2.0","timestamp":1705334590,"event":"apply_succeeded","actor":"auto","advice_id":"microcode-intel-missing","duration_ms":23000,"exit_code":0,"rollback_token_id":"microcode-intel-missing_1705334590"}
{"version":"1.2.0","timestamp":1705334600,"event":"advice_state_changed","actor":"advisor","advice_id":"microcode-intel-missing","old_state":"open","new_state":"applied_auto"}
{"version":"1.2.0","timestamp":1705334700,"event":"rollback_started","actor":"user","advice_id":"microcode-intel-missing","rollback_strategy":"remove_packages"}
{"version":"1.2.0","timestamp":1705334710,"event":"rollback_succeeded","actor":"user","advice_id":"microcode-intel-missing","duration_ms":10000,"exit_code":0}
{"version":"1.2.0","timestamp":1705334720,"event":"advice_state_changed","actor":"advisor","advice_id":"microcode-intel-missing","old_state":"applied_auto","new_state":"rolled_back"}
{"version":"1.2.0","timestamp":1705334730,"event":"user_changed_state","actor":"user","advice_id":"firewall-missing","action":"ignore","reason":"user manages firewall manually"}
{"version":"1.2.0","timestamp":1705334740,"event":"user_changed_state","actor":"user","advice_id":"swappiness-high","action":"snooze","duration":"30d","until":1707926740}
```

**Event Types**:

| Event | Actor | Description |
|-------|-------|-------------|
| `advice_shown` | advisor | Advice presented to user |
| `apply_started` | auto/user | Apply command initiated |
| `apply_succeeded` | auto/user | Apply completed successfully |
| `apply_failed` | auto/user | Apply failed (includes error) |
| `snapshot_created` | apply | State snapshot captured |
| `rollback_started` | user | Rollback initiated |
| `rollback_succeeded` | user | Rollback completed |
| `rollback_failed` | user | Rollback failed |
| `advice_state_changed` | advisor | State transition occurred |
| `user_changed_state` | user | User action (ignore/snooze) |
| `condition_reevaluated` | advisor | Detector re-ran |
| `facts_collected` | daemon | Facts refresh completed |

---

## 4. Rollback Token Schema (`rollback_tokens.jsonl`)

**Purpose**: Enable safe rollback for every autonomous action

```jsonl
{"version":"1.2.0","token_id":"microcode-intel-missing_1705334590","advice_id":"microcode-intel-missing","created_at":1705334590,"executed_by":"auto","command":"sudo pacman -S intel-ucode && sudo grub-mkconfig -o /boot/grub/grub.cfg","success":true,"exit_code":0,"output":"...","rollback_strategy":"remove_packages","rollback_cmd":"sudo pacman -Rns intel-ucode","snapshot_refs":{"package":"~/.local/state/anna/snapshots/packages/1705334575.json"},"wiki_ref":"https://wiki.archlinux.org/title/Microcode"}
{"version":"1.2.0","token_id":"vim-syntax-highlighting_1705334800","advice_id":"vim-syntax-highlighting","created_at":1705334800,"executed_by":"auto","command":"mkdir -p ~/.config/nvim && printf 'syntax on\\nset number\\n' >> ~/.config/nvim/init.vim","success":true,"exit_code":0,"output":"","rollback_strategy":"file_restore","rollback_cmd":"restore_from_snapshot","snapshot_refs":{"file":"~/.local/state/anna/snapshots/files/home-user-.config-nvim-init.vim/1705334795.meta.json"},"wiki_ref":"https://wiki.archlinux.org/title/Vim#Configuration"}
{"version":"1.2.0","token_id":"journald-size-unlimited_1705334900","advice_id":"journald-size-unlimited","created_at":1705334900,"executed_by":"auto","command":"sudo sed -i 's/#SystemMaxUse=/SystemMaxUse=500M/' /etc/systemd/journald.conf && sudo systemctl restart systemd-journald","success":true,"exit_code":0,"output":"","rollback_strategy":"file_restore","rollback_cmd":"restore_from_snapshot","snapshot_refs":{"file":"~/.local/state/anna/snapshots/files/etc-systemd-journald.conf/1705334895.meta.json"},"wiki_ref":"https://wiki.archlinux.org/title/Systemd/Journal#Journal_size_limit"}
```

**Rollback Strategies**:

| Strategy | Description | Implementation |
|----------|-------------|----------------|
| `remove_packages` | Installed packages | `sudo pacman -Rns <packages>` |
| `file_restore` | Modified config file | Copy from snapshot + restore perms |
| `service_restart` | Service state change | `systemctl restart <service>` |
| `service_disable` | Service enabled | `systemctl disable <service>` |
| `command_inverse` | Custom inverse command | Execute stored inverse command |
| `manual_required` | Cannot auto-rollback | Show instructions, require manual |

---

## 5. Snapshot Metadata Schema

### Package Snapshot (`snapshots/packages/<timestamp>.json`)

```json
{
  "version": "1.2.0",
  "created_at": 1705334575,
  "snapshot_type": "package",
  "advice_id": "microcode-intel-missing",

  "packages": {
    "installed": [
      {
        "name": "linux",
        "version": "6.7.1.arch1-1",
        "size": 150000000,
        "install_date": 1705330000
      },
      {
        "name": "systemd",
        "version": "255.2-1",
        "size": 20000000,
        "install_date": 1705320000
      }
    ],
    "checksum": "sha256:abc123def456...",
    "total_count": 1234
  },

  "added_by_action": [],
  "removed_by_action": []
}
```

### File Snapshot (`snapshots/files/<hash-path>/<timestamp>.meta.json`)

```json
{
  "version": "1.2.0",
  "created_at": 1705334795,
  "snapshot_type": "file",
  "advice_id": "vim-syntax-highlighting",

  "file": {
    "original_path": "~/.config/nvim/init.vim",
    "resolved_path": "/home/user/.config/nvim/init.vim",
    "backup_path": "./1705334795.orig",
    "checksum": "sha256:original123...",
    "size": 512,
    "mode": 33188,
    "owner": 1000,
    "group": 1000,
    "modified": 1705320000,
    "existed_before": true
  },

  "modifications": {
    "type": "append",
    "lines_added": 2,
    "content_preview": "syntax on\\nset number"
  }
}
```

### Service Snapshot (`snapshots/services/<timestamp>.json`)

```json
{
  "version": "1.2.0",
  "created_at": 1705334850,
  "snapshot_type": "service",
  "advice_id": "firewall-enable",

  "services": [
    {
      "name": "ufw",
      "enabled": false,
      "active": false,
      "preset": "disabled"
    }
  ]
}
```

---

## 6. Rule Definition Schema

**Purpose**: Define advisor rules with detection and remediation

```json
{
  "rule_id": "vim-syntax-highlighting",
  "version": "1.0",
  "category": "editor-ux",
  "tags": ["editor", "vim", "developer"],
  "risk": "low",

  "metadata": {
    "title": "Vim syntax highlighting not enabled",
    "description": "Enable syntax highlighting in Vim for better code readability",
    "impact": "Improves development experience with color-coded syntax",
    "wiki_ref": "https://wiki.archlinux.org/title/Vim#Configuration"
  },

  "detection": {
    "type": "file_regex",
    "files": [
      "~/.vimrc",
      "~/.config/nvim/init.vim",
      "~/.config/nvim/init.lua"
    ],
    "pattern": "^\\s*syntax\\s+on\\b",
    "invert": true,
    "comment": "Check if 'syntax on' is present in any Vim config"
  },

  "condition": {
    "type": "all",
    "checks": [
      {
        "type": "package_installed",
        "packages": ["vim", "neovim"],
        "operator": "any"
      },
      {
        "type": "detection_failed",
        "detection_ref": "main"
      }
    ]
  },

  "remediation": {
    "fix_cmd": "mkdir -p ~/.config/nvim && printf 'syntax on\\nset number\\nset ruler\\n' >> ~/.config/nvim/init.vim",
    "fix_description": "Create Neovim config with syntax highlighting enabled",
    "requires_sudo": false,
    "idempotent": true,
    "snapshot_type": "file",
    "snapshot_paths": ["~/.config/nvim/init.vim"]
  },

  "rollback": {
    "strategy": "file_restore",
    "automatic": true,
    "description": "Restore Neovim config to previous state"
  },

  "priority": {
    "base": 50,
    "workflow_modifiers": {
      "developer": 20,
      "server": -10
    }
  }
}
```

**Detection Types**:

| Type | Description | Example |
|------|-------------|---------|
| `file_regex` | Regex match in file(s) | Check for `syntax on` in vimrc |
| `file_exists` | File existence check | Check if ~/.gitconfig exists |
| `file_checksum` | File content hash | Detect modified config |
| `package_installed` | Package in pacman DB | Check if `ufw` installed |
| `package_missing` | Package not installed | Check if `intel-ucode` missing |
| `service_enabled` | Service enabled in systemd | Check if `sshd` enabled |
| `service_active` | Service currently running | Check if `firewalld` active |
| `command_output` | Shell command result | Check swappiness value |
| `config_value` | Config file key=value | Check `SystemMaxUse` in journald |

---

## 7. CLI Commands with State Management

### Advisor Commands

```bash
# Show open advice (default)
annactl advisor

# Show all advice including resolved
annactl advisor --all

# Show ignored advice
annactl advisor --show-ignored

# Filter by category
annactl advisor --category editor-ux

# Explain specific advice
annactl advisor --explain vim-syntax-highlighting

# Ignore advice permanently
annactl advisor --ignore vim-syntax-highlighting --reason "using defaults"

# Snooze advice temporarily
annactl advisor --snooze swappiness-high 30d

# Un-snooze advice
annactl advisor --unsnooze swappiness-high

# Refresh facts and re-evaluate
annactl advisor --refresh
```

### Apply Commands (State Updates)

```bash
# Apply interactively (asks for each)
annactl apply

# Auto-apply low-risk only
annactl apply --auto --yes

# Apply specific by ID
annactl apply --id vim-syntax-highlighting

# Dry-run (preview)
annactl apply --dry-run
```

**State Changes**:
- Creates rollback token
- Updates `advice_state.json`: `open` → `applied_auto`
- Logs to `audit.jsonl`: `apply_started`, `apply_succeeded`, `advice_state_changed`
- Creates snapshot in `snapshots/`

### Rollback Commands (State Updates)

```bash
# Rollback last action
annactl rollback --last

# Rollback specific action
annactl rollback --id microcode-intel-missing_1705334590

# Show rollback history
annactl rollback --list
```

**State Changes**:
- Restores from snapshot
- Updates `advice_state.json`: `applied_auto` → `rolled_back`
- Logs to `audit.jsonl`: `rollback_started`, `rollback_succeeded`, `advice_state_changed`
- Removes rollback token
- Re-evaluates condition: if still true, transitions `rolled_back` → `reverted_detected`

---

## 8. Re-evaluation Logic

**Triggers**:
1. Daemon start
2. Every 4 hours (±15min jitter)
3. Pacman hook (package changes)
4. Config file change (inotify)
5. Manual refresh (`annactl advisor --refresh`)

**Process**:

```
For each rule:
  1. Run detection check
  2. Get current advice state
  3. Apply transition logic:

     if condition == TRUE:
       if state == null:
         state = "open"
         log: advice_shown
       elif state == "applied_auto":
         state = "reverted_detected"
         log: advice_state_changed
       elif state == "applied_user":
         state = "reverted_detected"
         log: advice_state_changed
       elif state == "rolled_back":
         state = "reverted_detected"
         log: advice_state_changed
       elif state == "ignored":
         # stay ignored
       elif state == "snoozed":
         if now > snoozed_until:
           state = "open"
           log: advice_state_changed
         # else stay snoozed

     elif condition == FALSE:
       if state == "open":
         # Check for rollback token
         if rollback_token_exists():
           # Anna applied it
           state = "applied_auto"
         else:
           # User fixed it manually
           state = "applied_user"
         log: advice_state_changed
       elif state == "applied_auto":
         # stay applied_auto
       elif state == "applied_user":
         # stay applied_user
```

---

## 9. Privacy & Retention

**Privacy Guarantees**:
- ✅ All data stored locally (`~/.local/state/anna/`, `/var/lib/anna/`)
- ✅ No network transmission
- ✅ Shell history: Only command counts/frequency, not content
- ✅ Behavioral analysis: Opt-in via config
- ✅ User can inspect all data (`annactl status --data`)

**Retention Policy** (configurable):

```toml
# ~/.config/anna/config.toml
[retention]
snapshots_max_age_days = 30
snapshots_max_count = 100
audit_log_max_size_mb = 50
rollback_tokens_max_age_days = 90
```

**Cleanup** (`annactl doctor clean`):
- Remove snapshots older than 30 days
- Remove rollback tokens with no active advice
- Rotate audit log when > 50MB
- Compress old facts

---

## 10. Example: Complete Lifecycle

**T=0: Detection**
```json
// facts.json updated
{"packages": {"groups": {"editors": ["vim"]}}}

// Detection runs
Rule: vim-syntax-highlighting → condition TRUE

// advice_state.json updated
{"advice_id": "vim-syntax-highlighting", "state": "open", "first_seen": 1705334500}

// audit.jsonl appended
{"event": "advice_shown", "advice_id": "vim-syntax-highlighting"}
```

**T=10: User applies via Anna**
```bash
$ annactl apply --id vim-syntax-highlighting
```

```json
// Snapshot created
// snapshots/files/.../1705334510.meta.json + 1705334510.orig

// audit.jsonl
{"event": "apply_started", "advice_id": "vim-syntax-highlighting"}
{"event": "snapshot_created", "snapshot_type": "file"}
{"event": "apply_succeeded", "exit_code": 0}

// rollback_tokens.jsonl
{"token_id": "vim-syntax-highlighting_1705334520", "advice_id": "vim-syntax-highlighting"}

// advice_state.json updated
{"advice_id": "vim-syntax-highlighting", "state": "applied_auto"}

// audit.jsonl
{"event": "advice_state_changed", "old_state": "open", "new_state": "applied_auto"}
```

**T=20: User rolls back**
```bash
$ annactl rollback --last
```

```json
// audit.jsonl
{"event": "rollback_started", "advice_id": "vim-syntax-highlighting"}
{"event": "rollback_succeeded"}

// File restored from snapshot
// rollback token removed

// advice_state.json updated
{"advice_id": "vim-syntax-highlighting", "state": "rolled_back"}

// audit.jsonl
{"event": "advice_state_changed", "old_state": "applied_auto", "new_state": "rolled_back"}
```

**T=30: Re-evaluation detects revert**
```json
// Detection runs again
Rule: vim-syntax-highlighting → condition TRUE (syntax on still missing)

// advice_state.json updated
{"advice_id": "vim-syntax-highlighting", "state": "reverted_detected"}

// audit.jsonl
{"event": "condition_reevaluated", "result": "true"}
{"event": "advice_state_changed", "old_state": "rolled_back", "new_state": "reverted_detected"}
```

**T=40: User ignores**
```bash
$ annactl advisor --ignore vim-syntax-highlighting --reason "prefer defaults"
```

```json
// advice_state.json updated
{"advice_id": "vim-syntax-highlighting", "state": "ignored", "ignored_reason": "prefer defaults"}

// audit.jsonl
{"event": "user_changed_state", "action": "ignore", "reason": "prefer defaults"}
```

---

## Summary

This schema provides:

1. **Deterministic State**: Every piece of advice has a clear lifecycle
2. **Complete Audit**: Every action logged with wiki citations
3. **Safe Rollback**: Snapshots + tokens enable true reversal
4. **Privacy-First**: All data local, behavioral analysis opt-in
5. **User Control**: Ignore, snooze, un-ignore, refresh
6. **Re-evaluation**: Detects user changes and reverts

**Implementation Files**:
- `src/annad/src/facts_collector.rs` - Collect system facts
- `src/annad/src/advice_state.rs` - Manage advice lifecycle
- `src/annad/src/detection_engine.rs` - Run rule detectors
- `src/annad/src/snapshot_engine.rs` - Create/restore snapshots
- `src/annactl/src/apply_cmd.rs` - Apply with state tracking
- `src/annactl/src/rollback_cmd.rs` - Rollback with state tracking
- `src/annactl/src/advisor_cmd.rs` - Show advice + manage state

**File Paths**:
- Facts: `~/.local/state/anna/facts.json`
- State: `~/.local/state/anna/advice_state.json`
- Audit: `~/.local/state/anna/audit.jsonl`
- Tokens: `~/.local/state/anna/rollback_tokens.jsonl`
- Snapshots: `~/.local/state/anna/snapshots/`

---

**Everything is local. Everything is auditable. Everything is reversible.** 🌸
