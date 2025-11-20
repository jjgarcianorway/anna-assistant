# Anna Assistant v5.7.0-beta.153 Release Notes

**Release Date**: 2025-11-20
**Type**: Recipe Library Expansion
**Focus**: SSH, Firewall, and User Management

---

## 🎯 Overview

Beta.153 continues the recipe library expansion, adding 3 new system administration recipes focused on security, access control, and user management. This brings the total recipe count from 8 to 11, covering ~75 common Arch Linux tasks.

**Key Achievement**: Anna now handles SSH setup, firewall configuration, and user account management with zero-hallucination, tested, safe action plans.

---

## ✨ What's New

### Recipe Library Expansion (Beta.153)

#### New Recipe Modules

1. **ssh.rs** - SSH Installation and Configuration
   - Install OpenSSH server and client
   - Generate SSH key pairs (Ed25519)
   - Check SSH service status
   - Show SSH configuration files
   - Security warnings for remote access
   - Port configuration guidance

2. **firewall.rs** - UFW Firewall Management
   - Install UFW (Uncomplicated Firewall)
   - Enable/disable firewall with lockout warnings
   - Add firewall rules (allow ports/services)
   - Remove firewall rules
   - List active rules
   - Show firewall status
   - Critical SSH lockout prevention warnings

3. **users.rs** - User and Group Management
   - Create new user accounts
   - Remove user accounts (with data deletion warnings)
   - Add users to groups (docker, wheel, etc.)
   - List all user accounts
   - Show user information
   - Change user shell
   - Password management
   - Group context warnings (security implications)

---

## 📊 Recipe Coverage

### Before Beta.153 (8 recipes):
- Docker installation
- Wallpaper management
- Neovim installation
- Package repair
- Systemd service management
- Network diagnostics
- System updates
- AUR package management

### After Beta.153 (11 recipes):
- ✅ Docker installation
- ✅ Wallpaper management
- ✅ Neovim installation
- ✅ Package repair
- ✅ Systemd service management
- ✅ Network diagnostics
- ✅ System updates
- ✅ AUR package management
- ✅ **SSH installation and configuration** (NEW)
- ✅ **UFW firewall management** (NEW)
- ✅ **User and group management** (NEW)

**Total Coverage**: ~75 common Arch Linux admin tasks

---

## 🔧 Technical Details

### SSH Recipe (ssh.rs - ~650 lines)

**Operations**:
- `InstallServer`: Install OpenSSH, enable/start sshd
- `GenerateKeys`: Create Ed25519 key pair with proper permissions
- `CheckStatus`: Read-only status checks
- `ShowConfig`: Display configuration files

**Risk Levels**:
- MEDIUM: Server installation (opens remote access)
- LOW: Key generation
- INFO: Status and config display

**Security Emphasis**:
- Strong warnings about opening remote access
- Password authentication vs. key-based auth guidance
- Port change recommendations
- Firewall configuration reminders

---

### Firewall Recipe (firewall.rs - ~900 lines)

**Operations**:
- `Install`: Install UFW package
- `Enable`: Activate firewall (HIGH risk due to lockout potential)
- `Disable`: Deactivate firewall
- `AddRule`: Allow ports/services through firewall
- `RemoveRule`: Block ports/services
- `Status`: Show firewall state (read-only)
- `ListRules`: Display all configured rules (read-only)

**Risk Levels**:
- HIGH: Enabling firewall (potential SSH lockout)
- MEDIUM: Installing, disabling, modifying rules
- INFO: Status and listing operations

**Critical Safety Features**:
- SSH lockout warnings when enabling firewall
- Checks for existing SSH rules before activation
- Strong emphasis on having physical/console access
- Recovery instructions included

---

### Users Recipe (users.rs - ~900 lines)

**Operations**:
- `AddUser`: Create user account with home directory and groups
- `RemoveUser`: Delete user and all data (irreversible)
- `AddToGroup`: Grant group permissions
- `ListUsers`: Show all human users (UID >= 1000)
- `ShowUserInfo`: Display detailed user information
- `ChangeShell`: Modify default shell

**Risk Levels**:
- HIGH: Creating and deleting users
- MEDIUM: Adding to groups, changing shell
- INFO: Listing and showing user information

**Safety Features**:
- Warnings about deleting current user account
- Data loss warnings for user deletion
- Group permission context (docker = root access equivalent)
- Shell change requires logout/login to take effect

---

## 🧪 Testing

### Test Coverage

```bash
cargo test -p annactl recipes::
```

**Results**: 71/71 tests passing ✅ (43 from Beta.152 + 28 new)

Each new recipe includes comprehensive tests for:
- Pattern matching (positive and negative cases)
- Operation detection
- Plan structure validation
- Risk level correctness
- Security warning presence
- Edge case handling
- Metadata tracking

---

## 💡 Example Usage

### SSH Management

```bash
annactl "install SSH server"
annactl "generate SSH keys"
annactl "check SSH status"
annactl "show SSH configuration"
```

