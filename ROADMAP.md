# Anna Assistant - Feature Roadmap

This document tracks requested features and improvements from user feedback.

---

## ✅ COMPLETED (Beta.61-74)

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
**Status:** 🔜 PLANNED (Beta.87)

**User Feedback:**
- "annactl status should show the most relevant entries about the journalctl of annad, at least top 10"

**Planned Solution:**
- [ ] Add journalctl integration to `annactl status`
- [ ] Show top 10 relevant annad log entries
- [ ] Filter by severity (errors, warnings)
- [ ] Provide context for log entries

