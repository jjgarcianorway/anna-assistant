//! Hardware patterns - sensors, temperatures, battery, CPU, and diagnostics
//! v0.0.949: Initial hardware patterns for system monitoring

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Pattern with keywords, description, topic, and command templates
type HardwarePattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

/// Match common hardware-related questions
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // Temperature/thermal
    if let Some(u) = match_temperature(q) {
        return Some(u);
    }
    // Battery
    if let Some(u) = match_battery(q) {
        return Some(u);
    }
    // CPU
    if let Some(u) = match_cpu(q) {
        return Some(u);
    }
    // Storage/disks
    if let Some(u) = match_storage(q) {
        return Some(u);
    }
    // Devices
    if let Some(u) = match_devices(q) {
        return Some(u);
    }
    None
}

fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::Factual,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

/// Temperature and thermal queries
fn match_temperature(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HardwarePattern] = &[
        // CPU temperature
        (&["cpu", "temp"], "check CPU temperature", "sensors",
            &["sensors 2>/dev/null || echo 'Install: pacman -S lm_sensors && sensors-detect'",
              "cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null | awk '{print $1/1000\"°C\"}'",
              "cat /sys/class/hwmon/hwmon*/temp*_input 2>/dev/null | head -5"]),
        (&["cpu", "hot"], "check if CPU is hot", "sensors",
            &["sensors 2>/dev/null | grep -i temp", "cat /sys/class/thermal/thermal_zone0/temp"]),
        (&["processor", "temp"], "check processor temperature", "sensors",
            &["sensors 2>/dev/null | grep -E 'Core|Package'",
              "cat /sys/class/thermal/thermal_zone*/type /sys/class/thermal/thermal_zone*/temp 2>/dev/null"]),
        // GPU temperature
        (&["gpu", "temp"], "check GPU temperature", "sensors",
            &["nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader 2>/dev/null || echo 'No NVIDIA GPU'",
              "sensors 2>/dev/null | grep -i gpu", "cat /sys/class/drm/card0/device/hwmon/hwmon*/temp1_input 2>/dev/null"]),
        (&["graphics", "temp"], "check graphics temperature", "sensors",
            &["nvidia-smi 2>/dev/null | grep -i temp || sensors 2>/dev/null | grep -i gpu"]),
        // General temperature
        (&["system", "temp"], "check system temperatures", "sensors",
            &["sensors 2>/dev/null || echo 'Install lm_sensors'", "cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null"]),
        (&["all", "temp"], "show all temperatures", "sensors",
            &["sensors 2>/dev/null", "cat /sys/class/thermal/thermal_zone*/type /sys/class/thermal/thermal_zone*/temp 2>/dev/null"]),
        (&["sensor"], "show sensor readings", "sensors",
            &["sensors 2>/dev/null || echo 'Install: pacman -S lm_sensors'"]),
        // Fan speed
        (&["fan", "speed"], "check fan speed", "sensors",
            &["sensors 2>/dev/null | grep -i fan", "cat /sys/class/hwmon/hwmon*/fan*_input 2>/dev/null"]),
        (&["fan", "rpm"], "check fan RPM", "sensors",
            &["sensors 2>/dev/null | grep -i fan"]),
        // Thermal throttling
        (&["thermal", "throttl"], "check thermal throttling", "sensors",
            &["dmesg | grep -i thermal | tail -10", "journalctl -k | grep -i thermal | tail -10"]),
        (&["overheating"], "check for overheating", "sensors",
            &["sensors 2>/dev/null", "dmesg | grep -i 'thermal\\|overheat' | tail -10"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Battery queries
fn match_battery(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HardwarePattern] = &[
        // Battery status
        (&["battery", "status"], "check battery status", "power",
            &["upower -i /org/freedesktop/UPower/devices/battery_BAT0 2>/dev/null",
              "cat /sys/class/power_supply/BAT*/status /sys/class/power_supply/BAT*/capacity 2>/dev/null"]),
        (&["battery", "level"], "check battery level", "power",
            &["cat /sys/class/power_supply/BAT*/capacity 2>/dev/null",
              "upower -i /org/freedesktop/UPower/devices/battery_BAT0 2>/dev/null | grep percentage"]),
        (&["battery", "percent"], "check battery percentage", "power",
            &["cat /sys/class/power_supply/BAT*/capacity 2>/dev/null"]),
        (&["battery", "health"], "check battery health", "power",
            &["upower -i /org/freedesktop/UPower/devices/battery_BAT0 2>/dev/null | grep -E 'capacity|energy-full'",
              "cat /sys/class/power_supply/BAT*/charge_full /sys/class/power_supply/BAT*/charge_full_design 2>/dev/null"]),
        (&["battery", "charg"], "check if battery is charging", "power",
            &["cat /sys/class/power_supply/BAT*/status 2>/dev/null",
              "upower -i /org/freedesktop/UPower/devices/battery_BAT0 2>/dev/null | grep state"]),
        (&["battery", "time"], "check battery time remaining", "power",
            &["upower -i /org/freedesktop/UPower/devices/battery_BAT0 2>/dev/null | grep -E 'time to|percentage'",
              "acpi -b 2>/dev/null || echo 'Install: pacman -S acpi'"]),
        // Power
        (&["power", "usage"], "check power usage", "power",
            &["powertop --time=1 --csv=/tmp/powertop.csv 2>/dev/null && head -30 /tmp/powertop.csv",
              "cat /sys/class/power_supply/BAT*/power_now 2>/dev/null"]),
        (&["power", "consum"], "check power consumption", "power",
            &["cat /sys/class/power_supply/BAT*/power_now 2>/dev/null",
              "upower -d 2>/dev/null | grep -E 'energy-rate|power'"]),
        // AC adapter
        (&["ac", "connect"], "check AC connection", "power",
            &["cat /sys/class/power_supply/AC*/online 2>/dev/null",
              "upower -i /org/freedesktop/UPower/devices/line_power_AC 2>/dev/null"]),
        (&["plugged", "in"], "check if plugged in", "power",
            &["cat /sys/class/power_supply/AC*/online /sys/class/power_supply/BAT*/status 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// CPU queries
fn match_cpu(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HardwarePattern] = &[
        // CPU frequency
        (&["cpu", "freq"], "check CPU frequency", "cpu",
            &["lscpu | grep -i mhz", "cat /proc/cpuinfo | grep MHz | head -4",
              "cpupower frequency-info 2>/dev/null | grep -i 'current cpu'"]),
        (&["cpu", "speed"], "check CPU speed", "cpu",
            &["lscpu | grep -i mhz", "cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq 2>/dev/null | head -4"]),
        (&["clock", "speed"], "check clock speed", "cpu",
            &["cat /proc/cpuinfo | grep MHz | head -4"]),
        // CPU governor
        (&["cpu", "governor"], "check CPU governor", "cpu",
            &["cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor | head -1",
              "cpupower frequency-info 2>/dev/null | grep -i governor"]),
        (&["power", "mode"], "check power mode", "cpu",
            &["cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor | head -1",
              "powerprofilesctl get 2>/dev/null || echo 'powerprofilesctl not available'"]),
        // CPU usage
        (&["cpu", "usage"], "check CPU usage", "cpu",
            &["top -bn1 | head -5", "mpstat 1 1 2>/dev/null || echo 'Install: pacman -S sysstat'"]),
        (&["cpu", "load"], "check CPU load", "cpu",
            &["uptime", "cat /proc/loadavg"]),
        // CPU info
        (&["cpu", "info"], "show CPU information", "cpu",
            &["lscpu | head -20", "cat /proc/cpuinfo | head -30"]),
        (&["cpu", "model"], "show CPU model", "cpu",
            &["lscpu | grep -E 'Model name|Architecture|CPU MHz'", "cat /proc/cpuinfo | grep 'model name' | head -1"]),
        (&["cpu", "core"], "show CPU cores", "cpu",
            &["lscpu | grep -E 'CPU\\(s\\)|Core|Thread'", "nproc"]),
        // CPU architecture
        (&["cpu", "arch"], "check CPU architecture", "cpu",
            &["uname -m", "lscpu | grep Architecture"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Storage and disk queries
fn match_storage(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HardwarePattern] = &[
        // Disk health
        (&["disk", "health"], "check disk health", "storage",
            &["sudo smartctl -a /dev/sda 2>/dev/null | head -30 || echo 'Install: pacman -S smartmontools'",
              "sudo smartctl -H /dev/sda 2>/dev/null"]),
        (&["smart", "status"], "check SMART status", "storage",
            &["sudo smartctl -H /dev/sda 2>/dev/null",
              "sudo smartctl -a /dev/sda 2>/dev/null | grep -E 'SMART|Health'"]),
        (&["drive", "health"], "check drive health", "storage",
            &["sudo smartctl -H /dev/sda 2>/dev/null || echo 'Run: pacman -S smartmontools'"]),
        // Disk info
        (&["disk", "info"], "show disk information", "storage",
            &["lsblk -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT", "df -Th"]),
        (&["drive", "info"], "show drive information", "storage",
            &["lsblk -d -o NAME,SIZE,MODEL,SERIAL", "sudo hdparm -I /dev/sda 2>/dev/null | head -20"]),
        // SSD trim
        (&["ssd", "trim"], "check SSD TRIM", "storage",
            &["systemctl status fstrim.timer", "cat /etc/fstab | grep discard"]),
        (&["trim", "status"], "check TRIM status", "storage",
            &["lsblk -D", "systemctl status fstrim.timer 2>/dev/null"]),
        // NVMe
        (&["nvme", "info"], "show NVMe information", "storage",
            &["sudo nvme list 2>/dev/null || echo 'Install: pacman -S nvme-cli'",
              "sudo nvme smart-log /dev/nvme0 2>/dev/null"]),
        (&["nvme", "temp"], "check NVMe temperature", "storage",
            &["sudo nvme smart-log /dev/nvme0 2>/dev/null | grep -i temp"]),
        // Disk I/O
        (&["disk", "io"], "check disk I/O", "storage",
            &["iostat -x 1 1 2>/dev/null || echo 'Install: pacman -S sysstat'", "iotop -obn1 2>/dev/null | head -10"]),
        (&["io", "stat"], "show I/O statistics", "storage",
            &["iostat -x 1 1 2>/dev/null", "cat /proc/diskstats | head -10"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Device queries
fn match_devices(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[HardwarePattern] = &[
        // PCI devices
        (&["pci", "device"], "list PCI devices", "devices",
            &["lspci", "lspci -k | head -40"]),
        (&["list", "hardware"], "list hardware devices", "devices",
            &["lspci", "lsusb", "lsblk"]),
        // USB devices
        (&["usb", "device"], "list USB devices", "devices",
            &["lsusb", "lsusb -t"]),
        (&["list", "usb"], "list USB devices", "devices",
            &["lsusb -v 2>/dev/null | head -50", "lsusb -t"]),
        // Input devices
        (&["input", "device"], "list input devices", "devices",
            &["xinput list 2>/dev/null || cat /proc/bus/input/devices | head -30"]),
        (&["list", "input"], "list input devices", "devices",
            &["cat /proc/bus/input/devices | head -40"]),
        // Audio devices
        (&["audio", "device"], "list audio devices", "devices",
            &["aplay -l", "pactl list sinks short 2>/dev/null || wpctl status 2>/dev/null"]),
        (&["sound", "card"], "list sound cards", "devices",
            &["aplay -l", "cat /proc/asound/cards"]),
        // Block devices
        (&["block", "device"], "list block devices", "devices",
            &["lsblk", "lsblk -f"]),
        // Detect hardware
        (&["detect", "hardware"], "detect hardware", "devices",
            &["lspci -nn", "lsusb", "dmesg | tail -30"]),
        (&["new", "hardware"], "check for new hardware", "devices",
            &["dmesg | tail -30", "journalctl -b | grep -i 'new\\|detected' | tail -20"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature() {
        assert!(match_patterns("cpu temperature").is_some());
        assert!(match_patterns("gpu temp").is_some());
        assert!(match_patterns("fan speed").is_some());
    }

    #[test]
    fn test_battery() {
        assert!(match_patterns("battery status").is_some());
        assert!(match_patterns("battery level").is_some());
        assert!(match_patterns("battery charging").is_some());
    }

    #[test]
    fn test_cpu() {
        assert!(match_patterns("cpu frequency").is_some());
        assert!(match_patterns("cpu usage").is_some());
        assert!(match_patterns("cpu info").is_some());
    }

    #[test]
    fn test_storage() {
        assert!(match_patterns("disk health").is_some());
        assert!(match_patterns("smart status").is_some());
        assert!(match_patterns("nvme info").is_some());
    }

    #[test]
    fn test_devices() {
        assert!(match_patterns("pci devices").is_some());
        assert!(match_patterns("usb devices").is_some());
        assert!(match_patterns("audio devices").is_some());
    }
}
