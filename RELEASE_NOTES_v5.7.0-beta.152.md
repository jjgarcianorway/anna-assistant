# Anna Assistant v5.7.0-beta.152 Release Notes

**Release Date**: 2025-11-20
**Type**: Major Feature Update
**Focus**: Recipe Library Expansion

---

## 🎯 Overview

Beta.152 dramatically expands the deterministic recipe library from 4 recipes to 8 recipes, adding comprehensive coverage for systemd service management, network diagnostics, system updates, and AUR package installation.

**Key Achievement**: Anna now handles ~50 common Arch Linux system administration tasks with zero-hallucination, tested, safe action plans.

---

## ✨ What's New

### Recipe Library Expansion (Beta.152)

#### New Recipe Modules

1. **systemd.rs** - Service Management
   - Enable/disable services
   - Start/stop/restart services
   - Check service status
   - View service logs
   - Covers: NetworkManager, bluetooth, sshd, docker, nginx, apache, mysql, postgresql

2. **network.rs** - Network Diagnostics
   - Connectivity testing (gateway → external IP → DNS)
   - Interface status and configuration
   - WiFi diagnostics and troubleshooting
   - WiFi network scanning
   - DNS configuration checks
   - Static IP configuration guidance (instructional only, not automated)

3. **system_update.rs** - System Updates
   - Check for available updates
   - Full system upgrade (pacman -Syu)
   - Package-specific upgrades
   - Includes Arch News warnings and rollback instructions

4. **aur.rs** - AUR Package Management
   - Check for AUR helpers (yay, paru)
   - Install AUR helpers from source
   - Install packages from AUR with PKGBUILD review
   - Search AUR packages
   - Strong safety warnings for user-submitted packages

---

## 📊 Recipe Coverage

### Before Beta.152 (4 recipes):
- Docker installation
- Wallpaper management
- Neovim installation
- Package repair

### After Beta.152 (8 recipes):
- ✅ Docker installation
- ✅ Wallpaper management
- ✅ Neovim installation
- ✅ Package repair
- ✅ **Systemd service management** (NEW)
- ✅ **Network diagnostics** (NEW)
- ✅ **System updates** (NEW)
- ✅ **AUR package management** (NEW)

**Total Coverage**: ~50 common Arch Linux admin tasks

---

## 🔧 Technical Details

### Recipe Architecture

Each recipe follows a standardized pattern:

```rust
pub struct RecipeName;

impl RecipeName {
    /// Pattern matching - handles user request?
    pub fn matches_request(user_input: &str) -> bool;

    /// Generate validated ActionPlan
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan>;
}
```

### Recipe Integration

Recipes are tried **before** LLM generation in the query pipeline:

```
User Query
    ↓
Try Recipe Match (TIER 1)
    ↓ (if matched)
  Recipe ActionPlan ✅
    ↓ (if not matched)
  LLM JSON Generation (TIER 3)
```

This ensures:
- Zero hallucinations for common tasks
- Consistent, tested behavior
- Fast response (no LLM latency)
- Safe, validated commands

---

## 📚 Documentation

### New Documentation

- **`docs/RECIPES_ARCHITECTURE.md`** - Comprehensive recipe system documentation (~1,500 lines)
  - Architecture overview
  - Recipe patterns
  - Adding new recipes
  - Testing requirements
  - Design guidelines

### Updated Documentation

- **`README.md`** - Updated to Beta.152 with recipe library section
- **`Cargo.toml`** - Version bump to 5.7.0-beta.152

---

## 🧪 Testing

### Test Coverage

```bash
cargo test -p annactl recipes::
```

**Results**: 43/43 tests passing ✅

Each recipe includes comprehensive tests for:
- Pattern matching (positive and negative cases)
- Plan structure validation
- Risk level correctness
- Confirmation flag accuracy
- Edge case handling (no internet, low disk space, etc.)
- Metadata tracking

---

## 💡 Example Usage

### Systemd Service Management

```bash
annactl "enable NetworkManager service"
annactl "restart bluetooth"
annactl "show logs for sshd"
annactl "status of docker daemon"
```

### Network Diagnostics

```bash
annactl "check internet connection"
annactl "why is my wifi not working"
annactl "show available wifi networks"
annactl "check DNS settings"
```

### System Updates

```bash
annactl "check for system updates"
annactl "update system"
annactl "upgrade firefox package"
```

