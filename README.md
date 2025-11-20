# Anna Assistant

**Local Arch Linux System Assistant**

Anna is a local system assistant for Arch Linux that uses telemetry and a local LLM to help you understand and manage your system.

**Version:** 5.7.0-beta.173
**Status:** Beta - Active Development

---

## What is Anna?

Anna watches your Arch Linux system, collects telemetry, and uses a local language model (via Ollama) to answer questions and propose structured action plans.

**Core principles:**
- All data stays local (no cloud services)
- Telemetry-first (no hallucinations about your system)
- Transparent commands (shows exactly what will run)
- Requires approval before making changes

**What Anna is NOT:**
- ❌ Not a generic chatbot
- ❌ Not a remote management tool
- ❌ Not a fully autonomous agent
- ❌ Not production-ready (still beta)

---

## Installation

One-line install:

```bash
curl -fsSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash
```

The installer will:
1. Install `annad` (telemetry daemon) and `annactl` (CLI)
2. Set up systemd services
3. Configure Ollama for local LLM (if you choose)
4. Explain privacy and what data Anna collects

---

## How to Use Anna

Anna has **three commands**:

### 1. Interactive Mode - `annactl`

Opens the TUI (Terminal User Interface) for ongoing conversation:

```bash
annactl
```

### 2. System Status - `annactl status`

Quick one-line system health check:

```bash
annactl status
```

Shows: CPU, RAM, disk usage, daemon status, LLM availability.

### 3. One-Shot Query - `annactl "<question>"`

Ask a single question and get an answer:

```bash
annactl "how much RAM do I have?"
annactl "what is my CPU model?"
annactl "give me a full system report"
```

**Examples:**
```bash
# Hardware info
annactl "what CPU do I have?"
annactl "how much disk space is free?"

# System info
annactl "what desktop environment am I using?"
annactl "is my internet connection working?"

# Reports
annactl "give me a full system report"
```

---

## Current Capabilities (Beta.173)

### ✅ What Works

**Core Infrastructure:**
- ✅ Telemetry collection (CPU, RAM, disk, GPU, network, services, desktop environment)
- ✅ Daemon (`annad`) runs and monitors system
- ✅ CLI (`annactl`) communicates with daemon
- ✅ TUI interface for interactive sessions
- ✅ `annactl status` command for quick checks
- ✅ One-shot queries via `annactl "<question>"`

**Telemetry Truth System (Beta.150 - NEW):**
- ✅ Zero hallucinations - all system data verified or marked "Unknown"
- ✅ CLI and TUI produce identical answers for same query
- ✅ Real hostname detection (not "localhost")
- ✅ Accurate storage calculations (not "0.0 GB free")
- ✅ Unified system reports backed by real telemetry

**LLM Integration:**
- ✅ Local LLM via Ollama (automatic setup)
- ✅ Hardware-aware model selection
- ✅ Telemetry passed to LLM as structured JSON
- ✅ Conversational answers for info queries

**JSON ActionPlan System (Beta.150 - NEW):**
- ✅ V3 JSON dialogue for structured action plans
- ✅ Strict schema validation (analysis, goals, checks, commands, rollbacks)
- ✅ Command transparency (shows every command before execution)
- ✅ Enhanced confirmation flow (preview all commands before approval)
- ✅ Desktop environment detection (DE/WM/display protocol)
- ✅ Risk levels: INFO (blue), LOW (green), MEDIUM (yellow), HIGH (red)

