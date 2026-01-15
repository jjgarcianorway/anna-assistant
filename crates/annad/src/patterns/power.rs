//! Power management patterns for battery, suspend, hibernate, laptop power.
//! v0.0.960: Initial implementation.
//! v0.0.989: Added power button, WoL, power saving, auto suspend patterns

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
    // Phase 15: Power control patterns take priority (disable/prevent sleep)
    match_power_control(q)
        .or_else(|| match_battery(q))
        .or_else(|| match_power_state(q))
        .or_else(|| match_laptop(q))
        .or_else(|| match_power_settings(q))
        .or_else(|| match_power_advanced(q))
}

/// Phase 15: Power control patterns - disable/prevent sleep/suspend
/// These questions route to Power specialist, NOT Desktop.
fn match_power_control(q: &str) -> Option<DeepUnderstanding> {
    let q_lower = q.to_lowercase();

    // Keywords indicating power control intent
    let disable_keywords = ["disable", "prevent", "stop", "block", "never", "no more", "turn off", "cannot", "can't", "don't"];
    let power_keywords = ["sleep", "suspend", "hibernate", "idle", "power", "lid"];

    // Check if this is a power control question
    let has_disable = disable_keywords.iter().any(|k| q_lower.contains(k));
    let has_power = power_keywords.iter().any(|k| q_lower.contains(k));

    if !has_disable || !has_power {
        return None;
    }

    // Specific patterns for disable sleep/suspend
    let patterns: &[(&[&str], &str, &[&str])] = &[
        // Disable all sleep/suspend
        (&["disable", "sleep"], "disable system sleep across all layers", &[
            "cat /etc/systemd/logind.conf",
            "cat /etc/systemd/sleep.conf 2>/dev/null",
            "systemd-inhibit --list",
            "loginctl show-seat seat0 | grep -i idle",
        ]),
        (&["disable", "suspend"], "disable system suspend across all layers", &[
            "cat /etc/systemd/logind.conf",
            "systemctl status sleep.target suspend.target",
            "systemd-inhibit --list",
        ]),
        (&["prevent", "sleep"], "prevent system from sleeping", &[
            "cat /etc/systemd/logind.conf | grep -i idle",
            "cat /etc/systemd/sleep.conf 2>/dev/null",
            "systemd-inhibit --list",
        ]),
        (&["prevent", "suspend"], "prevent system from suspending", &[
            "cat /etc/systemd/logind.conf",
            "systemctl status suspend.target",
        ]),
        // Lid close behavior
        (&["lid", "close"], "configure lid close behavior", &[
            "cat /etc/systemd/logind.conf | grep -i lid",
            "loginctl show | grep -i lid",
        ]),
        (&["close", "lid"], "configure lid close behavior", &[
            "cat /etc/systemd/logind.conf | grep -i lid",
        ]),
        // Never sleep
        (&["never", "sleep"], "disable all sleep modes", &[
            "cat /etc/systemd/logind.conf",
            "cat /etc/systemd/sleep.conf 2>/dev/null",
            "systemctl mask sleep.target suspend.target hibernate.target hybrid-sleep.target",
        ]),
        // GDM specific
        (&["gdm", "sleep"], "configure GDM sleep behavior", &[
            "cat /etc/gdm/custom.conf 2>/dev/null",
            "cat /etc/systemd/logind.conf | grep -i idle",
        ]),
        (&["gdm", "suspend"], "configure GDM suspend behavior", &[
            "cat /etc/gdm/custom.conf 2>/dev/null",
            "cat /etc/systemd/logind.conf",
        ]),
        // Idle timeout
        (&["idle", "timeout"], "configure idle timeout", &[
            "cat /etc/systemd/logind.conf | grep -i idle",
            "loginctl show | grep -i idle",
        ]),
        (&["disable", "idle"], "disable idle actions", &[
            "cat /etc/systemd/logind.conf | grep -i idle",
        ]),
    ];

    for (keywords, desc, commands) in patterns {
        if keywords.iter().all(|k| q_lower.contains(k)) {
            return Some(DeepUnderstanding {
                interpreted_as: desc.to_string(),
                category: IntentCategory::HowTo, // This is a configuration task
                confidence: 0.95,
                topic: Some("power".to_string()),
                needs_confirmation: true, // Changing power policy needs confirmation
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }

    // Generic disable power management
    if has_disable && has_power {
        return Some(DeepUnderstanding {
            interpreted_as: "configure power management policy".to_string(),
            category: IntentCategory::HowTo,
            confidence: 0.85,
            topic: Some("power".to_string()),
            needs_confirmation: true,
            suggested_commands: vec![
                "cat /etc/systemd/logind.conf".to_string(),
                "cat /etc/systemd/sleep.conf 2>/dev/null".to_string(),
                "systemctl status sleep.target suspend.target".to_string(),
                "systemd-inhibit --list".to_string(),
            ],
            ..Default::default()
        });
    }

    None
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
        // Suspend settings
        (&["suspend", "settings"], "show suspend settings", "power",
         &["cat /sys/power/mem_sleep", "cat /etc/systemd/sleep.conf 2>/dev/null | grep -v '^#' | grep -v '^$'"]),
        // Hibernate config
        (&["hibernate", "config"], "show hibernate configuration", "power",
         &["cat /sys/power/disk", "grep swap /etc/fstab",
           "cat /etc/mkinitcpio.conf | grep resume"]),
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
        (&["power", "profiles"], "list power profiles", "power",
         &["powerprofilesctl list 2>/dev/null || echo 'power-profiles-daemon not installed'"]),
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
        // Thermal throttling
        (&["thermal", "throttling"], "check thermal throttling", "power",
         &["dmesg | grep -i throttl", "journalctl -b | grep -i 'thermal\\|throttl' | tail -15"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Advanced power patterns
fn match_power_advanced(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PowerPattern] = &[
        // Power button action
        (&["power", "button"], "show power button action", "power",
         &["cat /etc/systemd/logind.conf | grep -i powerkey",
           "echo 'Configure in /etc/systemd/logind.conf: HandlePowerKey='"]),
        (&["power", "button", "action"], "configure power button action", "power",
         &["cat /etc/systemd/logind.conf | grep -i handle",
           "echo 'Options: ignore, poweroff, reboot, halt, suspend, hibernate'"]),
        // Lid switch
        (&["lid", "switch", "action"], "show lid switch action", "power",
         &["cat /etc/systemd/logind.conf | grep -i lid",
           "loginctl show | grep -i lid"]),
        // Wake on LAN
        (&["wake", "on", "lan"], "check Wake on LAN status", "power",
         &["sudo ethtool <interface> | grep -i wake",
           "echo 'Enable: sudo ethtool -s <interface> wol g'"]),
        (&["wol", "status"], "check WoL status", "power",
         &["ip link show", "sudo ethtool eth0 2>/dev/null | grep -i wake || echo 'Check interface name'"]),
        // Power saving tips
        (&["power", "saving"], "power saving tips", "power",
         &["echo 'Install TLP: sudo pacman -S tlp && sudo systemctl enable --now tlp'",
           "echo 'Use powertop: sudo pacman -S powertop && sudo powertop --auto-tune'",
           "echo 'Reduce brightness, disable Bluetooth/WiFi when not needed'"]),
        (&["save", "power"], "how to save power", "power",
         &["echo 'TLP for laptops: sudo pacman -S tlp'",
           "echo 'Check current: powertop'"]),
        // Auto suspend
        (&["auto", "suspend"], "configure auto suspend", "power",
         &["cat /etc/systemd/logind.conf | grep -i idle",
           "echo 'IdleAction=suspend in /etc/systemd/logind.conf'",
           "echo 'IdleActionSec=30min'"]),
        // Battery calibration
        (&["battery", "calibration"], "battery calibration info", "power",
         &["echo 'To calibrate:'",
           "echo '1. Charge to 100%'",
           "echo '2. Drain to ~5% (don\\'t let it die)'",
           "echo '3. Charge to 100% uninterrupted'"]),
        // Power statistics
        (&["power", "statistics"], "show power statistics", "power",
         &["upower -d", "upower --dump"]),
        (&["power", "stats"], "show power stats", "power",
         &["upower -i /org/freedesktop/UPower/devices/battery_BAT0"]),
        // Powertop
        (&["powertop"], "powertop power analysis", "power",
         &["sudo powertop 2>/dev/null || echo 'Install: sudo pacman -S powertop'"]),
        // Sleep inhibitors
        (&["sleep", "inhibitor"], "show sleep inhibitors", "power",
         &["systemd-inhibit --list"]),
        (&["prevent", "sleep"], "check what prevents sleep", "power",
         &["systemd-inhibit --list", "cat /proc/acpi/wakeup"]),
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
        assert!(match_patterns("thermal throttling").is_some());
    }

    #[test]
    fn test_power_advanced() {
        assert!(match_patterns("power button action").is_some());
        assert!(match_patterns("wake on lan").is_some());
        assert!(match_patterns("power saving").is_some());
        assert!(match_patterns("auto suspend").is_some());
        assert!(match_patterns("battery calibration").is_some());
        assert!(match_patterns("power statistics").is_some());
    }

    // Phase 15: Power control tests
    #[test]
    fn test_disable_sleep_everywhere() {
        let result = match_patterns("disable sleep everywhere");
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.topic, Some("power".to_string()));
        assert!(u.needs_confirmation);
    }

    #[test]
    fn test_prevent_suspend_on_lid_close() {
        let result = match_patterns("prevent suspend on lid close");
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.topic, Some("power".to_string()));
    }

    #[test]
    fn test_never_sleep_even_on_gdm() {
        let result = match_patterns("never sleep even on GDM");
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.topic, Some("power".to_string()));
    }

    #[test]
    fn test_laptop_cannot_sleep_or_suspend() {
        let result = match_patterns("ensure my laptop cannot go to sleep or suspend");
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.topic, Some("power".to_string()));
    }
}
