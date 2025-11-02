# Arch Linux Advisor - System Optimization Recommendations

**Version:** 1.0 (Alpha)
**Status:** Active
**Added in:** Anna v0.12.3

## Overview

The Arch Linux Advisor analyzes your system configuration and provides actionable recommendations for optimization, security, and best practices. It runs deterministic rules against hardware profile and package inventory data to detect common misconfigurations and suggest improvements.

**Key Features:**
- **Deterministic** - Same system state always produces same advice
- **Fast** - Runs in 100-300ms after data collected
- **Safe** - No system modifications, only recommendations
- **Privacy-preserving** - All analysis done locally, no network calls
- **No root required** - Read-only access to system state

## What It Inspects

### Hardware Profile
- CPU model and vendor (AMD/Intel)
- GPU devices and loaded drivers
- Storage devices and controllers (NVMe/SATA/HDD)
- Network interfaces and drivers
- BIOS/board information (when available)
- Running kernel version

### Package Inventory
- All installed packages (`pacman -Qq`)
- Explicitly installed vs. dependencies
- Orphan packages (no longer needed)
- Package groups (base, base-devel, nvidia, vulkan, etc.)
- Recent package events (installs, upgrades, removals)
- AUR packages (sampled detection)

## Advisory Rules

The advisor implements 10 deterministic rules:

### 1. NVIDIA Kernel Headers (`nvidia-headers-missing`)

**Level:** Warning
**Category:** Drivers
**Trigger:** NVIDIA GPU with nvidia-dkms but no linux-headers

NVIDIA's DKMS driver requires matching kernel headers to build modules. Without headers, the driver may fail to load after kernel updates.

**Action:**
```bash
sudo pacman -S linux-headers
```

