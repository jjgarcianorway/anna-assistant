# Recipe Architecture

**Version**: Beta.152
**Last Updated**: 2025-11-20
**Status**: Production

---

## Overview

The Recipe system provides deterministic, zero-hallucination ActionPlans for common Arch Linux system administration tasks. Instead of relying on LLM-generated plans (which can be inconsistent or incorrect), recipes are hand-written Rust code that generates validated, testable ActionPlans.

### Key Principles

1. **Deterministic**: Same input always produces same output
2. **Tested**: Every recipe has comprehensive unit tests
3. **Safe**: Explicit risk levels, rollback plans, and confirmation flows
4. **Transparent**: All commands visible to user before execution
5. **Telemetry-aware**: Uses real system state to generate context-appropriate commands

---

## Architecture

### Directory Structure

```
crates/annactl/src/recipes/
├── mod.rs              # Recipe registry and dispatcher
├── docker.rs           # Docker installation
├── neovim.rs           # Neovim setup
├── packages.rs         # Package repair
├── wallpaper.rs        # Wallpaper management
├── systemd.rs          # Service management (Beta.152)
├── network.rs          # Network diagnostics (Beta.152)
├── system_update.rs    # System updates (Beta.152)
└── aur.rs              # AUR package management (Beta.152)
```

### Recipe Module Pattern

Every recipe module follows this structure:

```rust
pub struct RecipeName;

impl RecipeName {
    /// Pattern matching - does this recipe handle the user's request?
    pub fn matches_request(user_input: &str) -> bool {
        // Keyword matching logic
    }

    /// Generate ActionPlan for matched request
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        // Build structured plan with:
        // - Analysis
        // - Goals
        // - Necessary checks
        // - Command plan (with risk levels)
        // - Rollback plan
        // - User notes
        // - Metadata
    }
}

#[cfg(test)]
mod tests {
    // Comprehensive tests for pattern matching and plan generation
}
```

---

## Recipe Registry (mod.rs)

### try_recipe_match()

The recipe dispatcher tries each recipe in order of specificity:

```rust
pub fn try_recipe_match(
    user_input: &str,
    telemetry: &HashMap<String, String>,
) -> Option<Result<ActionPlan>>
```

**Match Order** (Beta.152):
1. AUR recipes (very specific - "install X from AUR", "yay", "paru")
2. System update recipes ("update system", "check for updates")
3. Systemd service management ("enable NetworkManager", "restart bluetooth")
4. Network diagnostics ("check internet", "why is my wifi not working")
5. Docker installation (Beta.151)
6. Wallpaper management (Beta.151)
7. Neovim installation (Beta.151)
8. Package repair (Beta.151)

If no recipe matches, returns `None` and the planner falls back to LLM-generated ActionPlan.

---

## Current Recipes

### Beta.151 Recipes

#### docker.rs
**Purpose**: Install and configure Docker with user group access

**Matches**:
- "install docker"
- "setup docker and enable it"
- "get container runtime"

**Actions**:
1. Install docker package
2. Enable service on boot
3. Start service immediately
4. Add user to docker group
5. Verify installation

**Risk Level**: MEDIUM
**Requires Confirmation**: Yes

---

#### wallpaper.rs
**Purpose**: Change desktop wallpaper (DE/WM-aware)

**Matches**:
- "change my wallpaper"
- "set new background"

**Detects**:
- GNOME → gsettings
- KDE Plasma → plasma-apply-wallpaperimage
- Hyprland → swww
- Sway → swaybg
- i3 → feh
- XFCE → xfconf-query

**Risk Level**: LOW
**Requires Confirmation**: Yes (modifies user settings)

---

#### neovim.rs
**Purpose**: Install Neovim with basic configuration

**Matches**:
- "install neovim"
- "setup nvim"

**Actions**:
1. Install neovim package
2. Create ~/.config/nvim/
3. Generate init.lua with sensible defaults
4. Verify installation

