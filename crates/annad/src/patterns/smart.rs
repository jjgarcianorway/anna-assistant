//! SMART disk health patterns for smartctl and disk monitoring.
//! v0.0.986: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a SMART-related DeepUnderstanding
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

type SmartPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match SMART patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_smart_status(q)
        .or_else(|| match_smart_attributes(q))
        .or_else(|| match_smart_tests(q))
        .or_else(|| match_disk_health(q))
        .or_else(|| match_nvme(q))
}

/// SMART status patterns
fn match_smart_status(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SmartPattern] = &[
        // SMART status
        (&["smart", "status"], "show SMART status", "smart",
         &["sudo smartctl -H /dev/sda", "sudo smartctl --scan"]),
        (&["smartctl", "status"], "show smartctl status", "smart",
         &["sudo smartctl -H /dev/sda"]),
        // Disk health
        (&["disk", "health"], "check disk health", "smart",
         &["sudo smartctl -H /dev/sda", "lsblk -o NAME,SIZE,TYPE,MODEL"]),
        (&["drive", "health"], "check drive health", "smart",
         &["sudo smartctl -H /dev/sda"]),
        (&["ssd", "health"], "check SSD health", "smart",
         &["sudo smartctl -a /dev/sda | grep -E 'Wear|Life|Health'"]),
        (&["hdd", "health"], "check HDD health", "smart",
         &["sudo smartctl -H /dev/sda"]),
        // SMART overview
        (&["smart", "info"], "show SMART information", "smart",
         &["sudo smartctl -i /dev/sda"]),
        (&["smart", "overview"], "show SMART overview", "smart",
         &["sudo smartctl -a /dev/sda | head -50"]),
        // Scan devices
        (&["smart", "scan"], "scan for SMART devices", "smart",
         &["sudo smartctl --scan"]),
        (&["smartctl", "scan"], "scan SMART devices", "smart",
         &["sudo smartctl --scan"]),
        // SMART capable
        (&["smart", "capable"], "check SMART capability", "smart",
         &["sudo smartctl -i /dev/sda | grep -i 'smart support'"]),
        (&["smart", "support"], "check SMART support", "smart",
         &["sudo smartctl -i /dev/sda | grep -i 'smart support'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// SMART attributes patterns
fn match_smart_attributes(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SmartPattern] = &[
        // All attributes
        (&["smart", "attribute"], "show SMART attributes", "smart",
         &["sudo smartctl -A /dev/sda"]),
        (&["smartctl", "attribute"], "show smartctl attributes", "smart",
         &["sudo smartctl -A /dev/sda"]),
        // Critical attributes
        (&["smart", "critical"], "show critical SMART attributes", "smart",
         &["sudo smartctl -A /dev/sda | grep -iE 'reallocated|pending|uncorrectable|wear'"]),
        // Reallocated sectors
        (&["reallocated", "sector"], "check reallocated sectors", "smart",
         &["sudo smartctl -A /dev/sda | grep -i reallocated"]),
        (&["bad", "sector"], "check for bad sectors", "smart",
         &["sudo smartctl -A /dev/sda | grep -iE 'reallocated|pending|uncorrectable'"]),
        // Power on hours
        (&["power", "hour"], "show power on hours", "smart",
         &["sudo smartctl -A /dev/sda | grep -i 'power_on_hours'"]),
        (&["disk", "age"], "check disk age", "smart",
         &["sudo smartctl -A /dev/sda | grep -iE 'power_on|start_stop'"]),
        // Temperature
        (&["disk", "temp"], "show disk temperature", "smart",
         &["sudo smartctl -A /dev/sda | grep -i temperature", "sudo hddtemp /dev/sda 2>/dev/null"]),
        (&["drive", "temp"], "show drive temperature", "smart",
         &["sudo smartctl -A /dev/sda | grep -i temperature"]),
        // Spin retry
        (&["spin", "retry"], "check spin retry count", "smart",
         &["sudo smartctl -A /dev/sda | grep -i spin"]),
        // Start/stop count
        (&["start", "stop", "count"], "show start/stop count", "smart",
         &["sudo smartctl -A /dev/sda | grep -i start_stop"]),
        // SSD specific
        (&["ssd", "wear"], "check SSD wear level", "smart",
         &["sudo smartctl -A /dev/sda | grep -iE 'wear|life|endurance'"]),
        (&["ssd", "life"], "check SSD life remaining", "smart",
         &["sudo smartctl -A /dev/sda | grep -iE 'life|wear|endurance'"]),
        (&["nand", "write"], "check NAND writes", "smart",
         &["sudo smartctl -A /dev/sda | grep -iE 'nand|written|tbw'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// SMART tests patterns
fn match_smart_tests(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SmartPattern] = &[
        // Self-test log
        (&["smart", "test", "log"], "show SMART test log", "smart",
         &["sudo smartctl -l selftest /dev/sda"]),
        (&["selftest", "log"], "show self-test log", "smart",
         &["sudo smartctl -l selftest /dev/sda"]),
        // Test results
        (&["smart", "test", "result"], "show SMART test results", "smart",
         &["sudo smartctl -l selftest /dev/sda"]),
        // Error log
        (&["smart", "error", "log"], "show SMART error log", "smart",
         &["sudo smartctl -l error /dev/sda"]),
        (&["disk", "error", "log"], "show disk error log", "smart",
         &["sudo smartctl -l error /dev/sda"]),
        // All logs
        (&["smart", "log"], "show SMART logs", "smart",
         &["sudo smartctl -l error /dev/sda", "sudo smartctl -l selftest /dev/sda"]),
        // Test capabilities
        (&["smart", "test", "capabil"], "show SMART test capabilities", "smart",
         &["sudo smartctl -c /dev/sda"]),
        // Test time
        (&["smart", "test", "time"], "show SMART test times", "smart",
         &["sudo smartctl -c /dev/sda | grep -i 'test'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Disk health patterns
fn match_disk_health(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SmartPattern] = &[
        // Failing disk
        (&["disk", "fail"], "check for disk failures", "smart",
         &["sudo smartctl -H /dev/sda", "dmesg | grep -iE 'error|fail|reset|timeout' | tail -20"]),
        (&["drive", "fail"], "check for drive failures", "smart",
         &["sudo smartctl -H /dev/sda"]),
        // Dying disk
        (&["disk", "dying"], "check if disk is dying", "smart",
         &["sudo smartctl -a /dev/sda | grep -iE 'reallocated|pending|uncorrectable|failing'"]),
        (&["drive", "dying"], "check if drive is dying", "smart",
         &["sudo smartctl -a /dev/sda | grep -iE 'reallocated|pending|uncorrectable|failing'"]),
        // Disk problems
        (&["disk", "problem"], "diagnose disk problems", "smart",
         &["sudo smartctl -H /dev/sda", "dmesg | grep -iE 'ata|sata|sd' | tail -20"]),
        // hdparm
        (&["hdparm"], "show hdparm info", "smart",
         &["sudo hdparm -I /dev/sda | head -30"]),
        // Disk info
        (&["disk", "model"], "show disk model", "smart",
         &["lsblk -o NAME,SIZE,MODEL", "sudo hdparm -I /dev/sda | grep Model"]),
        (&["drive", "model"], "show drive model", "smart",
         &["lsblk -o NAME,SIZE,MODEL"]),
        // Disk serial
        (&["disk", "serial"], "show disk serial", "smart",
         &["sudo hdparm -I /dev/sda | grep Serial", "lsblk -o NAME,SERIAL"]),
        // Disk firmware
        (&["disk", "firmware"], "show disk firmware", "smart",
         &["sudo hdparm -I /dev/sda | grep Firmware", "sudo smartctl -i /dev/sda | grep Firmware"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// NVMe patterns
fn match_nvme(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SmartPattern] = &[
        // NVMe health
        (&["nvme", "health"], "check NVMe health", "smart",
         &["sudo nvme smart-log /dev/nvme0", "sudo smartctl -a /dev/nvme0"]),
        (&["nvme", "smart"], "show NVMe SMART", "smart",
         &["sudo nvme smart-log /dev/nvme0"]),
        // NVMe list
        (&["nvme", "list"], "list NVMe devices", "smart",
         &["sudo nvme list"]),
        // NVMe info
        (&["nvme", "info"], "show NVMe info", "smart",
         &["sudo nvme id-ctrl /dev/nvme0 | head -30"]),
        // NVMe temperature
        (&["nvme", "temp"], "show NVMe temperature", "smart",
         &["sudo nvme smart-log /dev/nvme0 | grep -i temperature"]),
        // NVMe wear
        (&["nvme", "wear"], "check NVMe wear level", "smart",
         &["sudo nvme smart-log /dev/nvme0 | grep -iE 'percentage|endurance|life'"]),
        (&["nvme", "life"], "check NVMe life remaining", "smart",
         &["sudo nvme smart-log /dev/nvme0 | grep -i 'percentage'"]),
        // NVMe errors
        (&["nvme", "error"], "show NVMe errors", "smart",
         &["sudo nvme error-log /dev/nvme0"]),
        // NVMe namespace
        (&["nvme", "namespace"], "show NVMe namespaces", "smart",
         &["sudo nvme list-ns /dev/nvme0"]),
        // NVMe firmware
        (&["nvme", "firmware"], "show NVMe firmware", "smart",
         &["sudo nvme id-ctrl /dev/nvme0 | grep -i 'fr\\|mn'"]),
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
    fn test_smart_status() {
        assert!(match_patterns("smart status").is_some());
        assert!(match_patterns("disk health").is_some());
        assert!(match_patterns("ssd health").is_some());
    }

    #[test]
    fn test_smart_attributes() {
        assert!(match_patterns("smart attributes").is_some());
        assert!(match_patterns("reallocated sectors").is_some());
        assert!(match_patterns("disk temperature").is_some());
    }

    #[test]
    fn test_smart_tests() {
        assert!(match_patterns("smart test log").is_some());
        assert!(match_patterns("smart error log").is_some());
    }

    #[test]
    fn test_disk_health() {
        assert!(match_patterns("disk failing").is_some());
        assert!(match_patterns("disk problems").is_some());
        assert!(match_patterns("disk model").is_some());
    }

    #[test]
    fn test_nvme() {
        assert!(match_patterns("nvme health").is_some());
        assert!(match_patterns("nvme list").is_some());
        assert!(match_patterns("nvme temperature").is_some());
    }
}
