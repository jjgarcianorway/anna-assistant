# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Project Overview

**Anna Assistant v1.0 "Hildegard"** is a production-grade, autonomy-first, Wiki-native, rollback-safe system assistant for Arch Linux. Built in Rust, it provides intelligent system recommendations grounded in the Arch Wiki with safe autonomous action capabilities.

**Architecture**: Client-daemon model with strict privilege separation
- **annad**: Root daemon handling privileged operations via Unix socket
- **annactl**: Non-privileged CLI client for user interaction
- **anna_common**: Shared library for beautiful output and common utilities

**Design Philosophy**: Beauty, Intelligence, Safety
- All output uses the `anna_common::beautiful` library for consistent visual design
- Deep system awareness through hardware/software/storage profiling
- Autonomous actions with complete rollback and audit trails

---

## Essential Commands

### Build and Test
```bash
# Build release binaries
cargo build --release
# or
make build

# Run unit tests
cargo test --all
# or
make test

# Format code
cargo fmt --all
# or
make fmt

# Run linter (clippy)
cargo clippy --all-targets -- -D warnings
# or
make lint

# Run smoke tests (requires annad installed)
./tests/smoke_v0122.sh
# or
make smoke
```

### Development Workflow
```bash
# Install from local build (requires sudo)
sudo ./scripts/install.sh --from-local
# or
make install

# Check version consistency across files
./scripts/check-version-consistency.sh
# or
make check-version

# Uninstall Anna
sudo ./scripts/uninstall.sh
# or
make uninstall
```

### Release Management
```bash
# Bump version (updates VERSION file and Cargo.toml)
make bump VERSION=v1.0.0

# Create and push release (transactional)
./scripts/release.sh
# or
make release
```

### Service Management
```bash
# Check daemon status
sudo systemctl status annad

# View daemon logs
sudo journalctl -u annad -f

# Restart daemon after changes
sudo systemctl restart annad

# Verify socket permissions
ls -la /run/anna/annad.sock
# Should show: srwxrwx--- 1 anna anna (mode 0770)
```

### Running Anna
```bash
# Core commands
annactl version
annactl status
annactl doctor
annactl advisor
annactl report
annactl apply --dry-run

# Experimental commands (require ANNA_EXPERIMENTAL=1)
ANNA_EXPERIMENTAL=1 annactl radar
```

---

## Architecture

### Workspace Structure
This is a Cargo workspace with three members:
- `src/annad/` - Daemon binary (privileged, runs as root)
- `src/annactl/` - CLI client binary (unprivileged)
- `src/anna_common/` - Shared library

Version is centrally managed in workspace `Cargo.toml` at `[workspace.package]` and referenced by members using `version.workspace = true`.

### Communication Model
- **IPC**: JSON-RPC over Unix socket at `/run/anna/annad.sock`
- **Socket Permissions**: Mode 0770, group 'anna'
- **User Access**: Users must be in 'anna' group to communicate with daemon
- **Protocol**: Async Tokio-based RPC handlers in `annad/src/main.rs`

### Advisor System
The core intelligence is in `src/annad/src/advisor_v13.rs`:
- 20+ deterministic rules analyzing system state
- Each rule produces `Advice` with:
  - Risk level (Low/Medium/High)
  - Fix command for autonomous execution
  - Arch Wiki citations (`refs` field)
  - Detailed explanation and rationale
- Three input sources:
  - `HardwareProfile` - CPU, GPU, RAM, storage detection
  - `PackageInventory` - Installed packages, orphans, services
  - `BtrfsProfile` - Filesystem layout, snapshots, compression

### Autonomous Actions (v1.1+)
`src/annactl/src/apply_cmd.rs` implements safe command execution:
- **Risk Filtering**: Only auto-applies "Low" risk actions
- **Rollback Tokens**: Every action creates a rollback token in `~/.local/state/anna/rollback_tokens.jsonl`
- **Audit Logging**: All actions logged to `~/.local/state/anna/audit.jsonl`
- **State Snapshots**: Captures before/after state for future rollback capability