**Risk Level**: MEDIUM
**Requires Confirmation**: Yes (package installation)

---

#### packages.rs
**Purpose**: Fix broken pacman state and packages

**Matches**:
- "fix broken packages"
- "repair pacman"
- "package error"

**Actions**:
1. Remove stale lock files (safely)
2. Sync databases
3. Clean package cache
4. Check for updates
5. Verify package integrity
6. List orphaned packages

**Risk Level**: LOW to MEDIUM
**Requires Confirmation**: Only for lock removal

---

### Beta.152 Recipes

#### systemd.rs
**Purpose**: Manage systemd services (enable/disable/start/stop/restart/status/logs)

**Matches**:
- "enable NetworkManager service"
- "restart bluetooth"
- "start sshd"
- "stop nginx"
- "status of docker daemon"
- "show logs for sshd"

**Actions** (varies by sub-operation):
- **Enable**: Configure service to start on boot
- **Disable**: Prevent service from auto-starting
- **Start**: Start service immediately
- **Stop**: Stop running service
- **Restart**: Stop then start service
- **Status**: Show service status (read-only)
- **Logs**: Display recent journal entries (read-only)

**Risk Levels**:
- INFO: status, logs (read-only)
- MEDIUM: enable, disable, start, stop, restart

**Requires Confirmation**: Yes (except read-only operations)

---

#### network.rs
**Purpose**: Network diagnostics and configuration guidance

**Matches**:
- "check internet connection"
- "why is my wifi not working"
- "show available wifi networks"
- "check DNS settings"
- "configure static IP"

**Sub-operations**:
1. **Connectivity check**: Ping gateway → external IP → domain (layered testing)
2. **Interface status**: Show all network interfaces and IP addresses
3. **WiFi status**: Diagnose WiFi device, driver, connection issues
4. **WiFi list**: Scan and display available networks
5. **DNS check**: Show DNS configuration and test resolution
6. **Static IP guide**: Display step-by-step instructions (NOT automated)

**Risk Level**: INFO (read-only diagnostics only)
**Requires Confirmation**: No (no system changes)

**Important**: Static IP configuration is intentionally NOT automated (too risky). Recipe provides detailed manual instructions instead.

---

#### system_update.rs
**Purpose**: Check for and apply system updates via pacman

**Matches**:
- "check for system updates"
- "update system"
- "upgrade all packages"

**Sub-operations**:
1. **Check updates**: List available updates (read-only)
2. **Full upgrade**: Sync databases and upgrade all packages
3. **Package upgrade**: Update specific package

**Actions** (full upgrade):
1. Show packages to be updated
2. Sync databases and upgrade (pacman -Syu)
3. Clean old cache (paccache -rk3)
4. Check for failed services
5. Check for .pacnew/.pacsave files

**Risk Levels**:
- LOW: Checking for updates
- HIGH: Full system upgrade

**Requires Confirmation**: Yes (for upgrades)

**Safety**:
- Warns user to check Arch News first
- Shows all packages before upgrading
- Includes rollback instructions (downgrade from cache)
- Checks disk space before proceeding

---

#### aur.rs
**Purpose**: AUR package management via yay/paru

**Matches**:
- "install package from AUR"
- "do I have yay installed"
- "install yay"
- "search AUR for chrome"

**Sub-operations**:
1. **Check helper**: Verify yay/paru is installed
2. **Install helper**: Build and install yay from source
3. **Install package**: Review PKGBUILD and install from AUR
4. **Search package**: Search AUR by keyword

**Risk Levels**:
- INFO: Checking helper, searching
- HIGH: Installing helper, installing packages

**Requires Confirmation**: Yes (for installations)

**Critical Safety Features**:
- Always displays PKGBUILD before installing
- Shows security warnings about user-submitted packages
- Requires review of package source before proceeding
- Includes checklist for vetting AUR packages

---

## ActionPlan Structure

