# Anna Assistant v1.0.0-beta.49 - Release Summary

## 🎯 TUI Polish & Command Execution Fixes!

This release brings **major improvements** to the interactive TUI with better navigation, scrolling, and context-aware displays, plus critical fixes for command execution.

---

## 🆕 What's New in Beta.49

### Critical Bug Fixes

**Command Execution Fixed** 🔧
- Fixed shell command execution to support complex syntax
- Now properly handles `$(...)` command substitution
- Shell operators like `&&`, `||`, and `|` work correctly
- Pacman commands with nested commands now execute properly
- Multi-command operations fully supported

### TUI Experience Overhaul

**Navigation Improvements:**
- Category headers are no longer selectable - navigation automatically skips them
- Fixed critical indexing bug where entering a title would show the wrong advice
- Smooth keyboard navigation with up/down or j/k keys
- Details view now scrollable for long content

**Simplified Controls:**
- Changed apply shortcut from "a/y" to just "a" for clarity
- Clear, intuitive keyboard shortcuts throughout
- Better visual feedback for all actions

**Enhanced Details View:**
- Scrollable content with ↑/↓ or j/k keys
- Special banner for informational notices
- Better word wrapping based on terminal width
- Context-aware headings ("Why this matters" vs "Details")
- Clear distinction between actionable and informational items

**Context-Aware Health Display:**
- Health score now shows relevant breakdown based on sort mode
- When sorted by **Priority**: Shows 🔴 Critical, 🟡 Recommended, 🟢 Optional, ⚪ Cosmetic counts
- When sorted by **Risk**: Shows High/Med/Low risk breakdown
- When sorted by **Category**: Shows number of categories affected
- Always displays current filtered view, not total system state

**Better Sorting:**
- When sorted by Risk, category names shown in brackets for clarity
- Color coding improved to reduce confusion
- Each sort mode has appropriate visual hierarchy

**Improved Feedback:**
- Applied advice stays visible until action completes
- Success/failure messages are extremely clear
- No premature disappearance of items
- User has time to read results before list refreshes

---

## 📊 Current Feature Set

### 🔒 Security & Updates
- CPU microcode detection (Intel & AMD)
- SSH hardening recommendations
- Firewall status monitoring
- System update checking
- VPN setup recommendations
- Password manager suggestions

### 💻 Development
- **280+ detection rules** for comprehensive system analysis
- Language detection (Python, Rust, Go, JavaScript, Java, C++)
- LSP server recommendations
- Docker & virtualization support
- Git configuration checking
- Shell productivity tools

### 🎨 Desktop Environments
- **8 Desktop Environments**: GNOME, KDE, Cinnamon, XFCE, MATE, i3, Hyprland, Sway
- Window manager detection and configuration
- Compositor recommendations
- GPU-accelerated terminal support
- Status bars and application launchers

### 🎮 Gaming
- Steam, Lutris, Wine setup
- ProtonGE recommendations
- MangoHud for performance monitoring
- GPU driver optimization (Intel, AMD, Nvidia)

### 🔧 System Maintenance
- Orphan package cleanup
- AUR helper setup
- Package cache management
- Systemd health monitoring
- Boot performance optimization

### 🖥️ Interactive TUI
- Real-time system dashboard
- Sortable recommendations (by Category, Priority, Risk)
- Detailed recommendation view with scrolling
- Context-aware health metrics
- Keyboard-driven navigation
- Apply confirmation with safety checks

---

## 🚀 Installation

### New Installation

```bash
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo sh
```

### Upgrade from Previous Version

```bash
# Same command works for upgrades
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo sh

# Or use Anna's built-in updater
annactl update
```

The installer automatically:
1. Stops the old daemon
2. Installs new binaries
3. Restarts the daemon
4. Preserves your configuration

---

## 📖 Quick Start

```bash
# Get personalized recommendations
annactl advise

# Interactive TUI (NEW & IMPROVED!)
annactl tui

# Filter by category
annactl advise security
annactl advise hardware
annactl advise development

# Apply recommendations
annactl apply 1          # Apply single recommendation
annactl apply 1-5        # Apply range
annactl apply 1,3,5      # Apply multiple

# System health check
annactl doctor

# Auto-fix issues
annactl doctor --fix

# Check daemon status
annactl status

# View system report
annactl report
```

---

## 🔄 Upgrade Notes

### From Beta.45-48
- All configuration preserved automatically
- TUI navigation is now more intuitive
- Shell commands will execute correctly (especially pacman with $(...))
- Health score provides better context
- Details view scrolling available for long content

### Configuration
Anna's config is stored in `~/.config/anna/config.toml` (user) or `/etc/anna/config.toml` (system-wide)

---

## 🐛 Bug Fixes (Beta.49)

**Critical Fixes:**
- ✅ Fixed command execution for shell syntax (`$(...)`, `&&`, `|`)
- ✅ Fixed TUI category header selection bug
- ✅ Fixed wrong advice showing when navigating categories
- ✅ Fixed applied advice disappearing prematurely

**TUI Improvements:**
- ✅ Category headers no longer selectable
- ✅ Correct advice-to-index mapping throughout
- ✅ Details view scrolling for long content
- ✅ Improved informational notice display
- ✅ Better word wrapping for all terminal sizes
- ✅ Context-aware health score display
- ✅ Clearer risk sort with category indicators

**User Experience:**
- ✅ Simplified apply shortcut (just "a" now)
- ✅ All user messages extremely clear and actionable
- ✅ Better visual hierarchy in all views
- ✅ Consistent keyboard navigation patterns

---

## 🔮 What's Next

### Short Term (Beta.50-52)
- Ignore/dismiss functionality in TUI
- Category and priority filtering
- Rollback by history number
- Enhanced UI dialogs (whiptail/zenity/gum)

### Mid Term (Beta.53-60)
- Bundle rollback system
- Community data integration (privacy-preserving)
- Enhanced relevance filtering
- Workflow bundle expansion

### Long Term (v1.0.0)
- Full autonomy mode (with safety guardrails)
- Learning system refinements
- Multi-system support
- Plugin system for community extensions

---

## 💬 Community & Support

- **GitHub**: https://github.com/jjgarcianorway/anna-assistant
- **Issues**: https://github.com/jjgarcianorway/anna-assistant/issues
- **Contributing**: See CONTRIBUTING.md

---

## 🙏 Credits

**Built with:**
- Rust 🦀 - For speed, safety, and reliability
- Tokio - Async runtime
- Ratatui - Beautiful TUI framework
- Crossterm - Cross-platform terminal control
- owo-colors - Terminal color support
- Arch Wiki - Source of truth for all recommendations

**Philosophy:**
- **Privacy First** - Your data stays on your system
- **Non-Intrusive** - Anna suggests, you decide
- **Arch-Native** - Follows Arch philosophy and Wiki
- **Transparent** - Full audit logs, clear explanations
- **User-Focused** - Interface must be extremely clear

---

**Anna v1.0.0-beta.49** - Making Arch Linux easier, one recommendation at a time! 🌟
