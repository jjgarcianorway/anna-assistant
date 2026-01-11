//! Power management patterns for battery, suspend, hibernate, laptop power.
//! v0.0.960: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a power-related DeepUnderstanding
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

type PowerPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match power-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_battery(q)
        .or_else(|| match_power_state(q))
        .or_else(|| match_laptop(q))
        .or_else(|| match_power_settings(q))
}

/// Battery patterns
fn match_battery(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PowerPattern] = &[
        // Battery status
        (&["battery", "status"], "show battery status", "power",
         &["cat /sys/class/power_supply/BAT*/status", "upower -i /org/freedesktop/UPower/devices/battery_BAT0"]),
        (&["battery", "level"], "show battery level", "power",
         &["cat /sys/class/power_supply/BAT*/capacity", "upower -i /org/freedesktop/UPower/devices/battery_BAT0 | grep percentage"]),
        (&["battery", "health"], "show battery health", "power",
         &["upower -i /org/freedesktop/UPower/devices/battery_BAT0 | grep -E 'capacity|energy-full'"]),
        (&["battery", "info"], "show battery information", "power",
         &["upower -i /org/freedesktop/UPower/devices/battery_BAT0"]),
        (&["battery", "percentage"], "show battery percentage", "power",
         &["cat /sys/class/power_supply/BAT*/capacity"]),
        // Charging status
        (&["charging", "status"], "show charging status", "power",
         &["cat /sys/class/power_supply/*/status"]),
        (&["is", "charging"], "check if charging", "power",
         &["cat /sys/class/power_supply/BAT*/status"]),
        (&["power", "source"], "show power source", "power",
         &["cat /sys/class/power_supply/*/online", "upower -e | xargs -I{} upower -i {}"]),
        // Battery time remaining
        (&["battery", "time"], "show battery time remaining", "power",
         &["upower -i /org/freedesktop/UPower/devices/battery_BAT0 | grep 'time to'"]),
        (&["time", "remaining"], "show time remaining on battery", "power",
         &["upower -i /org/freedesktop/UPower/devices/battery_BAT0 | grep 'time to'"]),
        // Battery list
        (&["batteries"], "list batteries", "power",
         &["ls /sys/class/power_supply/ | grep BAT", "upower -e | grep battery"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Power state patterns (suspend, hibernate, sleep)
fn match_power_state(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PowerPattern] = &[
        // Suspend
        (&["suspend", "mode"], "show suspend modes", "power",
         &["cat /sys/power/mem_sleep"]),
        (&["suspend", "support"], "check suspend support", "power",
         &["cat /sys/power/state"]),
        (&["sleep", "modes"], "show available sleep modes", "power",
         &["cat /sys/power/state", "cat /sys/power/mem_sleep"]),
        // Hibernate
        (&["hibernate", "support"], "check hibernate support", "power",
         &["cat /sys/power/state | grep disk", "cat /sys/power/disk"]),
        (&["swap", "hibernate"], "check swap for hibernate", "power",
         &["cat /proc/swaps", "swapon --show"]),
        // Resume issues
        (&["resume", "issues"], "check for resume issues", "power",
         &["journalctl -b | grep -iE 'resume|suspend|sleep' | tail -20"]),
        (&["suspend", "logs"], "show suspend logs", "power",
         &["journalctl -b | grep -iE 'suspend|PM:' | tail -30"]),
        (&["wake", "issues"], "check wake from sleep issues", "power",
         &["journalctl -b | grep -iE 'wake|resume' | tail -20"]),
        // Last suspend
        (&["last", "suspend"], "show last suspend", "power",
         &["journalctl -b | grep -i 'suspend' | tail -10"]),
        (&["last", "sleep"], "show last sleep event", "power",
         &["journalctl -b | grep -i 'suspend\\|sleep' | tail -10"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Laptop-specific patterns
fn match_laptop(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PowerPattern] = &[
        // Lid
        (&["lid", "action"], "show lid close action", "power",
         &["cat /etc/systemd/logind.conf | grep -i lid", "loginctl show | grep -i lid"]),
        (&["lid", "switch"], "show lid switch status", "power",
         &["cat /proc/acpi/button/lid/*/state 2>/dev/null || loginctl show | grep -i lid"]),
        // Screen brightness
        (&["screen", "brightness"], "show screen brightness", "power",
         &["cat /sys/class/backlight/*/brightness", "cat /sys/class/backlight/*/actual_brightness"]),
        (&["brightness", "level"], "show brightness level", "power",
         &["cat /sys/class/backlight/*/brightness", "cat /sys/class/backlight/*/max_brightness"]),
        (&["backlight"], "show backlight info", "power",
         &["ls /sys/class/backlight/", "cat /sys/class/backlight/*/brightness"]),
        // Fan
        (&["fan", "speed"], "show fan speed", "power",
         &["sensors | grep -i fan", "cat /sys/class/hwmon/*/fan*_input 2>/dev/null"]),
        (&["fan", "control"], "show fan control info", "power",
         &["sensors | grep -i fan", "cat /sys/class/hwmon/*/pwm* 2>/dev/null | head -5"]),
        // Thermal
        (&["thermal", "zones"], "show thermal zones", "power",
         &["cat /sys/class/thermal/thermal_zone*/type", "cat /sys/class/thermal/thermal_zone*/temp"]),
        (&["cpu", "throttle"], "check CPU throttling", "power",
         &["dmesg | grep -i throttle", "journalctl -b | grep -i throttle | tail -10"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Power settings patterns
fn match_power_settings(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PowerPattern] = &[
        // TLP
        (&["tlp", "status"], "show TLP status", "power",
         &["tlp-stat -s 2>/dev/null || echo 'TLP not installed'"]),
        (&["tlp", "config"], "show TLP config", "power",
         &["tlp-stat -c 2>/dev/null || echo 'TLP not installed'"]),
        // Power profiles
        (&["power", "profile"], "show power profile", "power",
         &["powerprofilesctl get 2>/dev/null || echo 'power-profiles-daemon not installed'"]),
        (&["performance", "mode"], "check performance mode", "power",
         &["powerprofilesctl get", "cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor | uniq"]),
        // CPU governor
        (&["cpu", "governor"], "show CPU governor", "power",
         &["cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor | uniq"]),
        (&["available", "governors"], "list available CPU governors", "power",
         &["cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors"]),
        // Power consumption
        (&["power", "consumption"], "show power consumption", "power",
         &["upower -i /org/freedesktop/UPower/devices/battery_BAT0 | grep 'energy-rate'"]),
        (&["power", "draw"], "show power draw", "power",
         &["cat /sys/class/power_supply/BAT*/power_now 2>/dev/null"]),
        // ACPI
        (&["acpi", "info"], "show ACPI info", "power",
         &["acpi -V"]),
        (&["acpi", "events"], "show ACPI events", "power",
         &["journalctl -b | grep -i acpi | tail -20"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery() {
        assert!(match_patterns("battery status").is_some());
        assert!(match_patterns("battery level").is_some());
        assert!(match_patterns("charging status").is_some());
        assert!(match_patterns("battery health").is_some());
    }

    #[test]
    fn test_power_state() {
        assert!(match_patterns("suspend mode").is_some());
        assert!(match_patterns("hibernate support").is_some());
        assert!(match_patterns("sleep modes").is_some());
    }

    #[test]
    fn test_laptop() {
        assert!(match_patterns("screen brightness").is_some());
        assert!(match_patterns("fan speed").is_some());
        assert!(match_patterns("thermal zones").is_some());
    }

    #[test]
    fn test_power_settings() {
        assert!(match_patterns("tlp status").is_some());
        assert!(match_patterns("power profile").is_some());
        assert!(match_patterns("cpu governor").is_some());
    }
}
