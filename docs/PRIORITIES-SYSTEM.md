# Anna Priorities System

> **Philosophy**: Anna should be autonomous where it's safe and objective, persuasive where it's subjective.
>
> **Goal**: Small set of orthogonal knobs that actually matter, with strong defaults and complete reversibility.

---

## The Decision Stack (Order of Precedence)

```
┌─────────────────────────────────────────────────────────┐
│  1. Mandatory Baseline (Non-Negotiable)                │
│     • Security, boot integrity, filesystem health       │
│     • Risk: Varies, Auto-apply: Always (with rollback) │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│  2. User Priorities (Explicit Preferences)              │
│     • performance, aesthetics, stability, privacy       │
│     • Risk: Low-Medium, Auto-apply: If allowed          │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│  3. Contextual Heuristics (Observed, Local)             │
│     • Hardware capability, software footprint           │
│     • Risk: Low, Auto-apply: Tie-break only             │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│  4. Risk/Autonomy Matrix                                │
│     • Low → auto-apply if hands_off allows              │
│     • Medium → ask once, remember preference            │
│     • High → advice only                                │
└─────────────────────────────────────────────────────────┘
```

---

## 1. Mandatory Baseline (Layer 1)

These run **regardless of user priorities** — they're the "don't break the machine" layer.

### Categories

#### Security & Boot Integrity
- Firmware + microcode updates (Intel/AMD)
- Boot integrity (initramfs/grub correctness)
- Firewall enabled with sane defaults
- SSH key-based auth (disable password auth)
- CVE fixes with low risk

#### Filesystem Health
- Btrfs scrub scheduling
- SMART critical alerts
- Filesystem check flags (`fsck` errors)
- Disk space warnings (< 10% free)

#### USB & Device Policy
- USB auto-mount safe defaults
- Device permissions (udev rules)

### Characteristics
- **Risk**: Varies (but vetted for safety)
- **Auto-apply**: Always (with rollback tokens)
- **Override**: Cannot be disabled (but can be rolled back)
- **Wiki Citations**: Required for every action

**Example Rule**:
```yaml
id: microcode-intel-mandatory
layer: mandatory
category: security
risk: low
title: "Intel microcode not installed"
reason: "Critical security patches missing"
wiki_ref: "https://wiki.archlinux.org/title/Microcode"
fix_cmd: "sudo pacman -S intel-ucode && sudo grub-mkconfig -o /boot/grub/grub.cfg"
rollback_strategy: remove_packages
auto_apply: true
```

---

## 2. User Priorities (Layer 2)

Small, orthogonal set of dimensions with clear semantics.

### Priority Dimensions

```yaml
# ~/.config/anna/priorities.yaml

priorities:
  # Performance vs. Efficiency
  performance:
    value: balanced           # low, balanced, high
    description: "CPU/GPU utilization, compilation speed, responsiveness"

  # UI Responsiveness
  responsiveness:
    value: balanced           # low, balanced, high
    description: "Animation smoothness, input latency, frame rates"

  # Battery vs. Performance
  battery_life:
    value: balanced           # low, balanced, high
    description: "Power management aggressiveness"

  # Visual Beauty vs. Minimalism
  aesthetics:
    value: beautiful          # minimal, balanced, beautiful
    description: "Theming, animations, visual polish"

  # Update Cadence
  stability:
    value: conservative       # fast, balanced, conservative
    description: "Update frequency, package staleness tolerance"

  # Privacy Stance
  privacy:
    value: strict             # relaxed, balanced, strict
    description: "Telemetry, cloud features, behavioral analysis"

  # Autonomy Level
  hands_off:
    value: auto_low_risk      # advice_only, ask_first, auto_low_risk
    description: "How autonomous Anna should be"

# Metadata
last_updated: 1705334567
profile: workstation
```

### Value Semantics

#### `performance` (low, balanced, high)
- **low**: Minimize CPU/GPU usage, prefer efficiency
  - Disable heavy services, reduce compilation flags
  - Example: `-O2` instead of `-O3`, disable compositor effects
- **balanced**: Reasonable performance without waste
  - Enable performance features where beneficial
  - Example: Hardware acceleration enabled, moderate compilation