### Beautiful Output Library
`src/anna_common/src/beautiful.rs` provides consistent UI primitives:
- Pastel color palette with semantic meaning
- Rounded box drawing (╭─╮ ╰─╯)
- Unicode symbols (✓ ✗ ⚠ ℹ → ⏳)
- **NEVER use raw ANSI codes** in user-facing commands

### Experimental Features
Commands under development are gated behind `ANNA_EXPERIMENTAL=1`:
- Hidden from `--help` by default using `#[command(hide = true)]`
- Runtime guard using `require_experimental!()` macro
- Located in various `*_cmd.rs` files in annactl

---

## Code Patterns

### Adding a New CLI Command
1. Create `src/annactl/src/my_cmd.rs` with function signature `pub fn run_my_command() -> Result<()>`
2. Use `anna_common::{header, section, status, Level, TermCaps}` for output
3. Add module declaration in `src/annactl/src/main.rs`
4. Add to `Commands` enum in main.rs with doc comment for help text
5. Add handler in main match expression
6. For experimental features, add `#[command(hide = true)]` and `require_experimental!()` guard

### Adding Advisor Rules
In `src/annad/src/advisor_v13.rs`:
1. Create function `fn check_my_rule(hw: &HardwareProfile, pkg: &PackageInventory) -> Vec<Advice>`
2. Every rule MUST include:
   - Unique `id` string
   - `fix_cmd` with valid shell command
   - `fix_risk` starting with "Low", "Medium", or "High"
   - `refs` with Arch Wiki URL(s)
3. Add to the main advisor run function's rule execution list
4. Risk levels determine autonomous execution eligibility:
   - **Low**: Safe for auto-apply (install packages, clean cache)
   - **Medium**: Requires confirmation (remove packages)
   - **High**: Manual only (bootloader, kernel changes)

### Error Handling
Use `anyhow::Result` for all fallible functions:
```rust
use anyhow::{Context, Result};

fn my_function() -> Result<()> {
    let value = read_config()
        .context("Failed to read config.toml")?;
    Ok(())
}
```

### Async Operations
Daemon uses Tokio async runtime:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Async code here
}
```

---

## Testing Strategy

### Unit Tests
- Standard Rust tests with `cargo test`
- Located in `#[cfg(test)]` modules within source files

### Smoke Tests
- `tests/smoke_v0122.sh` - Basic CLI validation
- `tests/arch_advisor_smoke.sh` - Advisor engine testing
- `tests/arch_btrfs_smoke.sh` - Btrfs detection testing
- `tests/verify_v0122.sh` - Comprehensive system validation
- `tests/verify_socket_fix.sh` - Socket permissions testing

### Integration Tests
- Require annad installed and running
- Test actual RPC communication over Unix socket
- Validate privilege separation and permission model

---

## Version Management

**Single Source of Truth**: `Cargo.toml` workspace version
- Current version: `1.0.0-rc.18`
- Format: `MAJOR.MINOR.PATCH[-rc.N]`
- RC releases bump automatically via `scripts/release.sh`

**Version Consistency**:
- `scripts/check-version-consistency.sh` validates version is consistent across:
  - Workspace Cargo.toml
  - README.md version badge
  - CHANGELOG.md latest entry

**Bumping Versions**:
```bash
make bump VERSION=v1.0.0
# Updates VERSION file, Cargo.toml, prompts for CHANGELOG update
```

---

## Security Model

### Privilege Separation
- Daemon runs as root via systemd service
- CLI never requires sudo, communicates via socket
- Socket permissions restrict access to 'anna' group members
- All privileged operations go through RPC boundary

### Command Execution Safety
- Shell commands executed via `sh -c` (no shell injection)
- Risk levels gate autonomous execution
- All outputs captured for logging
- Exit codes tracked for success/failure detection

