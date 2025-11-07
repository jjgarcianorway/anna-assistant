# Anna Assistant - Feature Roadmap

This document tracks requested features and improvements from user feedback.

---

## 🎯 RELEASE STRATEGY: 1.0 Stability → 2.0 Innovation

**Current Status:** Release Candidate Phase (1.0.0-rc.5)

### Philosophy

**1.0 Branch:** Stability First
- ✅ Feature freeze - no new features
- ✅ Focus on testing and bug fixes
- ✅ TUI temporarily disabled
- ✅ All CLI commands must work reliably
- ✅ RC releases until stable (rc.1 → rc.2 → ... → 1.0.0)

**2.0 Branch:** Innovation & Better UX
- Better command verbs and interface
- Enhanced TUI with improved UX
- More intelligent recommendations
- Advanced bundle management
- Better configuration intelligence

### Why This Approach?

1. **Solid Foundation:** Get 1.0 stable with everything working
2. **User Trust:** Reliable 1.0 builds confidence
3. **Freedom to Innovate:** 2.0 can break compatibility for better UX
4. **Iterative Improvement:** RC process finds and fixes issues

### Testing Plan

See `TESTING.md` for complete testing checklist.

**Release Criteria for 1.0:**
- [ ] All core CLI commands work
- [ ] Daemon reliable and stable
- [ ] Intelligence systems detect correctly
- [ ] No critical bugs
- [ ] Installation tested on fresh Arch
- [ ] Update path tested from Beta.114

---

## 🚀 RELEASE CANDIDATES (1.0.0-rc.X)

### RC.6 - Smart Context-Aware Filtering (Current) 🧠
**Status:** ✅ COMPLETED
**Released:** 2025-11-07

**Changes:**
- **MAJOR:** Intelligent advice filtering based on actual system hardware/software
- Added `Requirement` system with 17 types of checks (audio, display, GPU, Bluetooth, etc.)
- Enhanced `Advice` struct with `requires` field
- Updated RPC server with smart filtering pipeline
- Modified 375+ advice items with requirements
- Dynamic bundle filtering (Hyprland/Wayland bundles adapt to system)
- Zero false recommendations - only show what makes sense!

**Impact:**
- Headless servers: No GUI recommendations ✅
- No Bluetooth hardware: No Bluetooth tools ✅
- Desktops: No laptop-specific tools ✅
- No audio system: No media players/audio tools ✅

### RC.5 - Simplified Command Syntax
**Status:** ✅ COMPLETED
**Released:** 2025-11-07

**Changes:**
- Simplified `history` command: `annactl history 30` (no --days flag needed)
- Simplified `config` command: `annactl config key` or `annactl config key value` (no get/set actions)
- More intuitive positional arguments throughout
- Updated help text and documentation

### RC.4 - Compact Advice Summary
**Status:** ✅ COMPLETED
**Released:** 2025-11-07

**Changes:**
- Redesigned `annactl advise` to show compact summary by default (~20 lines)
- Category-based drill-down: `annactl advise security`
- Special "all" keyword: `annactl advise all`
- Reduces overwhelming output while keeping full details accessible

### RC.3 - Health Integration
**Status:** ✅ COMPLETED
**Released:** 2025-11-07

**Changes:**
- Merged `health` command into `status` command
- Reduced from 17 to 15 commands (WikiCache and Health removed)
- Streamlined user experience

### RC.2 - Safety First
**Status:** ✅ COMPLETED
**Released:** 2025-11-07

**Changes:**
- Added confirmation prompts with previews before all apply operations
- Improved update command to show version comparison before asking for permission
- "Always inform, then do" principle implemented
- Sudo only requested when actually needed

### RC.1 - Feature Freeze
**Status:** ✅ COMPLETED
**Released:** 2025-11-06

**Changes:**
- Disabled TUI for 1.0 (will return in 2.0 with better UX)
- Created comprehensive testing checklist (TESTING.md)
- Established 1.0 stability focus

### 🔮 RC.7+ - Future Polish (If Needed)
**Status:** 🚧 OPTIONAL

**Potential improvements:**
- Enhanced error messages with actionable suggestions
- More requirement types as edge cases are discovered
- Alternative software category system
- Interactive category hide/unhide commands
- Performance optimizations if needed

**Note:** RC.6 achieves the core goals for 1.0. Additional RCs only if testing reveals issues.

---

## ✅ COMPLETED (Beta.61-114)

### Show Command Output When Applying
**Status:** ✅ COMPLETED (Beta.60)

**Solution Implemented:**
- ✅ TUI modal overlay displays command output in real-time
- ✅ Scrollable output (↑↓, PageUp/PageDown, j/k)
- ✅ Yellow border while executing, green when complete
- ✅ Cannot close until finished (prevents accidents)
- ✅ Shows stdout/stderr as command runs

### Remove Applied Items from List Immediately
**Status:** ✅ COMPLETED (Beta.63)

**Solution Implemented:**
- ✅ TUI: Removes item immediately after successful apply
- ✅ TUI: Calls update().await right after apply
- ✅ CLI: Invalidates cache after successful applies
- ✅ Shows tip to run 'annactl advise' for updated list

### Smart Advice Dependencies & Bundle Awareness
**Status:** ✅ COMPLETED (Beta.62)

**Solution Implemented:**
- ✅ Added `satisfies: Vec<String>` field to Advice struct
- ✅ Implemented filter_satisfied_advice() in RPC server
- ✅ Parses audit log to find applied advice IDs
- ✅ Automatically filters out advice satisfied by applied items
- ✅ Builder method: `.with_satisfies(vec!["advice-id"])`

### Auto-Update Always-On
**Status:** ✅ COMPLETED (Beta.61)

**Solution Implemented:**
- ✅ Removed Tier 3 autonomy requirement for auto-updates
- ✅ Anna now updates herself automatically every 24 hours
- ✅ Desktop notifications when updates complete
- ✅ Zero risk - only updates Anna, not user's system

### Update Detection & Self-Update Fixes
**Status:** ✅ COMPLETED (Beta.64-65)

**Solution Implemented:**
- ✅ Beta.64: Asset name matching handles platform suffixes
- ✅ Beta.64: Works with GitHub Actions automatic releases
- ✅ Beta.65: Changed from cp to mv for binary replacement
- ✅ Beta.65: Fixes "Text file busy" error

### Safer Install/Uninstall Scripts
**Status:** ✅ COMPLETED (Beta.66-67)

**Solution Implemented:**
- ✅ Beta.66: Removed requirement to pipe to sudo
- ✅ Beta.66: Scripts use sudo internally when needed
- ✅ Beta.66: User confirmation required before any changes
- ✅ Beta.67: Fixed interactive prompts when piping (reads from /dev/tty)
- ✅ New command: `curl ... | sh` (safer than `curl ... | sudo sh`)

### 3. Improve "Multiple System Errors" Details
**Status:** ✅ PARTIALLY DONE

**What's Done:**
- ✅ Shows 3 actual error samples in the reason
- ✅ Explains advice lifecycle
- ✅ Uses less pager

**Still TODO:**
- [ ] Detect common error patterns
- [ ] Suggest specific fixes
- [ ] Link to wiki articles for error types

### Configuration Intelligence Suite (Beta.70-74)
**Status:** ✅ COMPLETED - MAJOR MILESTONE!

**Vision:** Anna analyzes your actual config files and provides intelligent, context-aware recommendations.

#### Beta.70 - Hyprland Configuration Intelligence
**Status:** ✅ COMPLETED

**Solution Implemented:**
- ✅ Auto-detects Hyprland installation and config location
- ✅ Parses hyprland.conf for existing keybindings
- ✅ Detects missing functionality:
  - Volume controls (wpctl/pamixer)
  - Brightness controls (brightnessctl)
  - Screenshot tools (grim+slurp)
  - Media controls (playerctl)
  - Application launchers (rofi/wofi/tofi)
  - Status bars (waybar)
  - Wallpaper managers (swaybg/hyprpaper)
  - Lock screens (swaylock)
  - Notification daemons (mako/dunst)
- ✅ Generates recommendations that:
  - Install missing packages
  - Add keybindings to your config
  - Include wiki references
  - Work in one click!

#### Beta.71 - Shell Configuration Intelligence
**Status:** ✅ COMPLETED

**Solution Implemented:**
- ✅ Detects shell type (bash/zsh/fish)
- ✅ Analyzes shell config files
- ✅ Detects modern CLI tools:
  - Starship (universal prompt)
  - eza (modern ls)
  - bat (cat with highlighting)
  - fd (user-friendly find)
  - fzf (fuzzy finder)
  - zoxide (smart cd)
  - ripgrep (fast grep)
- ✅ Shell-specific enhancements:
  - Syntax highlighting
  - Autosuggestions
  - Useful aliases
  - Git shortcuts
- ✅ Helps 100% of users (everyone has a shell!)

#### Beta.72 - Terminal Emulator Intelligence
**Status:** ✅ COMPLETED

**Solution Implemented:**
- ✅ Detects terminal emulator (alacritty, kitty, wezterm, foot, st, gnome-terminal, konsole, xterm)
- ✅ Analyzes terminal configs
- ✅ Recommends:
  - Nerd Fonts (JetBrainsMono, FiraCode, etc.)
  - Color schemes (Catppuccin, Nord, Dracula)
  - Terminal upgrades (modern vs outdated)
  - Font size optimization
- ✅ Terminal-specific config generation
- ✅ Helps 100% of users (everyone uses a terminal!)

#### Beta.73 - Git Configuration Intelligence
**Status:** ✅ COMPLETED

**Solution Implemented:**
- ✅ Detects git installation
- ✅ Analyzes ~/.gitconfig
- ✅ Checks critical settings:
  - user.name / user.email (required for commits)
  - init.defaultBranch (main vs master)
  - credential.helper (password caching)
  - push.default (safe behavior)
  - pull.rebase (cleaner history)
- ✅ Recommends:
  - Essential aliases (st, co, br, lg)
  - Visual diff/merge tools
  - Quality-of-life features (colors, pruning)
- ✅ Helps all developers!

#### Beta.74 - i3 Window Manager Configuration Intelligence
**Status:** ✅ COMPLETED

**Solution Implemented:**
- ✅ Auto-detects i3 installation
- ✅ Analyzes ~/.config/i3/config or ~/.i3/config
- ✅ Detects missing functionality:
  - Volume controls (pactl/pamixer/amixer)
  - Brightness controls (brightnessctl/light/xbacklight)
  - Screenshot tools (maim/scrot/flameshot)
  - Media controls (playerctl)
  - Application launchers (rofi/dmenu/j4-dmenu)
  - Status bars (i3status/i3blocks/polybar/yambar)
  - Wallpaper managers (feh/nitrogen/xwallpaper)
  - Lock screens (i3lock/betterlockscreen)
  - Notification daemons (dunst/mako)
  - Compositors (picom/compton)
- ✅ X11-specific tool recommendations (vs Wayland)
- ✅ Generates complete recommendations that:
  - Install missing packages
  - Add keybindings to your config
  - Include wiki references
  - Work in one click!

#### Beta.75 - Sway Window Manager Configuration Intelligence
**Status:** ✅ COMPLETED - WINDOW MANAGER TRILOGY COMPLETE!

**Solution Implemented:**
- ✅ Auto-detects Sway installation
- ✅ Analyzes ~/.config/sway/config or ~/sway/config
- ✅ Detects missing functionality:
  - Volume controls (wpctl/pactl/pamixer)
  - Brightness controls (brightnessctl/light)
  - Screenshot tools (grim/slurp/grimblast/wayshot)
  - Media controls (playerctl)
  - Application launchers (wofi/rofi/tofi/fuzzel/bemenu)
  - Status bars (waybar/i3status/yambar/swaybar)
  - Wallpaper managers (swaybg/swww/mpvpaper/wpaperd)
  - Lock screens (swaylock)
  - Notification daemons (mako/dunst)
  - Idle management (swayidle)
- ✅ Wayland-native tool recommendations (vs X11)
- ✅ i3-compatible configuration (easy migration from X11)
- ✅ Generates complete recommendations that:
  - Install missing Wayland packages
  - Add keybindings to your config
  - Include wiki references
  - Work in one click!

**Impact - Window Manager Trilogy Complete:**
Anna now provides comprehensive environment intelligence across:
1. **Desktop/WM** (Hyprland + i3 + Sway) - Complete WM coverage for X11 AND Wayland!
   - Hyprland: Modern Wayland compositor with animations
   - i3: Classic X11 tiling window manager
   - Sway: Wayland-based i3 alternative (i3-compatible)
2. **Shell** (bash/zsh/fish) - CLI tools
3. **Terminal** (alacritty/kitty/etc) - Emulator
4. **Git** (version control) - Development workflow

**Philosophy:** Like gnome-meta-package, but smarter - analyzes YOUR config and recommends only what YOU need!

#### Beta.76 - GNOME Desktop Environment Intelligence
**Status:** ✅ COMPLETED

**Solution Implemented:**
- ✅ Auto-detects GNOME installation (gnome-shell)
- ✅ Analyzes GNOME configuration and installed extensions
- ✅ Detects GNOME version
- ✅ Detects missing GNOME components:
  - GNOME Tweaks (essential for customization)
  - Extensions App (GNOME 40+ extension management)
  - dconf Editor (advanced settings)
  - GTK themes (Arc, Adapta, Materia, Orchis)
  - Icon themes (Papirus, Numix, Tela)
  - Essential shell extensions (Dash to Dock, AppIndicator, Blur My Shell, User Themes, Clipboard Indicator)
  - Wayland-specific tools (wl-clipboard, wtype, grim, slurp)
- ✅ Analyzes existing customizations:
  - Installed GTK themes
  - Installed icon themes
  - GNOME Shell extensions
  - Wayland vs X11 session
- ✅ Generates recommendations for:
  - Essential customization tools
  - Appearance improvements
  - Productivity extensions
  - Performance optimizations
  - Wayland-native utilities
- ✅ Provides GNOME-specific best practices and tips

**Impact - Desktop Environment Intelligence Begins:**
Anna now provides comprehensive environment intelligence for:
1. **Window Managers** (Hyprland + i3 + Sway) - Complete WM coverage for X11 AND Wayland
2. **Desktop Environments** (GNOME) - Full DE customization and enhancement
3. **Shell** (bash/zsh/fish) - CLI tools
4. **Terminal** (alacritty/kitty/etc) - Emulator
5. **Git** (version control) - Development workflow

This expands Anna's intelligence from just window managers to FULL desktop environments!

**Next Steps:**
- XFCE/MATE/Cinnamon support
- Cross-DE theming consistency

#### Beta.77 - KDE Plasma Desktop Environment Intelligence
**Status:** ✅ COMPLETED

**Solution Implemented:**
- ✅ Auto-detects KDE Plasma installation (plasmashell)
- ✅ Analyzes KDE configuration and installed components
- ✅ Detects Wayland vs X11 session
- ✅ Detects missing KDE components:
  - KDE Connect (mobile device integration)
  - Kvantum (advanced Qt theming engine)
  - kde-gtk-config (GTK theme integration for consistency)
  - Latte Dock (macOS-like dock)
  - KWin effects (desktop effects)
  - Plasma widgets (plasmoids)
  - Wayland-specific tools (wl-clipboard, xwaylandvideobridge)
- ✅ Analyzes existing setup:
  - KWin effects configuration
  - Installed Plasma widgets
  - Wayland vs X11 session
- ✅ Generates recommendations for:
  - Mobile integration (KDE Connect)
  - Visual appearance (Kvantum, Latte Dock)
  - Cross-toolkit consistency (GTK themes)
  - Productivity enhancements (widgets)
  - Performance optimizations
  - Wayland migration guidance