### AUR Package Management

```bash
annactl "do I have yay installed"
annactl "install yay"
annactl "install google-chrome from AUR"
annactl "search AUR for spotify"
```

---

## ⚠️ Known Limitations

1. **Network Recipe Limitation**: Static IP configuration provides instructions only, does not automate (intentional for safety)

2. **AUR Recipe Limitation**: Requires manual PKGBUILD review before installation (intentional for security)

3. **LLM JSON Quality**: Complex multi-step queries that don't match recipes still fall back to conversational mode if LLM can't generate valid JSON

4. **Recipe Coverage**: 8 recipes cover ~50 tasks, but many more Arch Linux tasks exist. Recipe library will continue expanding.

---

## 🚀 Upgrade Instructions

### Automatic Update (Recommended)

Anna will auto-update within 10 minutes of release:

```bash
# Just wait, Anna updates herself
# You'll see a notification next time you interact:
✨ I Updated Myself!
I upgraded from v5.7.0-beta.151 to v5.7.0-beta.152
```

### Manual Update

```bash
# Stop the daemon
sudo systemctl stop annad

# Download new version
curl -L -o /tmp/annactl https://github.com/jjgarcianorway/anna-assistant/releases/download/v5.7.0-beta.152/annactl-5.7.0-beta.152-x86_64-unknown-linux-gnu
curl -L -o /tmp/annad https://github.com/jjgarcianorway/anna-assistant/releases/download/v5.7.0-beta.152/annad-5.7.0-beta.152-x86_64-unknown-linux-gnu

# Verify checksums
curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v5.7.0-beta.152/SHA256SUMS | sha256sum -c

# Install
sudo mv /tmp/annactl /usr/local/bin/annactl
sudo mv /tmp/annad /usr/local/bin/annad
sudo chmod +x /usr/local/bin/annactl /usr/local/bin/annad

# Restart daemon
sudo systemctl start annad
```

---

## 🎓 Design Philosophy

**Recipes are digital sysadmin manuals, not magic.**

Each recipe embodies the knowledge and best practices of experienced Arch Linux system administrators. They are:

- **Conservative**: Prefer safe, well-tested approaches
- **Educational**: Show users what commands accomplish the goal
- **Transparent**: Never hide what's being executed
- **Reversible**: Provide rollback when possible
- **Maintainable**: Simple, testable, documented code

Recipes should feel like having an experienced sysadmin guide you through a task, not having an AI make mysterious changes to your system.

---

## 📝 Changelog Summary

### Added
- **systemd.rs recipe**: Service management (enable/disable/start/stop/restart/status/logs)
- **network.rs recipe**: Network diagnostics and configuration guidance
- **system_update.rs recipe**: System update checking and upgrading
- **aur.rs recipe**: AUR package management with safety checks
- **docs/RECIPES_ARCHITECTURE.md**: Comprehensive recipe system documentation
- Recipe integration tests (43 tests total)
- Recipe metadata tracking

### Changed
- **Cargo.toml**: Version bumped from 5.7.0-beta.151 → 5.7.0-beta.152
- **README.md**: Updated to Beta.152 with recipe library section
- **recipes/mod.rs**: Enhanced with 4 new recipe registrations
- Recipe dispatch order optimized for specificity

### Fixed
- (No bug fixes in this release - pure feature addition)

---

## 🔮 Future Expansion

Planned recipes for future versions:

1. Firewall management (ufw, iptables)
2. User/group management (useradd, usermod, groups)
3. SSH configuration (install, keys, config)
4. GPU drivers (nvidia, amd, intel)
5. Development environments (rust, python, node, go)
6. Disk operations (mount, fstab, partitioning)
7. Backup operations (rsync, timeshift)
8. Boot management (grub, systemd-boot)
9. Sound configuration (pipewire, pulseaudio)
10. Bluetooth management (pairing, connecting)

---

## 🙏 Credits

This release represents significant expansion of Anna's deterministic recipe system, moving from proof-of-concept (4 recipes) to practical utility (8 recipes covering ~50 common tasks).

Built with Rust 🦀, tested on Arch Linux 🐧

---

**Full Architecture Details**: See `docs/RECIPES_ARCHITECTURE.md`

---

**Status**: Production-ready recipe infrastructure. Recipe coverage will continue expanding based on user needs.