**References:**
- [Arch Wiki: NVIDIA](https://wiki.archlinux.org/title/NVIDIA)
- [Arch Wiki: DKMS](https://wiki.archlinux.org/title/Dynamic_Kernel_Module_Support)

### 2. Vulkan Stack Missing (`vulkan-missing`)

**Level:** Info
**Category:** Graphics
**Trigger:** GPU present but vulkan-icd-loader not installed

Vulkan is the modern graphics API used by many games and applications. Missing the loader or vendor-specific ICD limits graphics capabilities.

**Action (NVIDIA):**
```bash
sudo pacman -S vulkan-icd-loader nvidia-utils
```

**Action (AMD):**
```bash
sudo pacman -S vulkan-icd-loader vulkan-radeon
```

**Action (Intel):**
```bash
sudo pacman -S vulkan-icd-loader vulkan-intel
```

**References:**
- [Arch Wiki: Vulkan](https://wiki.archlinux.org/title/Vulkan)

### 3. Microcode Missing (`microcode-amd-missing` / `microcode-intel-missing`)

**Level:** Warning
**Category:** System
**Trigger:** AMD/Intel CPU without amd-ucode/intel-ucode installed

CPU microcode provides critical security and stability fixes delivered by the manufacturer. Missing microcode leaves known vulnerabilities unpatched.

**Action (AMD):**
```bash
sudo pacman -S amd-ucode
# Then regenerate bootloader config
sudo grub-mkconfig -o /boot/grub/grub.cfg  # if using GRUB
```

**Action (Intel):**
```bash
sudo pacman -S intel-ucode
# Then regenerate bootloader config
sudo grub-mkconfig -o /boot/grub/grub.cfg  # if using GRUB
```

**References:**
- [Arch Wiki: Microcode](https://wiki.archlinux.org/title/Microcode)

### 10. Orphan Packages (`orphan-packages`)

**Level:** Info
**Category:** Maintenance
**Trigger:** One or more orphan packages detected

Orphan packages were installed as dependencies but are no longer required by any explicitly installed package. They waste disk space and create maintenance overhead.

**Action:**
```bash
# Review orphans
pacman -Qtd

# Remove after review
sudo pacman -Rns $(pacman -Qtdq)
```

**References:**
- [Arch Wiki: Pacman Tips - Removing Orphans](https://wiki.archlinux.org/title/Pacman/Tips_and_tricks#Removing_unused_packages_(orphans))

### 5. NVMe I/O Scheduler (`nvme-scheduler`)

**Level:** Info
**Category:** Performance
**Trigger:** NVMe device detected

NVMe devices have built-in sophisticated scheduling and perform best with the kernel's `none` scheduler, which bypasses the legacy block layer.

**Action:**
```bash
# Check current scheduler
cat /sys/block/nvme*/queue/scheduler

# Set to 'none' for best performance
echo none | sudo tee /sys/block/nvme*/queue/scheduler

# Make persistent (create udev rule)
echo 'ACTION=="add|change", KERNEL=="nvme[0-9]n[0-9]", ATTR{queue/scheduler}="none"' \
  | sudo tee /etc/udev/rules.d/60-ioschedulers.rules
```

**References:**
- [Arch Wiki: Improving Performance - I/O Schedulers](https://wiki.archlinux.org/title/Improving_performance#Input/output_schedulers)

### 6. TLP/Power Management (`power-management-missing`, `power-management-conflict`)

**Level:** Info/Warning
**Category:** Power
**Trigger:** Laptop detected without power management tool, or multiple conflicting tools installed

TLP and auto-cpufreq automatically optimize power settings for laptops (CPU scaling, disk power management, USB autosuspend, etc.), extending battery life by 20-40%. Running multiple tools simultaneously causes conflicts and unpredictable behavior.

**Action:**
```bash
# Install TLP (recommended for most users)
sudo pacman -S tlp tlp-rdw
sudo systemctl enable --now tlp.service

# Mask conflicting services
sudo systemctl mask systemd-rfkill.service systemd-rfkill.socket

# If conflict detected, remove other tools
sudo systemctl disable --now auto-cpufreq.service
sudo pacman -Rns auto-cpufreq laptop-mode-tools
```

**References:**
- [Arch Wiki: TLP](https://wiki.archlinux.org/title/TLP)
- [Arch Wiki: Power Management](https://wiki.archlinux.org/title/Power_management)

### 11. Kernel Headers for DKMS (`kernel-headers-missing`)

**Level:** Warning
**Category:** System
**Trigger:** DKMS modules detected but linux-headers not installed

Any DKMS-based driver (VirtualBox, NVIDIA, ZFS, etc.) requires kernel headers to build kernel modules. Missing headers cause driver failures after kernel updates.

**Action:**
```bash
sudo pacman -S linux-headers
```

**References:**
- [Arch Wiki: DKMS](https://wiki.archlinux.org/title/Dynamic_Kernel_Module_Support)

### 4. CPU Governor (`cpu-governor-laptop`)

**Level:** Info
**Category:** Performance
**Trigger:** Laptop detected (battery present)

CPU frequency scaling governors control performance vs power consumption. On laptops, the default governor may be set to 'powersave' which limits CPU frequency, reducing performance during intensive workloads.

**Action:**
```bash
# Check current governor
cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Install cpupower
sudo pacman -S cpupower

# Set to schedutil (balanced) or performance (max speed)
sudo cpupower frequency-set -g schedutil
```

**References:**
- [Arch Wiki: CPU Frequency Scaling](https://wiki.archlinux.org/title/CPU_frequency_scaling)

### 7. ZRAM/Swap Configuration (`zram-recommended`)

**Level:** Info
**Category:** Performance
**Trigger:** System with <8GB RAM and no ZRAM installed

ZRAM creates a compressed block device in RAM for swap, significantly faster than disk-based swap. On low-memory systems, ZRAM prevents thrashing and improves responsiveness under memory pressure.

**Action:**
```bash
# Install zram-generator
sudo pacman -S zram-generator

# Configure ZRAM (50% of RAM, zstd compression)
echo -e '[zram0]\nzram-size = ram / 2\ncompression-algorithm = zstd' | \
  sudo tee /etc/systemd/zram-generator.conf

# Enable and start
sudo systemctl daemon-reload
sudo systemctl start systemd-zram-setup@zram0.service
```

**References:**
- [Arch Wiki: Zram](https://wiki.archlinux.org/title/Zram)

### 8. Wayland/Xorg Acceleration (`nvidia-wayland-modesetting`)

**Level:** Info
**Category:** Graphics
**Trigger:** NVIDIA GPU with proprietary driver detected

Wayland requires DRM kernel modesetting (KMS) for proper operation. NVIDIA's proprietary driver disables KMS by default, causing Wayland sessions to fail or fall back to Xorg. Enabling modesetting is essential for Wayland support.

**Action:**
```bash
# Enable NVIDIA DRM modesetting
echo 'options nvidia-drm modeset=1' | \
  sudo tee /etc/modprobe.d/nvidia-drm-modesetting.conf

# Rebuild initramfs
sudo mkinitcpio -P

# Reboot, then verify
cat /sys/module/nvidia_drm/parameters/modeset
# Should show 'Y'
```

**References:**
- [Arch Wiki: NVIDIA DRM Kernel Mode Setting](https://wiki.archlinux.org/title/NVIDIA#DRM_kernel_mode_setting)
- [Arch Wiki: Wayland - NVIDIA](https://wiki.archlinux.org/title/Wayland#NVIDIA)

### 9. AUR Helpers (`aur-helper-missing`)

**Level:** Info
**Category:** Maintenance
**Trigger:** AUR packages detected but no AUR helper installed

AUR helpers like yay and paru automate building and updating AUR packages. Without one, users must manually check for updates, download PKGBUILDs, resolve dependencies, and run makepkg for each package.

**Action:**
```bash
# Install yay
git clone https://aur.archlinux.org/yay.git
cd yay && makepkg -si

# Or install paru
git clone https://aur.archlinux.org/paru.git
cd paru && makepkg -si

# Then use for AUR operations
yay -Syu  # Update all packages including AUR
```

**References:**
- [Arch Wiki: AUR Helpers](https://wiki.archlinux.org/title/AUR_helpers)

## Usage

### Basic Usage

```bash
# Run advisor with pretty output
annactl advisor arch

# JSON output for scripting
annactl advisor arch --json

# Explain specific advice
annactl advisor arch --explain nvidia-headers-missing
```

### Example Output

**Pretty TUI Mode:**
```
╭─ Arch Linux Advisor ──────────────────────────╮

Drivers Issues
──────────────────────────────────────────────────
⚠ NVIDIA DKMS requires kernel headers
  Reason: Running kernel 6.17.6-arch1-1 with nvidia-dkms but linux-headers not installed
  Action: sudo pacman -S linux-headers
  ID: nvidia-headers-missing

System Issues
──────────────────────────────────────────────────
⚠ AMD microcode not installed
  Reason: AMD CPU detected but amd-ucode package missing
  Action: sudo pacman -S amd-ucode
  ID: microcode-amd-missing

────────────────────────────────────────────────
2 issues found

For details: annactl advisor arch --explain <id>
```

**JSON Mode:**
```json
[
  {
    "id": "nvidia-headers-missing",
    "level": "warn",
    "category": "drivers",
    "title": "NVIDIA DKMS requires kernel headers",
    "reason": "Running kernel 6.17.6-arch1-1 with nvidia-dkms but linux-headers not installed",
    "action": "sudo pacman -S linux-headers",
    "refs": [
      "https://wiki.archlinux.org/title/NVIDIA",
      "https://wiki.archlinux.org/title/Dynamic_Kernel_Module_Support"
    ]
  }
]
```

## Data Sources

The advisor collects data from:

### Hardware Detection
- `lspci -mm -v` - PCI devices (GPU, storage, network)
- `lsblk -J` - Block devices and storage topology
- `lsusb` - USB devices
- `uname -r` - Running kernel version
- `/proc/cpuinfo` - CPU details
- `/sys/class/dmi/id/*` - Board/BIOS info (fallback if dmidecode unavailable)
- `/sys/class/power_supply/` - Battery detection
- `nvidia-smi` - NVIDIA GPU details (if available)

### Package Analysis
- `pacman -Qq` - All installed packages
- `pacman -Qeq` - Explicitly installed packages
- `pacman -Qtdq` - Orphan packages
- `pacman -Si` - Package repository info (sampled for AUR detection)
- `/var/log/pacman.log` - Recent package events

## Performance

- **First run (cold):** 1-2 seconds (data collection)
- **Subsequent runs:** 100-300ms (cached data, rules only)
- **Cache TTL:** 24 hours for hardware, 5 minutes for packages

## Privacy & Security

### What is Collected
- Hardware model names and specifications
- Package names and versions
- System configuration state

### What is NOT Collected
- User data or files
- Network traffic or browsing history
- Serial numbers or unique identifiers
- Personal information

### Where Data Stays
- All data collected locally on your machine
- No network calls made
- No telemetry sent anywhere
- Cache stored in `~/.cache/anna/` (user-readable only)

### Root Access
- **Not required** for standard operation
- `dmidecode` (board info) is optional; gracefully skipped if root not available
- All package and hardware queries use read-only, unprivileged APIs

## Opt-Out

To disable the advisor entirely:

```bash
# Don't run it - it's completely opt-in
# No background scanning, only runs when you invoke it
```

The advisor only runs when you explicitly call `annactl advisor arch`. There is no background scanning or automatic execution.

## Limitations (Alpha)

### Current Limitations
1. **AUR Detection** - Limited to sampling (100 packages max) for performance
2. **Scheduler Check** - NVMe scheduler cannot be read without root; advisory is informational only
3. **Rule Count** - 10 rules implemented in v0.12.3; more planned for future releases
4. **No Auto-Apply** - Advisor only suggests actions; user must execute manually

### Future Enhancements (Planned)
- Power management recommendations (laptop battery optimization)
- Security hardening checks (firewall, SSH config)
- Performance tuning (CPU governor, swappiness)
- Filesystem optimization (mount options, trim)
- Network stack tuning (TCP parameters, DNS)
- Machine learning-based recommendations (pattern detection)
- Auto-apply mode (with policy engine approval)

## Troubleshooting

### No advice shown but system has issues

**Cause:** Rules may not cover your specific issue yet
**Solution:** Current alpha has only 6 rules; more will be added

### "Failed to connect to annad" error

**Cause:** Daemon not running
**Solution:**
```bash
sudo systemctl status annad
sudo systemctl start annad
```

### Slow performance (>5 seconds)

**Cause:** First run collecting data
**Solution:** Subsequent runs use cached data and complete in <300ms

### Incorrect advice

**Cause:** Stale cached data
**Solution:** Force refresh by restarting daemon:
```bash
sudo systemctl restart annad
annactl advisor arch
```

### Permission denied errors

**Cause:** Missing read permissions on system files
**Solution:** Advisor should work without root; if seeing errors, check file permissions

## Extending the Advisor

### Adding Custom Rules (Developers)

Rules are defined in `src/annad/src/advisor_v13.rs`:

```rust
impl ArchAdvisor {
    fn check_my_rule(hw: &HardwareProfile, pkg: &PackageInventory) -> Vec<Advice> {
        let mut result = Vec::new();

        // Your detection logic
        if some_condition {
            result.push(Advice {
                id: "my-rule-id".to_string(),
                level: Level::Warn,
                category: "category".to_string(),
                title: "Short title".to_string(),
                reason: "Why this is an issue".to_string(),
                action: "How to fix it".to_string(),
                refs: vec!["https://wiki.archlinux.org/...".to_string()],
            });
        }

        result
    }
}
```

Then add to `run()` method:
```rust
pub fn run(hw: &HardwareProfile, pkg: &PackageInventory) -> Vec<Advice> {
    let mut advice = Vec::new();
    advice.extend(Self::check_my_rule(hw, pkg));
    // ...
    advice
}
```

## Testing

### Unit Tests

```bash
cargo test -p annad advisor
```

### Integration Test

```bash
# Run advisor on live system
annactl advisor arch

# Check JSON schema
annactl advisor arch --json | jq .

# Test degraded mode
TERM=dumb annactl advisor arch | cat
NO_COLOR=1 annactl advisor arch
```

## JSON Schema

See `docs/schemas/advisor_advice_v1.json` for the canonical JSON schema.

## References

- **Implementation:** `src/annad/src/advisor_v13.rs`
- **CLI Command:** `src/annactl/src/advisor_cmd.rs`
- **RPC Endpoint:** `src/annad/src/rpc_v10.rs::method_advisor_run`
- **Hardware Collector:** `src/annad/src/hardware_profile.rs`
- **Package Collector:** `src/annad/src/package_analysis.rs`

## Change Log

### v0.12.3 (2025-11-02) - Alpha Release
- Initial alpha release
- 6 deterministic rules implemented
- Hardware profile collection
- Package inventory analysis
- TUI and JSON output modes
- No root requirement
- Local-only operation

---

**Feedback:** Report issues at https://github.com/jjgarcianorway/anna-assistant/issues
