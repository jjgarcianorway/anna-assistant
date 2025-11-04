# Anna Architecture — v1.1 "Advisor Intelligence"

> A self-healing, Arch Wiki–guided, aesthetically consistent system companion
> that can act safely on its own advice while maintaining complete auditability
> and rollback integrity.

---

## Design Philosophy

Anna embodies three core principles:

1. **Beauty** — Calm, elegant, consistent visual output
2. **Intelligence** — Deep system awareness and contextual recommendations
3. **Safety** — Autonomous action with complete rollback and audit trails

Every action, every output, and every log reflects these principles.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      User Interface                         │
│  annactl (CLI) — 6 core commands + experimental features   │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ JSON-RPC over Unix socket
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                    Anna Daemon (annad)                       │
│                                                              │
│  ┌────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   Hardware     │  │    Software     │  │   Storage    │ │
│  │   Detection    │  │    Analysis     │  │   Intel      │ │
│  └────────┬───────┘  └────────┬────────┘  └──────┬───────┘ │
│           │                   │                   │         │
│           └───────────────────┼───────────────────┘         │
│                               │                             │
│                      ┌────────▼─────────┐                   │
│                      │  Advisor Engine  │                   │
│                      │  (20+ rules)     │                   │
│                      └────────┬─────────┘                   │
│                               │                             │
│                      ┌────────▼─────────┐                   │
│                      │  Recommendations │                   │
│                      │  + Arch Wiki refs│                   │
│                      └──────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
                       │
                       │ Actionable advice
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                  Autonomous Actions                          │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Risk Filter  │→ │  Execute     │→ │  Rollback Token  │  │
│  │ (low only)   │  │  + Audit Log │  │  + State Snapshot│  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. CLI Surface (annactl)

**Production Commands** (always visible):
- `version` — Version info
- `status` — System status
- `doctor` — Health checks
- `advisor` — System advice (Arch Wiki–guided)
- `report` — Health report with recommendations
- `apply` — **NEW** — Apply recommendations automatically

**Experimental Commands** (require `ANNA_EXPERIMENTAL=1`):
- Hidden by default for clean UX
- 17 development/diagnostic commands
- Gated with runtime guards

**Beautiful Output Library** (`anna_common::beautiful`):
- Consistent pastel color palette
- Rounded box drawing (╭─╮ ╰─╯)
- Unicode symbols (✓ ✗ ⚠ ℹ → ⏳)
- No raw ANSI codes in user-facing commands

---

### 2. Advisor System

**System Awareness**:
1. **Hardware Detection** (`HardwareProfile`)
   - CPU model, cores, frequency
   - GPU vendor, driver (NVIDIA/AMD/Intel)
   - RAM, storage (NVMe, SSD, HDD)
   - Firmware and kernel version

2. **Software Analysis** (`PackageInventory`)
   - Installed packages by group
   - Base, base-devel, AUR, nvidia, vulkan
   - Orphan packages
   - Service status

3. **Storage Intelligence** (`BtrfsProfile`)
   - Filesystem layout
   - Subvolumes and snapshots
   - Compression status
   - Scrub and balance recommendations

**Advisor Engine** (`advisor_v13.rs`):
- 20+ deterministic rules
- 10 system rules (drivers, microcode, power management)
- 10 Btrfs rules (snapshots, compression, scrub)
- Each rule produces `Advice`:
  ```rust
  pub struct Advice {
      pub id: String,
      pub level: Level,           // Info, Warn, Error
      pub category: String,        // "drivers", "graphics", "system"
      pub title: String,
      pub reason: String,
      pub action: String,
      pub explain: Option<String>,
      pub fix_cmd: Option<String>,  // Command to fix
      pub fix_risk: Option<String>, // "Low", "Medium", "High"
      pub refs: Vec<String>,        // Arch Wiki URLs
  }
  ```

