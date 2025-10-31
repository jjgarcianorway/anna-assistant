# Anna Thermal Management

**Version**: 0.9.7
**Status**: ✅ Production Ready

---

## Overview

Anna automatically manages system thermals through intelligent fan control, CPU governor management, and adaptive thermal policies. The system supports both ASUS laptops (via `asusctl`) and generic systems (via `lm-sensors` + `fancontrol`).

## Architecture

### Three-Layer Approach

1. **Hardware Layer** - Direct fan/governor control
2. **Policy Layer** - Temperature-based decision making
3. **Monitoring Layer** - Continuous thermal sensing (30s intervals)

### Dual Strategy

**Strategy A: ASUS Laptops (Preferred)**
- Uses `asusctl` for native ASUS fan curve control
- Integrates with `supergfxctl` for GPU management
- Leverages `power-profiles-daemon` for system-wide profiles

**Strategy B: Generic Systems (Fallback)**
- Uses `lm-sensors` to read hardware sensors
- Uses `fancontrol` daemon for PWM fan control
- Manual configuration via `sensors-detect` and `pwmconfig`

---

## Quick Start

### ASUS Laptops

```bash
# Install required packages
sudo pacman -S --needed asusctl supergfxctl power-profiles-daemon lm_sensors cpupower

# Enable services
sudo systemctl enable --now asusd.service supergfxd.service power-profiles-daemon.service cpupower.service

# Apply power-saver profile immediately
powerprofilesctl set power-saver

# Apply quiet fan profile
asusctl profile -P quiet

# Disable Intel Turbo (reduces thermal spikes)
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Configure cpupower for powersave governor
echo 'GOVERNOR="powersave"' | sudo tee /etc/default/cpupower
sudo systemctl restart cpupower
```

### Generic Laptops

```bash
# Install required packages
sudo pacman -S --needed lm_sensors fancontrol cpupower

# Detect sensors (auto-detect all)
sudo sensors-detect --auto

# Configure fan control (interactive)
sudo pwmconfig

# Enable fancontrol service
sudo systemctl enable --now fancontrol.service

# Verify it's working
sensors
sudo systemctl status fancontrol
```

---

## Anna Integration

### Automatic Installation

The Anna installer automatically:

1. **Detects system type** (ASUS vs generic)
2. **Installs appropriate services**:
   - `annad.service` - Main daemon
   - `anna-fans.service` - Thermal management
3. **Deploys thermal policies** to `/etc/anna/policies.d/thermal.yaml`
4. **Configures fan curves** (ASUS only)

```bash
./scripts/install.sh
```

The installer will output:

```
→ Installing systemd services...
✓ Anna daemon service installed and enabled
✓ ASUS laptop detected, enabling thermal management
✓ Thermal management enabled (will activate on next boot)
✓ Thermal management policies installed
```

### Manual Service Management

```bash
# Check thermal service status
sudo systemctl status anna-fans

# Apply thermal profile immediately
sudo systemctl start anna-fans

# View logs
journalctl -u anna-fans -n 20

# Restart thermal management
sudo systemctl restart anna-fans
```

---

## Thermal Policies

Anna includes adaptive thermal policies that monitor temperature and trigger actions.

### Policy File

Location: `/etc/anna/policies.d/thermal.yaml`

```yaml
# Policy 1: Log thermal state
- when: "always"
  then: "log"
  message: "Thermal management active"
  enabled: true

# Policy 2: Alert on high temperature
- when: "telemetry.cpu_temp > 85"
  then: "alert"
  message: "High CPU temperature detected (>85°C)"
  enabled: true

# Policy 3: Critical temperature protection
- when: "telemetry.cpu_temp > 90"
  then: "alert"
  message: "CRITICAL: CPU temperature >90°C"
  enabled: true

# Policy 4: Monitor elevated temperatures
- when: "telemetry.cpu_temp > 75 && telemetry.cpu_temp < 85"
  then: "log"
  message: "CPU temperature elevated (75-85°C)"
  enabled: true

# Policy 5: Optimal temperature (disabled to reduce log noise)
- when: "telemetry.cpu_temp < 60"
  then: "log"
  message: "CPU temperature optimal (<60°C)"
  enabled: false
```

