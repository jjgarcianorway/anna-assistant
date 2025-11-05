# Anna Assistant Development Roadmap

## Vision

Anna should be an intelligent, autonomous system administrator that:
1. **Learns** from your hardware, software, and behavior
2. **Recommends** actions based on Arch Wiki best practices
3. **Prioritizes** from critical security to cosmetic improvements
4. **Executes** safely with full audit trails and rollback capability

---

## Current Status (v1.0.0-beta.41)

### ✅ Complete
- [x] Core data models and types
- [x] Comprehensive system telemetry (hardware, packages, filesystems, services)
- [x] Unix socket IPC (daemon ↔ client communication)
- [x] Beautiful CLI with pastel colors and universal compatibility
- [x] **220+ intelligent recommendation rules** covering:
  - Security (CPU microcode, SSH hardening, firewall, VPN, antivirus)
  - 8 Desktop environments (GNOME, KDE, Cinnamon, XFCE, MATE, i3, Hyprland, Sway)
  - Performance (SSD, swap compression, firmware updates, parallel downloads)
  - Development (Docker, virtualization, 6 programming languages, LSP servers, shell tools)
  - **Multi-GPU Support (Intel, AMD/ATI, Nvidia)** - Hardware detection and driver recommendations
  - Hardware (Bluetooth, WiFi, printers, webcam, gamepads)
  - Multimedia (video/audio players, screen recording, codecs, video editing)
  - Gaming (Proton-GE, MangoHud, Wine)
  - Privacy (password managers, VPN, browser hardening)
  - Backup & snapshots (Timeshift, Snapper, rsync, borg)
  - System maintenance (orphans, systemd health, bootloader)
  - Productivity (mail clients, office suites, torrent clients)
  - Creative (GIMP, Inkscape, Kdenlive)
  - Networking (Samba, NFS, cloud storage, web servers, remote desktop)
  - Databases (PostgreSQL)
- [x] Action executor with dry-run support
- [x] Batch apply (by number, range, or ID)
- [x] Audit logging to JSONL
- [x] Automatic system monitoring and refresh
- [x] Smart notifications (GUI via notify-send, terminal via wall)
- [x] Filesystem watcher (detects package changes, reboots, config edits)
- [x] Install script with version embedding
- [x] GitHub Actions release pipeline with automated binary builds
- [x] Plain English system reports
- [x] Arch Wiki citations for all recommendations
- [x] Multi-user support with personalized advice
- [x] Priority-based recommendation system
- [x] Risk level categorization
- [x] **Configuration system** - TOML-based settings with annactl config command
- [x] **Snapshot system** - Btrfs/Timeshift/rsync support for safe rollback
- [x] **Category consistency** - All categories properly styled with emojis and colors
- [x] **Hardware video acceleration** - Recommendations for Intel, AMD, and Nvidia GPUs

### 🚧 In Progress
- [ ] **Autonomy tiers** - Graduated automatic maintenance (Tier 0-3)
- [ ] **TUI Dashboard** - Interactive configuration interface
- [ ] **Learning system** - Track user behavior and adapt
- [ ] **Arch Wiki caching** - Offline documentation access

---

## Phase 1: Intelligent Telemetry ✅ COMPLETED

### Goal
Make Anna understand your system deeply by analyzing:

#### Hardware Context
- CPU vendor/cores → optimize interrupt handling, suggest performance tools
- GPU vendor → recommend proper drivers, video acceleration
- RAM amount → suggest swap configuration, zram
- Storage type → btrfs/ext4 specific tools and maintenance

#### Software Environment
- Desktop environment (GNOME/KDE/i3) → suggest matching tools
- Display server (X11/Wayland) → compatibility warnings
- Shell (bash/zsh/fish) → shell-specific enhancements
- Package groups → detect base-devel, understand dev environment

#### User Behavior Analysis
**Command History** (`~/.bash_history`, `~/.zsh_history`):
- Frequency analysis → detect heavily-used tools
- Pattern detection → git user? docker user? vim user?
- Missing tools → suggest better alternatives

