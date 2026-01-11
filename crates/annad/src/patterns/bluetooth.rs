//! Bluetooth device and connection patterns.
//! v0.0.972: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a Bluetooth-related DeepUnderstanding
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

type BtPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match Bluetooth-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_bluetooth_status(q)
        .or_else(|| match_bluetooth_devices(q))
        .or_else(|| match_bluetooth_audio(q))
        .or_else(|| match_bluetooth_troubleshoot(q))
}

/// Bluetooth status patterns
fn match_bluetooth_status(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BtPattern] = &[
        // Bluetooth status
        (&["bluetooth", "status"], "show Bluetooth status", "bluetooth",
         &["bluetoothctl show", "systemctl status bluetooth"]),
        (&["bluetooth", "enabled"], "check if Bluetooth is enabled", "bluetooth",
         &["bluetoothctl show | grep Powered", "rfkill list bluetooth"]),
        (&["bluetooth", "on"], "check if Bluetooth is on", "bluetooth",
         &["bluetoothctl show | grep Powered"]),
        (&["bluetooth", "off"], "check if Bluetooth is off", "bluetooth",
         &["bluetoothctl show | grep Powered", "rfkill list bluetooth"]),
        // Bluetooth service
        (&["bluetooth", "service"], "show Bluetooth service status", "bluetooth",
         &["systemctl status bluetooth"]),
        (&["bluetooth", "running"], "check if Bluetooth is running", "bluetooth",
         &["systemctl is-active bluetooth"]),
        // Bluetooth adapter
        (&["bluetooth", "adapter"], "show Bluetooth adapter info", "bluetooth",
         &["bluetoothctl show", "hciconfig -a 2>/dev/null"]),
        (&["bluetooth", "controller"], "show Bluetooth controller", "bluetooth",
         &["bluetoothctl show"]),
        // Bluetooth version
        (&["bluetooth", "version"], "show Bluetooth version", "bluetooth",
         &["bluetoothctl --version", "hciconfig -a 2>/dev/null | grep -i version"]),
        // Rfkill
        (&["rfkill", "bluetooth"], "show rfkill Bluetooth status", "bluetooth",
         &["rfkill list bluetooth"]),
        (&["bluetooth", "blocked"], "check if Bluetooth is blocked", "bluetooth",
         &["rfkill list bluetooth"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Bluetooth device patterns
fn match_bluetooth_devices(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BtPattern] = &[
        // Paired devices
        (&["paired", "devices"], "show paired Bluetooth devices", "bluetooth",
         &["bluetoothctl devices Paired", "bluetoothctl devices"]),
        (&["bluetooth", "paired"], "show paired Bluetooth devices", "bluetooth",
         &["bluetoothctl devices Paired"]),
        // Connected devices
        (&["connected", "bluetooth"], "show connected Bluetooth devices", "bluetooth",
         &["bluetoothctl devices Connected", "bluetoothctl info"]),
        (&["bluetooth", "connected"], "show connected Bluetooth devices", "bluetooth",
         &["bluetoothctl devices Connected"]),
        // All devices
        (&["bluetooth", "devices"], "list Bluetooth devices", "bluetooth",
         &["bluetoothctl devices"]),
        (&["list", "bluetooth"], "list Bluetooth devices", "bluetooth",
         &["bluetoothctl devices"]),
        // Nearby devices
        (&["nearby", "bluetooth"], "scan for nearby Bluetooth devices", "bluetooth",
         &["echo 'Use: bluetoothctl scan on (then: devices)'", "bluetoothctl devices"]),
        (&["discover", "bluetooth"], "discover Bluetooth devices", "bluetooth",
         &["echo 'Use: bluetoothctl scan on'"]),
        // Device info
        (&["bluetooth", "device", "info"], "show Bluetooth device info", "bluetooth",
         &["echo 'Use: bluetoothctl info <MAC>'"]),
        // Trusted devices
        (&["trusted", "bluetooth"], "show trusted Bluetooth devices", "bluetooth",
         &["bluetoothctl devices Trusted"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Bluetooth audio patterns
fn match_bluetooth_audio(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BtPattern] = &[
        // Bluetooth headphones
        (&["bluetooth", "headphones"], "show Bluetooth headphones status", "bluetooth",
         &["bluetoothctl devices Connected", "pactl list cards short | grep -i blue"]),
        (&["bluetooth", "earbuds"], "show Bluetooth earbuds status", "bluetooth",
         &["bluetoothctl devices Connected", "pactl list cards short | grep -i blue"]),
        // Bluetooth audio
        (&["bluetooth", "audio"], "show Bluetooth audio status", "bluetooth",
         &["pactl list cards short | grep -i blue", "pactl list sinks short | grep -i blue"]),
        (&["bluetooth", "speaker"], "show Bluetooth speaker status", "bluetooth",
         &["bluetoothctl devices Connected", "pactl list sinks short | grep -i blue"]),
        // A2DP
        (&["a2dp", "profile"], "show A2DP profile status", "bluetooth",
         &["pactl list cards | grep -A 20 bluez | grep -E 'profile|Active'"]),
        (&["bluetooth", "a2dp"], "check Bluetooth A2DP", "bluetooth",
         &["pactl list cards | grep -A 20 bluez"]),
        // HSP/HFP
        (&["bluetooth", "microphone"], "check Bluetooth microphone", "bluetooth",
         &["pactl list sources short | grep -i blue"]),
        (&["bluetooth", "headset"], "show Bluetooth headset status", "bluetooth",
         &["bluetoothctl devices Connected", "pactl list cards | grep -A 10 bluez"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Bluetooth troubleshooting patterns
fn match_bluetooth_troubleshoot(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[BtPattern] = &[
        // Bluetooth logs
        (&["bluetooth", "logs"], "show Bluetooth logs", "bluetooth",
         &["journalctl -u bluetooth -n 30"]),
        (&["bluetooth", "errors"], "show Bluetooth errors", "bluetooth",
         &["journalctl -u bluetooth -p err -n 20"]),
        // Bluetooth not working
        (&["bluetooth", "not", "working"], "troubleshoot Bluetooth not working", "bluetooth",
         &["systemctl status bluetooth", "rfkill list", "bluetoothctl show"]),
        // Bluetooth connection issues
        (&["bluetooth", "connection", "issues"], "troubleshoot Bluetooth connection", "bluetooth",
         &["journalctl -u bluetooth -n 30", "bluetoothctl show"]),
        // Bluetooth firmware
        (&["bluetooth", "firmware"], "check Bluetooth firmware", "bluetooth",
         &["dmesg | grep -i bluetooth | tail -20"]),
        // Bluetooth driver
        (&["bluetooth", "driver"], "check Bluetooth driver", "bluetooth",
         &["lsmod | grep -i bt", "dmesg | grep -i bluetooth | tail -20"]),
        // Bluetooth kernel module
        (&["bluetooth", "module"], "show Bluetooth kernel modules", "bluetooth",
         &["lsmod | grep -iE 'bluetooth|btusb|btintel|btrtl'"]),
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
    fn test_bluetooth_status() {
        assert!(match_patterns("bluetooth status").is_some());
        assert!(match_patterns("bluetooth enabled").is_some());
        assert!(match_patterns("bluetooth adapter").is_some());
    }

    #[test]
    fn test_bluetooth_devices() {
        assert!(match_patterns("paired devices").is_some());
        assert!(match_patterns("bluetooth devices").is_some());
        assert!(match_patterns("connected bluetooth").is_some());
    }

    #[test]
    fn test_bluetooth_audio() {
        assert!(match_patterns("bluetooth headphones").is_some());
        assert!(match_patterns("bluetooth audio").is_some());
        assert!(match_patterns("bluetooth speaker").is_some());
    }

    #[test]
    fn test_bluetooth_troubleshoot() {
        assert!(match_patterns("bluetooth logs").is_some());
        assert!(match_patterns("bluetooth not working").is_some());
        assert!(match_patterns("bluetooth driver").is_some());
    }
}