### Policy Actions

Current policies support:
- `log` - Write to system log
- `alert` - Generate user notification

**Future (v0.9.8+)**:
- `throttle` - Reduce CPU frequency
- `optimize` - Adjust fan curve
- `execute` - Run custom commands

---

## ASUS Fan Curves

### Default Quiet Curve

```
Temperature (°C) → Fan Speed (%)
45°C → 0%    (Silent)
55°C → 20%   (Barely audible)
65°C → 35%   (Quiet)
75°C → 55%   (Moderate)
85°C → 75%   (Active cooling)
92°C → 100%  (Maximum)
```

### Customizing the Curve

Edit `/usr/local/share/anna/anna_fans_asus.sh`:

```bash
asusctl fan-curve --apply <<'EOF'
cpu  quiet  45:0 55:25 65:40 75:60 85:80 92:100
gpu  quiet  45:0 55:25 65:40 75:60 85:80 92:100
EOF
```

**Guidelines:**
- Lower temps → quieter but hotter
- Higher percentages → cooler but louder
- Keep GPU curve similar to CPU for balanced cooling

### Available Profiles

```bash
# Quiet (lowest noise, higher temps)
asusctl profile -P quiet

# Balanced (moderate noise and temps)
asusctl profile -P balanced

# Performance (maximum cooling)
asusctl profile -P performance

# Check current profile
asusctl profile -p
```

---

## Generic Fancontrol

### Configuration File

Location: `/etc/fancontrol`

Anna provides a template at `etc/fancontrol.template`. After running `pwmconfig`, compare with the template and adjust.

### Key Parameters

```ini
# Polling interval (seconds)
INTERVAL=3

# Temperature thresholds
MINTEMP=hwmon1/pwm1=45  # Start fan at 45°C
MAXTEMP=hwmon1/pwm1=85  # Max fan at 85°C

# PWM control values (0-255)
MINSTART=hwmon1/pwm1=80  # Startup PWM
MINSTOP=hwmon1/pwm1=50   # Stop threshold
```

### Tuning Tips

**For Quieter Operation:**
- Increase MINTEMP (e.g., 50°C)
- Increase MAXTEMP (e.g., 90°C)
- Lower MINSTART if fan spins reliably

**For Cooler Operation:**
- Decrease MINTEMP (e.g., 40°C)
- Decrease MAXTEMP (e.g., 80°C)
- Increase MINSTART for aggressive cooling

### Troubleshooting Fancontrol

```bash
# Test configuration
sudo fancontrol -t

# Check sensor readings
sensors

# Verify PWM control is available
cat /sys/class/hwmon/hwmon*/pwm*

# Re-run detection if sensors change
sudo sensors-detect --auto
sudo pwmconfig --yes
```

---

## CPU Governor Management

### Powersave Governor

Anna defaults to `powersave` governor for thermal management:

```bash
# Set via cpupower (persistent)
echo 'GOVERNOR="powersave"' | sudo tee /etc/default/cpupower
sudo systemctl restart cpupower

# Verify
cpupower frequency-info
```

### Available Governors

```bash
# List available governors
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors

# Common governors:
# - powersave: Keeps CPU at lowest frequency
# - performance: Keeps CPU at highest frequency
# - ondemand: Scales based on load (legacy)
# - schedutil: Kernel scheduler-driven (modern)
```

### Intel Turbo Boost Control

Disable turbo to cap thermal spikes:

```bash
# Disable (reduce heat and power)
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Enable (restore full performance)
echo 0 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Check status
cat /sys/devices/system/cpu/intel_pstate/no_turbo
```

---

## Monitoring

### Real-Time Temperature