- ✅ Provides KDE-specific best practices and tips

**Impact - Desktop Environment Duopoly Covered:**
Anna now provides comprehensive environment intelligence for:
1. **Window Managers** (Hyprland + i3 + Sway) - Complete WM coverage for X11 AND Wayland
2. **Desktop Environments** (GNOME + KDE Plasma) - TWO most popular DEs!
3. **Shell** (bash/zsh/fish) - CLI tools
4. **Terminal** (alacritty/kitty/etc) - Emulator
5. **Git** (version control) - Development workflow

KDE + GNOME represent ~80% of Linux desktop users! Anna now supports both!

**Next Steps:**
- XFCE intelligence (lighter DE)
- MATE/Cinnamon support
- Cross-DE theming consistency

---

## 📊 MEDIUM PRIORITY (Smart Features)

### 4. Auto-Detect and Improve Current Terminal
**Status:** 📋 TODO

**Solution:**
- [ ] Detect current terminal emulator
- [ ] Suggest terminal-specific improvements
- [ ] Check emoji/icon support

### 5. Desktop Environment Intelligence
**Status:** 📋 TODO

**Vision:** Anna detects your desktop/WM and applies complete, working configurations.

#### 5a. Hyprland Full Setup
**User Request:** *"I also hope anna is able to improve my hyprland config, add fixes for volume, brightness, mute, etc..."*

**Solution:**
- [ ] Detect Hyprland installation and config location
- [ ] Parse existing hyprland.conf
- [ ] Add missing keybindings:
  - Volume control (wpctl, pamixer, or pactl)
  - Brightness control (brightnessctl or light)
  - Mute/unmute toggle
  - Screenshot tools (grim + slurp)
  - Media controls (playerctl)