- **high**: Maximum throughput, utilize hardware fully
  - Aggressive optimization, parallel compilation
  - Example: `-O3 -march=native`, all hardware acceleration

#### `responsiveness` (low, balanced, high)
- **low**: Don't prioritize UI smoothness
  - Acceptable input lag, basic animations
  - Example: 30 FPS compositor, no input prediction
- **balanced**: Smooth for interactive use
  - 60 FPS target, reasonable input latency
  - Example: VRR enabled, basic compositing
- **high**: Zero-latency feel
  - 144 FPS+, aggressive input handling
  - Example: Gaming-optimized scheduler, direct rendering

#### `battery_life` (low, balanced, high)
- **low**: Always-on performance
  - No CPU frequency scaling, aggressive boost
  - Example: `performance` governor, no screen dimming
- **balanced**: Adaptive power management
  - Scale based on load
  - Example: `powersave` governor with boost, adaptive brightness
- **high**: Maximize battery
  - Aggressive power saving
  - Example: TLP aggressive, CPU throttling, screen dim

#### `aesthetics` (minimal, balanced, beautiful)
- **minimal**: Function over form
  - No theming, disable animations, monospace defaults
  - Example: Basic terminal colors, no compositor effects
- **balanced**: Tasteful polish
  - Consistent theme, subtle animations
  - Example: One cohesive theme, fade transitions
- **beautiful**: Full visual experience
  - Rich theming, smooth animations, visual effects
  - Example: Blur, shadows, gradients, custom fonts

#### `stability` (fast, balanced, conservative)
- **fast**: Bleeding edge
  - Update daily, experimental packages OK
  - Example: `linux-mainline`, AUR packages freely
- **balanced**: Stable with timely updates
  - Update weekly, vetted AUR packages
  - Example: `linux`, popular AUR helpers
- **conservative**: Rock-solid reliability
  - Update monthly, minimal AUR, LTS when available
  - Example: `linux-lts`, official repos only

#### `privacy` (relaxed, balanced, strict)
- **relaxed**: Convenience over privacy
  - Optional telemetry, cloud sync allowed
  - Example: Browser telemetry, cloud editors
- **balanced**: Privacy-conscious defaults
  - Local-first, opt-in telemetry
  - Example: Local analysis only, no tracking
- **strict**: Maximum privacy
  - No telemetry, no cloud, no behavioral analysis
  - Example: Tor, no JavaScript, local-only tools

#### `hands_off` (advice_only, ask_first, auto_low_risk)
- **advice_only**: Anna never auto-applies
  - Show recommendations, user applies manually
  - Example: `annactl advisor` shows advice, no auto-action
- **ask_first**: Anna asks for each action
  - Prompt for confirmation, remember preferences
  - Example: Apply prompt with [y/N/always/never]
- **auto_low_risk**: Anna applies safe changes
  - Auto-apply low-risk, prompt for medium/high
  - Example: Microcode auto-installs, compositor asks

---

## 3. Profiles (Presets)

```yaml
profiles:
  minimal:
    performance: high
    responsiveness: low
    battery_life: low
    aesthetics: minimal
    stability: conservative
    privacy: strict
    hands_off: advice_only
    description: "Maximum performance, minimal aesthetics, manual control"

  beautiful:
    performance: balanced
    responsiveness: high
    battery_life: balanced
    aesthetics: beautiful
    stability: balanced
    privacy: balanced
    hands_off: auto_low_risk
    description: "Visual polish, smooth interactions, reasonable autonomy"

  workstation:
    performance: high
    responsiveness: balanced
    battery_life: low
    aesthetics: balanced
    stability: balanced
    privacy: strict
    hands_off: auto_low_risk
    description: "Developer-focused, privacy-conscious, productive"

  gaming:
    performance: high
    responsiveness: high
    battery_life: low
    aesthetics: beautiful
    stability: fast
    privacy: relaxed
    hands_off: auto_low_risk
    description: "Maximum FPS, low latency, cutting-edge drivers"

  server:
    performance: balanced
    responsiveness: low
    battery_life: low
    aesthetics: minimal
    stability: conservative
    privacy: strict
    hands_off: auto_low_risk
    description: "Reliability over flash, security hardening, headless"

  laptop:
    performance: balanced
    responsiveness: balanced
    battery_life: high
    aesthetics: balanced
    stability: balanced
    privacy: balanced
    hands_off: auto_low_risk
    description: "Battery efficiency, balanced performance, portable"
```

