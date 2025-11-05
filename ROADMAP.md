# Anna Assistant - Feature Roadmap

This document tracks requested features and improvements from user feedback.

## 🔥 CRITICAL (Blocks Basic Functionality)

### 1. Show Command Output When Applying
**Status:** ⏳ IN PROGRESS

**Issue:** When applying recommendations, users can't see what's happening.
- Package installations appear frozen
- No progress indication
- Users think Anna is dead/hanging

**Solution:**
- [ ] Show command output in real-time in TUI (modal overlay)
- [ ] Stream stdout/stderr as command executes  
- [ ] Show progress for long-running operations
- [ ] Allow user to close output window when done

**User Quote:** *"I thought it was dead... we need an overlapping window terminal would be ideal)"*

---

## 🚨 HIGH PRIORITY (Major UX Issues)

### 2. Remove Applied Items from List Immediately
**Status:** 📋 TODO

**Issue:** Applied items disappear only after refresh, causing confusion.

**Solution:**
- [ ] Remove item from advice list immediately after successful apply
- [ ] Or show with strikethrough/greyed out
- [ ] Refresh only updates other items

### 2a. Smart Advice Dependencies & Bundle Awareness
**Status:** 📋 TODO

**User Insight:** *"some advices might apply other advices... for example fixing brightness keys and volume implies installing sound stack or brightnessctl... so other advices will need to be removed if a bundle is applied"*

**Issue:** Applying one recommendation can satisfy multiple others, creating redundancy.

**Examples:**
- Apply "Setup Hyprland volume controls" → Installs wpctl/pamixer
  - Should remove: "Install audio control tools"
  - Should remove: "Configure volume keys"
- Apply "Setup Gaming Environment" → Installs Steam, Lutris, Wine
  - Should remove: "Install Steam"
  - Should remove: "Install Wine"
  - Should remove: "Setup Proton"

**Solution:**
- [ ] Add `satisfies: []` field to advice - list of advice IDs this satisfies
- [ ] Add `satisfied_by: []` field to advice - what can satisfy this
- [ ] When applying advice, check what it satisfies:
  ```rust
  applied_advice.satisfies.iter().for_each(|id| {
      advice_list.retain(|a| a.id != id);
  });
  ```
- [ ] Bundle applications automatically mark related advice as satisfied
- [ ] Show user: "✓ Applied X which also satisfies: Y, Z"
- [ ] Smart filtering: Don't show advice already satisfied by bundles

**Example Implementation:**
```rust
Advice {
    id: "hyprland-volume-setup",
    title: "Setup Hyprland volume controls",
    satisfies: vec![
        "install-wpctl",
        "install-pamixer",
        "configure-volume-keys",
        "audio-control-missing"
    ],
    // ... rest of advice
}
```

**When Applied:**
```
✓ Applied: Setup Hyprland volume controls
→ This also satisfied:
  - Install audio control tools
  - Configure volume keys
  - Audio control recommendation

Removed 3 redundant recommendations from list.
```

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

1. **Immediate (Current Session):**
   - ✅ Better error details
   - ⏳ Command output display in TUI
   - Remove applied items immediately

2. **Next Week:**
   - Hyprland keybinding detection & improvement
   - GTK/Qt theme consistency
   - Terminal detection & configuration

3. **Next Month:**
   - Config file intelligence (vim, shell, git)
   - Desktop environment full setup
   - Notification daemon configuration

4. **Long-term (Ongoing):**
   - Wiki parsing and knowledge extraction
   - Full setup solutions (one-command environments)
   - Advanced intelligence and learning

---

**Last Updated:** 2025-11-05
**User Feedback Session:** Beta.60
**Your Vision:** *"Anna should improve whatever I'm using - Hyprland, GTK, Qt, vim, shell - everything configured perfectly with wiki knowledge"* ✨
