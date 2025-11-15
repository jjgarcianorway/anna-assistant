# HYPRLAND ECOSYSTEM RESEARCH
## Comprehensive Documentation for Production-Quality Hyprland Bundles

**Research Date:** November 7, 2025
**Sources:** Official Hyprland Wiki (wiki.hypr.land), Arch Wiki, Official GitHub repositories
**Purpose:** Building three-tier Hyprland bundles (minimal/default/signature)

---

## TABLE OF CONTENTS

1. [Core Hyprland Configuration](#1-core-hyprland-configuration)
2. [Official Hypr-Ecosystem Tools](#2-official-hypr-ecosystem-tools)
3. [Essential Wayland Compositor Tools](#3-essential-wayland-compositor-tools)
4. [Advanced Frameworks](#4-advanced-frameworks)
5. [NVIDIA Configuration](#5-nvidia-configuration)
6. [Configuration File Locations](#6-configuration-file-locations)
7. [Tool Matrix by Tier](#7-tool-matrix-by-tier)
8. [Integration Patterns](#8-integration-patterns)
9. [Smart Defaults & Recommended Keybindings](#9-smart-defaults--recommended-keybindings)
10. [Configuration Templates](#10-configuration-templates)

---

## 1. CORE HYPRLAND CONFIGURATION

### 1.1 Configuration File Structure

**Location:** `~/.config/hypr/hyprland.conf`

**Auto-generation:** Hyprland automatically generates an example config if none exists.

**Key Sections:**
```bash
# Monitors - Display configuration
monitor = DP-1,1920x1080@60,0x0,1

# Environment variables
env = XCURSOR_SIZE,24
env = HYPRCURSOR_SIZE,24

# Startup applications
exec-once = waybar
exec-once = hyprpaper
exec-once = mako

# Input configuration
input {
    kb_layout = us
    follow_mouse = 1
    touchpad {
        natural_scroll = false
    }
}

# General settings
general {
    gaps_in = 5
    gaps_out = 20
    border_size = 2
    col.active_border = rgba(33ccffee)
    col.inactive_border = rgba(595959aa)
    layout = dwindle
}

# Decoration
decoration {
    rounding = 10
    blur {
        enabled = true
        size = 3
        passes = 1
    }
    drop_shadow = yes
    shadow_range = 4
    shadow_render_power = 3
}

# Animations
animations {
    enabled = yes
    bezier = myBezier, 0.05, 0.9, 0.1, 1.05
    animation = windows, 1, 7, myBezier
    animation = windowsOut, 1, 7, default, popin 80%
    animation = fade, 1, 7, default
    animation = workspaces, 1, 6, default
}

# Window rules
windowrule = float, ^(pavucontrol)$
windowrulev2 = opacity 0.9 0.9, class:^(kitty)$

# Keybindings
bind = SUPER, Q, exec, kitty
bind = SUPER, C, killactive
bind = SUPER, M, exit
```

### 1.2 Configuration Variable Types

| Type | Description | Example |
|------|-------------|---------|
| **int** | Integer values | `border_size = 2` |
| **bool** | Boolean (true/false, yes/no, on/off, 0/1) | `enabled = true` |
| **float** | Floating point numbers | `shadow_range = 4.5` |
| **color** | Hex color with alpha | `rgba(33ccffee)` |
| **vec2** | Two floats separated by space | `gaps_in = 5 5` |
| **MOD** | Modifier keys | `SUPER`, `SUPER_SHIFT`, `CTRL_ALT` |
| **str** | String values | `kb_layout = us` |
| **gradient** | Color gradients | `rgba(11ee11ff) rgba(1111eeff) 45deg` |
| **font_weight** | 100-1000 or preset names | `bold`, `normal`, `600` |

### 1.3 Keywords Reference

| Keyword | Purpose | Example |
|---------|---------|---------|
| **monitor** | Display configuration | `monitor=DP-1,1920x1080@60,0x0,1` |
| **env** | Environment variables | `env = XCURSOR_SIZE,24` |
| **exec-once** | Run once on startup | `exec-once = waybar` |
| **exec** | Run on every reload | `exec = pkill waybar; waybar` |
| **source** | Include external files | `source = ~/.config/hypr/monitors.conf` |
| **bind** | Keybinding | `bind = SUPER, Q, exec, kitty` |
| **windowrule** | Window-specific rules | `windowrule = float, ^(pavucontrol)$` |
| **windowrulev2** | Enhanced window rules | `windowrulev2 = opacity 0.9, class:^(kitty)$` |

### 1.4 Keybinding Syntax

**Basic Format:**
```bash
bind = MODS, key, dispatcher, params
```

**Modifiers:**
- `SUPER` - Windows/Super key (primary modifier)
- `SHIFT`
- `CTRL`
- `ALT`
- `MOD2`, `MOD3` - Additional modifiers

**Combining Modifiers:**
```bash
bind = SUPER_SHIFT, Q, exec, firefox
bind = CTRL_ALT, Delete, exec, systemctl poweroff
```

**Mouse Binds:**
```bash
bindm = SUPER, mouse:272, movewindow
bindm = SUPER, mouse:273, resizewindow
```

**Bind Flags:**
- `l` - Locked (works when locked)
- `r` - Release (triggers on key release)
- `e` - Repeat (continues while held)
- `n` - Non-consuming (passes through)

### 1.5 Essential Dispatchers

**Window Management:**
- `exec [command]` - Execute shell command
- `killactive` - Close active window
- `togglefloating` - Toggle float/tiled
- `fullscreen [0/1/2]` - Fullscreen modes
- `pin` - Pin window to all workspaces
- `pseudo` - Toggle pseudotiling
- `movefocus [direction]` - Move focus (l/r/u/d)
- `movewindow [direction]` - Move window
- `swapwindow [direction]` - Swap with adjacent
- `centerwindow` - Center floating window
- `resizeactive [x y]` - Resize active window

**Workspace Operations:**
- `workspace [id]` - Switch to workspace
- `movetoworkspace [id]` - Move window to workspace
- `movetoworkspacesilent [id]` - Move without switching
- `togglespecialworkspace` - Toggle scratchpad
- `movecurrentworkspacetomonitor [monitor]` - Move workspace

**System Control:**
- `exit` - Exit Hyprland
- `dpms [on/off/toggle]` - Monitor power
- `exec [cmd]` - Execute command

### 1.6 Window Rules

**Syntax:**
```bash
windowrule = RULE, WINDOW_MATCH
windowrulev2 = RULE, MATCH_CRITERIA
```

**Common Rules:**
- `float` - Make window floating
- `tile` - Force tiled mode
- `opacity [active] [inactive]` - Set opacity
- `size [width] [height]` - Set window size
- `move [x] [y]` - Position window
- `center` - Center on screen
- `workspace [id]` - Assign to workspace
- `fullscreen` - Start fullscreen
- `pin` - Pin to all workspaces
- `noborder` - Remove borders
- `noshadow` - Remove shadow

**Matching Criteria:**
- `class:^(firefox)$` - By window class
- `title:^(.*Firefox.*)$` - By title (regex)
- `floating:1` - Match floating windows
- `fullscreen:1` - Match fullscreen

**Examples:**
```bash
windowrulev2 = float, class:^(pavucontrol)$
windowrulev2 = opacity 0.9 0.9, class:^(kitty)$
windowrulev2 = workspace 2, class:^(firefox)$
windowrulev2 = size 800 600, title:^(Calculator)$
windowrulev2 = center, floating:1
```

### 1.7 Animation Configuration

**Syntax:**
```bash
animation = NAME, ONOFF, SPEED, CURVE [,STYLE]
```

**Parameters:**
- **ONOFF:** `0` (disabled) or `1` (enabled)
- **SPEED:** Animation duration multiplier
- **CURVE:** Bezier curve name or "default"
- **STYLE:** Optional style parameters

**Available Animations:**
- `windows` - Window open/close
- `windowsIn` - Window appears
- `windowsOut` - Window disappears
- `windowsMove` - Window movement
- `fade` - Fade effects
- `fadeIn` - Fade in
- `fadeOut` - Fade out
- `border` - Border color transitions
- `workspaces` - Workspace switching
- `layers` - Layer surfaces (bars, notifications)

**Bezier Curves:**
```bash
bezier = NAME, X0, Y0, X1, Y1
```

**Tools for designing curves:**
- cssportal.com - Cubic Bezier designer
- easings.net - Pre-made easing functions

**Example Configurations:**

**Minimal (No Animations):**
```bash
animations {
    enabled = no
}
```

**Default (Smooth):**
```bash
animations {
    enabled = yes
    bezier = easeOutCubic, 0.33, 1, 0.68, 1
    
    animation = windows, 1, 5, easeOutCubic
    animation = windowsOut, 1, 5, default, popin 80%
    animation = fade, 1, 5, default
    animation = workspaces, 1, 6, default
}
```

**Signature (Elaborate):**
```bash
animations {
    enabled = yes
    bezier = overshot, 0.05, 0.9, 0.1, 1.05
    bezier = smoothOut, 0.36, 0, 0.66, -0.56
    bezier = smoothIn, 0.25, 1, 0.5, 1
    
    animation = windows, 1, 8, overshot, slide
    animation = windowsOut, 1, 8, smoothOut, slide
    animation = windowsMove, 1, 6, default
    animation = border, 1, 10, default
    animation = fade, 1, 10, smoothIn
    animation = fadeDim, 1, 10, smoothIn
    animation = workspaces, 1, 8, overshot, slidevert
}
```

---

## 2. OFFICIAL HYPR-ECOSYSTEM TOOLS

### 2.1 hyprpaper - Wallpaper Daemon

**Purpose:** Blazing fast Wayland wallpaper utility with IPC controls

**Package:** `hyprpaper`

**Config Location:** `~/.config/hypr/hyprpaper.conf` (optional)

**Configuration:**
```bash
# Preload images into memory (MUST use absolute paths)
preload = /home/user/wallpapers/image.png

# Set wallpaper for monitor
wallpaper = DP-1, /home/user/wallpapers/image.png

# Set for all monitors
wallpaper = , /home/user/wallpapers/image.png

# Splash screen over wallpaper
splash = true
splash_offset = 2.0
```

**Modes:**
- `cover` (default) - Scale to fill
- `contain:` prefix - Fit within bounds
- `tile:` prefix - Tile the image

**IPC Control:**
```bash
hyprctl hyprpaper preload /path/to/image.png
hyprctl hyprpaper wallpaper "DP-1,/path/to/image.png"
hyprctl hyprpaper listloaded
hyprctl hyprpaper listactive
```

**Startup:**
```bash
exec-once = hyprpaper
```

### 2.2 hypridle - Idle Management

**Purpose:** Hyprland's idle daemon for triggering actions on inactivity

**Package:** `hypridle`

**Config Location:** `~/.config/hypr/hypridle.conf` (REQUIRED)

**Configuration:**
```bash
general {
    lock_cmd = pidof hyprlock || hyprlock       # Lock command
    unlock_cmd = notify-send "Unlocked"         # Unlock command
    before_sleep_cmd = loginctl lock-session    # Before sleep
    after_sleep_cmd = hyprctl dispatch dpms on  # After sleep
    ignore_dbus_inhibit = false                 # Respect firefox/steam inhibits
    ignore_systemd_inhibit = false              # Respect systemd inhibits
}

# Dim screen after 5 minutes
listener {
    timeout = 300
    on-timeout = brightnessctl -s set 10%
    on-resume = brightnessctl -r
}

# Lock screen after 10 minutes
listener {
    timeout = 600
    on-timeout = loginctl lock-session
}

# Turn off screen after 15 minutes
listener {
    timeout = 900
    on-timeout = hyprctl dispatch dpms off
    on-resume = hyprctl dispatch dpms on
}

# Suspend after 30 minutes
listener {
    timeout = 1800
    on-timeout = systemctl suspend
}
```

**Startup:**
```bash
exec-once = hypridle
```

### 2.3 hyprlock - Lock Screen

**Purpose:** Hyprland's GPU-accelerated screen locking utility

**Package:** `hyprlock`

**Config Location:** `~/.config/hypr/hyprlock.conf` (REQUIRED)

**Widget-Based Configuration:**

**Background Widget:**
```bash
background {
    monitor =
    path = /home/user/Pictures/lockscreen.png
    blur_passes = 3
    blur_size = 8
}
```

**Input Field Widget:**
```bash
input-field {
    monitor =
    size = 200, 50
    position = 0, -80
    dots_center = true
    fade_on_empty = true
    font_color = rgb(202, 211, 245)
    inner_color = rgb(91, 96, 120)
    outer_color = rgb(24, 25, 38)
    outline_thickness = 5
    placeholder_text = <span foreground="##cad3f5">Password...</span>
    shadow_passes = 2
}
```

**Label Widget:**
```bash
label {
    monitor =
    text = cmd[update:1000] echo "$(date +"%-I:%M%p")"
    color = rgba(200, 200, 200, 1.0)
    font_size = 55
    font_family = Fira Semibold
    position = 0, 80
    halign = center
    valign = center
}
```

**Dynamic Variables:**
- `$TIME` - Current time (24h)
- `$TIME12` - Current time (12h)
- Uses `TZ` environment variable for timezone

**Signals:**
- `SIGUSR1` - Unlock hyprlock
- `SIGUSR2` - Update labels/images

**Keybinding:**
```bash
bind = SUPER, L, exec, hyprlock
```

**Integration with hypridle:**
```bash
general {
    lock_cmd = pidof hyprlock || hyprlock
}
```

### 2.4 hyprpicker - Color Picker

**Purpose:** Wayland color picker that doesn't suck

**Package:** `hyprpicker`

**No Configuration Required** - Launch with flags

**Usage:**
```bash
# Pick color and copy to clipboard
hyprpicker -a

# Pick color (hex output)
hyprpicker

# Pick color (RGB output)
hyprpicker -f rgb

# Zoom mode for precision
hyprpicker -z
```

**Keybinding:**
```bash
bind = SUPER, P, exec, hyprpicker -a
```

### 2.5 hyprpolkitagent - Polkit Authentication

**Purpose:** Qt/QML-based polkit authentication agent for GUI privilege escalation

**Package:** `hyprpolkitagent`

**No Configuration Required**

**Startup:**
```bash
exec-once = hyprpolkitagent
```

**Alternatives if not available:**
- `lxqt-policykit` - `exec-once = lxqt-policykit-agent`
- GNOME Polkit - Robust and compatible (avoid polkit-gnome, deprecated)

### 2.6 xdg-desktop-portal-hyprland

**Purpose:** XDG Desktop Portal backend for Hyprland (screensharing, file pickers)

**Packages:**
- `xdg-desktop-portal-hyprland` - Hyprland-specific portal
- `xdg-desktop-portal-gtk` - GTK portal (for file picker fallback)

**Installation:**
```bash
pacman -S xdg-desktop-portal-hyprland xdg-desktop-portal-gtk
```

**Auto-starts via D-Bus** - No manual configuration needed

**Testing:**
- Try screensharing in OBS or browser
- Qt menu should appear asking what to share

### 2.7 Other Official Tools

**hyprpm** - Plugin manager
```bash
hyprpm add <plugin-repo>
hyprpm enable <plugin>
```

**hyprsysteminfo** - System information utility
```bash
hyprsysteminfo
```

**hyprsunset** - Blue light filter
```bash
exec-once = hyprsunset
```

**hyprland-qt-support** - Qt Wayland support improvements

---

## 3. ESSENTIAL WAYLAND COMPOSITOR TOOLS

### 3.1 Status Bars

#### Waybar (RECOMMENDED)
**Purpose:** GTK status bar purpose-built for wlroots compositors

**Package:** `waybar`

**Config Location:** `~/.config/waybar/config` (JSONC)

**Style Location:** `~/.config/waybar/style.css`

**Setup:**
```bash
# Copy system defaults as starting point
mkdir -p ~/.config/waybar
cp /etc/xdg/waybar/config ~/.config/waybar/
cp /etc/xdg/waybar/style.css ~/.config/waybar/

# Replace Sway modules with Hyprland modules
sed -i 's/sway\/workspaces/hyprland\/workspaces/g' ~/.config/waybar/config
sed -i 's/sway\/mode/hyprland\/submap/g' ~/.config/waybar/config
```

**Key Modules:**
- `hyprland/workspaces` - Workspace switcher
- `hyprland/window` - Active window title
- `hyprland/submap` - Submap indicator
- `clock` - Date/time
- `battery` - Battery status
- `network` - Network info
- `pulseaudio` - Volume control
- `tray` - System tray

**Startup:**
```bash
exec-once = waybar
```

**Restart on reload:**
```bash
bind = SUPER_SHIFT, R, exec, pkill waybar; waybar &
```

**Complexity:** Low - Minimal coding required

**Resource Usage:** Low (~20-30MB RAM)

#### AGS (Aylur's GTK Shell)
**Purpose:** Framework for dynamic GTK windows using JavaScript

**Package:** `ags` (v2)

**Config Location:** `~/.config/ags/`

**Features:**
- Built-in Hyprland IPC service
- Notification handling
- Battery, network, audio services
- TypeScript/JavaScript configuration
- Highly customizable widgets

**Complexity:** High - Requires JavaScript/TypeScript knowledge

**Resource Usage:** Medium (~50-100MB RAM)

**Documentation:** aylur.github.io/ags

#### eww (ElKowars Wacky Widgets)
**Purpose:** Standalone widget system for X11 and Wayland

**Package:** `eww`

**Config Location:** `~/.config/eww/`

**Configuration:** Yuck DSL + CSS styling

**Features:**
- Custom widgets and bars
- Scripting support
- X11 and Wayland support
- System monitoring widgets

**Limitations:** Not compatible with GNOME on Wayland

**Complexity:** Medium-High - Custom DSL

**Resource Usage:** Low-Medium (~30-60MB RAM)

**Documentation:** elkowar.github.io/eww

#### Other Options
- **yambar** - Lightweight, C-based
- **lemonbar** - Minimalist script-driven bar

### 3.2 Application Launchers

#### Hyprlauncher (OFFICIAL)
**Purpose:** First-party launcher for Hyprland

**Package:** `hyprlauncher`

**Complexity:** Low

**Integration:** Native Hyprland support

**Status:** Official tool, recommended

#### Rofi (Wayland)
**Purpose:** Feature-rich launcher with Wayland support (v2.0+)

**Package:** `rofi-wayland` or `rofi` (v2.0+)

**Basic Usage:**
```bash
bind = SUPER, D, exec, rofi -show drun
bind = SUPER, R, exec, rofi -show run
bind = SUPER_SHIFT, Tab, exec, rofi -show window
```

**Complexity:** Low-Medium

**Features:** App launcher, window switcher, custom modes

#### Wofi
**Purpose:** GTK-based Wayland launcher

**Package:** `wofi`

**Basic Usage:**
```bash
bind = SUPER, D, exec, wofi --show drun
```

**Complexity:** Low

**Customization:** GTK themes and CSS

**Status:** Maintenance unclear (has "not actively maintained" banner but still receives updates)

#### Fuzzel
**Purpose:** Lightweight dmenu/rofi replacement for wlroots

**Package:** `fuzzel`

**Basic Usage:**
```bash
bind = SUPER, D, exec, fuzzel
```

**Complexity:** Low

**Features:** dmenu/rofi compatible, good documentation

**Performance:** Fast and lightweight

#### Tofi
**Purpose:** Extremely fast and simple dmenu/rofi replacement

**Package:** `tofi`

**Performance:** Can render within a single frame when configured correctly

**Limitations:** Missing some dmenu/rofi features (e.g., -p prompt flag as of mid-2024)

**Complexity:** Low

#### Bemenu
**Purpose:** Wayland-native dmenu replacement

**Package:** `bemenu`

**Complexity:** Low

**Resource Usage:** Minimal

#### Comparison Matrix

| Launcher | Speed | Features | Complexity | Wayland Native | Recommendation |
|----------|-------|----------|------------|----------------|----------------|
| Hyprlauncher | Fast | Medium | Low | Yes | Best for Hyprland |
| Rofi | Medium | High | Medium | Yes (v2.0+) | Power users |
| Wofi | Medium | Medium | Low | Yes | GTK users |
| Fuzzel | Fast | Medium | Low | Yes | Balanced choice |
| Tofi | Very Fast | Low | Low | Yes | Speed priority |
| Bemenu | Fast | Low | Low | Yes | Minimalists |

### 3.3 Notification Daemons

#### Mako (RECOMMENDED FOR MINIMAL/DEFAULT)
**Purpose:** Lightweight notification daemon for Wayland

**Package:** `mako`

**Config Location:** `~/.config/mako/config`

**Features:**
- Auto-starts on first notification
- Lightweight and simple
- Hyprland/Sway/river compatible

**Configuration:**
```ini
font=Monospace 10
background-color=#2e3440
text-color=#eceff4
border-color=#88c0d0
border-size=2
border-radius=10
default-timeout=5000
max-visible=5
```

**Startup:** Auto-starts (or explicit):
```bash
exec-once = mako
```

**Resource Usage:** Very Low (~5-10MB RAM)

**Complexity:** Low

#### SwayNC (Notification Center)
**Purpose:** Notification center with history and control

**Package:** `swaync`

**Features:**
- Auto-hides notifications after timeout
- Accessible notification center
- Custom Waybar module support
- Better than Mako for power users

**Complexity:** Medium

**Resource Usage:** Medium (~20-40MB RAM)

**Startup:**
```bash
exec-once = swaync
```

#### Dunst
**Purpose:** Minimalist notification daemon (X11/Wayland)

**Package:** `dunst`

**Features:**
- Wayland support since v1.6
- Most popular notification daemon
- Highly configurable

**Config Location:** `~/.config/dunst/dunstrc`

**Complexity:** Low-Medium

**Resource Usage:** Low (~10-20MB RAM)

**Note:** Mako borrows config format from Dunst, making migration easy

#### Comparison

| Daemon | Auto-start | History | Wayland Native | Resource | Recommendation |
|--------|-----------|---------|----------------|----------|----------------|
| Mako | Yes | No | Yes | Very Low | Minimal/Default |
| SwayNC | No | Yes | Yes | Medium | Power users |
| Dunst | No | Limited | Yes (1.6+) | Low | X11 migrants |

### 3.4 Screenshot Tools

#### Core Tools: Grim + Slurp
**Packages:** `grim`, `slurp`

**Purpose:**
- `grim` - Screenshot utility for Wayland
- `slurp` - Region selector for Wayland

**Basic Usage:**
```bash
# Region screenshot to clipboard
grim -g "$(slurp)" - | wl-copy

# Full screenshot to file
grim ~/Pictures/screenshot-$(date +%Y%m%d-%H%M%S).png

# Region screenshot to file
grim -g "$(slurp)" ~/Pictures/region-$(date +%Y%m%d-%H%M%S).png
```

#### Grimblast (OFFICIAL HYPRLAND TOOL)
**Package:** `grimblast` (from hyprwm/contrib)

**Purpose:** Convenient wrapper around grim/slurp with Hyprland integration

**Dependencies:**
- `grim`
- `slurp`
- `wl-clipboard` (wl-copy)
- `jq`
- `hyprpicker` (to freeze screen)
- `libnotify` (notify-send)

**Installation:**
```bash
pacman -S grim slurp wl-clipboard jq hyprpicker libnotify
yay -S grimblast
```

**Targets:**
- `area` - Select region or window
- `screen` - Current screen
- `output` - Specific output
- `active` - Active window

**Actions:**
- `copy` - To clipboard
- `save` - To file
- `copysave` - Both

**Usage:**
```bash
# Region to clipboard
grimblast copy area

# Active window to file
grimblast save active

# Full screen to clipboard and file
grimblast copysave screen
```

**Keybindings:**
```bash
# Region screenshot to clipboard
bind = SUPER_SHIFT, S, exec, grimblast copy area

# Full screenshot to clipboard
bind = SUPER_SHIFT, P, exec, grimblast copy screen

# Active window to file
bind = SUPER_SHIFT, A, exec, grimblast save active ~/Pictures/Screenshots/
```

#### Hyprshot
**Package:** `hyprshot`

**Purpose:** Hyprland-specific screenshot utility using mouse

**Usage:**
```bash
hyprshot -m region
hyprshot -m window
hyprshot -m output
```

**Keybindings:**
```bash
bind = SUPER, Print, exec, hyprshot -m region
```

### 3.5 Clipboard Managers

#### Core: wl-clipboard
**Package:** `wl-clipboard`

**Commands:**
- `wl-copy` - Copy to clipboard
- `wl-paste` - Paste from clipboard

**Usage:**
```bash
# Copy text
echo "hello" | wl-copy

# Copy image
wl-copy < image.png

# Paste
wl-paste > output.txt
```

#### cliphist (RECOMMENDED)
**Package:** `cliphist`

**Purpose:** Clipboard history manager with multimedia support

**Dependencies:**
- `wl-clipboard`
- `xdg-utils` (MIME detection)

**Setup:**
```bash
# Text history
exec-once = wl-paste --type text --watch cliphist store

# Image history
exec-once = wl-paste --type image --watch cliphist store
```

**Usage with Rofi:**
```bash
bind = SUPER, V, exec, cliphist list | rofi -dmenu | cliphist decode | wl-copy
```

**Usage with Wofi:**
```bash
bind = SUPER, V, exec, cliphist list | wofi -dmenu | cliphist decode | wl-copy
```

**Usage with Fuzzel:**
```bash
bind = SUPER, V, exec, cliphist list | fuzzel --dmenu | cliphist decode | wl-copy
```

#### Alternatives
- **clipman** - Text only, Wayland support
- **clipvault** - Like cliphist with extras (max age, min/max length)
- **clipse** - Single binary, TUI, text/images, themes, image preview

### 3.6 Terminal Emulators

#### Kitty (RECOMMENDED)
**Package:** `kitty`

**Features:**
- Fast, GPU-accelerated
- Ligatures, emojis, hyperlinks
- Tabs and layouts
- SSH file transfer
- Graphics protocol

**Wayland Support:** Yes (via glfw backend)

**Configuration:** Plain text (`~/.config/kitty/kitty.conf`)

**Resource Usage:** Low-Medium

**Performance:** Fastest Wayland terminal

**Keybinding:**
```bash
bind = SUPER, Q, exec, kitty
```

#### Alacritty
**Package:** `alacritty`

**Features:**
- Written in Rust
- Focus on speed and simplicity
- GPU-accelerated
- Minimal features (no tabs, no ligatures)

**Wayland Support:** Yes

**Configuration:** TOML or YAML (`~/.config/alacritty/alacritty.toml`)

**Resource Usage:** Very Low

**Performance:** Fastest with Kitty

#### Foot
**Package:** `foot`

**Features:**
- Wayland-native
- Extremely lightweight (13k LOC C)
- Fast and simple

**Wayland Support:** Native

**Configuration:** INI format (`~/.config/foot/foot.ini`)

**Resource Usage:** Very Low (lowest of all)

**Performance:** Very fast

**Note:** Default terminal in Arch Linux's Sway installation

#### WezTerm
**Package:** `wezterm`

**Features:**
- Incredibly feature-rich
- 3 image protocols
- Asciinema recording
- Hundreds of themes
- Lua configuration

**Wayland Support:** Yes

**Configuration:** Lua (`~/.config/wezterm/wezterm.lua`)

**Resource Usage:** Medium-High

**Performance:** Noticeably less performant than Alacritty/Kitty on some systems

**Complexity:** Higher (Lua config)

#### Comparison

| Terminal | Speed | Features | Config | Resource | GPU | Ligatures | Recommendation |
|----------|-------|----------|--------|----------|-----|-----------|----------------|
| Kitty | Fastest | High | Plain | Low-Med | Yes | Yes | Best balanced |
| Alacritty | Fastest | Low | TOML | Very Low | Yes | No | Minimalists |
| Foot | Very Fast | Medium | INI | Very Low | No | Yes | Ultra-minimal |
| WezTerm | Medium | Very High | Lua | Medium | Yes | Yes | Power users |

### 3.7 File Managers

#### Thunar (XFCE) - RECOMMENDED FOR MINIMAL
**Package:** `thunar`

**Desktop:** XFCE

**Wayland Support:** Yes (via XWayland)

**Features:**
- Simple, intuitive GUI
- Custom actions
- Lightweight
- Fast

**Resource Usage:** Low

**Complexity:** Low

**Community Ranking:** #2 for UNIX-like systems

#### Dolphin (KDE)
**Package:** `dolphin`

**Desktop:** KDE

**Wayland Support:** Native

**Features:**
- Split view (dual pane)
- Integrated terminal
- Service menus
- Tabs
- Advanced features

**Resource Usage:** Medium (KDE dependencies)

**Complexity:** Medium

**Recommendation:** Best for feature-rich needs

#### Nautilus (GNOME Files)
**Package:** `nautilus`

**Desktop:** GNOME

**Wayland Support:** Native (no XWayland)

**Features:**
- Fast, stable, clean
- Tabs
- Remote directory access
- GNOME integration

**Resource Usage:** Medium (GNOME dependencies)

**Complexity:** Low

#### Nemo (Cinnamon)
**Package:** `nemo`

**Desktop:** Cinnamon (fork of Nautilus)

**Wayland Support:** Via XWayland

**Features:**
- Third-party plugin support
- More customizable than Nautilus
- Familiar GNOME-like interface

**Resource Usage:** Medium

**Complexity:** Low-Medium

#### Comparison

| File Manager | DE | Wayland | Features | Resources | Dependencies | Recommendation |
|--------------|-----|---------|----------|-----------|--------------|----------------|
| Thunar | XFCE | XWayland | Medium | Low | Minimal | Minimal/Default |
| Dolphin | KDE | Native | High | Medium | Qt/KDE | Signature/Power |
| Nautilus | GNOME | Native | Medium | Medium | GTK/GNOME | GTK users |
| Nemo | Cinnamon | XWayland | Medium | Medium | GTK | Nautilus + |

---

## 4. ADVANCED FRAMEWORKS

### 4.1 AGS (Aylur's GTK Shell)

**Purpose:** Framework to create dynamic GTK windows using JavaScript

**Complexity:** High

**Resource Usage:** Medium (~50-100MB RAM)

**Configuration:** JavaScript/TypeScript

**Documentation:** aylur.github.io/ags

**Features:**
- Built-in services:
  - Hyprland IPC
  - Notifications
  - Battery monitoring
  - Network info
  - Audio control
  - MPRIS media control
- Massively expandable
- Works with any Wayland compositor
- Custom widgets, bars, launchers, etc.

**Use Cases:**
- Custom status bars
- Notification centers
- Control centers
- System widgets
- Complete desktop shells

**Version Note:** v2 is current (2025), many community configs use v1

**Learning Curve:** Steep - Requires JS/TS knowledge

**Recommendation:** Signature tier for users comfortable with scripting

### 4.2 eww (ElKowars Wacky Widgets)

**Purpose:** Standalone widget system for creating custom UI elements

**Complexity:** Medium-High

**Resource Usage:** Low-Medium (~30-60MB RAM)

**Configuration:** Yuck DSL + CSS

**Documentation:** elkowar.github.io/eww

**Features:**
- Custom widgets in any location
- System monitoring
- X11 and Wayland support
- Scriptable

**Limitations:**
- GNOME on Wayland incompatible (lacks necessary protocols)

**Use Cases:**
- Status bars
- System monitors
- Custom UI elements
- Desktop widgets

**Learning Curve:** Medium - Custom DSL (Yuck) + CSS

**Recommendation:** Signature tier for widget enthusiasts

### 4.3 Fabric

**Purpose:** Next-gen Python framework for desktop widgets

**Complexity:** Medium

**Resource Usage:** Medium

**Configuration:** Python

**Platform:** X11 and Wayland

**Backend:** GTK

**Features:**
- GTK-based widget creation
- Python scripting
- System widgets

**Use Cases:**
- Custom widgets
- System monitors
- Desktop enhancements

**Learning Curve:** Medium - Requires Python knowledge

**Recommendation:** Signature tier for Python developers

### 4.4 Quickshell

**Purpose:** Flexible toolkit for building desktop shells with QtQuick

**Complexity:** High

**Resource Usage:** Medium (~40-80MB RAM)

**Configuration:** QML

**Documentation:** quickshell.outfoxxed.me

**Features:**
- QtQuick/QML-based
- Wayland layer-shell integration
- Multiple layers: background, bottom, top, overlay
- Edge anchoring
- LSP support for QML
- Linux and BSD support
- X11 and Wayland (Wayland preferred)

**Use Cases:**
- Status bars
- Widgets
- Lockscreens
- Complete desktop shells

**Version:** 0.2.1 (2025)

**Learning Curve:** High - Requires QML knowledge

**Recommendation:** Signature tier for Qt/QML developers

### 4.5 Framework Comparison Matrix

| Framework | Language | Backend | Complexity | Resources | Platform | Best For |
|-----------|----------|---------|------------|-----------|----------|----------|
| AGS | JS/TS | GTK | High | Medium | Wayland/X11 | Feature-rich shells |
| eww | Yuck DSL | GTK | Medium-High | Low-Med | Wayland/X11 | Custom widgets |
| Fabric | Python | GTK | Medium | Medium | Wayland/X11 | Python devs |
| Quickshell | QML | Qt | High | Medium | Wayland/X11 | Qt ecosystem |

**Tier Recommendations:**
- **Minimal:** None (use simple tools)
- **Default:** None (Waybar sufficient)
- **Signature:** Any one framework based on user preference/skills

---

## 5. NVIDIA CONFIGURATION

### 5.1 Important Warnings

**Official Stance:** There is NO official Hyprland support for NVIDIA hardware.

**User Success:** Many users report success following these guidelines.

**Requirement:** Read ALL instructions completely before seeking help.

**Critical:** NVIDIA users MUST follow NVIDIA-specific configuration BEFORE launching Hyprland. Failure to do so will result in many bugs.

### 5.2 Driver Options

**Three Driver Configurations:**

1. **Proprietary Drivers** - Fully closed-source NVIDIA drivers
   - Full feature set
   - Best performance
   - Recommended for newer GPUs

2. **Open Drivers** - Proprietary with open-source kernel modules
   - Good performance
   - Newer approach
   - Recommended for newer GPUs

3. **Nouveau** - Community open-source implementation
   - Limited features
   - Lower performance
   - Not recommended for Hyprland

**Recommendation:** Use proprietary or open drivers for newer GPUs due to vital optimizations and power management support.

### 5.3 Required Environment Variables

**Add to `~/.config/hypr/hyprland.conf`:**

```bash
# NVIDIA environment variables
env = LIBVA_DRIVER_NAME,nvidia
env = __GLX_VENDOR_LIBRARY_NAME,nvidia
env = GBM_BACKEND,nvidia-drm
env = __GL_GSYNC_ALLOWED,1
env = __GL_VRR_ALLOWED,0
```

**Important Warnings:**

**LIBVA_DRIVER_NAME:**
- Setting to `nvidia` on Intel or hybrid setups WILL BREAK video acceleration
- Only use on NVIDIA-only systems
- Consult Arch Wiki Hardware Acceleration page before setting

**GBM_BACKEND:**
- May cause Firefox crashes
- Remove if experiencing Firefox issues

### 5.4 Hardware Acceleration Setup

**Package:** `libva-nvidia-driver` (official Arch repos)

**Purpose:** Enable hardware video acceleration on NVIDIA + Wayland

**Installation:**
```bash
pacman -S libva-nvidia-driver
```

**Testing:**
```bash
vainfo
# Should show NVIDIA as VA-API driver
```

### 5.5 Kernel Parameters

**For Proprietary NVIDIA Drivers:**

Add to bootloader (GRUB, systemd-boot, etc.):
```bash
nvidia_drm.modeset=1
```

**For GRUB (`/etc/default/grub`):**
```bash
GRUB_CMDLINE_LINUX_DEFAULT="... nvidia_drm.modeset=1"
```

Then regenerate config:
```bash
grub-mkconfig -o /boot/grub/grub.cfg
```

### 5.6 Configuration Recommendations

**Backend:**
```bash
env = AQ_DRM_DEVICES,/dev/dri/card1:/dev/dri/card0
```

**Cursor Fix:**
```bash
env = WLR_NO_HARDWARE_CURSORS,1
```

**Performance:**
```bash
env = __GL_THREADED_OPTIMIZATION,1
```

### 5.7 Common Issues & Workarounds

**Issue: Black screen on launch**
- Ensure `nvidia_drm.modeset=1` kernel parameter
- Verify drivers installed: `nvidia`, `nvidia-utils`, `nvidia-settings`

**Issue: Flickering or artifacts**
- Add `env = WLR_NO_HARDWARE_CURSORS,1`
- Try disabling VRR: `env = __GL_VRR_ALLOWED,0`

**Issue: Poor performance**
- Enable threaded optimization: `env = __GL_THREADED_OPTIMIZATION,1`
- Check GPU isn't throttling: `nvidia-smi`

**Issue: Firefox crashes**
- Remove `env = GBM_BACKEND,nvidia-drm`
- Use software rendering in Firefox as fallback

**Issue: Video acceleration not working**
- Install `libva-nvidia-driver`
- Verify with `vainfo`
- Check `LIBVA_DRIVER_NAME` is set correctly

### 5.8 Multi-GPU (Hybrid) Setup

**For Laptops with Intel + NVIDIA:**

**Primary GPU selection:**
```bash
env = AQ_DRM_DEVICES,/dev/dri/card1:/dev/dri/card0
```

**Prime offload for specific apps:**
```bash
# Run app on NVIDIA
__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia application
```

**Keybinding for NVIDIA launch:**
```bash
bind = SUPER_SHIFT, N, exec, __NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia kitty
```

### 5.9 NVIDIA Configuration Checklist

- [ ] Install NVIDIA proprietary drivers
- [ ] Add `nvidia_drm.modeset=1` kernel parameter
- [ ] Add all required environment variables to hyprland.conf
- [ ] Install `libva-nvidia-driver`
- [ ] Test with `vainfo`
- [ ] Add `WLR_NO_HARDWARE_CURSORS,1` if cursor issues
- [ ] Remove `GBM_BACKEND` if Firefox crashes
- [ ] Configure multi-GPU if hybrid system

---

## 6. CONFIGURATION FILE LOCATIONS

### 6.1 Core Hyprland Files

| Component | Config Location | Required |
|-----------|----------------|----------|
| Hyprland | `~/.config/hypr/hyprland.conf` | Yes |
| Hyprpaper | `~/.config/hypr/hyprpaper.conf` | No |
| Hypridle | `~/.config/hypr/hypridle.conf` | Yes (if using) |
| Hyprlock | `~/.config/hypr/hyprlock.conf` | Yes (if using) |

### 6.2 Essential Tools

| Tool | Config Location | Format | Required |
|------|----------------|--------|----------|
| Waybar | `~/.config/waybar/config` | JSONC | No (defaults exist) |
| Waybar Style | `~/.config/waybar/style.css` | CSS | No |
| Mako | `~/.config/mako/config` | INI | No |
| Dunst | `~/.config/dunst/dunstrc` | INI | No |
| Rofi | `~/.config/rofi/config.rasi` | Rasi | No |
| Wofi | `~/.config/wofi/` | CSS/conf | No |
| Kitty | `~/.config/kitty/kitty.conf` | Plain | No |
| Alacritty | `~/.config/alacritty/alacritty.toml` | TOML | No |
| Foot | `~/.config/foot/foot.ini` | INI | No |

### 6.3 Advanced Frameworks

| Framework | Config Location | Format |
|-----------|----------------|--------|
| AGS | `~/.config/ags/` | JS/TS |
| eww | `~/.config/eww/` | Yuck + CSS |
| Fabric | `~/.config/fabric/` | Python |
| Quickshell | `~/.config/quickshell/` | QML |

### 6.4 System Locations

| Purpose | Location | Notes |
|---------|----------|-------|
| Session startup | `/usr/share/wayland-sessions/hyprland.desktop` | Desktop entry |
| System configs | `/etc/xdg/hypr/` | System-wide defaults |
| Waybar defaults | `/etc/xdg/waybar/` | Copy for customization |

### 6.5 UWSM Integration

**For UWSM users:** Avoid placing environment variables in `hyprland.conf`

**Use instead:**
- `~/.config/uwsm/env` - Theming, xcursor, NVIDIA, toolkit vars
- `~/.config/uwsm/env-hyprland` - HYPR* and AQ_* vars

**Format:** `export KEY=VAL`

---

## 7. TOOL MATRIX BY TIER

### 7.1 Minimal Tier (Bare Essentials)

**Philosophy:** Functional Hyprland with minimal features, low resource usage

**Core Hyprland:**
- `hyprland` - Compositor
- `xdg-desktop-portal-hyprland` - Portal backend
- `xdg-desktop-portal-gtk` - File picker fallback
- `polkit` or `hyprpolkitagent` - Authentication

**Essential Tools:**
- `kitty` or `foot` - Terminal
- `thunar` - File manager
- `fuzzel` or `tofi` - Launcher
- `mako` - Notifications
- `grim` + `slurp` - Screenshots
- `wl-clipboard` - Clipboard
- `brightnessctl` - Brightness (laptops)
- `wireplumber` - Audio

**Wallpaper:** `swaybg` (simpler than hyprpaper)

**Lock Screen:** `swaylock`

**Status Bar:** None (use `hyprctl clients` / rofi for info)

**Total Packages:** ~15

**RAM Usage:** ~150-250MB (Hyprland + minimal tools)

**Config Complexity:** Low - Single hyprland.conf

### 7.2 Default Tier (Recommended)

**Philosophy:** Complete, polished Hyprland experience with reasonable resource usage

**Core Hyprland:**
- All from Minimal tier
- `hyprpaper` - Official wallpaper daemon
- `hypridle` - Idle management
- `hyprlock` - Official lockscreen
- `hyprpicker` - Color picker

**Essential Tools:**
- `kitty` - Terminal (with ligatures)
- `thunar` - File manager
- `fuzzel` or `rofi-wayland` - Launcher
- `mako` or `swaync` - Notifications
- `grimblast` - Enhanced screenshots
- `cliphist` - Clipboard history
- `brightnessctl` - Brightness
- `wireplumber` + `pamixer` - Audio
- `playerctl` - Media controls

**Status Bar:** `waybar` - Highly customizable

**File Manager:** `thunar` or `nautilus`

**Polkit:** `hyprpolkitagent`

**Total Packages:** ~25-30

**RAM Usage:** ~300-500MB

**Config Files:**
- `hyprland.conf`
- `hyprpaper.conf`
- `hypridle.conf`
- `hyprlock.conf`
- `waybar/config`
- `waybar/style.css`
- `mako/config` or `swaync/`

**Config Complexity:** Medium

### 7.3 Signature Tier (Premium)

**Philosophy:** Showcase configuration with advanced features, custom widgets, elaborate animations

**Core Hyprland:**
- All from Default tier

**Advanced Tools:**
- `wezterm` - Feature-rich terminal
- `dolphin` - Advanced file manager
- `rofi-wayland` - Versatile launcher
- `swaync` - Notification center
- `grimblast` - Screenshots
- `cliphist` - Clipboard history
- All media/system controls

**Widget Framework (Choose One):**
- `ags` - GTK Shell (JavaScript)
- `eww` - Wacky widgets (Yuck DSL)
- `fabric` - Python widgets
- `quickshell` - Qt/QML shell

**Status Bar:**
- Custom AGS bar, OR
- Custom eww bar, OR
- Heavily customized Waybar

**Animations:** Elaborate bezier curves, multiple animations

**Theming:**
- GTK theme coordination
- Qt theme coordination
- Cursor themes
- Icon themes
- Custom Waybar CSS
- Coordinated color schemes

**Additional:**
- `hyprsunset` - Blue light filter
- Custom scripts for dynamic theming
- Workspace-specific wallpapers
- Advanced window rules
- Submaps for complex keybindings

**Total Packages:** ~40-60+ (depends on framework)

**RAM Usage:** ~500MB-1GB+

**Config Files:**
- All from Default tier
- AGS/eww/fabric/quickshell configs (multi-file)
- GTK themes
- Qt themes
- Custom scripts
- Advanced hyprland.conf with submaps

**Config Complexity:** High - Multiple languages/frameworks

### 7.4 Package Lists by Tier

#### Minimal Tier Packages
```bash
# Core
hyprland xdg-desktop-portal-hyprland xdg-desktop-portal-gtk polkit

# Terminal & File Manager
foot thunar

# Launcher & Notifications
tofi mako

# Screenshots & Clipboard
grim slurp wl-clipboard

# System Control
brightnessctl wireplumber

# Lock
swaylock
```

#### Default Tier Packages
```bash
# Minimal tier +
hyprpaper hypridle hyprlock hyprpicker hyprpolkitagent

# Better tools
kitty grimblast cliphist pamixer playerctl

# Status bar
waybar

# Optional: rofi instead of tofi
rofi-wayland
```

#### Signature Tier Packages
```bash
# Default tier +
wezterm dolphin swaync

# Widget framework (choose one)
ags          # AGS
eww          # eww
# fabric     # Python framework
# quickshell # Qt framework

# Theming
gtk-theme-name qt6ct qt5ct
papirus-icon-theme xcursor-themes
hyprsunset

# Advanced
jq bc

# Development (if building custom widgets)
nodejs npm  # For AGS
rustup      # For eww
python      # For fabric
qt6-base    # For quickshell
```

---

## 8. INTEGRATION PATTERNS

### 8.1 Lock Screen + Idle Management

**Pattern:** hypridle triggers hyprlock

**hypridle.conf:**
```bash
general {
    lock_cmd = pidof hyprlock || hyprlock
    before_sleep_cmd = loginctl lock-session
}

listener {
    timeout = 300  # 5 minutes
    on-timeout = brightnessctl -s set 10%
    on-resume = brightnessctl -r
}

listener {
    timeout = 600  # 10 minutes
    on-timeout = loginctl lock-session
}

listener {
    timeout = 900  # 15 minutes
    on-timeout = hyprctl dispatch dpms off
    on-resume = hyprctl dispatch dpms on
}
```

**hyprland.conf:**
```bash
exec-once = hypridle
bind = SUPER, L, exec, loginctl lock-session
```

**Result:** Automatic dimming → locking → screen off → suspend chain

### 8.2 Screenshot + Clipboard + Notifications

**Pattern:** grimblast → wl-copy → notify-send

**hyprland.conf:**
```bash
# Region screenshot to clipboard
bind = SUPER_SHIFT, S, exec, grimblast copy area

# Notify success
bind = SUPER_SHIFT, S, exec, grimblast copy area && notify-send "Screenshot" "Copied to clipboard"

# Save to file with notification
bind = SUPER_SHIFT, A, exec, grimblast save active ~/Pictures/Screenshots/ && notify-send "Screenshot" "Saved to ~/Pictures/Screenshots/"
```

**Requirements:**
- `grimblast`
- `wl-clipboard`
- `mako` or `dunst` (for notify-send)

### 8.3 Media Controls + Waybar Integration

**Pattern:** playerctl → Waybar MPRIS module

**hyprland.conf:**
```bash
bind = , XF86AudioPlay, exec, playerctl play-pause
bind = , XF86AudioNext, exec, playerctl next
bind = , XF86AudioPrev, exec, playerctl previous
```

**waybar/config:**
```json
{
    "modules-left": ["hyprland/workspaces"],
    "modules-center": ["mpris"],
    "modules-right": ["pulseaudio", "clock"],
    
    "mpris": {
        "format": "{player_icon} {title} - {artist}",
        "format-paused": "{status_icon} {title} - {artist}",
        "player-icons": {
            "default": "",
            "spotify": ""
        },
        "status-icons": {
            "paused": ""
        }
    }
}
```

**Result:** Media keys control playback, Waybar shows current track

### 8.4 Wallpaper + Color Scheme Coordination

**Pattern:** hyprpaper → pywal/wpgtk → dynamic theming

**Advanced Integration:**
```bash
# hyprland.conf
exec-once = hyprpaper

# Script to change wallpaper and update colors
# ~/.config/hypr/scripts/wallpaper.sh
#!/bin/bash
WALLPAPER=$1
hyprctl hyprpaper preload "$WALLPAPER"
hyprctl hyprpaper wallpaper ",$WALLPAPER"
wal -i "$WALLPAPER"
# Reload waybar to apply colors
pkill waybar; waybar &
```

**Keybinding:**
```bash
bind = SUPER_SHIFT, W, exec, ~/.config/hypr/scripts/wallpaper.sh ~/Pictures/wallpaper.png
```

### 8.5 Application Launcher + Clipboard Manager

**Pattern:** rofi serves dual purpose

**hyprland.conf:**
```bash
# App launcher
bind = SUPER, D, exec, rofi -show drun

# Clipboard history
bind = SUPER, V, exec, cliphist list | rofi -dmenu | cliphist decode | wl-copy

# Window switcher
bind = SUPER, Tab, exec, rofi -show window
```

**Result:** Single launcher tool for multiple functions

### 8.6 Notification Center + Do Not Disturb

**Pattern:** swaync with DND toggle

**hyprland.conf:**
```bash
exec-once = swaync

# Toggle notification center
bind = SUPER, N, exec, swaync-client -t -sw

# Toggle Do Not Disturb
bind = SUPER_SHIFT, N, exec, swaync-client -d -sw
```

**waybar integration:**
```json
{
    "custom/notification": {
        "exec": "swaync-client -swb",
        "return-type": "json",
        "on-click": "swaync-client -t -sw",
        "on-click-right": "swaync-client -d -sw"
    }
}
```

### 8.7 Polkit Agent + Application Privilege Requests

**Pattern:** hyprpolkitagent auto-handles GUI privilege prompts

**hyprland.conf:**
```bash
exec-once = hyprpolkitagent
```

**Result:** GUI apps (file managers, software centers) can request sudo automatically

### 8.8 Workspace-Specific Wallpapers

**Pattern:** Workspace rules + hyprpaper IPC

**Script:**
```bash
#!/bin/bash
# ~/.config/hypr/scripts/workspace-wallpaper.sh
WORKSPACE=$(hyprctl activeworkspace -j | jq -r '.id')
WALLPAPER="$HOME/Pictures/wallpapers/workspace-$WORKSPACE.png"

if [ -f "$WALLPAPER" ]; then
    hyprctl hyprpaper wallpaper ",$WALLPAPER"
fi
```

**hyprland.conf:**
```bash
exec-once = hyprpaper
bind = SUPER, 1, workspace, 1
bind = SUPER, 1, exec, ~/.config/hypr/scripts/workspace-wallpaper.sh
# Repeat for other workspaces
```

### 8.9 AGS/eww Complete Desktop Shell

**Pattern:** Replace Waybar + launcher + notifications with unified framework

**With AGS:**
```javascript
// ~/.config/ags/config.js
const hyprland = await Service.import('hyprland');
const notifications = await Service.import('notifications');

// Custom bar, launcher, notification center all in one
App.config({
    windows: [
        Bar(),
        Launcher(),
        NotificationCenter()
    ]
});
```

**hyprland.conf:**
```bash
exec-once = ags

bind = SUPER, D, exec, ags -t launcher
bind = SUPER, N, exec, ags -t notification-center
```

**Result:** Unified, scriptable desktop shell with JavaScript

### 8.10 Dynamic Keybinding Submaps

**Pattern:** Mode-specific keybindings

**hyprland.conf:**
```bash
# Enter resize mode
bind = SUPER, R, submap, resize

submap = resize
bind = , h, resizeactive, -10 0
bind = , l, resizeactive, 10 0
bind = , k, resizeactive, 0 -10
bind = , j, resizeactive, 0 10
bind = , escape, submap, reset
submap = reset

# Enter system mode
bind = SUPER, Escape, submap, system

submap = system
bind = , l, exec, loginctl lock-session
bind = , e, exit
bind = , s, exec, systemctl suspend
bind = , r, exec, systemctl reboot
bind = , p, exec, systemctl poweroff
bind = , escape, submap, reset
submap = reset
```

**Result:** Vi-like modal keybindings for complex operations

---

## 9. SMART DEFAULTS & RECOMMENDED KEYBINDINGS

### 9.1 Minimal Tier Keybindings

```bash
# Define modifier
$mainMod = SUPER

# Core applications
bind = $mainMod, Q, exec, foot
bind = $mainMod, E, exec, thunar
bind = $mainMod, D, exec, tofi-drun

# Window management
bind = $mainMod, C, killactive
bind = $mainMod, M, exit
bind = $mainMod, V, togglefloating
bind = $mainMod, F, fullscreen
bind = $mainMod, P, pseudo

# Focus
bind = $mainMod, h, movefocus, l
bind = $mainMod, l, movefocus, r
bind = $mainMod, k, movefocus, u
bind = $mainMod, j, movefocus, d

# Workspaces
bind = $mainMod, 1, workspace, 1
bind = $mainMod, 2, workspace, 2
bind = $mainMod, 3, workspace, 3
bind = $mainMod, 4, workspace, 4
bind = $mainMod, 5, workspace, 5

# Move to workspace
bind = $mainMod SHIFT, 1, movetoworkspace, 1
bind = $mainMod SHIFT, 2, movetoworkspace, 2
bind = $mainMod SHIFT, 3, movetoworkspace, 3
bind = $mainMod SHIFT, 4, movetoworkspace, 4
bind = $mainMod SHIFT, 5, movetoworkspace, 5

# Mouse bindings
bindm = $mainMod, mouse:272, movewindow
bindm = $mainMod, mouse:273, resizewindow

# System
bind = $mainMod, L, exec, swaylock -f

# Screenshots
bind = $mainMod SHIFT, S, exec, grim -g "$(slurp)" - | wl-copy

# Volume
bind = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bind = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bind = , XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle

# Brightness (laptops)
bind = , XF86MonBrightnessUp, exec, brightnessctl set 5%+
bind = , XF86MonBrightnessDown, exec, brightnessctl set 5%-
```

### 9.2 Default Tier Keybindings

```bash
# All from Minimal tier, plus:

# Better terminal
bind = $mainMod, Q, exec, kitty

# Better launcher
bind = $mainMod, D, exec, rofi -show drun
bind = $mainMod, R, exec, rofi -show run

# Clipboard history
bind = $mainMod, V, exec, cliphist list | rofi -dmenu | cliphist decode | wl-copy

# Better screenshots
bind = $mainMod SHIFT, S, exec, grimblast copy area
bind = $mainMod SHIFT, P, exec, grimblast copy screen
bind = $mainMod SHIFT, A, exec, grimblast copy active

# Color picker
bind = $mainMod SHIFT, C, exec, hyprpicker -a

# Media controls
bind = , XF86AudioPlay, exec, playerctl play-pause
bind = , XF86AudioNext, exec, playerctl next
bind = , XF86AudioPrev, exec, playerctl previous

# Lock with loginctl (for hypridle integration)
bind = $mainMod, L, exec, loginctl lock-session

# Reload Waybar
bind = $mainMod SHIFT, R, exec, pkill waybar; waybar &

# Special workspace (scratchpad)
bind = $mainMod, S, togglespecialworkspace, magic
bind = $mainMod SHIFT, S, movetoworkspace, special:magic
```

### 9.3 Signature Tier Keybindings

```bash
# All from Default tier, plus:

# Advanced terminal
bind = $mainMod, Q, exec, wezterm

# Advanced file manager
bind = $mainMod, E, exec, dolphin

# AGS/eww launcher
bind = $mainMod, D, exec, ags -t launcher

# Notification center toggle
bind = $mainMod, N, exec, swaync-client -t -sw
bind = $mainMod SHIFT, N, exec, swaync-client -d -sw

# Submaps for complex operations
bind = $mainMod, R, submap, resize
submap = resize
bind = , h, resizeactive, -20 0
bind = , l, resizeactive, 20 0
bind = , k, resizeactive, 0 -20
bind = , j, resizeactive, 0 20
bind = , escape, submap, reset
submap = reset

bind = $mainMod, Escape, submap, system
submap = system
bind = , l, exec, loginctl lock-session
bind = , e, exit
bind = , s, exec, systemctl suspend
bind = , r, exec, systemctl reboot
bind = , p, exec, systemctl poweroff
bind = , escape, submap, reset
submap = reset

# More workspaces
bind = $mainMod, 6, workspace, 6
bind = $mainMod, 7, workspace, 7
bind = $mainMod, 8, workspace, 8
bind = $mainMod, 9, workspace, 9
bind = $mainMod, 0, workspace, 10

# Move windows to workspaces
bind = $mainMod SHIFT, 6, movetoworkspace, 6
bind = $mainMod SHIFT, 7, movetoworkspace, 7
bind = $mainMod SHIFT, 8, movetoworkspace, 8
bind = $mainMod SHIFT, 9, movetoworkspace, 9
bind = $mainMod SHIFT, 0, movetoworkspace, 10

# Move windows between monitors
bind = $mainMod CTRL, h, movecurrentworkspacetomonitor, l
bind = $mainMod CTRL, l, movecurrentworkspacetomonitor, r

# Window grouping
bind = $mainMod, G, togglegroup
bind = $mainMod, Tab, changegroupactive
```

### 9.4 Animation Presets

#### Minimal Tier (Disabled)
```bash
animations {
    enabled = no
}
```

#### Default Tier (Smooth & Performant)
```bash
animations {
    enabled = yes
    
    bezier = ease, 0.4, 0.0, 0.2, 1
    
    animation = windows, 1, 4, ease
    animation = windowsOut, 1, 4, ease, popin 80%
    animation = fade, 1, 4, ease
    animation = workspaces, 1, 5, ease
}
```

#### Signature Tier (Elaborate)
```bash
animations {
    enabled = yes
    
    bezier = overshot, 0.05, 0.9, 0.1, 1.05
    bezier = smoothOut, 0.36, 0, 0.66, -0.56
    bezier = smoothIn, 0.25, 1, 0.5, 1
    bezier = bounce, 0.68, -0.55, 0.265, 1.55
    
    animation = windows, 1, 8, overshot, slide
    animation = windowsIn, 1, 8, bounce, slide
    animation = windowsOut, 1, 8, smoothOut, slide
    animation = windowsMove, 1, 6, smoothOut
    animation = border, 1, 10, default
    animation = borderangle, 1, 8, default
    animation = fade, 1, 10, smoothIn
    animation = fadeDim, 1, 10, smoothIn
    animation = workspaces, 1, 8, overshot, slidevert
    animation = specialWorkspace, 1, 8, bounce, slidevert
}
```

### 9.5 Visual Settings by Tier

#### Minimal Tier
```bash
general {
    gaps_in = 3
    gaps_out = 6
    border_size = 2
    col.active_border = rgba(88c0d0ff)
    col.inactive_border = rgba(4c566aaa)
    layout = dwindle
}

decoration {
    rounding = 0
    drop_shadow = no
    blur {
        enabled = no
    }
}
```

#### Default Tier
```bash
general {
    gaps_in = 5
    gaps_out = 10
    border_size = 2
    col.active_border = rgba(88c0d0ff) rgba(8fbcbbff) 45deg
    col.inactive_border = rgba(4c566aaa)
    layout = dwindle
}

decoration {
    rounding = 8
    drop_shadow = yes
    shadow_range = 4
    shadow_render_power = 3
    col.shadow = rgba(1a1a1aee)
    
    blur {
        enabled = yes
        size = 3
        passes = 1
    }
}
```

#### Signature Tier
```bash
general {
    gaps_in = 8
    gaps_out = 16
    border_size = 3
    col.active_border = rgba(88c0d0ff) rgba(8fbcbbff) rgba(a3be8cff) 45deg
    col.inactive_border = rgba(4c566a66)
    layout = dwindle
}

decoration {
    rounding = 12
    drop_shadow = yes
    shadow_range = 16
    shadow_render_power = 3
    col.shadow = rgba(00000099)
    
    blur {
        enabled = yes
        size = 8
        passes = 3
        new_optimizations = yes
        noise = 0.01
        contrast = 0.9
        brightness = 0.8
    }
    
    dim_inactive = yes
    dim_strength = 0.1
}

group {
    col.border_active = rgba(88c0d0ff) rgba(8fbcbbff) 45deg
    col.border_inactive = rgba(4c566aaa)
    
    groupbar {
        font_size = 10
        gradients = yes
        render_titles = yes
    }
}
```

---

## 10. CONFIGURATION TEMPLATES

### 10.1 Minimal Tier Template

**~/.config/hypr/hyprland.conf:**
```bash
# Hyprland Configuration - Minimal Tier

# Monitor configuration
monitor=,preferred,auto,1

# Environment variables
env = XCURSOR_SIZE,24

# Startup applications
exec-once = foot --server
exec-once = mako

# Input configuration
input {
    kb_layout = us
    follow_mouse = 1
    
    touchpad {
        natural_scroll = false
    }
}

# General settings
general {
    gaps_in = 3
    gaps_out = 6
    border_size = 2
    col.active_border = rgba(88c0d0ff)
    col.inactive_border = rgba(4c566aaa)
    layout = dwindle
}

# Decoration
decoration {
    rounding = 0
    drop_shadow = no
    
    blur {
        enabled = no
    }
}

# Animations
animations {
    enabled = no
}

# Layouts
dwindle {
    pseudotile = yes
    preserve_split = yes
}

# Define modifier
$mainMod = SUPER

# Applications
bind = $mainMod, Q, exec, footclient
bind = $mainMod, E, exec, thunar
bind = $mainMod, D, exec, tofi-drun

# Window management
bind = $mainMod, C, killactive
bind = $mainMod, M, exit
bind = $mainMod, V, togglefloating
bind = $mainMod, F, fullscreen

# Focus
bind = $mainMod, h, movefocus, l
bind = $mainMod, l, movefocus, r
bind = $mainMod, k, movefocus, u
bind = $mainMod, j, movefocus, d

# Workspaces
bind = $mainMod, 1, workspace, 1
bind = $mainMod, 2, workspace, 2
bind = $mainMod, 3, workspace, 3
bind = $mainMod, 4, workspace, 4
bind = $mainMod, 5, workspace, 5

bind = $mainMod SHIFT, 1, movetoworkspace, 1
bind = $mainMod SHIFT, 2, movetoworkspace, 2
bind = $mainMod SHIFT, 3, movetoworkspace, 3
bind = $mainMod SHIFT, 4, movetoworkspace, 4
bind = $mainMod SHIFT, 5, movetoworkspace, 5

# Mouse
bindm = $mainMod, mouse:272, movewindow
bindm = $mainMod, mouse:273, resizewindow

# System
bind = $mainMod, L, exec, swaylock -f

# Screenshots
bind = $mainMod SHIFT, S, exec, grim -g "$(slurp)" - | wl-copy

# Volume
bind = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bind = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bind = , XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle

# Brightness
bind = , XF86MonBrightnessUp, exec, brightnessctl set 5%+
bind = , XF86MonBrightnessDown, exec, brightnessctl set 5%-

# Window rules
windowrulev2 = float, class:^(pavucontrol)$
windowrulev2 = float, class:^(thunar)$, title:^(File Operation Progress)$
```

### 10.2 Default Tier Template

**~/.config/hypr/hyprland.conf:**
```bash
# Hyprland Configuration - Default Tier

# Monitor configuration
monitor=,preferred,auto,1

# Environment variables
env = XCURSOR_SIZE,24
env = HYPRCURSOR_SIZE,24

# Startup applications
exec-once = hyprpaper
exec-once = waybar
exec-once = mako
exec-once = hypridle
exec-once = hyprpolkitagent
exec-once = wl-paste --type text --watch cliphist store
exec-once = wl-paste --type image --watch cliphist store

# Input configuration
input {
    kb_layout = us
    follow_mouse = 1
    
    touchpad {
        natural_scroll = false
        tap-to-click = true
    }
    
    sensitivity = 0
}

# General settings
general {
    gaps_in = 5
    gaps_out = 10
    border_size = 2
    col.active_border = rgba(88c0d0ff) rgba(8fbcbbff) 45deg
    col.inactive_border = rgba(4c566aaa)
    layout = dwindle
}

# Decoration
decoration {
    rounding = 8
    drop_shadow = yes
    shadow_range = 4
    shadow_render_power = 3
    col.shadow = rgba(1a1a1aee)
    
    blur {
        enabled = yes
        size = 3
        passes = 1
    }
}

# Animations
animations {
    enabled = yes
    
    bezier = ease, 0.4, 0.0, 0.2, 1
    
    animation = windows, 1, 4, ease
    animation = windowsOut, 1, 4, ease, popin 80%
    animation = fade, 1, 4, ease
    animation = workspaces, 1, 5, ease
}

# Layouts
dwindle {
    pseudotile = yes
    preserve_split = yes
}

# Gestures
gestures {
    workspace_swipe = yes
}

# Misc
misc {
    disable_hyprland_logo = yes
    disable_splash_rendering = yes
    force_default_wallpaper = 0
}

# Define modifier
$mainMod = SUPER

# Applications
bind = $mainMod, Q, exec, kitty
bind = $mainMod, E, exec, thunar
bind = $mainMod, D, exec, rofi -show drun
bind = $mainMod, R, exec, rofi -show run

# Window management
bind = $mainMod, C, killactive
bind = $mainMod, M, exit
bind = $mainMod, V, togglefloating
bind = $mainMod, F, fullscreen
bind = $mainMod, P, pseudo

# Focus
bind = $mainMod, h, movefocus, l
bind = $mainMod, l, movefocus, r
bind = $mainMod, k, movefocus, u
bind = $mainMod, j, movefocus, d

# Move windows
bind = $mainMod SHIFT, h, movewindow, l
bind = $mainMod SHIFT, l, movewindow, r
bind = $mainMod SHIFT, k, movewindow, u
bind = $mainMod SHIFT, j, movewindow, d

# Workspaces
bind = $mainMod, 1, workspace, 1
bind = $mainMod, 2, workspace, 2
bind = $mainMod, 3, workspace, 3
bind = $mainMod, 4, workspace, 4
bind = $mainMod, 5, workspace, 5

bind = $mainMod SHIFT, 1, movetoworkspace, 1
bind = $mainMod SHIFT, 2, movetoworkspace, 2
bind = $mainMod SHIFT, 3, movetoworkspace, 3
bind = $mainMod SHIFT, 4, movetoworkspace, 4
bind = $mainMod SHIFT, 5, movetoworkspace, 5

# Special workspace
bind = $mainMod, S, togglespecialworkspace, magic
bind = $mainMod SHIFT, S, movetoworkspace, special:magic

# Scroll through workspaces
bind = $mainMod, mouse_down, workspace, e+1
bind = $mainMod, mouse_up, workspace, e-1

# Mouse
bindm = $mainMod, mouse:272, movewindow
bindm = $mainMod, mouse:273, resizewindow

# System
bind = $mainMod, L, exec, loginctl lock-session

# Screenshots
bind = $mainMod SHIFT, S, exec, grimblast copy area
bind = $mainMod SHIFT, P, exec, grimblast copy screen
bind = $mainMod SHIFT, A, exec, grimblast copy active

# Color picker
bind = $mainMod SHIFT, C, exec, hyprpicker -a

# Clipboard history
bind = $mainMod, V, exec, cliphist list | rofi -dmenu | cliphist decode | wl-copy

# Volume
bind = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bind = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bind = , XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle

# Media
bind = , XF86AudioPlay, exec, playerctl play-pause
bind = , XF86AudioNext, exec, playerctl next
bind = , XF86AudioPrev, exec, playerctl previous

# Brightness
bind = , XF86MonBrightnessUp, exec, brightnessctl set 5%+
bind = , XF86MonBrightnessDown, exec, brightnessctl set 5%-

# Reload Waybar
bind = $mainMod SHIFT, R, exec, pkill waybar; waybar &

# Window rules
windowrulev2 = float, class:^(pavucontrol)$
windowrulev2 = float, class:^(thunar)$, title:^(File Operation Progress)$
windowrulev2 = opacity 0.9 0.9, class:^(kitty)$
windowrulev2 = workspace 2, class:^(firefox)$
```

**~/.config/hypr/hyprpaper.conf:**
```bash
preload = ~/Pictures/wallpapers/default.png
wallpaper = ,~/Pictures/wallpapers/default.png
splash = false
```

**~/.config/hypr/hypridle.conf:**
```bash
general {
    lock_cmd = pidof hyprlock || hyprlock
    before_sleep_cmd = loginctl lock-session
    after_sleep_cmd = hyprctl dispatch dpms on
}

listener {
    timeout = 300
    on-timeout = brightnessctl -s set 10%
    on-resume = brightnessctl -r
}

listener {
    timeout = 600
    on-timeout = loginctl lock-session
}

listener {
    timeout = 900
    on-timeout = hyprctl dispatch dpms off
    on-resume = hyprctl dispatch dpms on
}
```

**~/.config/hypr/hyprlock.conf:**
```bash
background {
    monitor =
    path = ~/Pictures/wallpapers/lockscreen.png
    blur_passes = 2
    blur_size = 4
}

input-field {
    monitor =
    size = 200, 50
    position = 0, -80
    dots_center = true
    fade_on_empty = true
    font_color = rgb(202, 211, 245)
    inner_color = rgb(91, 96, 120)
    outer_color = rgb(24, 25, 38)
    outline_thickness = 3
    placeholder_text = <span foreground="##cad3f5">Password...</span>
    shadow_passes = 2
}

label {
    monitor =
    text = cmd[update:1000] echo "$(date +"%-I:%M%p")"
    color = rgba(200, 200, 200, 1.0)
    font_size = 55
    font_family = Fira Semibold
    position = 0, 80
    halign = center
    valign = center
}
```

### 10.3 Signature Tier Template

**~/.config/hypr/hyprland.conf:**
```bash
# Hyprland Configuration - Signature Tier

# Source external configs
source = ~/.config/hypr/monitors.conf
source = ~/.config/hypr/env.conf
source = ~/.config/hypr/keybindings.conf
source = ~/.config/hypr/rules.conf

# Startup applications
exec-once = hyprpaper
exec-once = ags
exec-once = swaync
exec-once = hypridle
exec-once = hyprpolkitagent
exec-once = hyprsunset
exec-once = wl-paste --type text --watch cliphist store
exec-once = wl-paste --type image --watch cliphist store

# Input configuration
input {
    kb_layout = us
    follow_mouse = 1
    float_switch_override_focus = 2
    
    touchpad {
        natural_scroll = true
        tap-to-click = true
        clickfinger_behavior = true
    }
    
    sensitivity = 0
}

# General settings
general {
    gaps_in = 8
    gaps_out = 16
    border_size = 3
    col.active_border = rgba(88c0d0ff) rgba(8fbcbbff) rgba(a3be8cff) 45deg
    col.inactive_border = rgba(4c566a66)
    layout = dwindle
}

# Decoration
decoration {
    rounding = 12
    drop_shadow = yes
    shadow_range = 16
    shadow_render_power = 3
    col.shadow = rgba(00000099)
    
    blur {
        enabled = yes
        size = 8
        passes = 3
        new_optimizations = yes
        noise = 0.01
        contrast = 0.9
        brightness = 0.8
    }
    
    dim_inactive = yes
    dim_strength = 0.1
}

# Animations
animations {
    enabled = yes
    
    bezier = overshot, 0.05, 0.9, 0.1, 1.05
    bezier = smoothOut, 0.36, 0, 0.66, -0.56
    bezier = smoothIn, 0.25, 1, 0.5, 1
    bezier = bounce, 0.68, -0.55, 0.265, 1.55
    
    animation = windows, 1, 8, overshot, slide
    animation = windowsIn, 1, 8, bounce, slide
    animation = windowsOut, 1, 8, smoothOut, slide
    animation = windowsMove, 1, 6, smoothOut
    animation = border, 1, 10, default
    animation = borderangle, 1, 8, default
    animation = fade, 1, 10, smoothIn
    animation = fadeDim, 1, 10, smoothIn
    animation = workspaces, 1, 8, overshot, slidevert
    animation = specialWorkspace, 1, 8, bounce, slidevert
}

# Layouts
dwindle {
    pseudotile = yes
    preserve_split = yes
    smart_split = yes
    smart_resizing = yes
}

# Group
group {
    col.border_active = rgba(88c0d0ff) rgba(8fbcbbff) 45deg
    col.border_inactive = rgba(4c566aaa)
    
    groupbar {
        font_size = 10
        gradients = yes
        render_titles = yes
    }
}

# Gestures
gestures {
    workspace_swipe = yes
    workspace_swipe_fingers = 3
    workspace_swipe_distance = 300
}

# Misc
misc {
    disable_hyprland_logo = yes
    disable_splash_rendering = yes
    force_default_wallpaper = 0
    vfr = yes
    vrr = 1
    animate_manual_resizes = yes
    animate_mouse_windowdragging = yes
}
```

**~/.config/hypr/keybindings.conf:**
```bash
# Hyprland Keybindings - Signature Tier

$mainMod = SUPER

# Applications
bind = $mainMod, Q, exec, wezterm
bind = $mainMod, E, exec, dolphin
bind = $mainMod, D, exec, ags -t launcher
bind = $mainMod, R, exec, rofi -show run

# Window management
bind = $mainMod, C, killactive
bind = $mainMod, M, exit
bind = $mainMod, V, togglefloating
bind = $mainMod, F, fullscreen
bind = $mainMod, P, pseudo
bind = $mainMod, G, togglegroup
bind = $mainMod, Tab, changegroupactive

# Focus
bind = $mainMod, h, movefocus, l
bind = $mainMod, l, movefocus, r
bind = $mainMod, k, movefocus, u
bind = $mainMod, j, movefocus, d

# Move windows
bind = $mainMod SHIFT, h, movewindow, l
bind = $mainMod SHIFT, l, movewindow, r
bind = $mainMod SHIFT, k, movewindow, u
bind = $mainMod SHIFT, j, movewindow, d

# Workspaces (1-10)
bind = $mainMod, 1, workspace, 1
bind = $mainMod, 2, workspace, 2
bind = $mainMod, 3, workspace, 3
bind = $mainMod, 4, workspace, 4
bind = $mainMod, 5, workspace, 5
bind = $mainMod, 6, workspace, 6
bind = $mainMod, 7, workspace, 7
bind = $mainMod, 8, workspace, 8
bind = $mainMod, 9, workspace, 9
bind = $mainMod, 0, workspace, 10

# Move to workspace
bind = $mainMod SHIFT, 1, movetoworkspace, 1
bind = $mainMod SHIFT, 2, movetoworkspace, 2
bind = $mainMod SHIFT, 3, movetoworkspace, 3
bind = $mainMod SHIFT, 4, movetoworkspace, 4
bind = $mainMod SHIFT, 5, movetoworkspace, 5
bind = $mainMod SHIFT, 6, movetoworkspace, 6
bind = $mainMod SHIFT, 7, movetoworkspace, 7
bind = $mainMod SHIFT, 8, movetoworkspace, 8
bind = $mainMod SHIFT, 9, movetoworkspace, 9
bind = $mainMod SHIFT, 0, movetoworkspace, 10

# Special workspace
bind = $mainMod, S, togglespecialworkspace, magic
bind = $mainMod SHIFT, S, movetoworkspace, special:magic

# Move workspaces between monitors
bind = $mainMod CTRL, h, movecurrentworkspacetomonitor, l
bind = $mainMod CTRL, l, movecurrentworkspacetomonitor, r

# Scroll workspaces
bind = $mainMod, mouse_down, workspace, e+1
bind = $mainMod, mouse_up, workspace, e-1

# Mouse bindings
bindm = $mainMod, mouse:272, movewindow
bindm = $mainMod, mouse:273, resizewindow

# Resize submap
bind = $mainMod, R, submap, resize
submap = resize
bind = , h, resizeactive, -20 0
bind = , l, resizeactive, 20 0
bind = , k, resizeactive, 0 -20
bind = , j, resizeactive, 0 20
bind = , escape, submap, reset
submap = reset

# System submap
bind = $mainMod, Escape, submap, system
submap = system
bind = , l, exec, loginctl lock-session
bind = , l, submap, reset
bind = , e, exit
bind = , s, exec, systemctl suspend
bind = , s, submap, reset
bind = , r, exec, systemctl reboot
bind = , p, exec, systemctl poweroff
bind = , escape, submap, reset
submap = reset

# Screenshots
bind = $mainMod SHIFT, S, exec, grimblast copy area
bind = $mainMod SHIFT, P, exec, grimblast copy screen
bind = $mainMod SHIFT, A, exec, grimblast copy active

# Color picker
bind = $mainMod SHIFT, C, exec, hyprpicker -a

# Clipboard history
bind = $mainMod, V, exec, cliphist list | rofi -dmenu | cliphist decode | wl-copy

# Notifications
bind = $mainMod, N, exec, swaync-client -t -sw
bind = $mainMod SHIFT, N, exec, swaync-client -d -sw

# Volume
bind = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bind = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bind = , XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle

# Media
bind = , XF86AudioPlay, exec, playerctl play-pause
bind = , XF86AudioNext, exec, playerctl next
bind = , XF86AudioPrev, exec, playerctl previous

# Brightness
bind = , XF86MonBrightnessUp, exec, brightnessctl set 5%+
bind = , XF86MonBrightnessDown, exec, brightnessctl set 5%-

# Reload AGS
bind = $mainMod SHIFT, R, exec, ags -q; ags &
```

**~/.config/hypr/rules.conf:**
```bash
# Hyprland Window Rules - Signature Tier

# Floating windows
windowrulev2 = float, class:^(pavucontrol)$
windowrulev2 = float, class:^(nm-connection-editor)$
windowrulev2 = float, class:^(blueman-manager)$
windowrulev2 = float, class:^(org.kde.polkit-kde-authentication-agent-1)$

# Opacity
windowrulev2 = opacity 0.95 0.95, class:^(kitty)$
windowrulev2 = opacity 0.90 0.90, class:^(Code)$
windowrulev2 = opacity 0.85 0.85, class:^(thunar)$

# Workspace assignments
windowrulev2 = workspace 1, class:^(firefox)$
windowrulev2 = workspace 2, class:^(Code)$
windowrulev2 = workspace 3, class:^(discord)$
windowrulev2 = workspace 4, class:^(Spotify)$

# Size and position
windowrulev2 = size 800 600, title:^(Calculator)$
windowrulev2 = center, floating:1

# Special handling
windowrulev2 = idleinhibit focus, class:^(mpv)$
windowrulev2 = idleinhibit fullscreen, class:^(firefox)$
windowrulev2 = pin, title:^(Picture-in-Picture)$
```

---

## CONCLUSION

This research document provides a comprehensive foundation for building three-tier Hyprland bundles (minimal/default/signature) with:

1. **Complete configuration reference** from official sources
2. **Tool categorization** by purpose and complexity
3. **NVIDIA-specific guidance** for problematic hardware
4. **Integration patterns** for tool interoperability
5. **Smart defaults** for each tier
6. **Ready-to-use templates** for immediate deployment

All information sourced from official documentation as of November 7, 2025.

**Next Steps:**
1. Implement configuration generators for each tier
2. Create installation scripts per tier
3. Build testing infrastructure
4. Develop tier migration paths

---

**Document Version:** 1.0
**Last Updated:** November 7, 2025
**Research by:** Claude (Sonnet 4.5)
**Sources:** wiki.hypr.land, wiki.archlinux.org, GitHub official repositories