### Audit Trail
- Location: `~/.local/state/anna/audit.jsonl`
- Immutable append-only log
- Fields: timestamp, actor, action, result, details
- Used for compliance and debugging

---

## File Locations

**State Directory**: `~/.local/state/anna/`
- `actions.jsonl` - Registered actions
- `action_results.jsonl` - Action execution history
- `rollback_tokens.jsonl` - Rollback tokens for apply
- `audit.jsonl` - Audit log

**Config Directory**: `/etc/anna/`
- `config.toml` - Main configuration
- `policy.toml` - Policy definitions
- `CAPABILITIES.toml` - System capabilities
- `policies.d/*.yaml` - Additional policy files

**Logs**: `/var/log/anna/`
- `annad.log` - Daemon log
- `install.log` - Installation history
- `doctor.log` - Health check history

**Runtime**: `/run/anna/`
- `annad.sock` - Unix socket for RPC (mode 0770)
- `annad.pid` - Daemon PID file

---

## Common Development Tasks

### Testing Socket Communication
```bash
# Verify socket exists and has correct permissions
ls -la /run/anna/annad.sock

# Test basic RPC
annactl ping

# Test with experimental features
ANNA_EXPERIMENTAL=1 annactl radar
```

### Debugging Advisor Rules
```bash
# Run advisor with full output
annactl advisor

# Check hardware detection
ANNA_EXPERIMENTAL=1 annactl hardware-profile

# Check package analysis
ANNA_EXPERIMENTAL=1 annactl package-analysis
```

### Testing Apply/Rollback
```bash
# Preview what would be applied
annactl apply --dry-run

# Apply low-risk actions only
annactl apply --auto --yes

# Check rollback tokens
cat ~/.local/state/anna/rollback_tokens.jsonl

# Check audit log
cat ~/.local/state/anna/audit.jsonl
```

### Validating Beautiful Output
- All commands should use `anna_common::beautiful` primitives
- Run command and verify:
  - Consistent color scheme (pastels)
  - Rounded box drawing characters
  - Unicode status symbols
  - No raw ANSI escape codes visible

---

## Design Principles

1. **Never use raw ANSI codes** - Always use `anna_common::beautiful`
2. **Every autonomous action needs rollback token** - No exceptions
3. **All advisor rules cite Arch Wiki** - No recommendations without references
4. **Low-risk only for auto-apply** - Medium/High require explicit user consent
5. **Experimental features hidden by default** - Keep production CLI clean
6. **Privilege separation is sacred** - CLI never elevated, daemon always root
7. **Unix socket is the only IPC** - No HTTP, no DBus, no other channels

---

## Dependencies

### Core Libraries
- `tokio` - Async runtime for daemon
- `serde`/`serde_json` - Serialization for RPC
- `anyhow` - Error handling
- `clap` - CLI argument parsing
- `tracing` - Structured logging

### System Integration
- `nix` - Unix socket handling
- `sysinfo` - System metrics
- `rusqlite` - Local data persistence
- `toml`/`serde_yaml` - Config parsing

### UI/UX
- `ratatui` - TUI framework for experimental features
- `crossterm` - Terminal manipulation

---

## Troubleshooting

### Build Issues
```bash
# Clean and rebuild
cargo clean && cargo build --release

# Check for format issues
cargo fmt --all --check

# Verify clippy passes
cargo clippy --all-targets -- -D warnings
```

### Daemon Communication Issues
```bash
# Check daemon is running
sudo systemctl status annad

# Check user is in anna group
groups | grep anna
# If not: sudo usermod -aG anna $USER (then logout/login)

# Check socket permissions
stat /run/anna/annad.sock

# View recent daemon logs
sudo journalctl -u annad -n 50 --no-pager
```

### Version Inconsistencies
```bash
# Run version consistency check
./scripts/check-version-consistency.sh

# Verify versions match
annad --version
annactl version
grep version Cargo.toml
```