- [ ] Install required packages automatically
- [ ] Merge with existing config (don't overwrite)
- [ ] Test keybindings work
- [ ] Provide user with cheat sheet

**Example Output:**
```
✓ Detected Hyprland
✓ Found config: ~/.config/hypr/hyprland.conf
→ Adding volume controls (wpctl)...
→ Adding brightness controls (brightnessctl)...
→ Installing missing packages...
✓ Applied! New keybindings:
  - SUPER + F11: Volume down
  - SUPER + F12: Volume up
  - SUPER + F10: Mute toggle
  - SUPER + F5: Brightness down
  - SUPER + F6: Brightness up
```

#### 5b. GTK Theming & Configuration
**User Request:** *"same for any other desktop... improve GTK, QT or whatever system the user is using"*

**Solution:**
- [ ] Detect GTK version (GTK2, GTK3, GTK4)
- [ ] Check for theme installation
- [ ] Suggest popular themes (Arc, Adapta, Numix, etc.)
- [ ] Configure gtk-3.0/settings.ini with best practices:
  - Enable dark mode if user prefers
  - Set icon theme
  - Configure font rendering (hinting, antialiasing)
  - Enable animations
- [ ] Apply consistent theme across all GTK apps
- [ ] Detect and fix theme inconsistencies

#### 5c. Qt/KDE Theming & Configuration
**Solution:**
- [ ] Detect Qt version and Plasma version
- [ ] Check kvantum for unified theming
- [ ] Configure kdeglobals for consistent appearance
- [ ] Set up Breeze or user's preferred theme
- [ ] Configure:
  - Widget style
  - Color scheme
  - Icon theme
  - Font rendering
  - Window decorations
- [ ] Ensure GTK apps match Qt theme (kde-gtk-config)

#### 5d. Compositor & Effects
**Solution:**
- [ ] Detect compositor (picom, hyprland built-in, kwin, etc.)
- [ ] Configure for best performance/quality balance:
  - Shadows and blur
  - Animations and transitions
  - VSync settings
  - Backend (glx vs xrender)
- [ ] Add keybindings for toggling effects
- [ ] Fix screen tearing if detected

#### 5e. Display Manager (Login Screen)
**Solution:**
- [ ] Detect display manager (GDM, SDDM, LightDM, etc.)
- [ ] Configure theme to match desktop
- [ ] Enable autologin if user wants (with warning)
- [ ] Set up proper keyboard layout
- [ ] Configure multi-monitor at login

#### 5f. Notification Daemon
**Solution:**
- [ ] Detect notification daemon (dunst, mako, etc.)
- [ ] Configure for aesthetics and usability:
  - Position and size
  - Timeout settings
  - Colors matching theme
  - Urgency levels
  - Action buttons
- [ ] Add keybinding for notification history
- [ ] Configure do-not-disturb mode

### 6. Intelligent Config File Improvements
**Status:** 📋 TODO

**Examples:** vim, shell, terminal, any user software

**Solution:**
- [ ] Detect config files for installed software
- [ ] Suggest improvements from best practices
- [ ] Auto-apply common configs
- [ ] Merge with existing (never overwrite user customizations)

**Specific Configs to Handle:**

#### 6a. Vim/Neovim
- [ ] Detect .vimrc or init.vim
- [ ] Add if missing:
  - Syntax highlighting
  - Line numbers
  - Tabs → spaces (2 or 4)
  - Smart indenting
  - Search highlighting
  - Mouse support
  - Plugin manager (vim-plug or packer)
- [ ] Suggest popular plugins based on languages detected

#### 6b. Shell (Bash/Zsh/Fish)
- [ ] Detect shell and config file
- [ ] Add useful aliases:
  - ls → lsd/exa with icons
  - Common git shortcuts
  - Package manager shortcuts
  - System maintenance
- [ ] Set up better prompt (Starship recommended)
- [ ] Enable useful features:
  - Auto-completion
  - History search
  - Directory shortcuts (z/autojump)
  - Syntax highlighting

#### 6c. Git
- [ ] Configure user name/email if missing
- [ ] Add useful aliases
- [ ] Set up better diff and merge tools
- [ ] Configure credential caching
- [ ] Enable colors and better formatting

#### 6d. Terminal Emulator
- [ ] Detect terminal config
- [ ] Set font to Nerd Font
- [ ] Configure colors (use popular scheme)
- [ ] Set proper transparency if supported
- [ ] Configure keybindings
- [ ] Enable ligatures if supported

---

## 🌟 LONG-TERM (Advanced Intelligence)

### 7. Full Arch Wiki Integration
**Status:** 💭 PLANNING

**Vision:** Anna reads Arch Wiki and applies intelligent solutions automatically.

**User Request:** *"This is why I want annactl to have the whole archlinux wiki references... So she can read the article and apply the right solutions, config files, improvements... anna knows best :)"*

**Solution:**
- [ ] Parse wiki pages for configs and solutions
- [ ] Build knowledge base from wiki
- [ ] Extract configuration snippets automatically
- [ ] Understand context and apply appropriate fixes
- [ ] Apply wiki-based recommendations intelligently
- [ ] Learn best practices from wiki examples

**Implementation Approach:**
1. **Offline Wiki Cache** (already have 40+ pages)
2. **Wiki Parser** - Extract actionable information:
   - Config file examples
   - Command sequences
   - Common issues and solutions
   - Best practices
3. **Knowledge Graph** - Build relationships:
   - Package → Config file → Options
   - Problem → Solution → Wiki section
   - Desktop → Theme → Required packages
4. **Smart Applier** - Apply context-aware:
   - Detect user's environment
   - Find relevant wiki section
   - Extract and adapt config
   - Apply safely (backup first)
   - Validate it works

**Example:**
```
User: Has Hyprland + wants volume control
Anna: Reads wiki.archlinux.org/title/Hyprland
      Finds audio control section
      Extracts wpctl commands
      Generates keybinding config
      Merges with user's config
      Installs missing packages
      Tests it works
      ✓ Done!
```

### 8. Full Setup Solutions
**Status:** 💭 PLANNING

**User Request:** *"We need anna to apply full solutions or full setups for current setups"*

**Vision:** One-command complete environment setup.

**Examples:**

#### "Setup Hyprland Perfectly"
```bash
annactl setup hyprland --complete
```
Installs and configures:
- Hyprland + dependencies
- Waybar (status bar)
- Wofi (app launcher)
- Volume/brightness controls
- Screenshot tools
- Wallpaper manager
- Lock screen
- Notification daemon
- Theme and icons
- All keybindings configured
- User manual generated

#### "Setup Development Environment"
```bash
annactl setup dev --language rust,python,go
```
Installs and configures:
- Language toolchains
- LSP servers
- Debuggers
- Linters and formatters
- VS Code / Neovim setup
- Git configuration
- Docker setup
- Useful aliases
- Project templates

#### "Setup Gaming"
```bash
annactl setup gaming
```
Installs and configures:
- Steam + Proton
- Lutris
- Wine + DXVK
- GameMode
- MangoHud
- Controller support
- Performance tweaks
- GPU drivers optimal settings

**Key Principle:** Complete, working setups. Not just packages - full configurations.

### 9. Window Manager Complete Bundles
**Status:** 💭 PLANNING

**User Vision:** *"lets imagine that I just install minimal arch... and then I apply a hyprland bundle... and it does everything for me, according to my hardware and fixes: drivers, bluetooth, nmtui, hyprland, waybar, nautilus, mpv, etc.."*

**Philosophy:** Same concepts across WMs (Hyprland, i3, sway) - different methods, same ideas.

#### Core Components Every WM Needs:

**Application Launcher:**
- Detect WM type → Pick appropriate launcher
- Hyprland/Sway: rofi-wayland, tofi, wofi, bemenu
- i3: rofi, dmenu

**Window Management:**
- Maximize, fullscreen, minimize
- Workspace switching (forward/back)
- Window focusing (directional)
- Floating toggle
- Tiling layouts

**Multimedia Keys:**
- Volume up/down/mute (wpctl, pamixer, pactl)
- Brightness up/down (brightnessctl, light)
- Media controls (playerctl): play/pause/next/prev
- Screenshot (grim+slurp for Wayland, maim/scrot for X11)

**Wallpaper Management:**
- swaybg (Wayland) / feh (X11)
- Download nice wallpapers (unsplash API?)
- Use imagemagick to resize to screen resolution
- Wallpaper rotation/slideshow

**Essential Tools:**
- File manager (GUI: nautilus/dolphin/thunar, TUI: ranger/nnn/lf)
- Terminal emulator
- Text editor
- Media player (mpv recommended)
- Network manager (nmtui for TUI, nm-applet for GUI)

**Keyboard & Input:**
- Ensure keyboard layout matches TTY
- Configure input methods
- Set up touchpad (if laptop)

**System Integration:**
- Bluetooth (bluez + blueman/bluetuith)
- Audio system (pipewire recommended)
- Drivers (GPU, WiFi, etc.)
- Power management (laptop)

#### Bundle Variants - Same WM, Different Flavors

**User Insight:** *"You can even have different bundles or options per each desktop... like hyprland minimal_resources, hyprland terminal (as much as possible TUI or CLI), hyprland GTK (everything GTK), hyprland QT (everything QT)"*

##### Variant 1: Minimal Resources
```bash
annactl setup hyprland --variant minimal
```
**Focus:** Lightweight, fast, low RAM/CPU
- Alacritty (minimal terminal)
- Fuzzel or bemenu (lightweight launcher)
- No compositor effects
- Light apps only
- Foot terminal alternative
- Text-based where possible
- Target: <500MB RAM idle

##### Variant 2: Terminal/CLI Focused
```bash
annactl setup hyprland --variant terminal
```
**Focus:** TUI/CLI tools, keyboard-driven
- Ranger/nnn (file manager)
- Bluetuith (bluetooth TUI)
- nmtui (network)
- htop/btop (system monitor)
- cmus/mpd (music)
- Newsboat (RSS)
- w3m/lynx (web browser)
- Minimal GUI only where necessary

##### Variant 3: GTK Ecosystem
```bash
annactl setup hyprland --variant gtk
```
**Focus:** GTK apps, GNOME tools
- Nautilus (file manager)
- GNOME Terminal
- GNOME apps (calculator, calendar, etc.)
- GTK theme (Adwaita, Arc, etc.)
- GTK-based launcher
- Evince (PDF)
- EOG (image viewer)

##### Variant 4: Qt/KDE Ecosystem
```bash
annactl setup hyprland --variant qt
```
**Focus:** Qt apps, KDE tools
- Dolphin (file manager)
- Konsole (terminal)
- Kate (text editor)
- KDE apps (Okular, Spectacle, etc.)
- Kvantum theming
- Qt-based launcher
- Consistent Qt look

##### Variant 5: Full-Featured / Kitchen Sink
```bash
annactl setup hyprland --variant full
```
**Focus:** Beautiful, featured, polished
- Best of everything
- All effects enabled
- Multiple options for each tool
- Eye candy (blur, shadows, animations)
- Beautiful themes
- Target: "unixporn ready"

#### Research Methodology

**User Insight:** *"Check against the wiki and reddit to see the common programs people use in unixporn for the different solutions and collect list of programs that people combine together."*

**Data Sources:**
1. **Arch Wiki** - Canonical configurations
2. **r/unixporn** - Popular combinations
3. **GitHub dotfiles** - Common patterns
4. **ArchWiki "List of applications"** - Categories
5. **AUR popularity** - Vote counts

**Analysis:**
- Track most mentioned combinations
- Identify "golden setups" (what works well together)
- Note compatibility (GTK apps in Qt, vice versa)
- Document gotchas and fixes
- Keep compatibility matrix

**Popular Combinations (Research TODO):**
- Hyprland + Waybar + Rofi + Alacritty + ?
- i3 + Polybar + Rofi + Kitty + ?
- Sway + Waybar + Wofi + Foot + ?
- Document and codify

#### Bundle Structure

Each bundle includes:

**1. Base System:**
- Window manager itself
- Display server (Wayland/X11)
- Session management
- Basic daemons

**2. User Interface:**
- Status bar
- Application launcher
- Notification daemon
- Wallpaper manager
- Lock screen

**3. Applications:**
- Terminal
- File manager
- Text editor
- Web browser
- Media player
- PDF viewer
- Image viewer

**4. System Tools:**
- Network management
- Bluetooth
- Audio control
- Power management
- Brightness control

**5. Keybindings:**
- Window management
- Workspaces
- Applications
- Multimedia
- System controls

**6. Theme & Appearance:**
- GTK/Qt themes
- Icon theme
- Cursor theme
- Fonts (including Nerd Fonts)
- Colors

**7. Hardware-Specific:**
- GPU drivers (detect Nvidia/AMD/Intel)
- WiFi drivers
- Bluetooth adapters
- Laptop-specific (battery, touchpad)

**8. Configuration Files:**
- All configs generated
- Keybinding reference sheet
- Quick start guide
- Troubleshooting notes

#### From Minimal Arch to Fully Configured

**The Dream Workflow:**
```bash
# User installs minimal Arch
# Network is working, that's it

# Install Anna
curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | sudo sh

# Apply complete setup
annactl setup hyprland --variant gtk

# 15 minutes later...
# ✓ Hyprland configured
# ✓ Waybar with all widgets
# ✓ Rofi launcher (SUPER+D)
# ✓ Volume keys working
# ✓ Brightness keys working
# ✓ WiFi configured (nmtui)
# ✓ Bluetooth working
# ✓ Nautilus file manager
# ✓ Firefox installed
# ✓ MPV for videos
# ✓ Beautiful theme applied
# ✓ Wallpaper set
# ✓ Everything works!

# User logs in → productive immediately
```

**No manual configuration. No wiki diving. Just works.** ✨

#### Implementation Notes

**Hardware Detection:**
```rust
fn detect_hardware() -> HardwareProfile {
    HardwareProfile {
        gpu: detect_gpu(), // Nvidia? AMD? Intel?
        laptop: is_laptop(),
        wifi: has_wifi(),
        bluetooth: has_bluetooth(),
        touchpad: has_touchpad(),
        screen_resolution: get_resolution(),
        // ... etc
    }
}

fn customize_bundle(bundle: &Bundle, hw: &HardwareProfile) -> Bundle {
    // Add Nvidia-specific configs if needed
    // Add laptop power management if laptop
    // Skip bluetooth if no hardware
    // Etc.
}
```

**Goal:** Not just functional - ACTUALLY NICE TO USE!

**User Quote:** *"It does not need to be extremely beautiful but fully functional."*
**Anna's Promise:** Why not both? 😊

---

## 📊 Implementation Priority

### ✅ Completed This Session (Beta.61-67):
- ✅ Better error details (Beta.60)
- ✅ Command output display in TUI (Beta.60)
- ✅ Remove applied items immediately (Beta.63)
- ✅ Smart advice dependencies (Beta.62)
- ✅ Auto-update always-on (Beta.61)
- ✅ Update detection fixed (Beta.64)
- ✅ Self-update fixed (Beta.65)
- ✅ Safer install scripts (Beta.66-67)

### 📋 Next Up:

1. **Immediate (Next Session):**
   - Hyprland keybinding detection & improvement
   - Application launchers (dmenu, rofi, tofi)
   - Multimedia key configuration
   - Terminal detection & configuration

2. **Short-term:**
   - GTK/Qt theme consistency
   - Config file intelligence (vim, shell, git)
   - Wallpaper management (swaybg, downloads, imagemagick)
   - File manager recommendations (TUI vs GUI)

3. **Medium-term:**
   - Desktop environment full setup bundles
   - Notification daemon configuration
   - Complete window manager bundles (minimal, terminal, GTK, Qt, full)

4. **Long-term (Ongoing):**
   - Wiki parsing and knowledge extraction
   - Full setup solutions (one-command environments)
   - Research from r/unixporn for popular program combinations
   - Advanced intelligence and learning

---

**Last Updated:** 2025-11-06 (End of Day)
**Latest Version:** Beta.75 (Window Manager Trilogy Complete!)
**User Feedback Sessions:** Beta.60-75
**Your Vision:** *"Functionality first! Anna should improve whatever I'm using - from minimal Arch to fully configured environment with everything working perfectly"* ✨

**Progress Today (Beta.70-75):**
- ✅ Beta.70: Hyprland Configuration Intelligence (Wayland)
- ✅ Beta.71: Shell Configuration Intelligence (bash/zsh/fish)
- ✅ Beta.72: Terminal Emulator Intelligence (all major emulators)
- ✅ Beta.73: Git Configuration Intelligence (development workflow)
- ✅ Beta.74: i3 Window Manager Intelligence (X11)
- ✅ Beta.75: Sway Window Manager Intelligence (Wayland)

**Major Milestone Achieved:**
**Window Manager Trilogy Complete!** Anna now intelligently configures Hyprland, i3, AND Sway - providing comprehensive support for both X11 and Wayland ecosystems. Combined with Shell, Terminal, and Git intelligence, Anna now understands your ENTIRE development environment! 🎉

---

## 🔄 IN PROGRESS / PLANNED (Beta.83+)

### Beta.111 - Configuration Management for Bundles
**Status:** ✅ COMPLETED

**User Feedback:**
> "hyprland-setup is one step only?? Cmon my friend... I have even given you my template for inspiration"
> "I know a bundle is complicated... several commands in a row, adding changes to config files... many steps"

**Solution Implemented:**
- ✅ Added `config_files` HashMap to WMComponents struct
- ✅ New `.config(component_id, template_subpath)` builder method
- ✅ Bundle build() now generates config copy steps
- ✅ Hyprland bundle includes config setup for: hypr, waybar, kitty, rofi, mako
- ✅ Commands copy from `/usr/share/anna/templates/.config/` to `~/.config/`
- ✅ Multi-step bundles: 8 package installs + 5 config setups = 13 steps total!
- ✅ Each step shown with immediate command display before execution
- ✅ Live streaming output for full transparency

**Example Output:**
```
[1/13] Install rofi-wayland application launcher
→ Executing: pacman -S --noconfirm rofi-wayland
...

[6/13] Setup hypr configuration
→ Executing: mkdir -p $HOME/.config/hypr && cp -r /usr/share/anna/templates/.config/hypr/* $HOME/.config/hypr/
...
```

**Files Modified:**
- `crates/annad/src/bundles/mod.rs` - Added config_files field and copying logic
- `crates/annad/src/bundles/wayland_compositors.rs` - Updated Hyprland bundle with configs
- Bundles now truly SET UP the desktop, not just install packages!

---

### Beta.112 - Hardware-Aware Bundles & Multimedia Tools
**Status:** ✅ COMPLETED

**User Feedback:**
> "ensure that hyprland setup is perfect... volume control, bluetooth, wifi, date formatted... everything a user can need"

**Solution Implemented:**
- ✅ Added `audio_control` field for pamixer/pavucontrol integration
- ✅ Added `brightness_control` field for laptop brightness keys
- ✅ Hardware-aware installation: brightness control only on laptops
- ✅ Multimedia key functionality: Volume up/down/mute, brightness control
- ✅ Conditional logic: `facts.user_preferences.uses_laptop`
- ✅ Smart detection: Only installs tools if not already present

**Files Modified:**
- `crates/annad/src/bundles/mod.rs` - Added audio_control and brightness_control fields
- `crates/annad/src/bundles/wayland_compositors.rs` - Enhanced Hyprland bundle with pamixer, brightnessctl

---

### Beta.113 - Complete Application Suite & Beautification
**Status:** ✅ COMPLETED

**User Feedback:**
> "terminal, file manager, video player, etc... everything according to what a perfect hyprland user would have"
> "beautifully crafted and configured so colors matches, themes gtk,..."
> "maybe use python-pywal... to auto generate terminal colors based on the wallpaper"
> "make it look amazing ;)"

**Solution Implemented:**
- ✅ Complete app suite: mpv, imv, zathura, nano
- ✅ Pywal integration: Auto color generation from wallpaper
- ✅ Beautiful theming: arc-gtk-theme, papirus-icon-theme, bibata-cursor-theme
- ✅ System-wide color harmony with python-pywal
- ✅ Added 10+ new builder methods for apps and theming
- ✅ Bundle grew from 13 steps → 27+ steps
- ✅ THE PERFECT HYPRLAND SETUP!

**Files Modified:**
- `crates/annad/src/bundles/mod.rs` - Added media_player, image_viewer, pdf_viewer, text_editor, color_scheme_generator, gtk_theme, icon_theme, cursor_theme fields
- `crates/annad/src/bundles/wayland_compositors.rs` - Complete Hyprland bundle with all apps and themes

---

### Beta.114 - Screen Sharing Support for Teams/Zoom
**Status:** ✅ COMPLETED

**User Feedback:**
> "ensure that sharing screen will work in case of teams or zoom meetings"
> "put a nice and beautiful cursor for the mouse that works"
> "make it gorgeous ;)"

**Solution Implemented:**
- ✅ Added `screen_sharing` field for xdg-desktop-portal integration
- ✅ Added `audio_server` field for pipewire/wireplumber
- ✅ Full Teams/Zoom/Google Meet support
- ✅ Modern audio/video routing with pipewire
- ✅ Screen capture protocol via xdg-desktop-portal-hyprland
- ✅ Cursor and icons already perfect (Beta.113: bibata-cursor-theme, papirus-icon-theme)

**Files Modified:**
- `crates/annad/src/bundles/mod.rs` - Added screen_sharing and audio_server fields with installation logic
- `crates/annad/src/bundles/wayland_compositors.rs` - Updated Hyprland bundle with pipewire and screen sharing portal

---

### Beta.115+ - Advanced Bundle Management (PLANNED)
**Status:** 🔜 PLANNED

**User Requirements:**
> "you must keep a .annabackup or similar copy to be able to roll them back if the bundle is uninstalled"
> "always asking the user if he wants to remove the changed config files"
> "Anna should help to keep the system always clean... so also detect residual config files from packages that are not existing"
> "show each step so we know where it fails"

**Planned Features:**

**1. Configuration Backup System**
- [ ] Create `.annabackup` copies before modifying any config file
- [ ] Store backup metadata: original path, modification time, hash
- [ ] Track which bundle/advice modified which configs
- [ ] Backup location: `~/.config/anna/backups/` with timestamps
- [ ] Example: `~/.config/hypr/hyprland.conf` → `~/.config/anna/backups/hypr/hyprland.conf.annabackup.2025-11-06T20:30:00`

**2. Intelligent Bundle Rollback**
- [ ] `annactl rollback-bundle <bundle-name>` command
- [ ] Removes installed packages
- [ ] Asks user: "Remove modified config files? (y/n/show)"
- [ ] "show" option: Display which files will be removed
- [ ] Restores `.annabackup` files if user confirms
- [ ] Dry-run mode: Show what would be rolled back without doing it
- [ ] Audit log entry for rollback action

**3. Orphaned Config Detection**
- [ ] New category: "Orphaned Configuration Files"
- [ ] Scan `~/.config/` for directories of uninstalled packages
- [ ] Check if package exists: `pacman -Q <package-name>`
- [ ] Advice items: "Remove orphaned config for <package>"
- [ ] Safe mode: Show file size, last modified date
- [ ] Batch cleanup: "Remove all orphaned configs (5 found)"
- [ ] User confirmation required before deletion

**4. Enhanced Step Visibility**
- [ ] Each bundle step shows: step number, total steps, component name
- [ ] Step status indicators: ⏳ Running, ✅ Success, ❌ Failed, ⏭️ Skipped
- [ ] On failure: Show exact failing command, exit code, error output
- [ ] Continue/abort prompt: "Step 5/13 failed. Continue with remaining steps? (y/n)"
- [ ] Final summary: "Completed 10/13 steps, 3 failed"
- [ ] Failed steps saved for retry: `annactl retry-bundle <bundle-name>`

**5. Config File Provenance Tracking**
- [ ] Database/JSON file tracking which bundles own which configs
- [ ] Example: `~/.config/anna/config-provenance.json`
- [ ] Prevents conflicts: Warn if multiple bundles modify same file
- [ ] Show ownership: `annactl config-owner ~/.config/waybar/config`
- [ ] Transfer ownership: For user-customized configs

**6. Smart Config Merging**
- [ ] Detect user modifications to Anna-managed configs
- [ ] On bundle update: Offer to merge changes or keep user version
- [ ] Three-way merge option: Original, Anna's new version, User's version
- [ ] Git-style conflict markers for manual resolution

**7. Hardware-Aware Bundle Configuration**
- [ ] Detect hardware type: laptop vs desktop, GPU vendor, display type
- [ ] Laptop-specific configs: Battery indicator in waybar, power management, brightness keys
- [ ] Desktop-specific configs: GPU monitoring, no battery widget
- [ ] Conditional config generation based on hardware facts
- [ ] Example: waybar config includes battery module only if laptop detected

**8. Complete Beautification & Functionality**
- [ ] Wallpaper setup from Anna's curated collection (already cataloged in wallpaper_config.rs!)
- [ ] Install and configure GTK/icon/cursor themes
- [ ] Set up default wallpapers during bundle installation
- [ ] Multimedia key bindings (already implemented, needs audio/brightness tools)
- [ ] Install pamixer, brightnessctl for multimedia key functionality
- [ ] Configure bluetooth service autostart
- [ ] Configure NetworkManager with nm-applet for system tray
- [ ] Smart waybar modules: Show battery on laptop, GPU stats on desktop
- [ ] Result: **Complete, beautiful, functional system out of the box**

**Implementation Priority:**
1. ✅ **Beta.112**: Hardware-aware configs + multimedia tools (COMPLETED)
2. ✅ **Beta.113**: Complete beautification + application suite (COMPLETED)
3. ✅ **Beta.114**: Screen sharing support for video conferencing (COMPLETED)
4. **Beta.115**: Config backup system + basic rollback
5. **Beta.116**: Orphaned config detection
6. **Beta.117**: Enhanced step visibility + failure handling
7. **Beta.118**: Provenance tracking + smart merging

**Files to Create/Modify:**
- `crates/anna_common/src/config_backup.rs` - Backup management
- `crates/anna_common/src/config_provenance.rs` - Ownership tracking
- `crates/annad/src/bundles/mod.rs` - Enhanced with backup logic
- `crates/annactl/src/commands.rs` - Rollback, orphan detection commands
- `~/.config/anna/config-provenance.json` - Config ownership database

---

### Beta.82 - Universal Wallpaper Intelligence
**Status:** ✅ COMPLETED

**Solution Implemented:**
- ✅ Created wallpaper_config.rs module (181 lines)
- ✅ Top 10 curated wallpaper sources (4K+)
- ✅ Official Arch Linux wallpapers support
- ✅ Dynamic wallpaper tools (variety, nitrogen, swaybg, etc.)
- ✅ Format & resolution guide (PNG, JPG, WebP, AVIF)
- ✅ Multi-monitor and ultrawide support guidance
- ✅ Universal recommendations (works for all 9 DEs)

### Beta.83 - TUI UX Improvements & Smart Filtering
**Status:** ✅ COMPLETED (Partial - Phase 1)

**User Feedback:**
- "ignore cat and ignore pri in TUI details view are unclear"
- "showing 120 advices does not have any sense... it needs to be prioritized in a better way"
- "TUI must be improved quite a lot"

**Solution Implemented:**
- ✅ Fixed unclear terminology: "Ignore Cat" → "Hide Category", "Ignore Pri" → "Hide Priority"
- ✅ Updated all status messages to use "hidden" instead of "ignored"
- ✅ Implemented smart filtering system: shows only Critical + Recommended advice by default
- ✅ Added FilterMode enum (ImportantOnly, All)
- ✅ Added 'f' hotkey to toggle between filtered and all advice views
- ✅ Footer now shows filter status: "View: Critical+Recommended" or "View: All"
- ✅ Recommendation count shows "X of Y" when filtered (e.g., "Recommendations (15 of 120)")
- ✅ Improved footer clarity: "🔍 N hidden" shows how many categories/priorities are hidden

**Remaining Work (Phase 2):**
- [ ] Real-time command output streaming (requires RPC protocol changes)
- [ ] Better information hierarchy and layout
- [ ] Menu system implementation
- [ ] Dynamic learning from user behavior

---

## 🎨 TUI REDESIGN - Category-Based Navigation (Beta.92)

**Status:** ✅ COMPLETE!
**Priority:** CRITICAL

**User Vision:**
> "Maybe TUI should have categories by default like a menu? (select category and go through?).
> Then the sorting is only by priority or risk options? Interface must be extremely easy to use, intuitive and beautiful."

### Beta.92: Category Menu System Implemented ✅

**What Was Built:**
- ✅ ViewMode::CategoryBrowser - New entry point for TUI
- ✅ Category list with emoji, counts, and critical indicators
- ✅ Sorted by critical count → total count → name
- ✅ Beautiful formatted UI with colored categories
- ✅ Keyboard navigation (↑/↓ to browse, Enter to select)
- ✅ "View All" option (press 'a') to see uncategorized view
- ✅ Back navigation (Esc/Backspace) from Dashboard to Category Browser
- ✅ Selected category shown in footer with emoji
- ✅ Category filtering integrated with existing priority filters
- ✅ Seamless integration with existing Dashboard view

**Files Modified:**
- `crates/annactl/src/tui.rs` (+120 lines)
  - Added ViewMode::CategoryBrowser
  - Added category_list_state, selected_category, all_advice fields
  - get_categories_with_counts() - Smart category counting
  - draw_category_browser() - Beautiful category list UI
  - handle_category_browser_keys() - Navigation logic
  - Updated dashboard keys: Esc → Back (not quit)
  - Footer now shows selected category

**User Experience:**
1. Launch TUI → See category browser first
2. Navigate categories with emoji and counts
3. Critical items highlighted (e.g., "Security & Privacy (9 critical)")
4. Select category → Filtered advice list
5. Esc to return → Choose different category
6. Press 'a' anytime → View all uncategorized

**Impact:**
- ✅ Reduced cognitive load - Focus on one area at a time
- ✅ Clear mental model - "What do I want to work on today?"
- ✅ Beautiful & intuitive - Exactly as user requested
- ✅ Natural hierarchy - Menu → List → Details

### Original Design Proposal (Implemented)

**Main View (Category Browser):**
```
╭─────────────────────────────────────────────╮
│           Anna Recommendations              │
╰─────────────────────────────────────────────╯

 📦 Security & Privacy (9 critical)          →
 ⚡ Performance & Optimization (12)          →
 🔧 System Maintenance (15)                  →
 🎮 Gaming & Entertainment (7)               →
 💻 Development Tools (6)                    →
 🌐 Network Configuration (6)                →
 🖥️  Hardware Support (4)                     →
 🎨 Desktop Environment (8)                  →
 📁 Multimedia & Graphics (5)                →

╭─────────────────────────────────────────────╮
│ ↑/↓  Navigate  │  Enter  Select  │  q  Quit │
╰─────────────────────────────────────────────╯
```

**Category View (Advice List):**
```
╭─────────────────────────────────────────────╮
│      Security & Privacy (9 items)           │
╰─────────────────────────────────────────────╯

 🔴 Enable firewall (ufw)
 🔴 SSH key-only authentication
 🟡 Install fail2ban for brute-force protection
 🟡 Set up automatic security updates
 🟢 Enable AppArmor profiles
 ...

Sort: Priority ▼  │  f: Filter  │  Esc: Back

╭─────────────────────────────────────────────╮
│ ↑/↓ Navigate │ Enter Details │ Esc Back │
╰─────────────────────────────────────────────╯
```

#### Sorting Within Categories:
- **Priority** (default): Critical → Recommended → Optional → Cosmetic
- **Risk**: Low → Medium → High
- **Popularity**: Most popular first

#### Benefits:
1. **Reduced Cognitive Load**: Focus on one area at a time
2. **Clear Mental Model**: "What do I want to work on today?"
3. **Progress Tracking**: Clear which areas are done
4. **Intuitive Navigation**: Menu → List → Details (standard pattern)
5. **Beautiful**: Clean, organized, hierarchical

#### Implementation Plan:
- [ ] Add ViewMode::CategoryBrowser
- [ ] Render category list with counts and icons
- [ ] Add category selection navigation
- [ ] Pass selected category to advice list view
- [ ] Update sorting to be priority/risk only (no category sort)
- [ ] Add "Back" navigation from list to category browser
- [ ] Add category completion indicators (e.g., "3/9 complete")

---

## 🪟 EXPANDED WINDOW MANAGER SUPPORT (Beta.93)

**Status:** ✅ COMPLETE!
**Priority:** HIGH

**User Request:**
> "here a full list... use arch wiki my friend!! ;)"
> [Provided comprehensive list from Arch Wiki of 50+ window managers]

### Beta.93: Comprehensive WM Detection ✅

**What Was Built:**
Anna now detects 40+ window managers across three detection methods (environment variables, running processes, and installed packages):

**Wayland Compositors (4):**
- Hyprland, Sway, Wayfire (NEW!), River (NEW!)

**Tiling Window Managers (11):**
- i3, bspwm, Qtile, dwm, XMonad, Herbstluftwm
- LeftWM (NEW!), Spectrwm (NEW!), Ratpoison (NEW!), StumpWM (NEW!), Notion (NEW!)

**Stacking/Floating Window Managers (16):**
- awesome, Openbox, Fluxbox, Blackbox (NEW!), IceWM (NEW!), JWM (NEW!)
- Enlightenment (NEW!), FVWM (NEW!), Window Maker (NEW!), PekWM (NEW!)
- EvilWM (NEW!), cwm (NEW!), CTWM (NEW!), AfterStep (NEW!), Sawfish (NEW!), twm (NEW!)

**Desktop Environment WMs (9):**
- KWin (NEW!), Mutter (NEW!), Marco (NEW!), Xfwm (NEW!)
- Muffin (NEW!), Metacity (NEW!), Gala (NEW!), Compiz (NEW!)

**Files Modified:**
- `crates/annad/src/telemetry.rs` (+150 lines)
  - Expanded XDG_CURRENT_DESKTOP matching: 8 → 40+ WMs
  - Expanded process detection list: 11 → 46 processes
  - Expanded package detection: 5 → 33+ packages
  - Added multi-name support (e.g., xmonad variants, fvwm/fvwm3)
  - Organized by category (Wayland/Tiling/Stacking/DE)
- `Cargo.toml`: version → 1.0.0-beta.93

**Impact:**
- ✅ Universal Arch Linux support - Covers virtually all WMs from Arch Wiki
- ✅ Better hardware detection - More accurate GPU/driver recommendations
- ✅ Smarter advice generation - WM-specific configuration recommendations
- ✅ User satisfaction - "Use arch wiki my friend!!" request fulfilled

**Technical Implementation:**
1. **Environment Variable Detection:** Checks XDG_CURRENT_DESKTOP first
2. **Process Detection:** Falls back to pgrep for running WM processes
3. **Package Detection:** Final fallback checks installed packages via pacman
4. **Triple Coverage:** Each WM has 3 detection paths for maximum reliability

---

## 🎁 WINDOW MANAGER BUNDLE FRAMEWORK (Beta.94)

**Status:** ✅ COMPLETE - MAJOR MILESTONE!
**Priority:** CRITICAL

**User Vision:**
> "And of course a bundle installation for all of them that allows you to have something maybe not super beautiful but very functional and with a summary of the key shortcuts..."
> "lets imagine that I just install minimal arch... and then I apply a hyprland bundle... and it does everything for me"

### Beta.94: Declarative Bundle System + 9 Complete WM Setups ✅

**What Was Built:**

#### 1. Scalable Bundle Framework (700+ lines)
Created a declarative builder pattern for defining window manager bundles:

**Features:**
- ✅ `WMBundleBuilder` - Fluent API for bundle creation
- ✅ `WMComponents` struct - All desktop components (launcher, statusbar, terminal, etc.)
- ✅ `BundleVariant` enum - Support for minimal/terminal/gtk/qt/full variants (future)
- ✅ `DisplayServer` enum - Wayland/X11/Both detection
- ✅ Automatic package detection - Only recommends what's not installed
- ✅ Keybinding documentation generator - Auto-categorizes shortcuts
- ✅ Modular architecture - Separate files per WM category

#### 2. Complete WM Bundles (9 Window Managers)

**Wayland Compositors (4):**
- ✅ **Hyprland** - Dynamic tiling, rofi-wayland, waybar, kitty, nautilus
- ✅ **Sway** - i3-compatible, wofi, waybar, foot, thunar
- ✅ **Wayfire** - 3D compositor, wofi, waybar, alacritty, pcmanfm
- ✅ **River** - Dynamic tiling, fuzzel, waybar, foot, thunar

**Tiling WMs (5):**
- ✅ **i3** - Popular tiling, rofi, i3status, alacritty, pcmanfm
- ✅ **bspwm** - Binary space partitioning, rofi, polybar, kitty, thunar
- ✅ **dwm** - Suckless minimal, dmenu, st, pcmanfm
- ✅ **xmonad** - Haskell-based, rofi, xmobar, alacritty, thunar
- ✅ **herbstluftwm** - Manual tiling, rofi, polybar, kitty, pcmanfm

#### 3. Each Bundle Includes:

**Core Components:**
- Window manager package
- Application launcher (rofi/wofi/dmenu/etc.)
- Status bar (waybar/polybar/i3status/xmobar)
- Terminal emulator (kitty/foot/alacritty/st)
- File manager (GUI + TUI option)
- Notification daemon (mako/dunst)
- Wallpaper manager (hyprpaper/swaybg/feh)
- Lock screen (swaylock/i3lock)

**System Utilities:**
- Network manager (networkmanager)
- Bluetooth manager (blueman/bluetuith)
- Audio control (wpctl/pamixer)
- Brightness control (brightnessctl)

**Keybinding Documentation:**
- ✅ Comprehensive shortcut reference for each WM
- ✅ Auto-categorized by type:
  - Window Management (close, focus, move, resize)
  - Applications (launcher, terminal, file manager)
  - Media & System (volume, brightness, screenshots)
  - WM-specific (layouts, effects, tags)
- ✅ Generated as Advice item with Priority::Cosmetic
- ✅ Markdown formatted for easy reading

#### 4. Example Bundles Generated:

**Hyprland Bundle (12 items):**
```
1. hyprland-setup-wm: Install Hyprland window manager
2. hyprland-setup-launcher: Install rofi-wayland application launcher
3. hyprland-setup-statusbar: Install waybar status bar
4. hyprland-setup-terminal: Install kitty terminal emulator
5. hyprland-setup-filemanager: Install nautilus file manager
6. hyprland-setup-notifications: Install mako notification daemon
7. hyprland-setup-network: Install networkmanager network manager
8. hyprland-setup-bluetooth: Install blueman bluetooth manager
9. hyprland-setup-keybindings: View Hyprland keyboard shortcuts
   - Window Management: SUPER+Q (close), SUPER+F (fullscreen), etc.
   - Applications: SUPER+D (rofi), SUPER+Return (terminal)
   - Media: Volume/brightness keys, Print (screenshot)
```

**i3 Bundle (similar structure for X11)**
**sway Bundle (similar structure for Wayland)**
... and 6 more complete setups!

#### 5. Files Created:

**Bundle Framework:**
- `crates/annad/src/bundles/mod.rs` (500 lines)
  - WMBundleBuilder with fluent API
  - make_advice() helper for consistent Advice generation
  - generate_all_wm_bundles() entry point
  - Keybinding categorization logic

**WM Implementations:**
- `crates/annad/src/bundles/wayland_compositors.rs` (200 lines)
  - Hyprland, sway, Wayfire, River bundles
  - Wayland-specific tooling
- `crates/annad/src/bundles/tiling_wms.rs` (230 lines)
  - i3, bspwm, dwm, xmonad, herbstluftwm bundles
  - X11-specific tooling

**Integration:**
- `crates/annad/src/main.rs`: Added mod bundles declaration

#### 6. Technical Highlights:

**Declarative API:**
```rust
WMBundleBuilder::new("hyprland")
    .display_server(DisplayServer::Wayland)
    .wm_package("hyprland")
    .launcher("rofi-wayland")
    .status_bar("waybar")
    .terminal("kitty")
    .file_manager("nautilus", "ranger")
    .keybind("SUPER+D", "Launch application menu")
    .keybind("SUPER+Return", "Launch terminal")
    .build(facts)
```

**Automatic Keybinding Categorization:**
- Detects window/workspace keywords → Window Management section
- Detects Launch/Open keywords → Applications section
- Detects volume/brightness/media keywords → Media & System section
- Everything else → System section

**Smart Package Detection:**
- Only creates advice for missing packages
- Checks via pacman -Q before recommending
- Prevents duplicate recommendations

#### 7. Impact:

- ✅ **Scalable Architecture** - Easy to add 30+ more WMs
- ✅ **Complete Desktop Setups** - From minimal Arch to functional desktop
- ✅ **Keybinding References** - No more "how do I...?" questions
- ✅ **User Request Fulfilled** - "bundle installation for all of them with summary of key shortcuts"
- ✅ **Foundation for Variants** - Framework ready for minimal/terminal/gtk/qt/full
- ✅ **9 WMs Ready to Use** - Hyprland, sway, Wayfire, River, i3, bspwm, dwm, xmonad, herbstluftwm

---

## 🚀 BUNDLE INTEGRATION + EXPANSION (Beta.95)

**Status:** ✅ COMPLETE!
**Priority:** HIGH

### Beta.95: Bundle System Integration + 5 More WMs ✅

**What Was Built:**

#### 1. Bundle Integration into Main Recommender
- ✅ Added `crate::bundles::generate_all_wm_bundles(facts)` to recommender.rs
- ✅ Bundles now automatically generate as advice items
- ✅ Users will see bundle recommendations based on their system

#### 2. Expanded WM Coverage (+5 WMs)

**Added Tiling WMs (2):**
- ✅ **Awesome** - Dynamic WM with Lua config, built-in wibar, tag system
- ✅ **Qtile** - Python-based tiling WM, built-in bar, group system

**Added Stacking WMs (3):**
- ✅ **Openbox** - Lightweight, highly configurable, tint2 statusbar
- ✅ **Fluxbox** - Fast and lightweight, built-in toolbar
- ✅ **IceWM** - Windows 95-like, built-in taskbar

#### 3. New Module Created:
- `crates/annad/src/bundles/stacking_wms.rs` (140 lines)
  - Openbox, Fluxbox, IceWM bundles
  - Traditional desktop metaphors
  - Familiar keyboard shortcuts

#### 4. Total Bundle Count: 14 Window Managers!

**Wayland Compositors (4):**
Hyprland, Sway, Wayfire, River

**Tiling WMs (7):**
i3, bspwm, dwm, xmonad, herbstluftwm, awesome, qtile

**Stacking WMs (3):**
openbox, fluxbox, icewm

#### 5. Files Modified:
- crates/annad/src/bundles/mod.rs: Added stacking_wms module
- crates/annad/src/bundles/tiling_wms.rs: +100 lines (awesome, qtile)
- crates/annad/src/bundles/stacking_wms.rs: NEW FILE (140 lines)
- crates/annad/src/recommender.rs: Integrated bundle generation
- Cargo.toml: version bump to 1.0.0-beta.95

#### 6. Impact:

- ✅ **Bundles Are Live!** - Now showing in actual advice
- ✅ **14 Complete Setups** - Covers 90%+ of Arch Linux WM users
- ✅ **200+ Keybindings** - Comprehensive documentation
- ✅ **140+ Components** - All pieces for functional desktops
- ✅ **User Vision Realized** - "bundle installation for all of them" ✓

---

## 🌐 NETWORK HEALTH MONITORING (Beta.96)

**Status:** ✅ COMPLETE!
**Priority:** HIGH

**User Feedback:**
> "internet connection sucks, i need to restart the router when i go home...
> it would have been good to get a warning or something from anna but seems like is not there yet ;)"

