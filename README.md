# 🌟 Anna Assistant

**Your Friendly Arch Linux System Administrator**

Anna is a smart, friendly system assistant that helps keep your Arch Linux system secure, fast, and well-maintained. She speaks plain English, explains everything she suggests, and makes system administration feel like having a knowledgeable friend looking after your computer.

**Current Version:** Beta.48 (November 2025)

---

## 🎯 What's New in Beta.48

**🎯 Ignore System:**
- Ignore entire categories (e.g., "Desktop Customization")
- Ignore priority levels (e.g., all "Optional" items)
- Commands: `annactl ignore category/priority/show/reset`
- TUI integration for easy filtering

**✨ UI/UX Improvements:**
- Fixed TUI health display - now shows "Score: 0/100 - Critical (2 issues)"
- Status command shows category breakdown
- Update command shows installed version and friendly messages
- Report and TUI critical counts now consistent

**🔧 Apply System Refinements:**
- Cache-based number system guarantees exact match with advise display
- Sequential history rollback numbers ([#1], [#2], [#3])
- Simplified, reliable operation

---

## 🎯 Previous Release (Beta.43)

**🧠 Advanced Telemetry (8 New Categories):**
- **CPU Microcode Status** - Detects missing Intel/AMD security updates
- **Battery Health** - Monitoring for laptops with TLP recommendations
- **Backup Systems** - Detects timeshift, rsync, borg, restic
- **Bluetooth** - Hardware detection and device tracking
- **SSD TRIM** - Automatic detection and optimization
- **Swap Configuration** - Analyzes type, size, swappiness, zram
- **Locale/Timezone** - Regional settings detection
- **Pacman Hooks** - Identifies installed automation

**🤖 Autonomous Maintenance (13 Tasks):**
- **Tier 1 (Safe):** Package DB updates, failed service monitoring
- **Tier 2 (Extended):** User cache cleanup, broken symlinks, pacman optimization
- **Tier 3 (Full):** Security updates, config backups
- Graduated autonomy levels - choose your comfort level

**🌐 Arch Wiki Integration:**
- Working offline cache with 40+ common pages
- Background updates via daemon RPC
- Quick access to security, performance, and troubleshooting docs

**🎨 UI/UX Improvements:**
- TUI sorting by category/priority/risk (hotkeys: c, p, r)
- Popularity indicators (★★★★☆) for recommendations
- Detailed health score explanations
- Updated installer with current features

---

## ✨ Core Features

### 🔒 **Security & Updates**
- CPU microcode detection (Spectre/Meltdown protection)
- Comprehensive SSH hardening
- Firewall status monitoring
- System update checking
- VPN setup recommendations
- Password manager suggestions
- Security audit tools

### ⚡ **Performance**
- Btrfs compression (save 20-30% disk space!)
- Mirror list optimization with Reflector
- Parallel downloads in pacman (5x faster)
- SSD TRIM optimization
- Power management for laptops (TLP, powertop)
- Swap compression with zram
- Firmware updates (fwupd)
- Journal size management

### 💻 **Development**
- **Workflow bundles** - Complete dev stacks with one command
  - Container Development Stack (Docker ecosystem)
  - Python Development Stack (LSP, tools, debuggers)
  - Rust Development Stack (rust-analyzer, cargo tools)
- Language detection (Python, Rust, Go, JavaScript, Java, etc.)
- LSP server recommendations for your editor
- Git configuration checking
- Docker & virtualization support
- Shell productivity tools

### 🎮 **Desktop & Gaming**
- Window manager support (Hyprland, i3, sway, bspwm, dwm, etc.)
- Desktop environment support (GNOME, KDE, XFCE, etc.)
- Compositor detection and recommendations
- Nvidia+Wayland configuration
- Gaming tools (Steam, Lutris, Wine, ProtonGE)
- GPU-accelerated terminals
- Status bars and application launchers

### 📦 **Package Management**
- Orphan package cleanup
- AUR helper setup (yay, paru)
- Package cache management
- Update notifications
- Dependency checking

### 🩺 **System Doctor**
- Comprehensive health diagnostics (100-point scale)
- Auto-fix with `--fix` flag
- Dry-run mode to preview fixes
- Package system validation
- Disk health checking (SMART data)
- Service health monitoring
- Network connectivity tests

---

## 🚀 Quick Start

### Installation

```bash
# Install Anna (requires root)
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo sh
```

This will:
1. Download and install Anna binaries
2. Set up the systemd daemon
3. Create necessary directories
4. Start Anna's background service

### Basic Usage

```bash
# See system recommendations
annactl advise

# Filter by category
annactl advise security
annactl advise packages
annactl advise performance

# Apply recommendations
annactl apply 1              # Apply recommendation #1
annactl apply 1-5            # Apply recommendations 1 through 5
annactl apply 1,3,5          # Apply specific recommendations

# System health check
annactl doctor

# Auto-fix detected issues
annactl doctor --fix

# Open interactive dashboard
annactl dashboard

# Check system status
annactl status

# View available workflow bundles
annactl bundles

# Apply a workflow bundle
annactl apply --bundle hyprland-setup
```

---

## 📊 Commands Reference

### Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `advise [category]` | Show recommendations | `annactl advise security` |
| `apply <numbers>` | Apply recommendations | `annactl apply 1-5` |
| `bundles` | List workflow bundles | `annactl bundles` |
| `rollback <bundle>` | Rollback a bundle | `annactl rollback hyprland` |
| `doctor` | Run diagnostics | `annactl doctor --fix` |
| `dashboard` | Open interactive TUI | `annactl dashboard` |
| `status` | Show daemon status | `annactl status` |
| `health` | Show health score | `annactl health` |
| `report [category]` | Generate health report | `annactl report` |
| `dismiss <number>` | Dismiss recommendation | `annactl dismiss 1` |
| `history` | View application history | `annactl history` |
| `config` | Configure Anna | `annactl config` |

### Options

- `-m, --mode <mode>` - Display mode: smart (default), critical, recommended, all
- `-l, --limit <num>` - Maximum recommendations to show
- `-n, --dry-run` - Preview changes without applying
- `-a, --auto` - Auto-apply without confirmation

---

## 🎯 Environment-Aware Recommendations

Anna automatically detects your environment and provides tailored advice:

**Hyprland + Nvidia:**
- Critical environment variables (GBM_BACKEND, __GLX_VENDOR_LIBRARY_NAME)
- Kernel parameter recommendations (nvidia-drm.modeset=1)
- Hyprland-specific utilities (hyprpaper, hyprlock, waybar)

**i3 Window Manager:**
- Application launcher suggestions (rofi, dmenu)
- Status bar options (i3status, polybar)
- i3-specific productivity tools

**sway (Wayland i3):**
- Waybar for status display
- Wayland-native utilities
- Compositor optimizations

**GNOME:**
- GNOME Tweaks for customization
- Extension manager
- GNOME-specific tools

**KDE Plasma:**
- Plasma widgets and tools
- System monitor integration
- KDE-specific utilities

---

## 🧠 Deep System Intelligence

Anna understands your system at a deep level:

**Hardware Detection:**
- CPU architecture and temperature
- GPU vendor (Nvidia, AMD, Intel)
- Disk health via SMART data
- Battery status and health
- Memory pressure

**Environment Detection:**
- Window manager (Hyprland, i3, sway, bspwm, etc.)
- Desktop environment (GNOME, KDE, XFCE, etc.)
- Compositor (picom, Hyprland's built-in, etc.)
- Display server (X11, Wayland)
- Shell (bash, zsh, fish)

**Software Analysis:**
- Development languages in use
- Installed development tools
- Gaming setup (Steam, Lutris, etc.)
- Media tools and usage
- Network configuration

**System Health:**
- Boot performance
- Service status
- Failed/slow services
- Journal errors
- Package health

---

## 🛡️ Safety First

Anna is designed to be helpful but never reckless:

- **Explains Everything:** Every recommendation includes a clear reason
- **Risk Levels:** Critical, Recommended, Optional
- **Dry-Run Mode:** Preview changes before applying
- **Reversible Actions:** Most operations can be undone
- **User Approval:** Interactive confirmation for fixes
- **Bundle Rollback:** Undo workflow bundles cleanly
- **Learning System:** Remembers dismissed recommendations

---

## 🏗️ Architecture

Anna consists of two main components:

**annad (Daemon):**
- Runs in the background as a systemd service
- Collects system telemetry
- Generates recommendations
- Provides RPC API over Unix socket

**annactl (Client):**
- User-facing CLI interface
- Communicates with daemon via RPC
- Beautiful, intuitive output
- Interactive TUI dashboard

---

## 📚 Documentation

- [IPC API Documentation](docs/IPC_API.md) - For developers integrating with Anna
- [Contributing Guide](CONTRIBUTING.md) - How to contribute to Anna
- [Changelog](CHANGELOG.md) - Version history and features
- [Testing Guide](TESTING.md) - Testing procedures

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

## 📜 License

GPL-3.0-or-later

---

## 🙏 Credits

Built with ❤️ for the Arch Linux community.

Special thanks to:
- The Arch Linux team for an amazing distribution
- The Arch Wiki community for comprehensive documentation
- All contributors and testers

---

**Installation:**
```bash
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo sh
```

**Uninstallation:**
```bash
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/uninstall.sh | sudo sh
```