---

## 4. Contextual Heuristics (Layer 3)

Observed facts that **refine tie-breaks** but never override explicit priorities.

### Hardware Capability Signals

```yaml
hardware_signals:
  gpu:
    type: discrete             # integrated, discrete, none
    vendor: nvidia             # intel, amd, nvidia
    vram_gb: 8
    vulkan_capable: true
    driver_quality: good       # poor, fair, good, excellent

  display:
    refresh_rate: 144
    resolution: "2560x1440"
    hidpi: false
    vrr_capable: true

  cpu:
    cores: 16
    boost_capable: true
    tdp_class: desktop         # mobile, desktop, server

  memory:
    total_gb: 64
    swap_gb: 16

  battery:
    present: false
```

### Software Footprint Signals

```yaml
software_signals:
  compositor:
    current: hyprland          # none, i3, sway, hyprland, kwin, mutter
    wayland: true
    session_stable: true

  terminal:
    current: foot              # kitty, alacritty, foot, wezterm, urxvt
    gpu_accelerated: true
    truecolor_capable: true

  toolkit:
    gtk_version: 4
    qt_version: 6
    primary: gtk               # gtk, qt, both

  editors:
    primary: neovim            # vim, neovim, emacs, vscode, helix
    usage_hours_per_day: 8

  shell:
    type: zsh                  # bash, zsh, fish
    plugins: [zsh-autosuggestions, zsh-syntax-highlighting]
```

### Behavioral Signals (Opt-In)

```yaml
behavior_signals:
  workflow:
    type: developer            # developer, server_admin, creative, gamer, mixed
    confidence: 0.85

  terminal_usage:
    hours_per_day: 10
    top_commands:
      - { cmd: git, count: 450 }
      - { cmd: cargo, count: 320 }
      - { cmd: nvim, count: 280 }

  compositor_usage:
    cpu_time_percent: 5
    animations_visible: true
    screen_time_hours: 12
```

---

## 5. Decision Algorithm

### Scoring Function

For each candidate recommendation `R`:

```python
def score(R, priorities, hardware, software, behavior):
    # Layer 1: Mandatory baseline always scores highest
    if R.layer == "mandatory":
        return 1000  # Always applies

    # Layer 2: User priorities (main driver)
    user_score = 0
    for dimension in ["performance", "responsiveness", "battery_life", "aesthetics", "stability"]:
        alignment = compute_alignment(R, priorities[dimension])
        user_score += alignment * WEIGHTS[dimension]

    # Layer 3: Contextual heuristics (tie-break only)
    context_score = 0
    if fits_hardware(R, hardware):
        context_score += 10
    if fits_software(R, software):
        context_score += 10
    if fits_behavior(R, behavior):
        context_score += 5

    # Risk penalty
    risk_penalty = {
        "low": 0,
        "medium": -20,
        "high": -50,
    }[R.risk]

    # Final score
    return user_score + context_score + risk_penalty

def compute_alignment(R, priority_value):
    """
    How well does recommendation R align with priority_value?
    Returns: -100 (conflicts) to +100 (perfect match)
    """
    # Example: R wants "beautiful aesthetics", user set "minimal"
    if R.requires_aesthetics == "beautiful" and priority_value == "minimal":
        return -50  # Conflicts
    elif R.requires_aesthetics == "beautiful" and priority_value == "beautiful":
        return +100  # Perfect match
    elif R.requires_aesthetics == "balanced":
        return +50   # Neutral
    # ... (similar logic for other dimensions)

def auto_apply_decision(R, score, priorities):
    """Should Anna auto-apply recommendation R?"""

    # Mandatory: always auto-apply
    if R.layer == "mandatory":
        return True

    # Hands-off level
    hands_off = priorities["hands_off"]

    if hands_off == "advice_only":
        return False  # Never auto-apply

    if hands_off == "ask_first":
        return False  # Always ask (but remember answer)

    if hands_off == "auto_low_risk":
        # Auto-apply only low-risk with positive score
        return R.risk == "low" and score > 50

    return False
```

---

## 6. Desktop Environment Decision Examples

### Example 1: Hyprland vs. Sway