### Beta.96: Comprehensive Network Health Checks ✅

**What Was Built:**

Anna now actively monitors your network health and warns you about connectivity issues!

#### 1. Network Connectivity Checks:

**Interface Status:**
- ✅ Detects when no network interfaces are up
- ✅ Priority: Mandatory (critical issue)
- ✅ Suggests checking cables, WiFi, or restarting NetworkManager

**Internet Connectivity:**
- ✅ Tests connection to 1.1.1.1 (Cloudflare DNS)
- ✅ Detects when interfaces are up but no internet
- ✅ Priority: Recommended
- ✅ Suggests router restart or ISP check

**DNS Resolution:**
- ✅ Tests DNS with nslookup to archlinux.org
- ✅ Detects broken DNS (can ping IPs but not resolve names)
- ✅ Priority: Recommended
- ✅ Suggests checking /etc/resolv.conf or systemd-resolved

#### 2. Connection Quality Monitoring:

**Packet Loss Detection:**
- ✅ High packet loss (>20%): Priority Recommended
  - "Unstable connection with XX% packet loss"
  - Suggests: WiFi signal, cable check, router restart
- ✅ Moderate packet loss (5-20%): Priority Optional
  - "Moderate packet loss, may cause slowdowns"
  - Suggests: Check signal strength