Every recipe generates an `ActionPlan` with:

```rust
pub struct ActionPlan {
    /// Analysis of user request and system state
    pub analysis: String,

    /// List of goals to accomplish
    pub goals: Vec<String>,

    /// Pre-flight checks (optional dependencies, conflicts)
    pub necessary_checks: Vec<NecessaryCheck>,

    /// Main command sequence with risk levels
    pub command_plan: Vec<CommandStep>,

    /// Rollback commands (if things go wrong)
    pub rollback_plan: Vec<RollbackStep>,

    /// Detailed user-facing notes and warnings
    pub notes_for_user: String,

    /// Metadata (recipe tracking, template used)
    pub meta: PlanMeta,
}
```

### CommandStep

```rust
pub struct CommandStep {
    pub id: String,                    // Unique step identifier
    pub description: String,            // Human-readable description
    pub command: String,                // Actual shell command
    pub risk_level: RiskLevel,         // INFO, LOW, MEDIUM, HIGH
    pub rollback_id: Option<String>,   // Link to rollback step
    pub requires_confirmation: bool,   // User must approve before execution
}
```

---

## Risk Levels

| Level | Description | User Approval | Examples |
|-------|-------------|---------------|----------|
| **INFO** | Read-only, no system changes | No | systemctl status, ip addr show, pacman -Q |
| **LOW** | Minor changes, easily reversible | No | mkdir, sync package db |
| **MEDIUM** | Significant changes, rollback available | Yes | pacman -S, systemctl enable, usermod |
| **HIGH** | System-wide changes, potential downtime | Yes | pacman -Syu, AUR installation, service restarts |

---

## Adding New Recipes

### 1. Create Recipe Module

```bash
touch crates/annactl/src/recipes/my_recipe.rs
```

### 2. Implement Pattern

```rust
pub struct MyRecipe;

impl MyRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        // Pattern matching logic
        input_lower.contains("keyword") && input_lower.contains("action")
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        // Extract telemetry
        let has_internet = telemetry.get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        // Build necessary checks
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-something".to_string(),
                description: "Verify prerequisite".to_string(),
                command: "command-to-check".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        // Build command plan
        let command_plan = vec![
            CommandStep {
                id: "do-thing".to_string(),
                description: "Perform main action".to_string(),
                command: "actual-command".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("undo-thing".to_string()),
                requires_confirmation: true,
            },
        ];

        // Build rollback plan
        let rollback_plan = vec![
            RollbackStep {
                id: "undo-thing".to_string(),
                description: "Reverse the action".to_string(),
                command: "undo-command".to_string(),
            },
        ];

        // Build metadata
        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("my_recipe.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("my_recipe_template".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "Explanation of what's being done".to_string(),
            goals: vec!["Goal 1".to_string(), "Goal 2".to_string()],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Detailed instructions and warnings".to_string(),
            meta,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_request() {
        assert!(MyRecipe::matches_request("my keyword action"));
        assert!(!MyRecipe::matches_request("unrelated query"));
    }

    #[test]
    fn test_build_plan() {
        let telemetry = HashMap::new();
        let plan = MyRecipe::build_plan(&telemetry).unwrap();
        assert_eq!(plan.goals.len(), 2);
        // More assertions...
    }
}
```

### 3. Register in mod.rs

```rust
pub mod my_recipe;

pub fn try_recipe_match(...) -> Option<Result<ActionPlan>> {
    // Add at appropriate specificity level
    if my_recipe::MyRecipe::matches_request(user_input) {
        return Some(my_recipe::MyRecipe::build_plan(&telemetry_with_request));
    }

    // ... existing recipes ...
}
```

### 4. Add Tests

```rust
#[test]
fn test_recipe_matching() {
    let telemetry = HashMap::new();

    // Test your recipe
    assert!(try_recipe_match("my query", &telemetry).is_some());
}
```

---

## Testing

### Run Recipe Tests

