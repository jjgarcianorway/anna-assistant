//! Tests for journalctl module (v0.0.173).

use super::journalctl::*;

#[test]
fn test_parse_journalctl_priority_basic() {
    let output = "Dec 05 10:00:00 myhost systemd[1]: Failed to start Some Service.
Dec 05 10:01:00 myhost kernel: Error in something
Dec 05 10:02:00 myhost systemd[1]: Another error message
Dec 05 10:03:00 myhost nginx[1234]: Connection refused";

    let summary = parse_journalctl_priority(output);
    assert_eq!(summary.count_total, 4);
    assert_eq!(summary.top[0].key, "systemd"); // Most frequent
    assert_eq!(summary.top[0].count, 2);
}

#[test]
fn test_parse_journalctl_priority_empty() {
    let summary = parse_journalctl_priority("");
    assert_eq!(summary.count_total, 0);
    assert!(summary.top.is_empty());
}

#[test]
fn test_parse_journalctl_priority_stable_ordering() {
    let output = "Dec 05 10:00:00 host aaa[1]: msg
Dec 05 10:00:01 host bbb[1]: msg
Dec 05 10:00:02 host ccc[1]: msg";

    let summary1 = parse_journalctl_priority(output);
    let summary2 = parse_journalctl_priority(output);

    // Same input must produce identical output (deterministic)
    assert_eq!(summary1, summary2);
    // All have count 1, should be sorted alphabetically
    assert_eq!(summary1.top[0].key, "aaa");
    assert_eq!(summary1.top[1].key, "bbb");
    assert_eq!(summary1.top[2].key, "ccc");
}

#[test]
fn test_parse_boot_time_basic() {
    let output = "Startup finished in 2.5s (kernel) + 5.3s (userspace) = 7.8s";
    let info = parse_boot_time(output);
    assert!(info.raw_line.contains("Startup finished"));
    assert!(info.total_ms.is_some());
    // 7.8s = 7800ms
    assert_eq!(info.total_ms.unwrap(), 7800);
}

#[test]
fn test_parse_boot_time_empty() {
    let info = parse_boot_time("");
    assert!(info.raw_line.is_empty());
    assert!(info.total_ms.is_none());
}

#[test]
fn test_parse_failed_units_basic() {
    let output = "  UNIT                   LOAD   ACTIVE SUB    DESCRIPTION
● nginx.service         loaded failed failed Nginx Web Server
● redis.service         loaded failed failed Redis Database
0 loaded units listed.";

    let units = parse_failed_units(output);
    assert_eq!(units.len(), 2);
    assert_eq!(units[0].name, "nginx.service");
    assert_eq!(units[0].active_state, "failed");
    assert_eq!(units[1].name, "redis.service");
}

#[test]
fn test_parse_failed_units_empty() {
    let output = "0 loaded units listed.";
    let units = parse_failed_units(output);
    assert!(units.is_empty());
}

#[test]
fn test_parse_failed_units_variable_spacing() {
    // Real systemctl output often has variable spacing
    let output = "● foo.service            loaded  failed  failed  Some Description";
    let units = parse_failed_units(output);
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].name, "foo.service");
    assert_eq!(units[0].load_state, "loaded");
    assert_eq!(units[0].active_state, "failed");
}

// === v0.45.4: JSON parsing tests ===

#[test]
fn test_parse_journalctl_json_syslog_identifier() {
    let output = r#"{"SYSLOG_IDENTIFIER":"systemd","MESSAGE":"Starting service..."}
{"SYSLOG_IDENTIFIER":"systemd","MESSAGE":"Failed to start..."}
{"SYSLOG_IDENTIFIER":"nginx","MESSAGE":"Connection refused"}"#;

    let summary = parse_journalctl_json(output);
    assert_eq!(summary.count_total, 3);
    assert_eq!(summary.top[0].key, "systemd");
    assert_eq!(summary.top[0].count, 2);
    assert_eq!(summary.top[1].key, "nginx");
    assert_eq!(summary.top[1].count, 1);
}

#[test]
fn test_parse_journalctl_json_fallback_to_unit() {
    // No SYSLOG_IDENTIFIER, falls back to _SYSTEMD_UNIT
    let output = r#"{"_SYSTEMD_UNIT":"nginx.service","MESSAGE":"test"}"#;

    let summary = parse_journalctl_json(output);
    assert_eq!(summary.count_total, 1);
    assert_eq!(summary.top[0].key, "nginx"); // .service stripped
}

#[test]
fn test_parse_journalctl_json_unattributed() {
    // No identifying fields
    let output = r#"{"MESSAGE":"anonymous error"}"#;

    let summary = parse_journalctl_json(output);
    assert_eq!(summary.count_total, 1);
    assert_eq!(summary.top[0].key, "unattributed");
}

#[test]
fn test_parse_journalctl_auto_detect_json() {
    // parse_journalctl_priority should auto-detect JSON format
    let json_output = r#"{"SYSLOG_IDENTIFIER":"test","MESSAGE":"test"}"#;
    let summary = parse_journalctl_priority(json_output);
    assert_eq!(summary.top[0].key, "test");
}