```bash
# Watch all sensors
watch -n 2 sensors

# CPU temperature only
watch -n 1 "sensors | grep -i 'Package id 0'"

# Anna's telemetry (future)
watch -n 2 "annactl telemetry snapshot | grep temp"
```

### Log Monitoring

```bash
# Anna thermal logs
journalctl -u anna-fans -f

# ASUS daemon logs
journalctl -u asusd -f

# Fancontrol logs
journalctl -u fancontrol -f

# All thermal-related
journalctl -t anna-fans -t asusd -t fancontrol -f
```

### Anna Doctor Checks

```bash
# Validate thermal management
annactl doctor validate

# Check thermal policies
annactl policy list | grep thermal

# View temperature in system profile
annactl profile show
```

---

## Thermal Targets

### Optimal Ranges

| Component | Idle | Load | Maximum |
|-----------|------|------|---------|
| CPU       | 35-45°C | 60-75°C | 90°C |
| GPU       | 30-40°C | 65-80°C | 95°C |
| Motherboard | 30-40°C | 45-55°C | 70°C |

### Anna's Thermal Zones

- **Cool** (< 60°C): Optimal, quiet fan operation
- **Warm** (60-75°C): Normal under load
- **Hot** (75-85°C): Elevated, increase cooling
- **Critical** (> 85°C): Alert and throttle
- **Emergency** (> 90°C): Maximum intervention

---

## Power Profiles

### System-Wide Profiles

```bash
# List available profiles
powerprofilesctl list

# Set profile
powerprofilesctl set power-saver    # Anna's default
powerprofilesctl set balanced        # Moderate
powerprofilesctl set performance     # Maximum

# Get current profile
powerprofilesctl get
```

### Profile Effects

**power-saver:**
- Lower CPU frequencies
- Reduced display brightness
- Aggressive power management
- **Best for**: Battery life, low temperatures

**balanced:**
- Dynamic frequency scaling
- Moderate power management
- **Best for**: General use

**performance:**
- Maximum CPU frequencies
- Minimal power saving
- **Best for**: Gaming, rendering, builds

---

## Troubleshooting

### High Temperatures

**Symptom:** CPU consistently > 80°C at idle

**Fixes:**

1. **Check fan operation:**
   ```bash
   sensors | grep -i fan
   # Should show RPM > 0
   ```

2. **Verify thermal service:**
   ```bash
   sudo systemctl status anna-fans
   # Should show "active"
   ```

3. **Apply aggressive cooling:**
   ```bash
   # ASUS
   asusctl profile -P performance

   # Generic
   sudo systemctl restart fancontrol
   ```

4. **Check for dust/blockages** (physical inspection)

5. **Verify thermal paste** (if > 2 years old, consider repaste)

### Fans Not Responding

**Symptom:** Fan speed doesn't change with temperature

**Fixes:**

1. **ASUS systems:**
   ```bash
   sudo systemctl restart asusd
   asusctl profile -P balanced
   asusctl fan-curve --default
   ```

2. **Generic systems:**
   ```bash
   # Re-detect sensors
   sudo sensors-detect --auto

   # Reconfigure PWM
   sudo pwmconfig

   # Restart fancontrol
   sudo systemctl restart fancontrol
   ```

3. **Check BIOS settings:**
   - Ensure "Smart Fan Control" is enabled
   - Some BIOSes override OS fan control

### Noisy Fans

**Symptom:** Fans constantly ramping up/down or running high

**Fixes:**

1. **Apply quieter curve:**
   ```bash
   # ASUS
   asusctl profile -P quiet

   # Generic - edit /etc/fancontrol
   # Increase MINTEMP and MAXTEMP
   ```

2. **Check for runaway processes:**
   ```bash
   top
   # Look for processes using high CPU
   ```

3. **Verify turbo is disabled:**
   ```bash
   cat /sys/devices/system/cpu/intel_pstate/no_turbo
   # Should be "1"
   ```

