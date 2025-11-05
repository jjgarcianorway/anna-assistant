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
