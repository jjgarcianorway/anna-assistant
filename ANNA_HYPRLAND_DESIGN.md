# Anna Hyprland System Design
**Version:** 1.0
**Target Release:** RC.9.5 - RC.9.7
**Philosophy:** One intelligent bundle that adapts to YOUR system

---

## 1. THE ANNA WAY - Intelligent Adaptation

**Traditional Approach (❌):**
```bash
annactl bundle install hyprland-minimal    # User guesses
annactl bundle install hyprland-default    # User guesses
annactl bundle install hyprland-signature  # User guesses
```

**The Anna Way (✅):**
```bash
annactl bundle install hyprland
→ Anna analyzes your system (RAM, GPU, CPU, existing tools)
→ Anna generates the PERFECT config for YOUR hardware
→ Looks amazing within YOUR resource constraints
→ Adapts as your system evolves
```

### Intelligence Sources
- **Hardware:** RAM, GPU vendor/model, CPU cores
- **Existing tools:** Terminals, launchers, file managers already installed
- **User preferences:** Detected from dotfiles, running processes
- **Resource usage:** Current system load, available resources
- **Display:** Resolution, refresh rate, multi-monitor setup

---

## 2. ANNA COLOR SCHEME - "Arctic Dusk"

Inspired by Arch's technical elegance with warm undertones. Soft, professional, calming.

### 2.1 Core Identity Colors

| Role | Name | Hex | RGB | Usage |
|------|------|-----|-----|-------|
| **Primary** | Arctic Teal | `#5FBDBD` | rgb(95, 189, 189) | Main accent, focused elements |
| **Secondary** | Warm Amber | `#E8B67A` | rgb(232, 182, 122) | Secondary highlights, success states |
| **Tertiary** | Soft Lavender | `#B396C9` | rgb(179, 150, 201) | Tertiary elements, special states |

### 2.2 Semantic Colors

| Role | Name | Hex | RGB | Usage |
|------|------|-----|-----|-------|
| **Success** | Mint Green | `#88C0A0` | rgb(136, 192, 160) | Completed actions, healthy states |
| **Warning** | Honey Gold | `#E5C07B` | rgb(229, 192, 123) | Warnings, important info |
| **Error** | Coral Pink | `#E87D7D` | rgb(232, 125, 125) | Errors, critical states |
| **Info** | Sky Blue | `#7EB8DA` | rgb(126, 184, 218) | Informational, links |

### 2.3 Dark Theme Base

| Element | Name | Hex | Usage |
|---------|------|-----|-------|
| **BG Primary** | Deep Slate | `#1E1E2E` | Main background |
| **BG Secondary** | Darker Slate | `#181825` | Panels, sidebars |
| **BG Tertiary** | Charcoal | `#11111B` | Terminal, code blocks |
| **FG Primary** | Cloud White | `#E0E0E6` | Main text |
| **FG Secondary** | Silver Gray | `#A6A6B3` | Secondary text |
| **FG Tertiary** | Slate Gray | `#6C6C7A` | Disabled, placeholders |
| **Border** | Steel Blue | `#3B3B52` | Borders, separators |
| **Border Active** | Arctic Teal | `#5FBDBD` | Active borders, focus |

### 2.4 Light Theme Base

| Element | Name | Hex | Usage |
|---------|------|-----|-------|
| **BG Primary** | Frost White | `#F5F5F9` | Main background |
| **BG Secondary** | Pale Blue | `#EBEBF0` | Panels, sidebars |
| **BG Tertiary** | Cloud Gray | `#DCDCE5` | Terminal (light mode) |
| **FG Primary** | Deep Charcoal | `#2E2E3A` | Main text |
| **FG Secondary** | Dim Gray | `#5A5A6B` | Secondary text |
| **FG Tertiary** | Light Gray | `#8A8A9B` | Disabled, placeholders |
| **Border** | Silver | `#C5C5D0` | Borders, separators |
| **Border Active** | Arctic Teal | `#5FBDBD` | Active borders, focus |