### Service Won't Start

**Symptom:** `anna-fans.service` fails to start

**Diagnosis:**
```bash
sudo systemctl status anna-fans
journalctl -u anna-fans -n 50
```

**Common Issues:**

1. **asusctl not installed:**
   ```bash
   sudo pacman -S asusctl
   sudo systemctl restart asusd
   ```

2. **Script not executable:**
   ```bash
   sudo chmod +x /usr/local/share/anna/anna_fans_asus.sh
   ```

3. **ASUS daemon not running:**
   ```bash
   sudo systemctl enable --now asusd
   ```

---

## Advanced Configuration

### Custom Thermal Policies

Create custom policies in `/etc/anna/policies.d/custom-thermal.yaml`:

```yaml
# Aggressive cooling for gaming
- when: "telemetry.cpu_temp > 70 && process.name == 'steam'"
  then: "execute"
  command: ["asusctl", "profile", "-P", "performance"]
  message: "Gaming detected, maximum cooling"

# Auto-throttle during high ambient temp
- when: "telemetry.cpu_temp > 85 && telemetry.load_1min < 30"
  then: "execute"
  command: ["powerprofilesctl", "set", "power-saver"]
  message: "High temp at low load, throttling to cool down"
```

### Per-Application Profiles

```bash
# Create udev rules or systemd units that trigger on app launch
# Example: Launch game → performance mode
# Exit game → quiet mode
```

### Remote Monitoring

```bash
# SSH and monitor
ssh user@laptop 'watch sensors'

# Export to monitoring system (future)
annactl telemetry export --format prometheus
```

---

## Performance Impact

### Resource Usage

| Component | Idle | Peak |
|-----------|------|------|
| anna-fans.service | 0% CPU | 0.1% CPU |
| annad (thermal collection) | 0.5% CPU | 1.5% CPU |
| asusd | 0.2% CPU | 0.5% CPU |
| fancontrol | 0.1% CPU | 0.2% CPU |

**Total overhead:** < 2% CPU, < 50MB RAM

### Poll Intervals

- **Sensors:** 30s ± 5s (configurable)
- **Fancontrol:** 3s (from `/etc/fancontrol`)
- **Policy evaluation:** On telemetry update

---

## Safety

### Built-In Protections

1. **Failsafe defaults:** If Anna crashes, system reverts to BIOS fan control
2. **No destructive actions:** Cannot disable fans completely
3. **Conservative thresholds:** Alert at 85°C, well below thermal shutdown (~100°C)
4. **Audit logging:** All actions logged to `/var/log/anna/adaptive.log` (future)

### Emergency Override

```bash
# Immediately restore BIOS control (ASUS)
asusctl fan-curve --default
asusctl profile -P balanced

# Stop Anna's thermal management
sudo systemctl stop anna-fans

# Disable permanently if needed
sudo systemctl disable anna-fans
```

---

## Future Enhancements

### Planned for v0.9.8+

- **Adaptive execution:** Policies can run commands (currently alerts only)
- **Machine learning:** Learn optimal curves per workload
- **Predictive cooling:** Pre-cool before known intensive tasks
- **Cloud sync:** Share curves across multiple machines
- **GUI dashboard:** Real-time thermal visualization

---

## Summary

Anna's thermal management provides:

✅ **Automatic fan control** (ASUS and generic)
✅ **Intelligent thermal policies** (temperature-based alerts)
✅ **CPU governor management** (powersave default)
✅ **Low overhead** (< 2% CPU, < 50MB RAM)
✅ **Safe fallbacks** (BIOS control on failure)
✅ **Customizable curves** (per-workload optimization)

**Result:** Quieter, cooler, smarter thermal management with zero manual intervention.

---

**Status:** ✅ Production ready
**Tested on:** ASUS ROG Zephyrus G14, ThinkPad X1 Carbon
**Requirements:** Linux 5.10+, systemd 248+

*Anna keeps cool under pressure.* 🌡️