**Arch Wiki Integration**:
- Every recommendation includes `refs: Vec<String>`
- Citations to official Arch Wiki articles
- Examples:
  - `https://wiki.archlinux.org/title/NVIDIA`
  - `https://wiki.archlinux.org/title/Microcode`
  - `https://wiki.archlinux.org/title/Btrfs`

---

### 3. Autonomous Actions — **NEW in v1.1**

**Apply Command** (`apply_cmd.rs`):
```bash
annactl apply                    # Interactive mode (ask for each)
annactl apply --dry-run          # Preview what would happen
annactl apply --auto --yes       # Auto-apply low-risk only
annactl apply --id microcode-amd # Apply specific by ID
```

**Safety Model**:
1. **Risk Filtering**:
   - Only applies advice with `fix_risk` starting with `"Low"` or containing `"safe"`
   - Medium/High risk requires manual confirmation

2. **Rollback Tokens**:
   ```rust
   pub struct RollbackToken {
       pub advice_id: String,
       pub executed_at: u64,
       pub command: String,
       pub success: bool,
       pub output: String,
       pub state_snapshot: Option<StateSnapshot>,
   }
   ```
   - Saved to `~/.local/state/anna/rollback_tokens.jsonl`
   - Used for future rollback capability

3. **Audit Logging**:
   - Every action logged to audit trail
   - Actor: "advisor" (for autonomous actions)
   - Result: success/failure
   - Details: command executed + Arch Wiki reference

**Execution Modes**:
- **Interactive** — Ask user for each recommendation (default)
- **DryRun** — Show what would be applied without executing
- **Auto** — Apply all low-risk recommendations automatically
- **Specific** — Apply single recommendation by ID

---

## Data Flow

### Advisor Flow
```
User runs: annactl advisor
  ↓
1. Connect to annad via Unix socket
2. Send JSON-RPC: advisor_run
3. Daemon collects:
   - Hardware profile (lshw, lscpu, lsblk)
   - Package inventory (pacman -Q)
   - Btrfs profile (btrfs fi show, btrfs sub list)
4. Run 20+ advisor rules
5. Generate Advice list with Arch Wiki refs
6. Return to annactl
7. Display with beautiful formatting
```

### Apply Flow
```
User runs: annactl apply --auto --yes
  ↓
1. Fetch advisor recommendations
2. Filter to actionable (has fix_cmd)
3. Filter to low-risk only
4. For each recommendation:
   a. Execute fix_cmd via shell
   b. Capture output + exit code
   c. Create rollback token
   d. Save token to disk
   e. Log to audit trail
5. Display summary (✓ Applied: N, ✗ Failed: M)
```

---

## Security & Safety

**Principle: Never run destructive commands without explicit consent**

1. **Risk Levels**:
   - **Low** — Safe for auto-apply (e.g., install microcode, clean cache)
   - **Medium** — Requires confirmation (e.g., remove orphan packages)
   - **High** — Manual only (e.g., bootloader changes)

2. **Execution Guards**:
   - Commands run via `sh -c` (no shell injection)
   - Capture stdout + stderr for logging
   - Track exit code for success detection

3. **Rollback Support**:
   - Every action creates a rollback token
   - Future: `annactl rollback --last` to undo
   - State snapshots (packages installed, files modified)

4. **Audit Trail**:
   - Location: `~/.local/state/anna/audit.jsonl`
   - Fields: timestamp, actor, action, result, details
   - Immutable append-only log

---

## File Locations

**State Directory**: `~/.local/state/anna/`
- `actions.jsonl` — Registered actions
- `action_results.jsonl` — Action execution history
- `rollback_tokens.jsonl` — **NEW** — Rollback tokens for apply
- `audit.jsonl` — **NEW** — Audit log

**Config Directory**: `/etc/anna/`
- `config.toml` — Main configuration
- `policies.d/*.yaml` — Policy files

**Logs**: `/var/log/anna/`
- `annad.log` — Daemon log
- `install.log` — Installation history
- `doctor.log` — Health check history