**Scenario**: User has Intel iGPU, 60Hz display, Sway installed, `aesthetics=beautiful`

**Hyprland Migration Rule**:
```yaml
id: desktop-hyprland-migration
layer: optional
category: desktop-env
risk: medium
title: "Migrate to Hyprland for visual effects"

condition:
  current_compositor: sway
  priorities:
    aesthetics: beautiful
  hardware:
    gpu_type: [integrated, discrete]
    wayland_stable: true

recommendation:
  type: migration_offer
  from: sway
  to: hyprland
  reason: "Hyprland offers richer animations and blur effects aligned with your 'aesthetics=beautiful' preference"
  wiki_ref: "https://wiki.archlinux.org/title/Hyprland"

action:
  prompt: true  # Always ask, never auto-migrate
  preview: true  # Show config diff
  snapshot: true  # Backup Sway config
  rollback_cmd: "restore_compositor sway"

guardrails:
  - "Never auto-migrate compositor"
  - "Offer theme overlay for Sway first"
  - "Full rollback path with saved Sway config"
```

**Anna's Output**:
```
╭─ Desktop Environment ────────────────────────────────────╮

⚠  Hyprland migration available

   Because: You set aesthetics=beautiful, Intel GPU detected
   Current: Sway (stable)
   Offered: Hyprland with animations and blur effects

   Risk: Medium — compositor migration
   Wiki: https://wiki.archlinux.org/title/Hyprland

   Options:
   1. Theme current Sway setup (low risk) ← Recommended
   2. Migrate to Hyprland (medium risk, full preview)
   3. Ignore this suggestion

   [1/2/3]:
```

### Example 2: Foot 24-bit Color

**Scenario**: User has `foot` terminal, no truecolor config, `aesthetics=balanced`

**Foot Truecolor Rule**:
```yaml
id: terminal-foot-truecolor
layer: optional
category: terminal-ux
risk: low
title: "Enable 24-bit colors in foot terminal"

condition:
  terminal: foot
  truecolor_enabled: false
  priorities:
    aesthetics: [balanced, beautiful]

recommendation:
  type: config_edit
  file: ~/.config/foot/foot.ini
  reason: "24-bit color improves code syntax highlighting and theme consistency"
  wiki_ref: "https://wiki.archlinux.org/title/Foot#Configuration"

action:
  fix_cmd: |
    mkdir -p ~/.config/foot
    cat >> ~/.config/foot/foot.ini << 'EOF'
    [colors]
    alpha=0.95
    foreground=ebdbb2
    background=282828
    # Gruvbox theme with 24-bit colors
    EOF
  auto_apply: true  # Low risk + balanced aesthetics = auto-apply
  snapshot: true
  rollback_strategy: file_restore

guardrails:
  - "Never remove existing config"
  - "Append to existing file or create new"
  - "Backup original before any edit"
```

**Anna's Output** (auto-applied):
```
╭─ Terminal UX ────────────────────────────────────────────╮

✓ Enabled 24-bit colors in foot terminal

  Config: ~/.config/foot/foot.ini
  Theme: Gruvbox (pastel)
  Reason: Aligned with aesthetics=balanced
  Risk: Low — config file edit only

  Rollback: annactl rollback --last

  Wiki: https://wiki.archlinux.org/title/Foot#Configuration
```

### Example 3: NVIDIA + Wayland

**Scenario**: User has NVIDIA proprietary driver, Wayland issues detected

**Wayland Warning Rule**:
```yaml
id: desktop-nvidia-wayland-issues
layer: optional
category: desktop-env
risk: high
title: "NVIDIA proprietary driver has known Wayland issues"

condition:
  gpu_vendor: nvidia
  driver: proprietary
  wayland: true
  issues_detected: [flickering, tearing, crashes]

recommendation:
  type: advice_only
  reason: "NVIDIA proprietary + Wayland has stability issues on your hardware"
  wiki_ref: "https://wiki.archlinux.org/title/NVIDIA#Wayland"

action:
  auto_apply: false  # High risk, advice only
  options:
    - title: "Stay on Xorg with i3"
      risk: low
      wiki: "https://wiki.archlinux.org/title/I3"

    - title: "Try Sway with NVIDIA workarounds"
      risk: medium
      wiki: "https://wiki.archlinux.org/title/Sway#NVIDIA"
      fix_cmd: "enable_nvidia_wayland_workarounds"

    - title: "Install open-source nouveau driver"
      risk: high
      wiki: "https://wiki.archlinux.org/title/Nouveau"
      reason: "Loses CUDA/performance but better Wayland support"
```

