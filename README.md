# 🌟 Anna Assistant

**Your Friendly Arch Linux System Administrator**

```
╭─────────────────────────────────────────────╮
│   Intelligent • Safe • Beautiful • Human    │
╰─────────────────────────────────────────────╯
```

Anna is a smart, friendly system assistant that helps keep your Arch Linux system secure, fast, and well-maintained. She speaks plain English, explains everything she suggests, and makes system administration feel like having a knowledgeable friend looking after your computer.

---

## ✨ What Anna Does

### 🔒 **Security & Updates**
- Detects missing CPU microcode (Spectre/Meltdown protection)
- Comprehensive SSH hardening (10 security rules)
- Checks firewall status (UFW/iptables)
- Monitors for available system updates
- VPN setup (WireGuard, OpenVPN)
- Password manager recommendations (KeePassXC)
- Rootkit detection (rkhunter)
- Antivirus scanning (ClamAV)
- LUKS encryption awareness

### ⚡ **Performance**
- Suggests Btrfs compression (save 20-30% disk space!)
- Optimizes mirror lists with Reflector
- Enables parallel downloads in pacman (5x faster)
- Recommends SSD TRIM for longevity (fstrim timer, noatime, discard)
- Power management for laptops (TLP, powertop)
- Swap compression with zram (faster, less wear)
- Firmware updates (fwupd)
- DNS optimization (systemd-resolved)
- Journal size management

### 💻 **Development**
- Detects which languages you actually use (Python, Rust, Go, JavaScript)
- Suggests LSP servers and tools for your workflow
- Finds missing configurations (git, bat, starship, zoxide)
- Build optimization (sccache for Rust)
- Docker & Docker Compose setup
- Virtualization (QEMU/KVM, virt-manager, libvirt)
- Shell productivity tools (fzf, tmux, bash-completion)
- Archive utilities (zip, rar, p7zip, unarchiver)

### 🎨 **Desktop & Terminal**
- Modern GPU-accelerated terminals (Alacritty, Kitty, WezTerm)
- **8 desktop environments** (GNOME, KDE, Cinnamon, XFCE, MATE, i3, Hyprland, Sway)
- Status bars (Waybar, i3blocks) for tiling WMs
- Application launchers (Rofi, Wofi)
- Notification daemons (Dunst, Mako)
- Compositor support (Picom for X11, XWayland compatibility)
- Screenshot tools (grim/slurp for Wayland, maim/scrot for X11)
- Laptop-specific (touchpad, backlight, battery optimization)
- Webcam support (v4l-utils)
- Audio enhancements (EasyEffects, pavucontrol)

### 🎮 **Hardware & Gaming**
- Gamepad driver detection (Xbox, PlayStation, Nintendo controllers)
- Steam and gaming optimizations
- Hardware accelerated video decoding
- Proton-GE for better game compatibility
- MangoHud performance overlay
- Wine for Windows applications

### 🔌 **Hardware Support**
- Bluetooth stack setup (bluez)
- WiFi firmware detection and installation
- USB automount (udisks2)
- NetworkManager for easy WiFi management
- Printer setup (CUPS)
- Webcam support (v4l-utils)
- Multiple monitor configuration

### 🎬 **Multimedia**
- Video players (VLC, mpv) with codec support
- FFmpeg for media processing
- YouTube downloader (yt-dlp)
- Image manipulation (ImageMagick)
- GStreamer plugins for media playback
- Screen recording (OBS Studio, SimpleScreenRecorder)
- Audio enhancements (EasyEffects, pavucontrol)

### 🔤 **Fonts & Rendering**
- Nerd Fonts for terminal icons
- Emoji font support
- CJK (Chinese, Japanese, Korean) fonts
- Better font rendering

### 🧹 **Maintenance**
- Cleans up orphaned packages
- Monitors systemd health
- Checks GPU drivers
- System snapshots (Timeshift, Snapper, snap-pac)
- Backup solutions (rsync, borg, duplicity)
- Locale and timezone configuration
- NTP time synchronization
- Bootloader optimization (GRUB timeout, quiet boot)
- **Automatic refresh** on system changes (package installs, reboots, config changes)
- **Smart notifications** (GUI via notify-send, terminal via wall)

### 🔐 **Privacy & Productivity**
- Browser recommendations (Firefox, Chromium hardening)
- Password managers (KeePassXC, Bitwarden)
- VPN tools (WireGuard, OpenVPN)
- Android integration (KDE Connect, scrcpy)
- System monitoring tools (htop, btop, iotop)

---

## 🚀 Quick Start

### One-Line Installation

```bash
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo sh
```

The installer automatically:
- Downloads the latest release
- Installs Anna system-wide
- Sets up the background service
- Shows you what Anna can do!

### Try It Out

```bash
# See what Anna suggests for your system
annactl advise

# Check Anna's status
annactl status

# Get a full system health report (in plain English!)
annactl report

# Apply recommendations by number
annactl apply --nums 1        # Apply first recommendation
annactl apply --nums 1-5      # Apply recommendations 1 through 5
annactl apply --nums 1,3,5-7  # Apply multiple recommendations

# Apply by ID
annactl apply --id orphan-packages

# See what would happen without actually doing it
annactl apply --nums 1 --dry-run
```

---

## 🎯 Why Anna?

**She speaks human** - No jargon, no cryptic messages. Anna explains things like a friend would.

> "Your SSD needs regular 'TRIM' operations to stay fast and last longer. Think of it like taking out the trash - it tells the SSD which data blocks are no longer in use."

**She's smart about context** - Anna won't suggest Python tools just because you have Python installed. She checks if you *actually use* Python by analyzing your command history and files.