**Examples**:
- Sees `ls` used 500 times → suggest `eza` (better ls)
- Sees `cat` heavily → suggest `bat` (syntax highlighting)
- Sees `grep` often → suggest `ripgrep` (faster)
- Sees `find` → suggest `fd` (modern alternative)

**File Type Detection** (non-intrusive):
- Check common directories: `~/Documents`, `~/Downloads`, `~/Projects`
- Detect: `.py`, `.rs`, `.js`, `.go`, `.mp4`, `.mp3`, etc.
- Purpose: Understand workflows without reading file contents

**Development Environment**:
- Programming languages used → suggest LSP servers, formatters
- Build systems present (make/cmake/cargo) → suggest ccache, sccache
- VCS usage → git hooks, diff tools
- Containers → docker/podman optimizations

**Media Usage**:
- Video files but no player → suggest mpv/vlc
- Audio files → suggest proper codecs, players
- Image editing → suggest GIMP, krita based on file types

---

## Phase 2: Priority-Based Recommendations ✅ COMPLETED

### Recommendation Categories & Priorities

#### 1. **MANDATORY** (Security & Drivers)
**Must be addressed for safe operation**

Examples:
- CPU microcode missing (Intel/AMD)
- GPU driver not installed
- SSH allows root login
- No firewall configured
- Kernel modules failing to load
- System time sync disabled (chrony/systemd-timesyncd)

#### 2. **RECOMMENDED** (Quality of Life)
**Significant improvements to usability**

Examples based on detected usage:
- **Python developer** → python-lsp-server, black, mypy
- **Rust developer** → rust-analyzer, clippy
- **Git user** → delta (better diffs), lazygit
- **Vim user** → syntax plugins for detected languages
- **Shell user** → starship prompt, fzf, zoxide
- Package cache > 2GB → clean with `paccache`
- Orphan packages detected → safe removal
- Failed systemd services → investigation needed

#### 3. **OPTIONAL** (Optimizations)
**Nice-to-have performance/convenience**

Examples:
- Multi-core system → irqbalance
- SSD detected → fstrim timer
- Laptop detected → tlp, powertop
- Frequent compilation → ccache/sccache
- Slow mirrors → reflector
- AUR usage detected → paru/yay helper
- Preload for faster app startup

#### 4. **COSMETIC** (Beautification)
**Aesthetics and minor enhancements**

Examples:
- `ls` → `eza` (colored, git-aware ls)
- `cat` → `bat` (syntax highlighting)
- `grep` → `ripgrep` (faster, prettier)
- `find` → `fd` (modern, intuitive)
- `du` → `dust` (visual disk usage)
- `top` → `btop` (beautiful system monitor)
- Nerd fonts for terminal icons
- Starship or oh-my-zsh themes

---

## Phase 3: Smart Recommendation Engine ✅ COMPLETED

### Rule Structure

```rust
struct RecommendationRule {
    category: &'static str,      // "security", "drivers", "development", etc.
    priority: Priority,           // Mandatory/Recommended/Optional/Cosmetic
    check: fn(&SystemFacts) -> Option<Advice>,
}
```

### Intelligence Examples

**Example 1: Python Developer**
```
Detected:
- .py files in ~/Projects
- python3 in command history (125 uses)
- pip in command history (45 uses)

Recommendations (priority order):
1. [RECOMMENDED] python-lsp-server - LSP for editor integration
2. [RECOMMENDED] python-black - Code formatting
3. [OPTIONAL] ipython - Better REPL
4. [OPTIONAL] python-poetry - Dependency management
5. [COSMETIC] python-rich - Beautiful terminal output
```

**Example 2: Vim Power User**
```
Detected:
- vim in command history (890 uses)
- .py, .rs, .js files present
- No nvim installed

Recommendations:
1. [RECOMMENDED] neovim - Modern vim with LSP support
2. [RECOMMENDED] Syntax highlighting for detected languages
3. [OPTIONAL] vim-plug - Plugin manager
4. [COSMETIC] vim color schemes
```