```bash
cargo test -p annactl recipes::
```

### Test Coverage Requirements

Every recipe MUST have tests for:

1. **Pattern matching**:
   - Positive matches (should match)
   - Negative matches (should NOT match)

2. **Plan generation**:
   - Verify structure (goals, checks, commands, rollback)
   - Verify risk levels are appropriate
   - Verify confirmation flags are correct

3. **Edge cases**:
   - No internet warning (if applicable)
   - Low disk space warning (if applicable)
   - Missing dependencies (if applicable)

4. **Metadata**:
   - Verify template_used is correct
   - Verify recipe_module is tracked

---

## Integration with Planner Pipeline

```
User Query
    ↓
try_recipe_match()
    ↓
┌─────────────────┐
│ Recipe Matches? │
└────┬────────┬───┘
     YES      NO
      ↓        ↓
   Recipe   LLM JSON
   Plan     Generation
      ↓        ↓
   ┌──────────┐
   │ Execute  │
   │ Commands │
   └──────────┘
```

The recipe system is the **first priority** in the query pipeline:

1. **Recipes tried first**: Deterministic, tested, safe
2. **LLM fallback**: Only for queries with no matching recipe
3. **Conversational fallback**: If LLM JSON fails to parse

---

## Recipe Coverage (Beta.152)

| Category | Recipes | Example Queries |
|----------|---------|-----------------|
| **Package Management** | docker, neovim, packages, system_update, aur | "install docker", "update system", "fix broken packages", "install from AUR" |
| **Service Management** | systemd | "enable NetworkManager", "restart bluetooth", "status of sshd" |
| **Network** | network | "check internet", "why is wifi not working", "show available networks" |
| **Desktop** | wallpaper | "change my wallpaper" |

**Total**: 8 recipes covering ~50 common Arch Linux admin tasks

---

## Future Expansion

Potential recipes for future versions:

1. **Firewall management** (ufw, iptables)
2. **User/group management** (useradd, usermod, groups)
3. **SSH configuration** (install, keys, config)
4. **GPU drivers** (nvidia, amd, intel)
5. **Development environments** (rust, python, node, go)
6. **Disk operations** (mount, fstab, partitioning)
7. **Backup operations** (rsync, timeshift)
8. **Boot management** (grub, systemd-boot)
9. **Sound configuration** (pipewire, pulseaudio)
10. **Bluetooth management** (pairing, connecting)

---

## Design Guidelines

### DO

✅ Use real telemetry to generate contextual commands
✅ Show all commands explicitly before execution
✅ Provide rollback instructions for write operations
✅ Include clear warnings for risky operations
✅ Test thoroughly with multiple scenarios
✅ Document assumptions and prerequisites
✅ Use appropriate risk levels
✅ Require confirmation for medium/high risk operations

### DON'T

❌ Hide commands from the user
❌ Make assumptions about system state without checking
❌ Execute high-risk operations without confirmation
❌ Generate commands dynamically at runtime (use templates)
❌ Rely on external services for recipe logic
❌ Create recipes for uncommon edge cases
❌ Automate operations that could brick the system

---

## Recipe Philosophy

**Recipes are digital sysadmin manuals, not magic.**

Each recipe embodies the knowledge and best practices of experienced Arch Linux system administrators. They are:

- **Conservative**: Prefer safe, well-tested approaches
- **Educational**: Show users what commands accomplish the goal
- **Transparent**: Never hide what's being executed
- **Reversible**: Provide rollback when possible
- **Maintainable**: Simple, testable, documented code

Recipes should feel like having an experienced sysadmin guide you through a task, not having an AI make mysterious changes to your system.

---

## Version History

- **Beta.151**: Initial recipe system (docker, neovim, packages, wallpaper)
- **Beta.152**: Major expansion (systemd, network, system_update, aur) - 4 new recipes, 8 total

---

**End of RECIPES_ARCHITECTURE.md**