### Firewall Management

```bash
annactl "install firewall"
annactl "allow SSH through firewall"
annactl "enable firewall"
annactl "firewall status"
annactl "list firewall rules"
annactl "remove port 80 from firewall"
```

### User Management

```bash
annactl "add user john"
annactl "add user to docker group"
annactl "list users"
annactl "show info for user alice"
annactl "change shell to zsh"
annactl "remove user testuser"
```

---

## ⚠️ Known Limitations

1. **SSH Recipe**: Does not automate SSH key distribution to remote servers (manual copy-paste required for security)

2. **Firewall Recipe**: HIGH risk warning for enabling firewall - users must ensure SSH access is configured first to avoid lockout

3. **Users Recipe**: User deletion is irreversible - no automatic backup of user data before removal

4. **General**: Recipe coverage at 11 recipes (~75 tasks), many more Arch Linux tasks exist

---

## 🚀 Upgrade Instructions

### Automatic Update (Recommended)

Anna will auto-update within 10 minutes of release:

```bash
# Just wait, Anna updates herself
# You'll see a notification next time you interact:
✨ I Updated Myself!
I upgraded from v5.7.0-beta.152 to v5.7.0-beta.153
```

### Manual Update

```bash
# Stop the daemon
sudo systemctl stop annad

# Download new version
curl -L -o /tmp/annactl https://github.com/jjgarcianorway/anna-assistant/releases/download/v5.7.0-beta.153/annactl-5.7.0-beta.153-x86_64-unknown-linux-gnu
curl -L -o /tmp/annad https://github.com/jjgarcianorway/anna-assistant/releases/download/v5.7.0-beta.153/annad-5.7.0-beta.153-x86_64-unknown-linux-gnu

# Verify checksums
curl -L https://github.com/jjgarcianorway/anna-assistant/releases/download/v5.7.0-beta.153/SHA256SUMS | sha256sum -c

# Install
sudo mv /tmp/annactl /usr/local/bin/annactl
sudo mv /tmp/annad /usr/local/bin/annad
sudo chmod +x /usr/local/bin/annactl /usr/local/bin/annad

# Restart daemon
sudo systemctl start annad
```

---

## 🎓 Design Philosophy

**"Recipes are experienced sysadmin guidance, not magic."**

Each recipe reflects best practices for Arch Linux system administration:

- **SSH Recipe**: Emphasizes key-based authentication and minimal attack surface
- **Firewall Recipe**: Prioritizes preventing lockouts over convenience
- **Users Recipe**: Warns extensively about data loss and permission escalation

Recipes feel like having an experienced sysadmin walk you through a task, not having an AI make opaque changes to your system.

---

## 📝 Changelog Summary

### Added
- **ssh.rs recipe**: SSH server installation, key generation, status checking, configuration display
- **firewall.rs recipe**: UFW installation, enable/disable, rule management, status display
- **users.rs recipe**: User creation/deletion, group management, shell changes, user listing
- 28 new recipe tests (71 total)
- Enhanced pattern matching for shell operations
- Improved username extraction with keyword exclusion

### Changed
- **Cargo.toml**: Version bumped from 5.7.0-beta.152 → 5.7.0-beta.153
- **README.md**: Updated to Beta.153 with new recipes listed
- **recipes/mod.rs**: Added 3 new recipe registrations and integration tests
- Recipe dispatch order optimized for new recipes

### Fixed
- (No bug fixes in this release - pure feature addition)

---

## 🔮 Future Expansion

Planned recipes for future versions:

1. ~~SSH configuration (install, keys, config)~~ ✅ **Done in Beta.153**
2. ~~Firewall management (ufw, iptables)~~ ✅ **Done in Beta.153**
3. ~~User/group management (useradd, usermod, groups)~~ ✅ **Done in Beta.153**
4. GPU drivers (nvidia, amd, intel)
5. Development environments (rust, python, node, go)
6. Disk operations (mount, fstab, partitioning)
7. Backup operations (rsync, timeshift)
8. Boot management (grub, systemd-boot)
9. Sound configuration (pipewire, pulseaudio)
10. Bluetooth management (pairing, connecting)

---

## 📊 Progress Tracking

**Recipe Development Timeline**:
- Beta.151: 4 recipes (docker, neovim, packages, wallpaper)
- Beta.152: +4 recipes = 8 total (systemd, network, system_update, aur)
- Beta.153: +3 recipes = 11 total (ssh, firewall, users)

**Next milestone**: 15 recipes covering 100+ common Arch Linux tasks

---

## 🙏 Credits

This release continues the systematic expansion of Anna's deterministic recipe system, focusing on security-critical system administration tasks.

Built with Rust 🦀, tested on Arch Linux 🐧

---

**Full Architecture Details**: See `docs/RECIPES_ARCHITECTURE.md`

---

**Status**: Production-ready recipe infrastructure. Recipe coverage continues expanding based on user needs and system administration best practices.