**Recipe Library (Beta.173):
- ✅ 71 deterministic recipes for common Arch Linux tasks
- ✅ Systemd service management (enable/disable/start/stop/restart/status)
- ✅ Network diagnostics and configuration guidance
- ✅ System updates (check/upgrade packages)
- ✅ AUR package installation with safety checks
- ✅ SSH server installation and key management
- ✅ UFW firewall configuration and rule management
- ✅ User and group management (add/remove/modify)
- ✅ Rust development environment setup (rustup, cargo, tools)
- ✅ Python development environment setup (pip, venv, tools)
- ✅ Node.js development environment setup (npm, project init, tools)
- ✅ NVIDIA GPU driver installation and configuration (CUDA, Xorg)
- ✅ AMD GPU driver installation and configuration (ROCm, Mesa)
- ✅ Intel GPU driver installation and configuration (Mesa, VA-API)
- ✅ Docker Compose installation and project management
- ✅ PostgreSQL database installation and management
- ✅ Nginx web server installation and configuration
- ✅ System monitoring tools installation (htop, btop, glances)
- ✅ Backup solutions setup (rsync, borg backup)
- ✅ Performance tuning (CPU governor, swappiness)
- ✅ Web browser installation (Firefox, Chrome, Chromium, Brave) (NEW)
- ✅ Media players and codecs (VLC, MPV, ffmpeg, GStreamer) (NEW)
- ✅ Productivity applications (LibreOffice, GIMP, Inkscape) (NEW)
- ✅ Terminal tools (alacritty, kitty, tmux, screen) (NEW)
- ✅ Shell environments (zsh, fish, oh-my-zsh) (NEW)
- ✅ Compression tools (zip, 7zip, unrar) (NEW)
- ✅ Communication apps (Discord, Slack, Telegram, Signal) (NEW)
- ✅ Text editors (VS Code, Sublime Text, Neovim) (NEW)
- ✅ File sync tools (Syncthing, rclone) (NEW)
- ✅ Gaming platform (Steam, Proton, multilib support) (NEW)
- ✅ Windows compatibility (Wine, Lutris, Winetricks) (NEW)
- ✅ Gamepad/controller support (jstest-gtk, xboxdrv) (NEW)
- ✅ Security tools (fail2ban, AIDE intrusion detection) (NEW)
- ✅ Antivirus (ClamAV virus scanning and protection) (NEW)
- ✅ VPN tools (WireGuard, OpenVPN) (NEW)
- ✅ Virtualization (QEMU/KVM, VirtualBox, virt-manager) (NEW)
- ✅ Container tools (Podman, Buildah, Skopeo, rootless) (NEW)
- ✅ Virtual networks (libvirt network management) (NEW)
- ✅ Audio systems (PipeWire, PulseAudio, ALSA) (NEW)
- ✅ Music players (Spotify, MPD, ncmpcpp) (NEW)
- ✅ Audio recording (Audacity, Ardour, JACK)
- ✅ Video editing and graphics (Kdenlive, OpenShot, Blender)
- ✅ Desktop environments (GNOME, KDE Plasma, XFCE, i3, Sway)
- ✅ Display managers (SDDM, GDM, LightDM)
- ✅ Printing system (CUPS, printer management) (NEW)
- ✅ Bluetooth (bluez, device pairing, management)
- ✅ Cloud storage (Nextcloud, Dropbox clients)
- ✅ File managers (Nautilus, Dolphin, Thunar, ranger) (NEW)
- ✅ Archive managers (File Roller, Ark, Xarchiver) (NEW)
- ✅ PDF readers (Evince, Okular, Zathura, MuPDF) (NEW)
- ✅ Screenshot tools (Flameshot, Spectacle, scrot, GNOME Screenshot)
- ✅ Screencast/recording (OBS Studio, SimpleScreenRecorder, Peek)
- ✅ Remote desktop (Remmina, TigerVNC, AnyDesk)
- ✅ Application launchers (Rofi, dmenu, Wofi)
- ✅ Clipboard managers (Clipman, Clipmenu, CopyQ)
- ✅ Notification daemons (Dunst, Mako, notify-osd)
- ✅ Email clients (Thunderbird, Mutt, Evolution, Geary)
- ✅ Password managers (KeePassXC, Bitwarden, Pass)
- ✅ Torrent clients (qBittorrent, Transmission, Deluge, rTorrent)
- ✅ IRC/Chat clients (WeeChat, Irssi, HexChat, Pidgin)
- ✅ Image viewers (feh, sxiv, Geeqie, imv)
- ✅ Note-taking apps (Obsidian, Joplin, Typora, Zettlr)
- ✅ Calendar applications (GNOME Calendar, KOrganizer, calcurse, Orage)
- ✅ Task management (Taskwarrior, todoman, GNOME To Do)
- ✅ Diagram tools (draw.io Desktop, Dia, Graphviz, PlantUML)
- ✅ Ebook readers (Calibre, Foliate, Zathura, Okular) (NEW)
- ✅ RSS/feed readers (Newsboat, Liferea, Akregator, RSS Guard) (NEW)
- ✅ System info tools (Fastfetch, Neofetch, Screenfetch, inxi, hwinfo) (NEW)
- ✅ Zero-hallucination, tested, safe action plans (289 tests passing)
- ✅ See `docs/RECIPES_ARCHITECTURE.md` for details

### 🔧 Partially Implemented

**What exists but needs work:**
- 🔧 LLM JSON output quality - Model doesn't consistently generate valid ActionPlan JSON for complex multi-step queries
- ✅ Recipe coverage - 53 recipes implemented (Beta.167) - Expanding beyond core
- 🔧 Template matching - Works for simple queries, limited coverage
- 🔧 Action execution - Infrastructure ready, execution depends on recipe or LLM JSON quality

### 📋 Roadmap