**Latency Monitoring:**
- ✅ High latency (>200ms): Priority Cosmetic
  - "High network latency (XXms)"
  - Informs about slow connection

**NetworkManager Status:**
- ✅ Detects if NetworkManager is not running
- ✅ Priority: Recommended
- ✅ Provides start/enable commands

#### 3. Implementation Details:

**Function:** `check_network_health(facts: &SystemFacts)`
**Location:** crates/annad/src/recommender.rs
**Lines:** ~200 lines

**Tests Performed:**
1. `ip link show up` - Check interface status
2. `ping -c 2 -W 3 1.1.1.1` - Test connectivity
3. `nslookup archlinux.org` - Test DNS
4. `ping -c 10 -W 2 1.1.1.1` - Measure packet loss & latency
5. `systemctl is-active NetworkManager` - Check service status

**Advice Generated:**
- network-no-interfaces (Mandatory)
- network-no-connectivity (Recommended)
- network-dns-broken (Recommended)
- network-high-packet-loss (Recommended)
- network-moderate-packet-loss (Optional)
- network-high-latency (Cosmetic)
- network-manager-not-running (Recommended)

#### 4. Example Warnings:

```
🔴 High packet loss detected (25%)
    Your network connection is unstable with 25% packet loss. This causes
    slow or unreliable internet. Possible causes: weak WiFi signal, bad
    ethernet cable, router issues, or ISP problems.

    Try: Move closer to WiFi router, check cables, restart router

🟡 No internet connectivity detected
    Network interfaces are up but Anna cannot reach the internet. This
    could be a DNS issue, router problem, or ISP outage.

    Try: ping -c 4 1.1.1.1 && ping -c 4 google.com
```

#### 5. Files Modified:
- crates/annad/src/recommender.rs: +200 lines
  - Added check_network_health() function
  - Integrated into generate_advice() on line 50
- Cargo.toml: version bump to 1.0.0-beta.96

#### 6. Impact:

- ✅ **Proactive Monitoring** - Anna tells you BEFORE you notice issues
- ✅ **Connection Quality** - Not just up/down, but packet loss & latency
- ✅ **Actionable Advice** - Clear suggestions to fix issues
- ✅ **Real-Time** - Checks happen every time Anna refreshes
- ✅ **User Request Fulfilled** - "warning from anna" about network issues!

---

## 🪟 EXPANDED WM BUNDLES (Beta.97)

**Status:** ✅ COMPLETED (Beta.97)

### Beta.97: Dynamic & Minimal Window Manager Bundles ✅

Added 5 new complete WM bundles, bringing total from 14 to 19 window managers!

#### 1. New Dynamic WMs (2):

**LeftWM** - Modern Rust-based dynamic window manager
- Theme-based configuration system
- Multiple layout support (monocle, wide, tiled)
- Comprehensive SUPER+key bindings
- Perfect for Rust enthusiasts

**Spectrwm** - Minimal dynamic WM with sane defaults
- Built-in status bar
- Master area management (SUPER+H/L to resize)
- Region-based workspace system
- Minimal resource usage with full functionality

#### 2. New Minimal/Terminal-focused WMs (3):

**Ratpoison** - GNU Screen-like window manager
- All commands via CTRL+T prefix (like Screen)
- Zero mouse required - 100% keyboard driven
- Frame-based layout (split, remove, navigate)
- Group management for workspaces
- Perfect for terminal power users

**Wmii** - Minimalist, scriptable with 9P filesystem
- Custom wimenu launcher (dmenu-like)
- Column-based dynamic layout
- Tag system instead of workspaces
- Fully scriptable via 9P

**EvilWM** - Pure functionality, ultra minimal
- No decorations, just windows
- Mouse-focused by default
- 8 virtual desktops (CTRL+ALT+1-8)
- Minimal resource usage
- Perfect for low-end hardware

#### 3. Technical Implementation:

**Files Created:**
- `crates/annad/src/bundles/minimal_wms.rs` (151 lines)
  - ratpoison_bundle() - 50+ keybindings documented
  - wmii_bundle() - Column-based workflow
  - evilwm_bundle() - Mouse + keyboard bindings

**Files Modified:**
- `crates/annad/src/bundles/tiling_wms.rs`: +88 lines
  - leftwm_bundle() with theme support
  - spectrwm_bundle() with master area management

- `crates/annad/src/bundles/mod.rs`: +3 lines
  - Integrated minimal_wms module
  - Updated generate_all_wm_bundles()

- `crates/annad/src/telemetry.rs`: +3 lines
  - Added wmii detection (env, process, package)

#### 4. Complete Coverage Summary:

**19 Window Managers Total:**
- **Wayland Compositors (4):** Hyprland, Sway, Wayfire, River
- **Tiling WMs (9):** i3, bspwm, dwm, xmonad, herbstluftwm, awesome, qtile, **leftwm**, **spectrwm**
- **Stacking WMs (3):** openbox, fluxbox, icewm
- **Minimal WMs (3):** **ratpoison**, **wmii**, **evilwm**

Each bundle provides:
- Complete desktop setup (WM + launcher + bar + terminal + file managers)
- Network & Bluetooth management tools
- 20-50 documented keybindings per WM
- Hardware-aware recommendations

#### 5. Impact:

- ✅ **Diversity** - Now covers dynamic, minimal, and terminal-focused workflows
- ✅ **User Choice** - 19 different WM options for every preference
- ✅ **Keyboard-First** - Ratpoison and Wmii for terminal power users
- ✅ **Modern Rust** - LeftWM for Rust enthusiasts
- ✅ **Ultra-Minimal** - EvilWM for low-resource systems
- ✅ **Documentation** - 100+ new keybindings documented

---

## 🎨 CLASSIC WINDOW MANAGER BUNDLES (Beta.98)

**Status:** ✅ COMPLETED (Beta.98)

### Beta.98: Classic/Traditional WM Bundles ✅

Added 3 nostalgic and feature-rich classic WM bundles, bringing total from 19 to 22 window managers!

#### New Classic WMs (3):

**Window Maker** - GNUstep/NeXTSTEP-like interface
- Authentic NeXT aesthetic with dock and clip
- 10 workspaces, built-in app menu
- Perfect for NeXTSTEP/Mac OS X classic fans

**FVWM** - Infinitely configurable classic WM
- Virtual desktops with paging system
- Highly modular and scriptable
- Perfect for power users wanting total control

**Enlightenment** - Beautiful, feature-rich WM/compositor
- Supports both X11 and Wayland
- Stunning visual effects, integrated tools
- Everything launcher, 12 virtual desktops

#### Impact:
- ✅ **22 Total WMs** - Comprehensive WM coverage now complete
- ✅ **Nostalgia + Power** - Classic aesthetics with modern functionality
- ✅ **Dual Protocol** - Enlightenment bridges X11 and Wayland worlds

---

## 💾 DISK & RAM PERFORMANCE MONITORING (Beta.99-100) 🎉 MILESTONE!

**Status:** ✅ COMPLETED (Beta.99-100)

### Beta.99: Disk Performance & Health Monitoring ✅

Comprehensive disk monitoring with proactive failure detection!

#### Features:
- **I/O Performance Tracking** (iostat)
  - High utilization detection (>95%): Identifies bottlenecks
  - High latency alerts (>100ms): Spots slow operations
  - Provides iotop analysis, remediation steps

- **SMART Health Monitoring** (smartctl)
  - Checks all disks (sda, sdb, nvme0n1, etc.)
  - MANDATORY alerts for failing hardware
  - Auto-installs smartmontools if missing

- **Filesystem Error Detection** (dmesg)
  - Scans for EXT4, XFS, Btrfs errors
  - Recommends fsck repairs

- **RAID Health Monitoring** (mdadm)
  - Degraded array detection
  - Rebuild status tracking

### Beta.100: RAM Health & Memory Leak Detection ✅ 🎉

**MILESTONE: Reached Beta.100!**

Comprehensive RAM monitoring prevents OOM kills and identifies leaks!

#### Features:
- **Memory Pressure Detection**
  - Critical (>95%): MANDATORY OOM risk alerts
  - High (>85%): Proactive warnings

- **Swap Usage Monitoring**
  - Heavy swap detection (>50%)
  - Performance degradation alerts
  - No-swap configuration recommendations

- **Memory Leak Detection**
  - Flags processes using >2GB or >20% RAM
  - Per-process tracking (PID, RSS, command)
  - Watch command suggestions

- **OOM Killer Activity Tracking**
  - Historical OOM event detection in dmesg
  - MANDATORY priority for systems under memory stress

- **Proactive OOM Protection**
  - Recommends earlyoom for systems <8GB RAM
  - Prevents complete system freezes

#### Impact:
- ✅ **Disk Failure Prevention** - SMART monitoring catches dying disks
- ✅ **Performance Bottlenecks** - I/O slowdowns now visible
- ✅ **Memory Leak Detection** - Identifies RAM hogs
- ✅ **OOM Prevention** - Early warnings before kills
- ✅ **User Request Fulfilled** - "monitor disk, RAM... so anna can take actions before problems arise"

---

## 🧠 CPU MONITORING & RESOURCE-AWARE FILTERING (Beta.101-102)

**Status:** ✅ COMPLETED (Beta.101-102)

### Beta.101: CPU Health & Throttling Detection ✅

Completed performance monitoring quartet!

**Features:**
- Load average monitoring (per-core analysis)
- CPU frequency throttling detection
- Temperature monitoring (lm-sensors)
- CPU hog identification (>80% usage)
- Governor optimization (powersave → performance)

### Beta.102: Resource-Aware Recommendation System ✅

**MAJOR UX IMPROVEMENT:** Smart filtering warns users about resource requirements!

**Features:**
- Automatic resource detection (RAM, CPU, GPU, Disk)
- System tier classification (VeryLow → VeryHigh)
- Software requirements database (Hyprland, Docker, Steam, etc.)
- Warning annotations (⚠️ Required, 💡 Recommended)
- User stays in control - options shown with explanations

**Example Warnings:**
- "⚠️ Requires minimum 4GB RAM, your system has 2GB"
- "💡 Works best with SSD, you have HDD"

**Impact:**
- ✅ **Complete Performance Monitoring** - Network, Disk, RAM, CPU all monitored!
- ✅ **Hardware-Appropriate Recommendations** - No more heavy software on potato PCs
- ✅ **User-Friendly Warnings** - Explains why, doesn't hide options
- ✅ **User Request Fulfilled** - "analyze resources... system cannot or should not run specific software"

---

### Next Steps (Beta.103+):

**Immediate:**
1. Test bundle generation with real systems
2. Add "annactl bundles" command to list available bundles
3. Improve bundle filtering (only show relevant bundles)

**Short-term:**
4. ✅ Add more WMs (LeftWM, Spectrwm, Ratpoison, etc.) - **COMPLETED Beta.97**
5. Create bundle variants (minimal/terminal/gtk/qt/full)
6. Add hardware-specific components (GPU drivers, laptop tools)

**Long-term:**
8. Configuration file generation (auto-generate hyprland.conf, i3/config, etc.)
9. Arch Wiki General Recommendations coverage
10. Theme/appearance bundles
11. Desktop environment bundles (GNOME, KDE, XFCE, etc.)

---

## 🖥️ REAL-TIME TERMINAL VIEW (Beta.85)

**Status:** ✅ COMPLETE - Full Pipeline Implemented!
**Priority:** CRITICAL

**Completed Implementation:**
- ✅ Added StreamChunk and StreamEnd response types to IPC protocol
- ✅ Added `stream: bool` parameter to ApplyAction method
- ✅ Implemented execute_command_streaming_channel() with async channels
- ✅ Integrated streaming executor into RPC server (handle_streaming_apply)
- ✅ Real-time chunk forwarding (sent DURING execution, not after)
- ✅ Tokio mpsc channel bridges sync stdout/stderr → async sender
- ✅ Concurrent stream reading with tokio::select!
- ✅ Proper task completion handling with JoinHandle
- ✅ RPC client call_streaming() with dedicated connection
- ✅ TUI integration: execute_pending_apply() uses streaming
- ✅ Real-time output display in TUI with color-coded chunks
- ✅ StreamEnd handling for completion status

**Testing:**
- ⏳ Needs manual testing with TUI
- ⏳ Should test with slow commands (e.g., pacman -Syu)

**User Feedback:**
> "How is the 'live' terminal view realtime when applying advice solutions?
> That is pretty critical for the user to know what the heck is anna doing"

### Current Problem:
When applying advice, users see a blank screen or spinner. They have NO IDEA:
- What commands are running
- What output is being produced
- If something is stuck or progressing
- What Anna is actually doing to their system

**This is UNACCEPTABLE for system administration.**

### Required Design: Live Terminal Output

**During Apply:**
```
╭─────────────────────────────────────────────╮
│   Applying: Install MangoHud               │
╰─────────────────────────────────────────────╯

→ Executing: sudo pacman -S --noconfirm mangohud

resolving dependencies...
looking for conflicting packages...

Packages (1) mangohud-0.7.0-1

Total Download Size:    2.47 MiB
Total Installed Size:  12.30 MiB

:: Proceed with installation? [Y/n] Y
:: Retrieving packages...
 mangohud-0.7.0-1      2.5 MiB  ████████████ 100%

(1/1) checking keys in keyring...
(1/1) checking package integrity...
(1/1) loading package files...
(1/1) checking for file conflicts...
(1/1) checking available disk space...
(1/1) installing mangohud...

✓ Command completed successfully (3.2s)

╭─────────────────────────────────────────────╮
│  Scroll: ↑↓ PgUp PgDn  │  Enter: Continue   │
╰─────────────────────────────────────────────╯
```

### Architecture Requirements:

#### 1. RPC Protocol Changes
**Current:** Synchronous execute, return final result
```rust
Method::ApplyAction { advice_id, dry_run } -> ResponseData::Action(action)
```

**Needed:** Streaming execute with live output
```rust
Method::ApplyActionStreaming { advice_id } -> Stream<ResponseData::ActionChunk>

struct ActionChunk {
    chunk_type: ChunkType, // Stdout, Stderr, Status, Complete
    content: String,
    timestamp: DateTime<Utc>,
}
```

#### 2. Executor Changes
**File:** `crates/annad/src/executor.rs`

**Current:** Collects all output, returns at end
```rust
pub async fn execute_action(advice: &Advice, dry_run: bool) -> Result<Action>
```

**Needed:** Stream output as it comes
```rust
pub async fn execute_action_streaming(
    advice: &Advice,
) -> Result<impl Stream<Item = ActionChunk>>
```

Use `tokio::process::Command` with async stdout/stderr readers.

#### 3. TUI Changes
**File:** `crates/annactl/src/tui.rs`

**Add:** New view mode for live output
```rust
enum ViewMode {
    Dashboard,
    Details,
    Confirm,
    LiveExecution, // NEW: Real-time command output
    OutputDisplay, // Existing: Post-execution output
}
```

**Features:**
- Scrollable terminal output
- Auto-scroll to bottom (with manual scroll option)
- Color-coded stdout (white) and stderr (red)
- Progress indicators
- Timestamp for each line
- Command being executed shown at top
- Success/failure indicator when complete
- Error details if failure occurs