---

## 7. Guardrails

### Never Override Without Consent

❌ **NEVER**:
- Auto-migrate compositor (Sway → Hyprland, i3 → Sway)
- Auto-migrate terminal (foot → kitty, alacritty → wezterm)
- Auto-change desktop environment
- Auto-enable experimental features
- Override explicit `ignored` advice

✅ **ALWAYS**:
- Snapshot before any config edit
- Offer "theme current setup" before migration
- Show full preview with diffs
- Require confirmation for migrations
- Provide rollback tokens

### Risk Matrix

| Risk | Auto-Apply | Confirmation | Rollback | Examples |
|------|-----------|--------------|----------|----------|
| Low | If `hands_off` allows | No | Token + snapshot | Terminal colors, editor syntax |
| Medium | Never | Yes + remember | Token + snapshot | Compositor theme, package cleanup |
| High | Never | Yes + preview | Manual or guided | Compositor migration, driver change |
| Mandatory | Always | No (logs only) | Token + snapshot | Microcode, firewall, boot integrity |

### Config Edit Rules

For every config file edit:
1. **Check existence**: If file exists, read it first
2. **Snapshot**: Create timestamped backup
3. **Non-destructive**: Append or patch, never truncate
4. **Idempotent**: Check if change already applied
5. **Verify**: Validate syntax after edit
6. **Rollback token**: Store restore path

**Example** (Hyprland theme overlay):
```bash
# Before
$ cat ~/.config/hypr/hyprland.conf
# User's existing config
exec-once = waybar

# Anna's change (non-destructive)
$ cat ~/.config/hypr/hyprland.conf
# User's existing config
exec-once = waybar

# Anna theme overlay (added at end)
# Added by Anna on 2024-01-20
source = ~/.config/hypr/anna-theme.conf

$ cat ~/.config/hypr/anna-theme.conf
# Anna's theme overlay
# Rollback: annactl rollback --id hyprland-theme-overlay
decoration {
    blur {
        enabled = true
        size = 3
    }
    drop_shadow = true
}
```

---

## 8. CLI Interface

### Set Priorities

```bash
# Apply profile
annactl priorities profile apply beautiful
annactl priorities profile apply workstation

# Fine-tune individual priorities
annactl priorities set aesthetics=beautiful
annactl priorities set performance=high stability=conservative

# Batch update
annactl priorities set aesthetics=beautiful performance=high stability=conservative

# Show current priorities
annactl priorities show

# Show profile definitions
annactl priorities profile list
annactl priorities profile show workstation
```

### Manage Advice

```bash
# Show advice filtered by priorities
annactl advisor                          # Shows all open advice
annactl advisor --category desktop-env   # Filter by category
annactl advisor --all                    # Include resolved advice

# Explain priority alignment
annactl advisor --explain hyprland-migration
# Output:
#   Score: 65/100
#   Alignment:
#     ✓ aesthetics=beautiful → +80 (theme matches)
#     ~ performance=high → +20 (Hyprland slightly heavier than Sway)
#     ✓ hardware=capable → +10 (Intel GPU sufficient)
#   Risk: Medium → -20
#   Decision: Prompt user (migration requires consent)

# Override advice
annactl advisor --ignore hyprland-migration --reason "prefer sway speed"
annactl advisor --snooze foot-truecolor 30d
```

---

## 9. Implementation Files

```
src/annad/src/
├── priorities.rs              # Priority system core
├── priority_scorer.rs         # Scoring algorithm
├── profiles.rs                # Profile definitions
└── heuristics.rs              # Contextual signals

src/annactl/src/
├── priorities_cmd.rs          # CLI: annactl priorities
└── advisor_cmd.rs             # Updated: show priority alignment

~/.config/anna/
├── priorities.yaml            # User priorities
└── profiles/                  # Custom profiles
    └── my-custom.yaml

/etc/anna/
└── priorities.d/              # System defaults
    ├── minimal.yaml
    ├── beautiful.yaml
    ├── workstation.yaml
    ├── gaming.yaml
    ├── server.yaml
    └── laptop.yaml
```