---

## Future Enhancements

1. **Rollback Command**:
   ```bash
   annactl rollback --last           # Undo last applied action
   annactl rollback --id <advice_id> # Undo specific action
   ```

2. **State Snapshots**:
   - Capture package list before/after
   - Capture file checksums for modified files
   - Enable true rollback, not just undo

3. **Behavioral Analysis**:
   - Parse shell history to infer workflow patterns
   - Classify machine type (dev, gaming, server)
   - Tailor recommendations to user behavior

4. **Multi-Distro Support**:
   - Debian/Ubuntu advisor rules
   - Fedora/RHEL advisor rules
   - OpenSUSE advisor rules

5. **ML-Based Prediction**:
   - Learn from user accept/reject patterns
   - Predict which recommendations user will accept
   - Prioritize recommendations by relevance

---

## Code Structure

```
anna-assistant/
├── src/
│   ├── annactl/           # CLI binary
│   │   ├── main.rs        # Command routing
│   │   ├── advisor_cmd.rs # Advisor command
│   │   ├── apply_cmd.rs   # **NEW** Apply command
│   │   ├── doctor_cmd.rs  # Doctor command
│   │   ├── report_cmd.rs  # Report command
│   │   ├── status_cmd.rs  # Status command
│   │   └── ...
│   ├── annad/             # Daemon binary
│   │   ├── main.rs        # RPC server
│   │   ├── advisor_v13.rs # Advisor engine
│   │   ├── hardware_profile.rs
│   │   ├── package_analysis.rs
│   │   ├── storage_btrfs.rs
│   │   └── ...
│   └── anna_common/       # Shared library
│       ├── beautiful.rs   # Beautiful output
│       ├── tui.rs         # Terminal UI primitives
│       └── ...
├── docs/
│   ├── ARCHITECTURE.md    # This file
│   └── ...
└── scripts/
    ├── install.sh         # Installer
    └── ...
```

---

## Testing

**Manual Testing**:
```bash
# 1. Build
cargo build --release

# 2. Test CLI surface
./target/release/annactl --help              # 6 commands visible
./target/release/annactl apply --help        # Apply command visible

# 3. Test experimental guard
./target/release/annactl radar               # Error: experimental
ANNA_EXPERIMENTAL=1 annactl radar            # Works

# 4. Test apply (requires annad running)
# sudo systemctl start annad
./target/release/annactl advisor             # See recommendations
./target/release/annactl apply --dry-run     # Preview
./target/release/annactl apply --auto --yes  # Auto-apply low-risk
```

**Validation Checklist**:
- [ ] CLI shows 6 core commands
- [ ] Experimental commands hidden by default
- [ ] Apply command visible and documented
- [ ] Apply --dry-run shows preview
- [ ] Apply --auto filters to low-risk only
- [ ] Rollback tokens saved to disk
- [ ] Audit log created
- [ ] Beautiful output everywhere

---

## Version History

### v1.1 "Advisor Intelligence" (Current)
- **NEW**: `apply` command for autonomous action
- Rollback token system
- Audit logging for all autonomous actions
- Risk-based filtering (low-risk auto-apply)
- Arch Wiki citations in all recommendations

### v1.0 "Production Ready"
- CLI simplification (6 core commands)
- Experimental command gating
- Beautiful output library standardization
- 20+ advisor rules with Arch Wiki refs
- Hardware/software/storage awareness

---

## Contributing

See `CONTRIBUTING.md` for development guidelines.

**Key Design Decisions**:
1. Only low-risk actions may auto-apply
2. Every autonomous action must have a rollback token
3. All recommendations must cite Arch Wiki
4. Beautiful output everywhere — no raw ANSI codes
5. Experimental features behind `ANNA_EXPERIMENTAL=1`

---

## License

See `LICENSE` file.

---

**Built with ❤️ and Rust**
**Guided by the Arch Wiki**
**Designed for humans**