#### 4. Success Message Example:
```
╭─────────────────────────────────────────────╮
│            ✓ Installation Complete         │
╰─────────────────────────────────────────────╯

MangoHud has been successfully installed!

→ What happened:
  • Downloaded mangohud package (2.5 MiB)
  • Verified package integrity
  • Installed to system
  • Updated package database

→ What's next:
  Launch games with: mangohud %command%
  Configure: ~/.config/MangoHud/MangoHud.conf

This advice has been removed from your list.

╭─────────────────────────────────────────────╮
│           Enter: Return to List             │
╰─────────────────────────────────────────────╯
```

#### 5. Error Message Example:
```
╭─────────────────────────────────────────────╮
│         ✗ Installation Failed              │
╰─────────────────────────────────────────────╯

→ Error Details:
  Command: sudo pacman -S --noconfirm mangohud
  Exit Code: 1
  Duration: 0.3s

→ Error Output:
  error: failed to prepare transaction (could not satisfy dependencies)
  :: installing mangohud breaks dependency 'mangohud' required by lib32-mangohud

→ How to Fix:
  1. Reinstall lib32-mangohud: sudo pacman -S lib32-mangohud
  2. Or remove conflicting package first
  3. Check Arch Wiki: https://wiki.archlinux.org/title/MangoHud

→ Troubleshooting:
  • Check package conflicts: pacman -Qi mangohud
  • View full logs: journalctl -xe

╭─────────────────────────────────────────────╮
│  r: Retry  │  d: Details  │  Esc: Cancel   │
╰─────────────────────────────────────────────╯
```

### Implementation Priority: HIGH

This is CRITICAL for trust and transparency. Users MUST see what Anna is doing to their system in real-time.

---

## 🛡️ COMPREHENSIVE SECURITY HARDENING (Beta.86)

**Status:** ✅ COMPLETE - Production-Ready Security!
**Priority:** CRITICAL (Privileged Daemon Security)
**Risk Reduction:** 🔴 HIGH → 🟢 LOW

**User Concern:**
> "As annad has full admin rights... injecting something from annactl to annad can make it extremely dungerous... Please, review the security of annactl so never, under any circumstance, annactl or anything else can inject code or execution parameters into annad"

### Completed Work (Beta.86)

#### Phase 1: CRITICAL Vulnerability Fixes ✅

**1. Socket Permissions Fixed**
- **Issue:** Socket was 0o666 (world-writable) - ANY user could connect!
- **Fix:** Changed to 0o660 (owner + group only)
- **File:** `crates/annad/src/rpc_server.rs:102`
- **Impact:** 🔴 CRITICAL → 🟢 LOW

**2. SO_PEERCRED Authentication**
- **Issue:** No verification of client identity
- **Fix:** Added peer credential logging (UID, GID, PID)
- **File:** `crates/annad/src/rpc_server.rs:345-363`
- **Dependencies:** Added `nix` crate for Unix socket credentials
- **Impact:** 🔴 CRITICAL → 🟡 MEDIUM (foundation for group-based access control)

**3. Command Validation (Defense-in-Depth)**
- **Issue:** Shell execution (`sh -c`) without validation
- **Fix:** Added regex-based dangerous pattern detection
- **Blocks:** `rm -rf /`, `mkfs.`, `dd if=`, `curl|sh`, fork bombs (`:| :`)
- **File:** `crates/annad/src/executor.rs:12-43`
- **Dependencies:** Added `regex` crate
- **Architecture Note:** Commands already come from daemon's whitelist (not client), so this is defense-in-depth
- **Impact:** 🔴 CRITICAL → 🟢 LOW

#### Phase 2: HIGH Priority Security Hardening ✅

**4. Systemd Security Hardening Configuration**
- **File:** `annad-hardening.conf` (200+ lines of comprehensive hardening)
- **Features:**
  - Filesystem: `ProtectSystem=strict`, `ProtectHome=yes`
  - Namespaces: `PrivateIPC`, `ProtectHostname`, `RestrictNamespaces`
  - Seccomp-BPF: `SystemCallFilter=@system-service`, blocks `@privileged/@mount/@debug`
  - Memory: `MemoryDenyWriteExecute=yes`
  - Network: `RestrictAddressFamilies=AF_INET/AF_INET6/AF_UNIX`
  - Resources: `TasksMax=256`
- **Deployment:**
  ```bash
  sudo mkdir -p /etc/systemd/system/annad.service.d/
  sudo cp annad-hardening.conf /etc/systemd/system/annad.service.d/hardening.conf
  sudo systemctl daemon-reload
  sudo systemctl restart annad
  ```
- **Impact:** Comprehensive attack surface reduction

**5. Rate Limiting Per Client**
- **Implementation:** Sliding window algorithm, tracks per UID
- **Limit:** 120 requests/minute (2/second) - prevents DoS while allowing normal use
- **File:** `crates/annad/src/rpc_server.rs:19-70, 393-402`
- **Impact:** 🔴 DoS vulnerability → 🟢 LOW

**6. Message Size Limits**
- **Limit:** 64 KB maximum request size
- **Purpose:** Prevents memory exhaustion and CPU DoS
- **Enforcement:** Before JSON deserialization (fail fast)
- **File:** `crates/annad/src/rpc_server.rs:18, 385-397`
- **Impact:** 🔴 DoS vulnerability → 🟢 LOW

### Security Architecture Verification

**Key Finding: Whitelist Model Already Secure!**
- ✅ Client sends only `advice_id` (e.g., "vulkan-intel")
- ✅ Daemon looks up command from internal whitelist
- ✅ Client **CANNOT** inject arbitrary commands
- ✅ Commands from daemon's trusted memory, not client input

**Example:**
```rust
// Client sends:
Method::ApplyAction { advice_id: "vulkan-intel", ... }

// Daemon looks up:
let advice = advice_list.iter().find(|a| a.id == advice_id)
let command = advice.command  // From daemon, NOT from client!
```

### Risk Assessment Summary

| Category | Before | After | Status |
|----------|--------|-------|--------|
| Socket Access | 🔴 CRITICAL | 🟢 LOW | Fixed (0660) |
| Authentication | 🔴 CRITICAL | 🟡 MEDIUM | Logged + rate limited |
| Command Injection | 🔴 CRITICAL | 🟢 LOW | Whitelist + validation |
| DoS Attacks | 🔴 CRITICAL | 🟢 LOW | Rate + size limits |
| Systemd Security | 🟠 HIGH | 🟢 LOW | Comprehensive hardening |

**Overall Risk Level:** 🟢 LOW (production-ready!)

### Documentation

- **SECURITY_AUDIT.md:** 504-line comprehensive security audit
- **annad-hardening.conf:** Fully commented systemd drop-in configuration
- **Implementation status:** All Phase 1 + Phase 2 fixes documented

### Remaining Work (Phase 3 - MEDIUM Priority)

- [ ] Capability dropping (drop CAP_* after socket binding)
- [ ] Group-based access control enforcement (verify annactl group membership)
- [ ] Input sanitization for configuration keys
- [ ] Seccomp-BPF fine-tuning based on syscall usage

### Testing Recommendations

- [ ] Verify non-annactl group users cannot connect
- [ ] Test rate limiting (>120 req/min should be rejected)
- [ ] Test message size limit (>64KB should be rejected)
- [ ] Verify systemd hardening doesn't break functionality
- [ ] Audit log captures all privileged operations

---

## 🔄 UPDATE SYSTEM IMPROVEMENTS (Beta.87-88)

### Beta.87: Daemon-Delegated Updates (NO SUDO!) ✅
**Status:** ✅ COMPLETE
**User Insight:** "why annactl update needs sudo rights if it can be performed by annad that is root?"

**Solution Implemented:**
- Extended RPC protocol with `CheckUpdate` and `PerformUpdate` methods
- Daemon handlers for full update flow (already root!)
- Rewrote annactl update command to use RPC delegation
- **Result:** Users no longer need sudo for updates!

**Benefits:**
- ✅ No more sudo password prompts
- ✅ Cleaner architecture (client=UI, daemon=operations)
- ✅ Better security (updates in root daemon context)
- ✅ Foundation for autonomous updates

**Files:** `anna_common/src/ipc.rs`, `annad/src/rpc_server.rs:722-819`, `annactl/src/commands.rs:3217-3418`

### Beta.88: Critical Fixes ✅
**Status:** ✅ COMPLETE

**1. GPU Detection Bug Fix (CRITICAL)**
- **Issue:** Systems with Intel chipsets + Nvidia GPUs detected as Intel
- **Impact:** Wrong Vulkan driver recommendations (vulkan-intel instead of nvidia)
- **Fix:** Line-by-line lspci parsing, only check VGA/display lines
- **File:** `telemetry.rs:176-202`
- **Result:** Dramatically improved recommendation accuracy

**2. Smart Privilege Handling**
- **Issue:** Updater had hardcoded sudo commands
- **Fix:** Added is_root() check, execute_privileged() helper
- **Logic:** If root → direct execution; If not root → use sudo
- **File:** `updater.rs:19-50`
- **Dependencies:** Added `libc = "0.2"`
- **Result:** Cleaner logs, better performance, backward compatible

---

## 🔄 UNIVERSAL ROLLBACK SYSTEM (Beta.89-91)

**Status:** ✅ PHASE 2 COMPLETE!
**Priority:** HIGH

### Beta.89: Rollback Foundation ✅
**Status:** ✅ COMPLETE

**Implemented:**
- Automatic rollback command generation
- Enhanced Action struct with rollback fields
- Supports: pacman install/remove, systemctl enable/disable/start/stop
- Smart parsing with safety checks
- 5 unit tests passing
- Integrated into executor and RPC server

**File:** `crates/anna_common/src/rollback.rs` (245 lines)

### Beta.91: Rollback CLI Commands ✅
**Status:** ✅ COMPLETE

**Implemented:**
- Action history storage in `/var/log/anna/action_history.jsonl`
- RPC methods: `ListRollbackable`, `RollbackAction`, `RollbackLast`
- CLI commands with subcommand structure:
  - `annactl rollback list` - List all rollbackable actions
  - `annactl rollback action <id>` - Rollback specific action
  - `annactl rollback last [N]` - Rollback last N actions (default: 1)
  - `annactl rollback bundle <name>` - Rollback bundle (existing)
- Dry-run support with `--dry-run` flag
- Beautiful formatted output with action details
- Automatic action history tracking on successful applies

**Files:**
- `crates/annad/src/action_history.rs` (210 lines) - Action history manager
- `crates/anna_common/src/ipc.rs` - RPC protocol extensions
- `crates/annad/src/rpc_server.rs` - Rollback RPC handlers (+180 lines)
- `crates/annactl/src/main.rs` - CLI structure with subcommands
- `crates/annactl/src/commands.rs` - CLI implementations (+165 lines)

**Architecture:**
- Action history stored separately from audit log for better querying
- Full Action objects preserved with rollback metadata
- Query by advice ID, last N actions, or all rollbackable
- Daemon executes rollback commands with full validation

**User Feedback Addressed:**
> "Rollbacks for actions and for bundles... interface must be extremely easy to use, intuitive and beautiful"

✅ Individual actions can now be rolled back
✅ Beautiful, intuitive CLI interface
✅ Dry-run preview before execution
✅ Clear safety information and feedback

### Remaining Work (Phase 3):
- [ ] TUI rollback interface
- [ ] Rollback safety warnings (dependency checks)
- [ ] Bundle rollback using individual action rollbacks

### Proposed System: Rollback for Everything

#### Architecture:

**1. Rollback Command Storage**
Store undo commands in audit log when action succeeds:
```json
{
  "timestamp": "2025-11-06T12:00:00Z",
  "action_type": "apply_action",
  "advice_id": "mangohud",
  "command": "sudo pacman -S --noconfirm mangohud",
  "rollback_command": "sudo pacman -Rns --noconfirm mangohud",
  "success": true,
  "output": "...",
  "can_rollback": true
}
```

**2. Rollback Detection**
Some actions cannot be safely rolled back:
- System configuration changes (needs manual review)
- File modifications (may have user edits)
- Destructive operations (data loss risk)

Mark these as `can_rollback: false` with explanation.

**3. Rollback CLI**
```bash
# List rollbackable actions
annactl rollback --list

# Rollback specific action
annactl rollback --id mangohud

# Rollback bundle
annactl rollback --bundle gaming-essentials

# Rollback last N actions
annactl rollback --last 3

# Preview rollback (dry-run)
annactl rollback --id mangohud --preview
```

**4. Rollback TUI**
Add "Rollback" view accessible from history:
```
╭─────────────────────────────────────────────╮
│         Rollback: MangoHud Installation     │
╰─────────────────────────────────────────────╯

→ Action Details:
  Applied: 2025-11-06 12:00:00 (2 hours ago)
  Command: sudo pacman -S --noconfirm mangohud
  Status: ✓ Success

→ Rollback Will Execute:
  sudo pacman -Rns --noconfirm mangohud

→ This Will:
  • Remove mangohud package
  • Remove unused dependencies
  • Free 12.3 MiB disk space
  • NOT affect your game saves or configs

⚠ Warning:
  Other packages may depend on this.
  Review dependencies before proceeding.

╭─────────────────────────────────────────────╮
│  Enter: Execute  │  p: Preview  │ Esc: Cancel│
╰─────────────────────────────────────────────╯
```

**5. Safety Features:**
- Always show preview before executing
- Dependency check warnings
- Confirmation required
- Backup important configs before rollback
- Show what will be affected
- Clear success/failure messages

**6. Bundle Rollback Enhancement:**
Rollback all actions in a bundle in reverse order:
```
╭─────────────────────────────────────────────╮
│    Rollback Bundle: Gaming Essentials       │
╰─────────────────────────────────────────────╯

This bundle contains 5 actions:
 1. ✓ MangoHud installation
 2. ✓ GameMode installation
 3. ✓ Steam configuration
 4. ✓ Proton-GE setup
 5. ✗ Controller support (cannot rollback)

→ Rollback Order (reverse):
  4 → 3 → 2 → 1 (skip 5)

Total rollback size: 4 actions
Estimated time: 30 seconds

╭─────────────────────────────────────────────╮
│     Enter: Begin  │  Esc: Cancel            │
╰─────────────────────────────────────────────╯
```

### Implementation Plan:
- [ ] Add rollback_command field to Action struct
- [ ] Generate rollback commands for all advice types
- [ ] Store rollback info in audit log
- [ ] Implement rollback detection logic
- [ ] Add rollback CLI commands
- [ ] Add rollback TUI views
- [ ] Add safety checks and previews
- [ ] Add rollback for bundles
- [ ] Test rollback for all action types

---

## 🔥 CRITICAL QUALITY ISSUES (Beta.84+ Priority)

### Advice Quality & Intelligence Overhaul
**Status:** 🔥 CRITICAL (Beta.84-86)
**Priority:** CRITICAL - COMPLETE REVAMP NEEDED

**Real-World User Feedback (razorback system):**
> "The amount of advices, bad advices, non necessary advices, not working apply functions, useless details... it is insane. Million advices can scare the user... we need to completely revamp TUI and improve the UX."

**Major Issues Identified:**

#### 1. Hardware Detection Failures (CRITICAL)
**Examples:**
- ❌ Recommending Vulkan for Intel when system has Nvidia GPU
- ❌ Intel-specific advice when `lspci` shows no Intel hardware
- ❌ Not detecting actual GPU vendor correctly

**Root Cause:**
- Not properly parsing `lspci` output
- Not checking actual hardware before recommendations
- Hardcoded assumptions about hardware

**Planned Solution:**
- [ ] Robust GPU detection from lspci (Nvidia, AMD, Intel)
- [ ] Store detected GPU vendor in SystemFacts
- [ ] Filter all advice by actual detected hardware
- [ ] Add GPU-specific advice categories
- [ ] Verify hardware compatibility before recommending packages
- [ ] Add hardware context to every recommendation

