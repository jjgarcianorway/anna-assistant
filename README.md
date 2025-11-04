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

### 🔒 **Security**
- Detects missing CPU microcode (Spectre/Meltdown protection)
- Checks firewall status (UFW/iptables)
- Warns about security vulnerabilities with clear explanations

### ⚡ **Performance**
- Suggests Btrfs compression (save 20-30% disk space!)
- Optimizes mirror lists with Reflector
- Enables parallel downloads in pacman (5x faster)
- Recommends SSD TRIM for longevity

### 💻 **Development**
- Detects which languages you actually use (Python, Rust, Go)
- Suggests LSP servers and tools for your workflow
- Finds missing configurations (git, bat, starship, zoxide)

### 🎨 **Beautification**
- Enables colorful terminal output
- Suggests modern CLI tools (eza, bat, ripgrep, fd)
- Recommends shell enhancements

### 🧹 **Maintenance**
- Cleans up orphaned packages
- Monitors systemd health
- Checks GPU drivers
- Detects system updates

### 📦 **Power User Features**
- AUR helper recommendations (yay/paru)
- NetworkManager setup for WiFi
- Comprehensive system checks

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

# Get a full system health report
annactl report
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

**Version**: v1.0.0-beta.8
**Status**: Beta - Ready for testing!

### What's Working

✅ **12 intelligent detection rules** covering security, performance, and usability
✅ **Human-friendly messages** - every word in plain English
✅ **Perfect terminal formatting** - beautiful pastel colors
✅ **Smart detection** - only suggests what you actually need
✅ **Automatic installation** - one command and you're done
✅ **Background daemon** - runs quietly, always ready
✅ **Arch Wiki citations** - every recommendation has references
✅ **Priority system** - Mandatory > Recommended > Optional > Cosmetic

### Coming Soon

🚧 **SSH hardening detection** - check for weak algorithms
🚧 **Professional system reports** - detailed admin-style analysis
🚧 **Shell beautification** - oh-my-zsh, starship, oh-my-posh
🚧 **Snapshot systems** - Timeshift, Snapper recommendations
🚧 **Gaming support** - Steam, multilib, gamemode detection
🚧 **Desktop environment context** - Hyprland, GNOME, KDE specific advice

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

MIT License - See [LICENSE](LICENSE) for details.

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