### 2.5 Terminal ANSI Colors

**Dark Theme:**
```
Black:         #1E1E2E
Red:           #E87D7D
Green:         #88C0A0
Yellow:        #E5C07B
Blue:          #7EB8DA
Magenta:       #B396C9
Cyan:          #5FBDBD
White:         #E0E0E6

Bright Black:  #6C6C7A
Bright Red:    #F09090
Bright Green:  #9DD5B5
Bright Yellow: #F0D699
Bright Blue:   #93CCE8
Bright Magenta:#C9ADDC
Bright Cyan:   #7AD3D3
Bright White:  #F5F5F9
```

**Light Theme:**
```
Black:         #2E2E3A
Red:           #D06B6B
Green:         #6FA887
Yellow:        #D4A860
Blue:          #5B9AC2
Magenta:       #9B7DB0
Cyan:          #4AA5A5
White:         #F5F5F9

Bright Black:  #5A5A6B
Bright Red:    #E87D7D
Bright Green:  #88C0A0
Bright Yellow: #E5C07B
Bright Blue:   #7EB8DA
Bright Magenta:#B396C9
Bright Cyan:   #5FBDBD
Bright White:  #FFFFFF
```

---

## 3. ADAPTIVE ARCHITECTURE

### 3.1 Resource Tiers (Auto-Detected)

Anna automatically assigns your system to a tier based on:

| Tier | RAM | GPU | CPU | Target Profile |
|------|-----|-----|-----|----------------|
| **Efficient** | ≤8GB | iGPU/no GPU | ≤4 cores | Old laptop, VM, budget build |
| **Balanced** | 8-16GB | Mid-range GPU | 4-8 cores | Typical desktop, modern laptop |
| **Performance** | ≥16GB | High-end GPU | ≥8 cores | Gaming rig, workstation, enthusiast |

### 3.2 Component Selection Matrix

Anna chooses components based on your tier and existing tools:

#### Status Bar / Panel

| Tier | Primary Choice | Fallback | RAM Usage | Features |
|------|----------------|----------|-----------|----------|
| **Efficient** | yambar | waybar (minimal) | ~10-20MB | Simple modules, CPU/RAM, workspaces |
| **Balanced** | waybar | ashell | ~30-60MB | Custom modules, system tray, media controls |
| **Performance** | HyprPanel | AGS custom | ~80-150MB | GUI config, dashboard, full theming |

#### Launcher

| Tier | Primary | Fallback | Notes |
|------|---------|----------|-------|
| **Efficient** | fuzzel | bemenu | Fast, minimal dependencies |
| **Balanced** | wofi | rofi-wayland | Balance of features/speed |
| **Performance** | rofi-wayland | hyprlauncher | Full theming, plugins |

#### Notifications

| Tier | Choice | Config Complexity |
|------|--------|-------------------|
| **Efficient** | mako | Simple, lightweight |
| **Balanced** | mako | Themed, with actions |
| **Performance** | swaync | Full notification center |

#### Terminal (User's existing terminal preferred)

Detection order: kitty → foot → alacritty → wezterm → fallback to kitty

#### File Manager (User's existing preferred)

Detection order: thunar → dolphin → nautilus → pcmanfm → fallback to thunar

#### Wallpaper & Lock

| Component | All Tiers | Notes |
|-----------|-----------|-------|
| Wallpaper | hyprpaper | Official, efficient |
| Lock | hyprlock | Official, GPU-accelerated |
| Idle | hypridle | Official, battery-friendly |

### 3.3 Animation & Visual Settings

#### Efficient Tier
```toml
animations = true  # Keep basic, reduce visual debt
bezier = linear    # No fancy curves
speed = 1.2        # Slightly faster (feels snappier)
blur = false       # Save GPU cycles
shadows = false    # Save GPU cycles
rounding = 4       # Subtle rounding
```

