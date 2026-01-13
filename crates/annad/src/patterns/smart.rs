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
/// v0.3.30: Use device discovery instead of hardcoding /dev/sda (fails on NVMe systems)
fn match_smart_status(q: &str) -> Option<DeepUnderstanding> {
    // v0.3.30: Dynamic device discovery - find boot disk, handle NVMe
    // Get disk that contains root filesystem: lsblk -no PKNAME $(findmnt -no SOURCE /)
    let discover_boot_disk = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                              [ -n \"$DISK\" ] && sudo smartctl -H /dev/$DISK 2>/dev/null || echo 'Could not determine boot disk'";
    let discover_disk_info = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                              [ -n \"$DISK\" ] && sudo smartctl -i /dev/$DISK 2>/dev/null || echo 'Could not determine boot disk'";
    let discover_disk_attrs = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                               [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | head -50 || echo 'Could not determine boot disk'";

    let patterns: &[SmartPattern] = &[
        // SMART status - use discovery, then show all devices
        (&["smart", "status"], "show SMART status", "smart",
         &[discover_boot_disk, "sudo smartctl --scan", "lsblk -o NAME,SIZE,TYPE,MODEL"]),
        (&["smartctl", "status"], "show smartctl status", "smart",
         &[discover_boot_disk, "sudo smartctl --scan"]),
        // Disk health - discover boot disk first
        (&["disk", "health"], "check disk health", "smart",
         &[discover_boot_disk, "lsblk -o NAME,SIZE,TYPE,MODEL,MOUNTPOINT"]),
        (&["drive", "health"], "check drive health", "smart",
         &[discover_boot_disk]),
        (&["ssd", "health"], "check SSD health", "smart",
         &[discover_disk_attrs, "lsblk -d -o NAME,SIZE,ROTA,MODEL"]),
        (&["hdd", "health"], "check HDD health", "smart",
         &[discover_boot_disk]),
        // SMART overview
        (&["smart", "info"], "show SMART information", "smart",
         &[discover_disk_info]),
        (&["smart", "overview"], "show SMART overview", "smart",
         &[discover_disk_attrs]),
        // Scan devices - no discovery needed
        (&["smart", "scan"], "scan for SMART devices", "smart",
         &["sudo smartctl --scan", "lsblk -d -o NAME,SIZE,TYPE,MODEL"]),
        (&["smartctl", "scan"], "scan SMART devices", "smart",
         &["sudo smartctl --scan"]),
        // SMART capable
        (&["smart", "capable"], "check SMART capability", "smart",
         &[discover_disk_info]),
        (&["smart", "support"], "check SMART support", "smart",
         &[discover_disk_info]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// SMART attributes patterns
/// v0.3.30: Use device discovery instead of hardcoding /dev/sda
fn match_smart_attributes(q: &str) -> Option<DeepUnderstanding> {
    // v0.3.30: Dynamic device discovery - all commands find boot disk first
    // Base discovery prefix
    const DISCOVER: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                            [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null";
    const DISCOVER_CRIT: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                 [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | grep -iE 'reallocated|pending|uncorrectable|wear'";
    const DISCOVER_REALLOC: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                    [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | grep -i reallocated";
    const DISCOVER_BAD: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | grep -iE 'reallocated|pending|uncorrectable'";
    const DISCOVER_POWER: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                  [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | grep -i power_on_hours";
    const DISCOVER_AGE: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | grep -iE 'power_on|start_stop'";
    const DISCOVER_TEMP: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                 [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | grep -i temperature";
    const DISCOVER_SPIN: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                 [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | grep -i spin";
    const DISCOVER_START: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                  [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | grep -i start_stop";
    const DISCOVER_WEAR: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                 [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | grep -iE 'wear|life|endurance'";
    const DISCOVER_NAND: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                 [ -n \"$DISK\" ] && sudo smartctl -A /dev/$DISK 2>/dev/null | grep -iE 'nand|written|tbw'";

    let patterns: &[SmartPattern] = &[
        // All attributes
        (&["smart", "attribute"], "show SMART attributes", "smart",
         &[DISCOVER]),
        (&["smartctl", "attribute"], "show smartctl attributes", "smart",
         &[DISCOVER]),
        // Critical attributes
        (&["smart", "critical"], "show critical SMART attributes", "smart",
         &[DISCOVER_CRIT]),
        // Reallocated sectors
        (&["reallocated", "sector"], "check reallocated sectors", "smart",
         &[DISCOVER_REALLOC]),
        (&["bad", "sector"], "check for bad sectors", "smart",
         &[DISCOVER_BAD]),
        // Power on hours
        (&["power", "hour"], "show power on hours", "smart",
         &[DISCOVER_POWER]),
        (&["disk", "age"], "check disk age", "smart",
         &[DISCOVER_AGE]),
        // Temperature
        (&["disk", "temp"], "show disk temperature", "smart",
         &[DISCOVER_TEMP]),
        (&["drive", "temp"], "show drive temperature", "smart",
         &[DISCOVER_TEMP]),
        // Spin retry
        (&["spin", "retry"], "check spin retry count", "smart",
         &[DISCOVER_SPIN]),
        // Start/stop count
        (&["start", "stop", "count"], "show start/stop count", "smart",
         &[DISCOVER_START]),
        // SSD specific
        (&["ssd", "wear"], "check SSD wear level", "smart",
         &[DISCOVER_WEAR]),
        (&["ssd", "life"], "check SSD life remaining", "smart",
         &[DISCOVER_WEAR]),
        (&["nand", "write"], "check NAND writes", "smart",
         &[DISCOVER_NAND]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// SMART tests patterns
/// v0.3.30: Use device discovery instead of hardcoding /dev/sda
fn match_smart_tests(q: &str) -> Option<DeepUnderstanding> {
    // v0.3.30: Dynamic device discovery for all test commands
    const DISCOVER_SELFTEST: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                     [ -n \"$DISK\" ] && sudo smartctl -l selftest /dev/$DISK 2>/dev/null || echo 'Could not determine boot disk'";
    const DISCOVER_ERROR: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                  [ -n \"$DISK\" ] && sudo smartctl -l error /dev/$DISK 2>/dev/null || echo 'Could not determine boot disk'";
    const DISCOVER_CAPS: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                 [ -n \"$DISK\" ] && sudo smartctl -c /dev/$DISK 2>/dev/null || echo 'Could not determine boot disk'";
    const DISCOVER_CAPS_TEST: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                      [ -n \"$DISK\" ] && sudo smartctl -c /dev/$DISK 2>/dev/null | grep -i 'test' || echo 'Could not determine boot disk'";

    let patterns: &[SmartPattern] = &[
        // Self-test log
        (&["smart", "test", "log"], "show SMART test log", "smart",
         &[DISCOVER_SELFTEST]),
        (&["selftest", "log"], "show self-test log", "smart",
         &[DISCOVER_SELFTEST]),
        // Test results
        (&["smart", "test", "result"], "show SMART test results", "smart",
         &[DISCOVER_SELFTEST]),
        // Error log
        (&["smart", "error", "log"], "show SMART error log", "smart",
         &[DISCOVER_ERROR]),
        (&["disk", "error", "log"], "show disk error log", "smart",
         &[DISCOVER_ERROR]),
        // All logs
        (&["smart", "log"], "show SMART logs", "smart",
         &[DISCOVER_ERROR, DISCOVER_SELFTEST]),
        // Test capabilities
        (&["smart", "test", "capabil"], "show SMART test capabilities", "smart",
         &[DISCOVER_CAPS]),
        // Test time
        (&["smart", "test", "time"], "show SMART test times", "smart",
         &[DISCOVER_CAPS_TEST]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Disk health patterns
/// v0.3.30: Use device discovery instead of hardcoding /dev/sda
fn match_disk_health(q: &str) -> Option<DeepUnderstanding> {
    // v0.3.30: Dynamic device discovery for all health commands
    const DISCOVER_HEALTH: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                   [ -n \"$DISK\" ] && sudo smartctl -H /dev/$DISK 2>/dev/null || echo 'Could not determine boot disk'";
    const DISCOVER_ALL: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                [ -n \"$DISK\" ] && sudo smartctl -a /dev/$DISK 2>/dev/null | grep -iE 'reallocated|pending|uncorrectable|failing' || echo 'Could not determine boot disk'";
    const DISCOVER_HDPARM: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                   [ -n \"$DISK\" ] && sudo hdparm -I /dev/$DISK 2>/dev/null | head -30 || echo 'Could not determine boot disk'";
    const DISCOVER_MODEL: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                  [ -n \"$DISK\" ] && sudo hdparm -I /dev/$DISK 2>/dev/null | grep Model || echo 'Could not determine boot disk'";
    const DISCOVER_SERIAL: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                                   [ -n \"$DISK\" ] && sudo hdparm -I /dev/$DISK 2>/dev/null | grep Serial || lsblk -o NAME,SERIAL";
    const DISCOVER_FW: &str = "DISK=$(lsblk -no PKNAME $(findmnt -no SOURCE /) 2>/dev/null | head -1); \
                               [ -n \"$DISK\" ] && (sudo hdparm -I /dev/$DISK 2>/dev/null | grep Firmware; sudo smartctl -i /dev/$DISK 2>/dev/null | grep Firmware) || echo 'Could not determine boot disk'";

    let patterns: &[SmartPattern] = &[
        // Failing disk
        (&["disk", "fail"], "check for disk failures", "smart",
         &[DISCOVER_HEALTH, "dmesg | grep -iE 'error|fail|reset|timeout' | tail -20"]),
        (&["drive", "fail"], "check for drive failures", "smart",
         &[DISCOVER_HEALTH]),
        // Dying disk
        (&["disk", "dying"], "check if disk is dying", "smart",
         &[DISCOVER_ALL]),
        (&["drive", "dying"], "check if drive is dying", "smart",
         &[DISCOVER_ALL]),
        // Disk problems
        (&["disk", "problem"], "diagnose disk problems", "smart",
         &[DISCOVER_HEALTH, "dmesg | grep -iE 'ata|sata|sd' | tail -20"]),
        // hdparm
        (&["hdparm"], "show hdparm info", "smart",
         &[DISCOVER_HDPARM]),
        // Disk info
        (&["disk", "model"], "show disk model", "smart",
         &["lsblk -o NAME,SIZE,MODEL", DISCOVER_MODEL]),
        (&["drive", "model"], "show drive model", "smart",
         &["lsblk -o NAME,SIZE,MODEL"]),
        // Disk serial
        (&["disk", "serial"], "show disk serial", "smart",
         &[DISCOVER_SERIAL]),
        // Disk firmware
        (&["disk", "firmware"], "show disk firmware", "smart",
         &[DISCOVER_FW]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// NVMe patterns
/// v0.3.30: Use device discovery instead of hardcoding /dev/nvme0
fn match_nvme(q: &str) -> Option<DeepUnderstanding> {
    // v0.3.30: Dynamic NVMe device discovery
    const DISCOVER_NVME_SMART: &str = "NVME=$(ls /dev/nvme0n1 2>/dev/null && echo nvme0 || ls /dev/nvme* 2>/dev/null | grep -oE 'nvme[0-9]+' | head -1); \
                                       [ -n \"$NVME\" ] && sudo nvme smart-log /dev/$NVME 2>/dev/null || echo 'No NVMe device found'";
    const DISCOVER_NVME_SMARTCTL: &str = "NVME=$(ls /dev/nvme0n1 2>/dev/null && echo nvme0 || ls /dev/nvme* 2>/dev/null | grep -oE 'nvme[0-9]+' | head -1); \
                                          [ -n \"$NVME\" ] && sudo smartctl -a /dev/$NVME 2>/dev/null || echo 'No NVMe device found'";
    const DISCOVER_NVME_ID: &str = "NVME=$(ls /dev/nvme0n1 2>/dev/null && echo nvme0 || ls /dev/nvme* 2>/dev/null | grep -oE 'nvme[0-9]+' | head -1); \
                                    [ -n \"$NVME\" ] && sudo nvme id-ctrl /dev/$NVME 2>/dev/null | head -30 || echo 'No NVMe device found'";
    const DISCOVER_NVME_TEMP: &str = "NVME=$(ls /dev/nvme0n1 2>/dev/null && echo nvme0 || ls /dev/nvme* 2>/dev/null | grep -oE 'nvme[0-9]+' | head -1); \
                                      [ -n \"$NVME\" ] && sudo nvme smart-log /dev/$NVME 2>/dev/null | grep -i temperature || echo 'No NVMe device found'";
    const DISCOVER_NVME_WEAR: &str = "NVME=$(ls /dev/nvme0n1 2>/dev/null && echo nvme0 || ls /dev/nvme* 2>/dev/null | grep -oE 'nvme[0-9]+' | head -1); \
                                      [ -n \"$NVME\" ] && sudo nvme smart-log /dev/$NVME 2>/dev/null | grep -iE 'percentage|endurance|life' || echo 'No NVMe device found'";
    const DISCOVER_NVME_LIFE: &str = "NVME=$(ls /dev/nvme0n1 2>/dev/null && echo nvme0 || ls /dev/nvme* 2>/dev/null | grep -oE 'nvme[0-9]+' | head -1); \
                                      [ -n \"$NVME\" ] && sudo nvme smart-log /dev/$NVME 2>/dev/null | grep -i 'percentage' || echo 'No NVMe device found'";
    const DISCOVER_NVME_ERROR: &str = "NVME=$(ls /dev/nvme0n1 2>/dev/null && echo nvme0 || ls /dev/nvme* 2>/dev/null | grep -oE 'nvme[0-9]+' | head -1); \
                                       [ -n \"$NVME\" ] && sudo nvme error-log /dev/$NVME 2>/dev/null || echo 'No NVMe device found'";
    const DISCOVER_NVME_NS: &str = "NVME=$(ls /dev/nvme0n1 2>/dev/null && echo nvme0 || ls /dev/nvme* 2>/dev/null | grep -oE 'nvme[0-9]+' | head -1); \
                                    [ -n \"$NVME\" ] && sudo nvme list-ns /dev/$NVME 2>/dev/null || echo 'No NVMe device found'";
    const DISCOVER_NVME_FW: &str = "NVME=$(ls /dev/nvme0n1 2>/dev/null && echo nvme0 || ls /dev/nvme* 2>/dev/null | grep -oE 'nvme[0-9]+' | head -1); \
                                    [ -n \"$NVME\" ] && sudo nvme id-ctrl /dev/$NVME 2>/dev/null | grep -i 'fr\\|mn' || echo 'No NVMe device found'";

    let patterns: &[SmartPattern] = &[
        // NVMe health
        (&["nvme", "health"], "check NVMe health", "smart",
         &[DISCOVER_NVME_SMART, DISCOVER_NVME_SMARTCTL]),
        (&["nvme", "smart"], "show NVMe SMART", "smart",
         &[DISCOVER_NVME_SMART]),
        // NVMe list
        (&["nvme", "list"], "list NVMe devices", "smart",
         &["sudo nvme list"]),
        // NVMe info
        (&["nvme", "info"], "show NVMe info", "smart",
         &[DISCOVER_NVME_ID]),
        // NVMe temperature
        (&["nvme", "temp"], "show NVMe temperature", "smart",
         &[DISCOVER_NVME_TEMP]),
        // NVMe wear
        (&["nvme", "wear"], "check NVMe wear level", "smart",
         &[DISCOVER_NVME_WEAR]),
        (&["nvme", "life"], "check NVMe life remaining", "smart",
         &[DISCOVER_NVME_LIFE]),
        // NVMe errors
        (&["nvme", "error"], "show NVMe errors", "smart",
         &[DISCOVER_NVME_ERROR]),
        // NVMe namespace
        (&["nvme", "namespace"], "show NVMe namespaces", "smart",
         &[DISCOVER_NVME_NS]),
        // NVMe firmware
        (&["nvme", "firmware"], "show NVMe firmware", "smart",
         &[DISCOVER_NVME_FW]),
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