**Every suggestion is backed by Arch Wiki** - All recommendations link to official documentation so you can learn more.

**Beautiful terminal experience** - Pastel colors, perfect formatting, emoji where it helps. The best-looking CLI you'll use.

---

## 📊 Current Status

**Version**: v1.0.0-beta.17
**Status**: Beta - Feature-rich and stable!

### What's Working

✅ **130+ intelligent detection rules** covering security, hardware, desktop, multimedia, development, system optimization, and more
✅ **Automatic system monitoring** - refreshes advice on package changes, reboots, config edits
✅ **Multi-user support** - personalized advice based on desktop environment, shell, display server
✅ **Batch apply** - apply recommendations by number, range (1-5), or multiple (1,3,5-7)
✅ **Smart notifications** - GUI notifications (notify-send) and terminal broadcasts (wall) for critical issues
✅ **Plain English reports** - conversational system health summaries
✅ **Human-friendly messages** - every word in plain English with clear explanations
✅ **Perfect terminal formatting** - beautiful pastel colors with numbered advice
✅ **Context-aware** - only suggests what you actually need based on your usage
✅ **Automatic installation** - one command and you're done
✅ **Background daemon** - runs quietly, always watching your system
✅ **Arch Wiki citations** - every recommendation has references
✅ **Risk levels** - High (critical) > Medium (recommended) > Low (optional)

### Coming Soon

🚧 **Policy-based auto-apply** - let Anna automatically fix low-risk issues
🚧 **Arch Wiki caching** - offline access to documentation

---

## 🏗️ Architecture

Anna is built with Rust for safety, speed, and reliability.

```
┌─────────────┐
│   annactl   │  ← You interact with this (CLI)
└──────┬──────┘
       │ Unix Socket IPC
┌──────▼──────┐
│    annad    │  ← Background daemon (runs as systemd service)
│             │
│  ┌────────┐ │
│  │Telemetry│ │  Collects system facts
│  └────────┘ │
│  ┌────────┐ │
│  │Recommender│  Generates advice
│  └────────┘ │
│  ┌────────┐ │
│  │Executor │ │  (Future) Runs approved actions
│  └────────┘ │
└─────────────┘
```

**Three crates:**
- `annad` - The daemon (privileged, collects data, generates advice)
- `annactl` - The CLI client (user-facing interface)
- `anna_common` - Shared types and beautiful output formatting

---

## 🎨 What Makes Anna Special

### She Explains Things

Instead of: `AMD CPU detected without microcode updates`

Anna says:
> "Your AMD processor needs microcode updates to protect against security vulnerabilities like Spectre and Meltdown. Think of it like a security patch for your CPU itself."

### She Uses Context

Anna won't spam you with irrelevant suggestions. She checks:
- Do you have the hardware? (SSD → TRIM suggestions)
- Do you actually use this? (Python files + Python commands → Python tools)
- Is it already configured? (NetworkManager installed → check if enabled)

### She Prioritizes

**Mandatory** (🔴) - Security critical (microcode, firewall)
**Recommended** (🟡) - Significant improvements (parallel downloads, TRIM)
**Optional** (🟢) - Performance tweaks (noatime)
**Cosmetic** (🔵) - Pretty things (colored output)

---

## 🔒 Safety & Privacy

- **Fully offline** - No phone home, no telemetry sent anywhere
- **Runs locally** - All data stays on your machine
- **Open source** - See exactly what Anna does
- **Arch Wiki grounded** - Official documentation, not random internet advice
- **Audit logging** - Every action is logged (future feature)
- **Dry-run mode** - See what would happen before doing it (future feature)

---

## 🤝 Contributing

Anna is actively developed and we'd love your help!

**Ways to contribute:**
- Try Anna and report issues
- Suggest new detection rules
- Improve documentation
- Add support for more configurations
- Help make messages even friendlier

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## 📜 License

**GNU General Public License v3.0 (GPLv3)**

Anna Assistant is free and open source software licensed under GPLv3.

This means:
- ✅ **Free to use, fork, and share**
- ✅ **Must cite original source** when distributing
- ✅ **Must remain open source** (copyleft protection)
- ✅ **Must disclose modifications**

This ensures Anna remains free for everyone while protecting the work and giving proper attribution.

See [LICENSE](LICENSE) for full details.

---

## 🌐 Philosophy

Anna believes system administration should be:
- **Accessible** - You don't need to be a Linux expert
- **Transparent** - Always explain why, not just what
- **Beautiful** - Terminal UIs can be gorgeous
- **Helpful** - Like having a knowledgeable friend
- **Smart** - Context-aware, not just rule-based
- **Safe** - Security and stability first

Anna evolves from a diagnostic tool into an intelligent system administrator that understands your system better than you do, learns your habits, and keeps your machine secure, fast, and reliable — quietly, intelligently, beautifully.

---

## 📸 Screenshots

(Coming soon - we want to show you the beautiful terminal output!)

---

## 🙏 Credits

Built with ❤️ for the Arch Linux community.

**Technologies:**
- Rust 🦀 - For speed, safety, and reliability
- Tokio - Async runtime
- Serde - Serialization
- Sysinfo - System information gathering
- Arch Wiki - The source of all truth

---

**Built with Rust • Powered by Arch Wiki • Privacy First • Human Friendly**

[⭐ Star us on GitHub](https://github.com/jjgarcianorway/anna-assistant) • [📦 Latest Release](https://github.com/jjgarcianorway/anna-assistant/releases) • [🐛 Report Issues](https://github.com/jjgarcianorway/anna-assistant/issues)