**Next priorities:**
- LLM fine-tuning for JSON ActionPlan generation
- Expand deterministic recipe library for common tasks
- QA test suite expansion (currently 20 questions, goal: 700)
- Repository cleanup and documentation accuracy

---

## Architecture

Anna uses a **4-tier query system**:

```
User Query
    ↓
TIER 0: System Report (instant, zero-latency)
    ↓ (if not matched)
TIER 1: Deterministic Recipes (hard-coded, zero hallucination)
    ↓ (if not matched)
TIER 2: Template Matching (fast, accurate for simple queries)
    ↓ (if not matched)
TIER 3: V3 JSON Dialogue (LLM generates ActionPlan)
    ↓ (if not matched or error)
TIER 4: Conversational Answer (LLM explains)
```

**For actionable queries** (install, fix, configure):
- LLM generates **JSON ActionPlan** with:
  - `analysis`: Why this solution
  - `goals`: What will be achieved
  - `necessary_checks`: Pre-flight validation commands
  - `command_plan`: Exact commands with risk levels
  - `rollback_plan`: How to undo changes
  - `notes_for_user`: Plain English explanation

**Current limitation:** LLM doesn't consistently output valid JSON. Falls back to conversational mode.

---

## Privacy

**What Anna collects (locally):**
- System metrics: CPU, RAM, disk, GPU, network status
- Installed packages and running services
- Desktop environment (DE, WM, display server)
- System logs (for diagnostics)

**What Anna does NOT do:**
- ❌ Read your personal files or documents
- ❌ Send data to external servers
- ❌ Track you for advertising
- ❌ Run commands without your explicit approval
- ❌ Connect to cloud services (unless you configure remote LLM)

**All data stays on your machine.** Anna is 100% local by default.

Ask anytime: `annactl "what do you store about me?"`

---

## Requirements

- **OS:** Arch Linux (x86_64)
- **Systemd:** Required for daemon
- **RAM:** 4GB minimum, 8GB+ recommended for local LLM
- **Disk:** ~5GB for Ollama + model
- **Optional:** Ollama (auto-installed for local LLM)

---

## Development

**Build from source:**
```bash
git clone https://github.com/jjgarcianorway/anna-assistant
cd anna-assistant
cargo build --release
```

**Run tests:**
```bash
cargo test --workspace
```

**Test harness:**
```bash
cd tests/qa
python3 run_qa_suite.py --count 5
```

---

## Documentation

**User docs:**
- This README - current state and usage
- `VERSION_150_TELEMETRY_TRUTH.md` - telemetry truth system
- `VERSION_150_OPTION_A.md` - JSON ActionPlan architecture
- `tests/qa/README.md` - QA test harness

**Internal/developer docs:**
- `ARCHITECTURE.md` - system design
- `docs/runtime_llm_contract.md` - LLM JSON schema
- `crates/annactl/src/system_prompt_v3_json.rs` - LLM system prompt

**Archived docs:**
- `archived-docs/` - obsolete documentation from previous iterations
- `docs/archive/` - legacy architecture docs

---

## Known Issues (Beta.150)

1. **LLM JSON Quality:** Local LLM (llama3.1:8b) doesn't consistently generate valid ActionPlan JSON. Falls back to conversational mode. Solution: Fine-tune model or use JSON-mode-capable models (qwen2.5-coder:14b).

2. **Test Pass Rate:** QA suite shows 0% pass rate due to issue #1. Infrastructure is complete, model needs work.

3. **Limited Recipe Coverage:** Deterministic recipes only cover a handful of queries. Needs expansion for common tasks (package management, networking, systemd).

4. **Documentation Drift:** Many old docs describe features that no longer exist or are outdated. Cleanup in progress.

---

## Version History

**Beta.150 (2025-11-20):**
- ✅ Telemetry truth enforcement - zero hallucinations
- ✅ Unified system reports (CLI/TUI identical)
- ✅ Fixed storage bug (0.0 GB → actual free space)
- ✅ Fixed hostname detection (shows real hostname, not "localhost")
- ✅ Re-enabled V3 JSON dialogue
- ✅ Command transparency (shows all commands before execution)
- ✅ Enhanced confirmation flow
- ✅ QA test harness (20 questions, expandable to 700)

**Previous versions:** See `CHANGELOG.md`

---

## License

GNU General Public License v3 (GPLv3) - See `LICENSE` for details.

---

## Support

**Issues:** https://github.com/jjgarcianorway/anna-assistant/issues
**Source:** https://github.com/jjgarcianorway/anna-assistant

**Philosophy:** Anna is designed to be self-explanatory. If you need extensive docs, we've failed. Just ask Anna.