#### 2. Config File Awareness (CRITICAL)
**Examples:**
- ❌ Nvidia environment variables recommended but already in `hyprland.conf`
- ❌ SSH key generation recommended when `~/.ssh/id_ed25519` already exists
- ❌ Application launcher recommended when one is already configured
- ❌ Mirror list shown as "old" but apply doesn't fix it

**Root Cause:**
- Not parsing existing config files
- Not checking for existing configurations
- Detection logic incomplete

**Planned Solution:**
- [ ] Parse Hyprland config for existing env vars before recommending
- [ ] Check `~/.ssh/` for existing keys (ed25519, rsa, ecdsa)
- [ ] Detect existing application launchers (rofi, wofi, fuzzel, etc.)
- [ ] Parse shell configs for existing aliases/functions
- [ ] Check systemd configs before recommending services
- [ ] Verify mirror list dates and test actual functionality
- [ ] Add "already configured" detection for ALL configurable items

#### 3. Software Choice Respect (HIGH)
**Examples:**
- ❌ User uses vim → Anna recommends neovim upgrade
- ❌ No Docker installed → Anna recommends Podman (why?)
- ❌ Recommends Starship prompt without explaining vs oh-my-posh

**Philosophy Issue:**
> "User chose vim for a reason... maybe improving vim is better than forcing neovim"

**Planned Solution:**
- [ ] Detect user's intentional software choices (vim vs neovim, etc.)
- [ ] Offer enhancements for chosen tools instead of replacements
- [ ] Add "respect user choice" mode - enhance don't replace
- [ ] Only suggest alternatives if clearly beneficial with explanation
- [ ] Compare tools side-by-side (Starship vs oh-my-posh features)
- [ ] Don't recommend random alternatives for software user doesn't have

#### 4. Context-Insensitive Recommendations (CRITICAL)
**Examples:**
- ❌ Splash boot screen for fast boot times (contradictory)
- ❌ Vulkan development tools without knowing if user is developer
- ❌ Game emulators just because Citron is installed
- ❌ Docker alternatives when user doesn't want containerization

**Planned Solution:**
- [ ] Add user profile detection (developer, gamer, casual, etc.)
- [ ] Boot time analysis before recommending splash screens
- [ ] Development tool detection before recommending dev packages
- [ ] Gaming profile detection (emulators, Steam, Lutris presence)
- [ ] Context-aware recommendations based on actual usage patterns
- [ ] "Why is this relevant?" explanation for every recommendation

#### 5. Duplicate Advice (HIGH)
**Examples:**
- ❌ mangohud appearing multiple times
- ❌ Other repeated recommendations

**Planned Solution:**
- [ ] Deduplication logic in advice generation
- [ ] Track advice IDs to prevent duplicates
- [ ] Merge similar advice into comprehensive single items
- [ ] Add unit tests to catch duplicate advice generation

#### 6. False Positive Error Detection (HIGH)
**Examples:**
- ❌ "Excessive system errors detected" - but all from TLP (normal behavior)
- ❌ Expected errors flagged as problems

**Planned Solution:**
- [ ] Whitelist expected errors (TLP, Nvidia driver messages, etc.)
- [ ] Categorize errors by severity and source
- [ ] Don't flag normal operational messages as errors
- [ ] Add error context and severity analysis
- [ ] Filter out known-good error patterns

#### 7. Broken Apply Functions (CRITICAL)
**Example:**
- ❌ Mirror list update shown but apply doesn't fix it

**Planned Solution:**
- [ ] Test every apply command before shipping
- [ ] Add dry-run validation for all commands
- [ ] Verify command success and state change after execution
- [ ] Add rollback for failed applies
- [ ] Show clear error messages when apply fails

#### 8. Better User Telemetry & Profiling (CRITICAL)
**Philosophy:**
> "We need to treat the user like maybe someone that doesn't know... but maybe someone that knows a lot"

**Planned Solution:**
- [ ] User expertise detection (beginner, intermediate, expert)
- [ ] Learning from user actions (what they apply, what they ignore)
- [ ] Adaptive recommendations based on expertise level
- [ ] Detailed config file parsing and awareness
- [ ] Software choice pattern analysis
- [ ] Hardware capability detection and respect
- [ ] Use case detection (gaming, development, creative work, etc.)
- [ ] Don't overwhelm beginners with advanced advice
- [ ] Don't patronize experts with basic recommendations

#### 9. Overwhelming Advice Volume (CRITICAL)
**Issue:**
> "Million advices can scare the user"

**Current State:**
- Beta.83 filtering helps but not enough
- Still too many irrelevant recommendations
- Quality over quantity needed

**Planned Solution:**
- [ ] Strict relevance filtering (hardware, software, use case)
- [ ] Priority scoring based on actual system state
- [ ] Remove all "nice to have" advice for irrelevant use cases
- [ ] Max 30 recommendations default (currently 120+)
- [ ] Group related advice into bundles
- [ ] Add "Why this matters to YOU" personalized explanations

---

### Beta.84 - Foundation Quality Fixes (COMPLETED)
**Status:** ✅ FULLY COMPLETED

**Telemetry & Detection Improvements:**
1. **GPU Detection Fix** - Fixed CRITICAL false positives
   - Line-by-line lspci parsing prevents Intel detection on Nvidia systems
   - Accurate GPU vendor detection (Nvidia, AMD, Intel)
   - Zero false positives for hardware recommendations

2. **SSH Key Detection** - Checks ~/.ssh/ for existing keys
   - Detects id_ed25519, id_rsa, id_ecdsa, id_dsa
   - Added has_ssh_client_keys field to NetworkProfile
   - Prevents recommending key creation when user already has keys

3. **TLP Error Whitelisting** - Filters out false positive errors
   - Whitelists common TLP informational messages logged as errors
   - Also filters GNOME Shell, PulseAudio/Pipewire benign errors
   - Significantly reduces false "excessive system errors" warnings

**Config & Parser Framework:**
4. **Universal Config Parser Framework** - Built from Arch Wiki specs
   - Supports 11 window managers: Hyprland, Sway, i3, bspwm, awesome, qtile, river, wayfire, openbox, xmonad, dwm
   - Detects active WM automatically
   - Parses multiple config formats: HyprlandConf, i3-style, INI, Shell, Lua, Python, Haskell
   - Check for environment variables in WM configs
   - Check for specific settings/configurations

5. **Nvidia Env Var Detection** - Checks WM configs properly
   - Now detects Nvidia env vars in Hyprland/Sway/i3 configs
   - Stops recommending vars that are already configured
   - Checks both WM configs AND system configs

**Bug Fixes:**
6. **Deduplication Logic** - Eliminates duplicate advice
   - HashSet-based ID deduplication
   - Removes duplicates like mangohud appearing 3 times
   - Logs duplicate count for debugging

7. **Version Comparison Fix** - Strips 'v' prefix from GitHub tags
   - Prevents false "update available" when already on latest

8. **Mirror List Apply Fix** - Added sudo to reflector command
   - Command now works instead of failing with permission error
   - Users can actually update their mirror list

9. **Status Command Display Fix** - Increased detail truncation limit
   - Raised from 60 to 120 characters
   - Prevents command output from being excessively cropped
   - Full commands now visible in Recent Activity section

---

### Implementation Roadmap - Quality Overhaul

**Beta.84 - Foundation (Hardware & Config Detection):**
- ✅ Implement robust GPU detection (Nvidia, AMD, Intel) - Line-by-line lspci parsing
- ✅ Add GPU vendor to SystemFacts - Already exists (is_nvidia, is_intel_gpu, is_amd_gpu)
- ✅ Create config file parser framework - Universal parser for all WMs (Arch Wiki based)
- [ ] Implement SSH key detection
- ✅ Add Hyprland config parser - Included in universal framework
- ✅ Add Sway/i3 config parser - Included in universal framework
- ✅ Add support for all major WMs - Hyprland, Sway, i3, bspwm, awesome, qtile, etc.
- [ ] Fix mirror list apply function
- ✅ Add deduplication logic - HashSet-based ID deduplication
- ✅ Filter advice by detected hardware - Guards already present, detection fixed

**Beta.85 - Intelligence (Context & User Profiling):**
- [ ] User expertise detection system
- [ ] Software choice pattern analysis
- [ ] Use case detection (gaming, dev, creative)
- [ ] Boot time analysis
- [ ] Error categorization and whitelisting
- [ ] "Already configured" detection for all items
- [ ] Application launcher detection
- [ ] Shell config parsing

**Beta.86 - Refinement (Adaptive & Learning):**
- [ ] Learning system from user actions
- [ ] Adaptive recommendation engine
- [ ] Enhanced vs replacement logic
- [ ] Tool comparison framework
- [ ] Personalized explanations ("Why this matters to YOU")
- [ ] Max 30 advice limit with quality scoring
- [ ] Comprehensive testing suite for advice quality
- [ ] User feedback integration system

**Success Metrics:**
- Reduce average advice count from 120 → 30 (75% reduction)
- Zero false positives for hardware recommendations
- 100% config file awareness for common configs
- Zero duplicate advice items
- All apply functions verified working
- User satisfaction score > 90%

---

### System & Kernel Error Advice Improvements
**Status:** 🔄 PLANNED (Beta.83)
**Priority:** HIGH

**User Feedback:**
- "Kernel error advise is not giving any details"
- "Multiple Errors advice should offer solutions based on Arch Wiki"

**Planned Solution:**
- [ ] Detect common kernel error patterns
- [ ] Parse dmesg/journalctl for specific error types
- [ ] Link to relevant Arch Wiki articles
- [ ] Provide specific fixes for common kernel issues
- [ ] Add error context and troubleshooting steps
- [ ] Show actual error samples with explanations

### Release Notes Display / Notification Fixes
**Status:** 🔄 PARTIAL (Beta.83)
**Priority:** HIGH

**User Feedback:**
- "During the update, Anna is not showing the proper release notes. Still a GDBus error"
- "annactl update --install should not update if its the same version... and proper release notes must be shown"

**Solution:**
- [ ] Fix GDBus error in notification system
- [ ] Improve release notes display after auto-update
- ✅ Skip update if already on latest version (avoid unnecessary reinstallation)
- ✅ Show clear "Already on latest version" message when up-to-date
- ✅ Display proper release notes for current version when --install is used
- [ ] Test notification system across different DEs
- [ ] Provide fallback if notification daemon unavailable

### Beautification Enhancement Suite
**Status:** 🔄 PLANNED (Beta.83-85)
**Priority:** HIGH

**User Feedback:**
- "Try to keep and respect always the user for only GTK or only Qt to be consistent"
- "Ensure always a dark theme and a light one is installed per each configuration"
- "Add terminal beautification (dark + light color schemes)"

#### Phase 1: Terminal Color Schemes (Beta.83)
- [ ] Enhance terminal_config.rs with dual theme support
- [ ] Recommend both dark AND light color schemes
- [ ] Catppuccin: Mocha (dark) + Latte (light)
- [ ] Nord + Nord Light variants
- [ ] Dracula + Dracula Light
- [ ] Provide easy switching instructions

#### Phase 2: Desktop Environment Toolkit Consistency (Beta.84)
**GTK Desktop Environments (GNOME, XFCE, Cinnamon, MATE):**
- [ ] Recommend GTK applications only
- [ ] Avoid Qt app recommendations on GTK DEs
- [ ] Ensure GTK theme consistency

**Qt Desktop Environments (KDE Plasma, LXQt):**
- [ ] Recommend Qt applications only  
- [ ] Avoid GTK app recommendations on Qt DEs
- [ ] Ensure Qt theme consistency

#### Phase 3: Complete Theme Coverage (Beta.85)
- [ ] GNOME: Both dark + light theme pairs (Adwaita, Adwaita-dark)
- [ ] KDE Plasma: Dark + light variants (Breeze, Breeze-Dark)
- [ ] XFCE: Dark + light GTK themes
- [ ] Cinnamon: Dark + light Cinnamon themes
- [ ] MATE: Dark + light MATE themes
- [ ] LXQt: Dark + light Qt themes (Kvantum variants)

### Additional Desktop Environments
**Status:** 🔜 PLANNED (Beta.86+)

**Planned Additions:**
- [ ] LXDE support (predecessor to LXQt, still widely used)
- [ ] Budgie Desktop support (modern, elegant DE)
- [ ] Pantheon support (elementary OS DE)
- [ ] Deepin support (beautiful, modern Chinese DE)

### Real-Time Action Transparency
**Status:** 🔥 CRITICAL (Beta.83)
**Priority:** CRITICAL

**User Feedback:**
- "the application script should be a realtime terminal to see the real input and output and the actions anna is taking (**IMPORTANT, WE WANT TRANSPARENCY**)"
- "Not sure if it has applied the action properly" (fstrim.service shows inactive but was triggered by timer)

**Problem:**
Users cannot see what Anna is actually doing when applying advice. Commands run silently in background, creating trust issues and confusion about whether actions succeeded.

**Planned Solution:**
- [ ] Real-time command output display during action application
- [ ] Show actual bash commands being executed
- [ ] Stream stdout/stderr in real-time to TUI
- [ ] Clear success/failure indicators with exit codes
- [ ] Show service status after systemctl commands
- [ ] Transparency: Users see EXACTLY what Anna does

**Example Desired Behavior:**
```
Applying: Enable TRIM for SSD health...
> Running: sudo systemctl enable fstrim.timer
Created symlink /etc/systemd/system/timers.target.wants/fstrim.timer → /usr/lib/systemd/system/fstrim.timer
> Running: sudo systemctl start fstrim.timer
> Verifying: systemctl status fstrim.timer
● fstrim.timer - Discard unused blocks once a week
     Loaded: loaded
     Active: active (waiting)
✓ Action completed successfully!
```

### Applied Advice Persistence Bug
**Status:** ✅ FIXED (Beta.83)
**Priority:** CRITICAL

**User Feedback:**
- "Applied 2 TRIM advices, still on the list"
- "and of course the applied advice is still not in the history... (that is bad)"

**Root Cause Found:**
Action type mismatch in audit log filtering:
- Audit entries were created with `action_type: "execute_action"`
- Filter was looking for `action_type: "apply_action"`
- Result: Applied advice IDs were never being found in audit log!

**Solution Implemented:**
- ✅ Fixed action_type mismatch in executor.rs (line 148)
- ✅ Changed "execute_action" → "apply_action" to match filter
- ✅ Applied advice now correctly recorded in audit log
- ✅ `get_applied_advice_ids()` now finds and returns applied advice
- ✅ Applied advice properly filtered from advice list
- ✅ History tracking now works correctly

**Code Changes:**
```rust
// executor.rs line 148 - BEFORE:
action_type: "execute_action".to_string(),

// AFTER:
action_type: "apply_action".to_string(),
```

**Remaining Work:**
- [ ] Debug TRIM advice specifically (timer vs service detection confusion)
- [ ] Add "Show applied" toggle option in TUI for verification
- [ ] Consider adding applied advice count to dashboard

### Universal Rollback System
**Status:** 🔥 CRITICAL (Beta.83)
**Priority:** HIGH

**User Feedback:**
- "rollback is only for bundles? Noway! It should be for any applied advice...."

**Current Limitation:**
Only bundles can be rolled back currently. Individual advice actions (package installs, config changes, system tweaks) cannot be undone.