**Example 3: Media Consumer**
```
Detected:
- 50+ .mp4 files in ~/Videos
- No video player installed

Recommendations:
1. [RECOMMENDED] mpv - Lightweight video player
2. [OPTIONAL] ffmpeg - Video conversion tools
3. [OPTIONAL] Hardware video acceleration (based on GPU)
```

---

## Phase 4: Arch Wiki Integration 🚧 IN PROGRESS

### Local Wiki Cache (Planned)
- [ ] Download wiki pages for installed packages
- [ ] Cache recommendations with wiki citations
- [ ] Update weekly via cron/systemd timer

### Wiki-Grounded Advice ✅ COMPLETED
Every recommendation includes:
- [x] Direct link to relevant wiki page
- [x] Citation of specific wiki section (in explanations)
- [x] Command examples from wiki

Example:
```
[MANDATORY] Install AMD microcode
Reason: AMD CPU detected without microcode updates
Wiki: https://wiki.archlinux.org/title/Microcode#AMD
Quote: "For AMD processors, install the amd-ucode package."
Command: pacman -S amd-ucode
```

---

## Phase 5: Advanced Features 🚧 IN PROGRESS

### Autonomous Tiers (Planned)
- [x] **Tier 0** (default): Advise only - IMPLEMENTED
- [ ] **Tier 1**: Auto-apply Low risk + Mandatory priority
- [ ] **Tier 2**: Auto-apply Low/Medium risk
- [ ] **Tier 3**: Fully autonomous (with safeguards)

### Rollback System (Planned)
- [ ] Snapshot before risky operations
- [ ] Rollback tokens for reversible actions
- [x] Audit log of all changes - IMPLEMENTED

### Periodic Monitoring ✅ COMPLETED
- [x] Refresh telemetry automatically via filesystem watcher
- [x] New recommendations based on system changes
- [x] Alert on new critical issues via notifications

---

## Implementation Order

### Completed ✅
1. ✅ Fix compilation errors (add new fields to SystemFacts)
2. ✅ Implement enhanced telemetry functions
3. ✅ Update recommender with priority system
4. ✅ Add 130+ intelligent rules (far exceeded 20+ goal!)
5. ✅ Systemd service file
6. ✅ Periodic telemetry refresh (via filesystem watcher)
7. ✅ Command history analysis
8. ✅ Development tool detection
9. ✅ Package recommendation based on usage
10. ✅ Beautification suggestions
11. ✅ Desktop environment detection (8 DEs supported)
12. ✅ Gaming optimizations
13. ✅ Privacy & security tools
14. ✅ Backup & snapshot integration
15. ✅ Multi-user support with personalized recommendations

### Next Steps (Short Term)
16. [ ] Wiki caching system (offline documentation)
17. [ ] Policy-based auto-apply (Tier 1-3 autonomy)
18. [ ] Rollback system with snapshots
19. [ ] More sophisticated behavior analysis
20. [ ] Performance benchmarking and optimization

---

## Design Principles

1. **Privacy First**: Analyze patterns, not content. Never read private files.
2. **Non-Intrusive**: Suggest, don't impose. User stays in control.
3. **Arch-Native**: Follow Arch philosophy and wiki recommendations.
4. **Transparent**: Full audit logs, clear explanations.
5. **Safe**: Dry-run by default, rollback capability, risk levels.

---

## Questions to Answer

For your workflow, what would be most useful?

1. **Security hardening** (firewall, SSH, audit)?
2. **Development tools** (LSP, formatters, build cache)?
3. **Performance optimization** (irqbalance, preload, zram)?
4. **Quality of life** (better CLI tools, shell enhancements)?
5. **Media/desktop** (players, codecs, themes)?

Anna should learn what matters to **you** and prioritize accordingly.
