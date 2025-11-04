# Anna Assistant Development Roadmap

## Vision

Anna should be an intelligent, autonomous system administrator that:
1. **Learns** from your hardware, software, and behavior
2. **Recommends** actions based on Arch Wiki best practices
3. **Prioritizes** from critical security to cosmetic improvements
4. **Executes** safely with full audit trails and rollback capability

---

## Current Status (v1.0.0-alpha.3)

### ✅ Complete
- [x] Core data models and types
- [x] Basic system telemetry (hardware, packages)
- [x] Unix socket IPC (daemon ↔ client communication)
- [x] Beautiful CLI with pastel colors
- [x] 5 basic recommendation rules
- [x] Action executor with dry-run support
- [x] Audit logging to JSONL
- [x] Install script with version embedding
- [x] GitHub Actions release pipeline

### 🚧 In Progress
- [ ] **Enhanced Telemetry** - Behavioral analysis
- [ ] **Intelligent Recommender** - Priority-based suggestions
- [ ] **Comprehensive Rules** - 20+ recommendation types

---

## Phase 1: Intelligent Telemetry (CURRENT)

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

## Phase 2: Priority-Based Recommendations

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

## Phase 3: Smart Recommendation Engine

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

## Phase 4: Arch Wiki Integration

### Local Wiki Cache
- Download wiki pages for installed packages
- Cache recommendations with wiki citations
- Update weekly via cron/systemd timer

### Wiki-Grounded Advice
Every recommendation includes:
- Direct link to relevant wiki page
- Citation of specific wiki section
- Command examples from wiki

Example:
```
[MANDATORY] Install AMD microcode
Reason: AMD CPU detected without microcode updates
Wiki: https://wiki.archlinux.org/title/Microcode#AMD
Quote: "For AMD processors, install the amd-ucode package."
Command: pacman -S amd-ucode
```

---

## Phase 5: Advanced Features

### Autonomous Tiers
- **Tier 0** (default): Advise only
- **Tier 1**: Auto-apply Low risk + Mandatory priority
- **Tier 2**: Auto-apply Low/Medium risk
- **Tier 3**: Fully autonomous (with safeguards)

### Rollback System
- Snapshot before risky operations
- Rollback tokens for reversible actions
- Audit log of all changes

### Periodic Monitoring
- Refresh telemetry every 6 hours
- New recommendations based on behavior changes
- Alert on new security issues

---

## Implementation Order

### Next Steps (Immediate)
1. ✅ Fix compilation errors (add new fields to SystemFacts)
2. ✅ Implement enhanced telemetry functions
3. ✅ Update recommender with priority system
4. ✅ Add 20+ intelligent rules

### Short Term (This Week)
5. Systemd service file
6. Periodic telemetry refresh
7. Command history analysis
8. Development tool detection

### Medium Term
9. Wiki caching system
10. More sophisticated behavior analysis
11. Package recommendation based on usage
12. Beautification suggestions

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