---

## 10. Examples: Full Decision Trace

### Example A: Laptop with Beautiful Priorities

**User Priorities**:
```yaml
performance: balanced
responsiveness: balanced
battery_life: high
aesthetics: beautiful
stability: balanced
privacy: strict
hands_off: auto_low_risk
```

**Hardware**:
- Intel i7 mobile CPU
- Intel Iris Xe iGPU
- 16 GB RAM
- 60 Hz 1920x1080 display
- Battery present

**Decisions**:

| Advice | Score | Decision | Reason |
|--------|-------|----------|--------|
| `microcode-intel-mandatory` | 1000 | ✅ Auto-apply | Mandatory baseline |
| `compositor-sway-theme` | 75 | ✅ Auto-apply | Low risk + aesthetics match |
| `compositor-hyprland-migration` | 45 | ⚠️  Prompt | Medium risk + battery concern |
| `terminal-foot-truecolor` | 80 | ✅ Auto-apply | Low risk + aesthetics match |
| `cpu-governor-powersave` | 85 | ✅ Auto-apply | Low risk + battery priority |
| `gpu-power-saving` | 80 | ✅ Auto-apply | Low risk + battery priority |

**Output**:
```
╭─ Recommendations ────────────────────────────────────────╮

✓ Applied automatically (5 actions):
  1. Intel microcode installed
  2. Sway theme enhanced (blur, shadows)
  3. Foot terminal: 24-bit colors enabled
  4. CPU governor: powersave mode
  5. Intel GPU: power-saving features enabled

⚠  Needs your decision (1 action):
  • Migrate to Hyprland for richer effects?
    Score: 45/100 (borderline due to battery_life=high)
    [View details / Apply / Ignore / Snooze]

  Rollback any action: annactl rollback --list
```

### Example B: Gaming Workstation with Minimal Priorities

**User Priorities**:
```yaml
performance: high
responsiveness: high
battery_life: low
aesthetics: minimal
stability: fast
privacy: relaxed
hands_off: auto_low_risk
```

**Hardware**:
- AMD Ryzen 9 5950X
- NVIDIA RTX 3080
- 64 GB RAM
- 144 Hz 2560x1440 display
- No battery

**Decisions**:

| Advice | Score | Decision | Reason |
|--------|-------|----------|--------|
| `microcode-amd-mandatory` | 1000 | ✅ Auto-apply | Mandatory baseline |
| `compositor-hyprland-gaming` | 90 | ✅ Auto-apply | Low risk + performance/responsiveness |
| `cpu-governor-performance` | 95 | ✅ Auto-apply | Low risk + performance match |
| `gpu-max-performance` | 95 | ✅ Auto-apply | Low risk + performance match |
| `terminal-alacritty-minimal` | 85 | ✅ Auto-apply | Low risk + minimal aesthetics |
| `compositor-effects-disable` | -30 | ❌ Skip | Conflicts with hyprland-gaming |

---

## 11. Migration from Current System

**Phase 1**: Add priorities system
- Create `priorities.yaml` schema
- Implement scoring algorithm
- Add `annactl priorities` commands

**Phase 2**: Update advisor rules
- Add priority alignment to existing rules
- Create new rules with priority scoring
- Test with all 6 profiles

**Phase 3**: Integrate with apply/rollback
- Use scores to determine auto-apply
- Add priority explanation to outputs
- Update audit logs with priority context

---

## Summary

This priorities system:

✅ **Respects User Agency** — Small set of clear knobs, strong defaults
✅ **Balances Autonomy** — Auto-apply where safe, ask where subjective
✅ **Preserves Beauty** — Aesthetics is explicit dimension, not override
✅ **Maintains Safety** — Mandatory baseline always runs, everything reversible
✅ **Stays Local** — All decisions deterministic, no cloud/ML required
✅ **Provides Transparency** — Score breakdown shows why Anna decided

**Taste never becomes tyranny. Safety never becomes nagging. Beauty never becomes bloat.**

---

**Files Created**:
- `docs/PRIORITIES-SYSTEM.md` - Complete specification (this file)
- Ready for implementation with example rules included

🌸