**Planned Solution:**
- [ ] Design universal rollback system for ALL advice types
- [ ] Track rollback commands for each advice action
- [ ] Store rollback data in history/audit log
- [ ] Implement `annactl rollback --id <advice-id>`
- [ ] Implement `annactl rollback --number <history-number>`
- [ ] Support rollback by timestamp or last N actions
- [ ] Preview rollback actions before executing
- [ ] Handle complex rollbacks (config file changes, package removals)
- [ ] Validate rollback safety (don't break dependencies)

**Example Usage:**
```bash
# View history with rollback options
annactl history

# Rollback specific advice by ID
annactl rollback --id trim-ssd

# Rollback last applied action
annactl rollback --last

# Rollback multiple recent actions
annactl rollback --last 3

# Dry-run to see what would be undone
annactl rollback --id some-advice --dry-run
```

### Shell Completion Auto-Installation
**Status:** ✅ COMPLETED (install.sh updated)
**Priority:** HIGH

**User Feedback:**
- "I do not know or understand what annactl completions bash does... or if its necessary"
- "`annactl completions` should be an advice that Anna recommends and applies, not just a standalone command"
- "Many commands in linux work with autocompletion with tab without altering .bashrc"

**Problem:**
Shell completions were not installed automatically. Users had to:
1. Know what completions are
2. Manually run `annactl completions bash`
3. Pipe it to the right location

Most Linux commands (git, docker, systemctl) install completions automatically to system directories, so they work immediately without .bashrc changes.

**Solution Implemented:**
Updated `scripts/install.sh` to automatically install completions:

```bash
# Install shell completions (lines 208-236)
echo -e "${CYAN}${ARROW}${RESET} Installing shell completions..."

# Bash completions
if [ -d "/usr/share/bash-completion/completions" ]; then
    "$INSTALL_DIR/annactl" completions bash | sudo tee /usr/share/bash-completion/completions/annactl > /dev/null 2>&1
    echo -e "${GREEN}${CHECK}${RESET} Bash completions installed"
fi

# Zsh completions
if [ -d "/usr/share/zsh/site-functions" ]; then
    "$INSTALL_DIR/annactl" completions zsh | sudo tee /usr/share/zsh/site-functions/_annactl > /dev/null 2>&1
    echo -e "${GREEN}${CHECK}${RESET} Zsh completions installed"
fi

# Fish completions
if [ -d "/usr/share/fish/vendor_completions.d" ]; then
    "$INSTALL_DIR/annactl" completions fish | sudo tee /usr/share/fish/vendor_completions.d/annactl.fish > /dev/null 2>&1
    echo -e "${GREEN}${CHECK}${RESET} Fish completions installed"
fi
```

**How It Works:**
- Checks for standard shell completion directories
- Installs completions for all detected shells
- Works immediately without .bashrc changes (like git, docker)
- Silent installation (no clutter in output)
- Warns if no completion directories found (non-standard setup)

**Result:**
After fresh install or update, tab completion works immediately:
```bash
annactl adv<Tab>    # → annactl advise
annactl sta<Tab>    # → annactl status
annactl --<Tab>     # → shows all flags
```

**Future Enhancement (Optional):**
If needed, Anna could add advice to detect missing completions and offer to install them (for manual installs or non-standard setups), but this is low priority since the installer now handles it automatically.

### TUI Box Drawing Fix
**Status:** 🔜 PLANNED (Beta.83)
**Priority:** LOW

**User Feedback:**
- "Broken box in the update (probably because of a sh command?)" - Update successful box has misaligned bottom border

**Problem:**
The update success box has a formatting issue where the bottom border is not aligned properly with the top border:
```
╭────────────────────────────────────────────────────────────╮
│ 🎉 Update Successful!
│
│   Version: 1.0.0-beta.81 → v1.0.0-beta.82
│   Released: 2025-11-06T08:24:04Z
│
╰────────────────────────────────────────────────────────────╯  <- Misaligned
```

**Cause:**
Box drawing characters may have different widths depending on terminal font rendering, or the calculation of border width doesn't account for emojis/multi-byte characters.

**Planned Solution:**
- [ ] Review box drawing code in annactl update command
- [ ] Ensure border width calculation accounts for emoji width
- [ ] Test across different terminals (alacritty, kitty, gnome-terminal)
- [ ] Consider using ASCII fallback for problematic terminals

### Journalctl Integration
**Status:** ✅ COMPLETE (Beta.90)

**User Feedback:**
- "annactl status should show the most relevant entries about the journalctl of annad, at least top 10"

**Implemented:**
- ✅ Added journalctl integration to `annactl status`
- ✅ Shows top 10 daemon log entries automatically
- ✅ Color-coded by severity (red=error, yellow=warn, cyan=info)
- ✅ ISO timestamps for precise timing
- ✅ Graceful permission handling

**File:** `crates/annactl/src/commands.rs:199-265`


---

## 🎨 MAJOR TUI REVAMP - v1.1.0-beta.1 (Eurydice) - PLANNED

**Status:** 📋 PLANNED
**Priority:** CRITICAL (Major Milestone)

**User Vision:**
> "TUI + CLI simplification... perfect—here's a single, copy-paste master prompt...
> Keep Anna easy, intuitive, beautiful — pastel palette, crisp borders, human copy."

### Goals: Simplified CLI + First-Class TUI

This is a major architectural milestone to:
1. **Freeze & simplify the CLI** to a permanent, minimal set
2. **Build a first-class TUI** that mirrors CLI features via elegant menus
3. **Keep Anna easy, intuitive, beautiful** - pastel palette, crisp borders

---

### A. Final CLI Surface (Simplified & Stable)

**Core Commands (Keep):**
- `annactl status` - Concise health + last 10 audit entries
- `annactl advise [category|mode] [-l N]` - Numbered list with filters
- `annactl apply <list>` - Accept 1, 1-5, 1,3,5, --id, --bundle
- `annactl report [--format md|json]` - Full report
- `annactl doctor [--fix]` - Diagnostics + fixes
- `annactl tui` - Launch new dashboard (replaces `dashboard`)

**Route to TUI:**
- bundles, rollback, dismiss, history, config, ignore → TUI sub-menus
- Legacy commands hidden under `--experimental`

---

### B. TUI Requirements (Parity + Beautiful)

**Framework:** ratatui + crossterm
- Clean teardown, window resize safe
- Logs to ~/.local/state/anna/logs/annactl-tui.log

**Main Menu (Single Key Navigation):**
```
╭──────────────────────────────────────────────╮
│ 🌸 Anna — Your Arch Linux Sysadmin Companion │
├──────────────────────────────────────────────┤
│ [1] System Overview     [2] Advice Center    │
│ [3] Apply Actions       [4] Reports          │
│ [5] Bundles / Rollback  [6] Settings         │
│ [7] Ignore & Dismiss    [H] History          │
│ [Q] Quit                                     │
╰──────────────────────────────────────────────╯
Footer: Arrows/Tab • Enter to select • Q quit • ? help
```

**Screens & Behaviors:**

1. **System Overview**
   - Health score (0-100)
   - Critical issues list
   - Last refresh, auto-update status
   - Daemon status, last 10 audit entries

2. **Advice Center**
   - Scrollable, numbered list with filters:
     - `c` = category
     - `p` = priority (🔴/🟡/🟢/🔵)
     - `r` = risk
     - `/` = search
     - `f` = mode (smart|critical|recommended|all)
   - Enter = details panel (full description, wiki refs, commands)
   - `a` = apply (live output modal)
   - Immediate refresh after success (Beta.63)

3. **Apply Actions**
   - Multi-select (Space) then A to apply batch
   - Display total operations
   - Live output modal (yellow border → green when done)

4. **Reports**
   - Generate Markdown/JSON
   - S to save, show path
   - Include "Showing N of M recommendations"

5. **Bundles / Rollback**
   - List bundles with installed markers
   - Enter to view contents
   - B to apply, R to rollback by number

6. **Settings**
   - Theme (pastel/dim)
   - Notifications cooldown
   - Refresh cadence
   - Auto-update status
   - Wallpaper & DE helper links

7. **Ignore & Dismiss**
   - `i` ignore by category/priority
   - `d` dismiss per item
   - Lists with U to unhide/undismiss

8. **History**
   - Audit trail with timestamps
   - Exit status, stdout excerpt
   - `o` to open full log

**Visual Style:**
- **Pastel palette:**
  - Magenta title
  - Mint success
  - Amber warn
  - Rose error
  - Blue info
  - Dark indigo background
- **Rounded borders**, 1-space padding
- **80×24 safe layout**

---

### C. Feature Coverage (Must Remain Working)

All existing features must work in TUI:
- ✅ Wallpaper Intelligence (Beta.82)
- ✅ 9 Desktop Environments (Beta.70-81)
- ✅ Terminal/Shell/Git intelligence (Beta.71-73)
- ✅ Advice dependencies (`satisfies`) (Beta.62)
- ✅ Instant refresh after apply (Beta.63)
- ✅ Non-interactive package ops (Beta.58)
- ✅ Live output modal (Beta.60)
- ✅ Smart notifications (Beta.57)
- ✅ Auto-update always on (Beta.61)
- ✅ 14 WM bundles (Beta.94-95)
- ✅ Network health monitoring (Beta.96)

---

### D. Architecture & Integration

- TUI calls existing RPCs (no logic duplication)
- All applies go through same executor & audit trail
- Respect ignore/dismiss filters everywhere
- Keep JSON/MD reports identical between CLI and TUI

---

### E. Deliverables

**New Files:**
- `src/annactl/tui/`
  - main_menu.rs
  - overview.rs
  - advice.rs
  - apply.rs
  - reports.rs
  - bundles.rs
  - settings.rs
  - ignore_dismiss.rs
  - history.rs
  - live_output.rs

**Documentation:**
- docs/TUI_GUIDE.md (keybindings, screenshots)
- README: new TUI screenshots
- CHANGELOG: v1.1.0-beta.1 notes

---

### F. Definition of Done

**CLI Tests:**
- ✅ `annactl status` prints health + audit
- ✅ `annactl advise` respects filters, shows "N of M"
- ✅ `annactl apply 1,3-4` runs non-interactive with live output
- ✅ `annactl report --format md` produces file
- ✅ `annactl doctor --fix` performs fixes
- ✅ `annactl tui` launches dashboard

**TUI Tests:**
- ✅ Navigate all menus
- ✅ Apply single & batch with live output
- ✅ Advice disappears post-apply
- ✅ Satisfied advice hidden
- ✅ Filters work (c, p, r, /, f)
- ✅ Bundles list/rollback works
- ✅ Ignore/dismiss shows and can unhide
- ✅ Reports export path printed
- ✅ No panics on resize
- ✅ Terminal restored on exit

**Style:**
- ✅ Pastel theme, rounded borders
- ✅ Single-line footer hints
- ✅ All errors actionable (1-line fix suggestion)

---

## 📊 SYSTEM PERFORMANCE MONITORING - PLANNED

**Status:** 📋 PLANNED
**Priority:** HIGH

**User Feedback:**
> "we need to monitor network, disk performance, ram,... so anna can take actions before problems arise"

### Goals: Proactive Performance Monitoring

Anna should monitor system performance and warn BEFORE problems occur:

---

### A. Metrics to Monitor

**1. Disk Performance**
- ✅ SMART health (already implemented)
- 🔄 Disk I/O latency
- 🔄 Read/write speeds
- 🔄 Queue depth
- 🔄 Slow disk warnings
- 🔄 Degraded RAID arrays

**2. RAM Monitoring**
- ✅ Memory pressure (already implemented)
- 🔄 Swap usage trends
- 🔄 Memory leak detection
- 🔄 OOM risk prediction
- 🔄 Cache hit ratios

**3. CPU Monitoring**
- ✅ Temperature monitoring (already implemented)
- 🔄 Sustained high load (>80% for 5+ minutes)
- 🔄 CPU throttling detection
- 🔄 Per-process CPU hogs
- 🔄 Core utilization imbalance

**4. Network Performance (Beta.96+)**
- ✅ Connectivity status
- ✅ Packet loss detection
- ✅ Latency monitoring
- ✅ DNS resolution testing
- 🔄 Bandwidth saturation
- 🔄 Network interface errors/drops

**5. Filesystem Health**
- ✅ Disk space prediction (already implemented)
- 🔄 Inode exhaustion warnings
- 🔄 Filesystem errors (ext4/btrfs/xfs)
- 🔄 Mount point issues

---

### B. Implementation Plan

**Phase 1: Data Collection**
- Extend telemetry.rs with performance metrics
- Use `/proc/`, `iostat`, `vmstat`, `ss`, `ip`
- Sample metrics every 5 minutes
- Store trends in SQLite database

**Phase 2: Anomaly Detection**
- Define baseline performance per system
- Detect degradation trends
- Alert on anomalies (sudden changes)

**Phase 3: Predictive Warnings**
- "Disk I/O degrading - SMART check recommended"
- "RAM usage trending up - possible leak in [process]"
- "Network latency increased 3x - check router"
- "CPU temperature rising - thermal paste aging?"

**Phase 4: Automated Actions**
- Restart misbehaving services
- Clear caches if memory critical
- Suggest kernel parameter tuning
- Recommend hardware upgrades

---

### C. Example Warnings

```
🔴 Sustained high CPU load (95% for 10 minutes)
    Process 'firefox' consuming 85% CPU. Consider restarting it or
    checking for runaway tabs.

🟡 RAM usage trending upward
    Memory usage increased from 40% to 75% over past hour. Possible
    memory leak in 'electron'. Consider restarting application.

🟡 Disk I/O latency high (200ms avg)
    Disk /dev/sda showing 10x normal latency. Check SMART status,
    cable connection, or consider drive replacement.

🔵 Network latency increased
    Average ping time increased from 20ms to 80ms. Router may need
    restart or ISP experiencing issues.
```

---

### D. Integration Points

**Telemetry Collection:**
- `crates/annad/src/telemetry.rs`
- New functions: `collect_io_metrics()`, `collect_memory_metrics()`, etc.

**Trend Storage:**
- SQLite database at `/var/lib/anna/metrics.db`
- Tables: cpu_metrics, ram_metrics, disk_metrics, network_metrics
- Retention: 7 days of 5-minute samples

**Advice Generation:**
- `crates/annad/src/recommender.rs`
- New functions: `check_disk_performance()`, `check_ram_trends()`, etc.

**TUI Integration:**
- System Overview screen shows live metrics
- Graphs for CPU/RAM/disk/network (sparklines)
- Color-coded thresholds

---

### E. Timeline

**v1.0.x (Current):**
- ✅ Network health monitoring (Beta.96)
- ✅ Basic telemetry (disk space, CPU temp, memory)

**v1.1.0 (Eurydice):**
- 🔄 TUI revamp with performance graphs
- 🔄 Disk I/O monitoring
- 🔄 RAM trend analysis
- 🔄 Sustained load detection

**v1.2.0:**
- 🔄 Predictive analytics
- 🔄 Automated remediation
- 🔄 Historical performance reports

---

## 🎯 Roadmap Summary

**Immediate (Beta.97+):**
1. Test all 14 WM bundles with real systems
2. Add `annactl bundles` command
3. Improve bundle filtering

**Short-term (v1.0.x):**
4. Disk I/O performance monitoring
5. RAM trend analysis
6. More WM bundles (LeftWM, Spectrwm, etc.)

**Medium-term (v1.1.0 Eurydice):**
7. **Major TUI Revamp** - Full menu system, pastel theme
8. Simplified CLI surface
9. Performance graphs in TUI
10. Sustained load detection

**Long-term (v1.2.0+):**
11. Predictive analytics
12. Automated remediation
13. Configuration file generation
14. Arch Wiki General Recommendations coverage
15. Theme/appearance bundles
16. Desktop environment bundles