#### Balanced Tier
```toml
animations = true
bezier = easeOutQuart  # Smooth but not slow
speed = 1.0
blur = true (size: 3, passes: 1)  # Light blur
shadows = true (range: 4)          # Subtle shadows
rounding = 8                       # Moderate rounding
```

#### Performance Tier
```toml
animations = true
bezier = customElastic  # Elaborate curves
speed = 0.8             # Slightly slower (more visible)
blur = true (size: 8, passes: 3)  # Heavy blur
shadows = true (range: 20)         # Strong shadows
rounding = 12                      # Full rounding
```

### 3.4 NVIDIA Detection & Configuration

Anna detects NVIDIA GPUs and automatically applies required env vars:

```bash
# Critical NVIDIA environment variables
env = LIBVA_DRIVER_NAME,nvidia
env = XDG_SESSION_TYPE,wayland
env = GBM_BACKEND,nvidia-drm
env = __GLX_VENDOR_LIBRARY_NAME,nvidia
env = WLR_NO_HARDWARE_CURSORS,1

# Hyprland cursor settings
cursor {
    no_hardware_cursors = true
}
```

---

## 4. CONFIGURATION GENERATION

### 4.1 Smart Detection Logic

```rust
pub struct SystemProfile {
    // Hardware
    pub ram_gb: u64,
    pub gpu_vendor: GpuVendor,  // NVIDIA, AMD, Intel, None
    pub cpu_cores: usize,
    pub resolution: (u32, u32),
    pub refresh_rate: u32,

    // Existing tools (detected)
    pub terminal: Option<String>,      // kitty, foot, alacritty...
    pub file_manager: Option<String>,  // thunar, dolphin...
    pub launcher: Option<String>,      // rofi, wofi, fuzzel...

    // Derived
    pub tier: ResourceTier,  // Efficient, Balanced, Performance
}

pub enum ResourceTier {
    Efficient,    // ≤8GB RAM, iGPU
    Balanced,     // 8-16GB, discrete GPU
    Performance,  // ≥16GB, high-end GPU
}
```

### 4.2 Keybinding Intelligence

Anna uses detected tools in keybindings:

```bash
# Terminal (uses detected terminal)
bind = SUPER, Return, exec, {detected_terminal}

# File manager (uses detected)
bind = SUPER, E, exec, {detected_file_manager}

# Launcher (uses tier-appropriate)
bind = SUPER, D, exec, {tier_appropriate_launcher}

# Screenshots (always grim+slurp)
bind = , Print, exec, grim -g "$(slurp)" - | wl-copy
```

### 4.3 Configuration File Structure

```
~/.config/anna-theme/
├── colors.toml                 # Anna color definitions
├── hyprland/
│   ├── hyprland.conf          # Generated main config
│   ├── keybindings.conf       # Smart keybindings
│   ├── windowrules.conf       # Common rules
│   ├── nvidia.conf            # NVIDIA-specific (if detected)
│   └── startup.conf           # Exec-once commands
├── hyprpaper/
│   └── hyprpaper.conf         # Wallpaper config
├── hypridle/
│   └── hypridle.conf          # Idle management
├── hyprlock/
│   └── hyprlock.conf          # Lock screen
├── waybar/ (or ashell/hyprpanel)
│   ├── config                 # Bar configuration
│   └── style.css              # Anna-themed styling
└── terminal/
    ├── kitty.conf             # Anna colors for kitty
    ├── foot.ini               # Anna colors for foot
    └── alacritty.toml         # Anna colors for alacritty
```

### 4.4 Template System

Anna uses Tera templates with system profile data:

```bash
# Example template snippet (hyprland.conf)
general {
    gaps_in = {{ gaps_in }}
    gaps_out = {{ gaps_out }}
    border_size = {{ border_size }}
    col.active_border = rgb({{ color_accent }})
    col.inactive_border = rgb({{ color_border }})
}

{% if tier == "Performance" %}
decoration {
    rounding = 12
    blur {
        enabled = true
        size = 8
        passes = 3
    }
    drop_shadow = yes
    shadow_range = 20
}
{% elif tier == "Balanced" %}
decoration {
    rounding = 8
    blur {
        enabled = true
        size = 3
        passes = 1
    }
    drop_shadow = yes
    shadow_range = 4
}
{% else %}
decoration {
    rounding = 4
    blur {
        enabled = false
    }
    drop_shadow = no
}
{% endif %}
```

---

## 5. USER EXPERIENCE

### 5.1 Installation Flow

```bash
$ annactl bundle install hyprland

🔍 Analyzing your system...
   • RAM: 16GB
   • GPU: NVIDIA RTX 3060
   • CPU: 8 cores
   • Existing: kitty, thunar

✨ Profile: Balanced Tier
   Bar: waybar (themed)
   Launcher: wofi
   Terminal: kitty (existing)
   File manager: thunar (existing)
   Animations: Smooth
   Theme: Anna Arctic Dusk (Dark)

📦 Installing packages...
   [████████████████] 100%

⚙️  Generating configuration...
   • ~/.config/hypr/hyprland.conf
   • ~/.config/waybar/ (Anna theme)
   • ~/.config/anna-theme/
   • NVIDIA optimizations applied

✅ Installation complete!

💡 Next steps:
   • Log out and select "Hyprland" from your display manager
   • Or run: Hyprland
   • Keybindings: Super+Return (terminal), Super+D (launcher)
   • Customize: Edit ~/.config/anna-theme/colors.toml
```

### 5.2 Customization

Users can override Anna's choices:

```bash
# Force a different tier
$ annactl bundle install hyprland --tier performance

# Use specific bar
$ annactl bundle install hyprland --bar hyprpanel

# Light theme
$ annactl bundle install hyprland --theme light

# Reconfigure after hardware upgrade
$ annactl bundle reconfigure hyprland
```

### 5.3 Future: Dynamic Adaptation

Future versions could:
- Detect laptop on battery → reduce animations
- Detect high load → simplify effects
- Learn user preferences over time
- Suggest tier upgrade after hardware changes

---

## 6. IMPLEMENTATION PLAN

### Phase 1: Foundation (RC.9.5)
1. ✅ Research complete
2. ✅ Design complete
3. Remove all other bundles (gnome, kde, i3, etc.)
4. Implement resource detection
5. Create Anna color scheme files
6. Basic template system

### Phase 2: Core Components (RC.9.6)
7. Implement Efficient tier (yambar/waybar minimal)
8. Implement Balanced tier (waybar themed)
9. Tool detection (terminal, file manager, launcher)
10. NVIDIA detection and configuration
11. Keybinding intelligence

### Phase 3: Advanced (RC.9.7)
12. Implement Performance tier (HyprPanel/AGS)
13. Complete theme system (GTK, Qt, terminals)
14. Configuration reconfiguration command
15. User customization options
16. Polish and testing

---

## 7. SUCCESS CRITERIA

- ✅ ONE bundle that works for everyone
- ✅ Automatically detects and adapts to system resources
- ✅ Uses existing tools when possible
- ✅ Anna color scheme consistently applied
- ✅ NVIDIA systems work out of box
- ✅ Looks amazing at ALL resource tiers
- ✅ Easy to customize
- ✅ Respects user's existing setup

---

## 8. COMPARISON: Before vs After

### Before (Multiple Bundles)
- User must choose between 3 bundles
- Might choose wrong one for their system
- Duplicate code maintenance
- Inconsistent experience

### After (One Intelligent Bundle)
- Anna chooses the best configuration
- Always optimal for the system
- Single codebase with templates
- Consistent Anna experience
- Adapts as system evolves

---

**The Anna Philosophy:**
*"Your system, perfectly configured, beautifully themed, intelligently adapted."*

🌸 Anna - Your Arch Linux Assistant
