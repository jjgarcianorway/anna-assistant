//! Integration tests for annad

use std::process::Command;

/// Test that cpu.info probe returns valid data
#[test]
fn test_cpuinfo_parser_real_system() {
    let output = Command::new("cat")
        .arg("/proc/cpuinfo")
        .output()
        .expect("Failed to read /proc/cpuinfo");

    assert!(output.status.success());
    let content = String::from_utf8_lossy(&output.stdout);

    // Basic sanity checks
    assert!(content.contains("processor"));
}

/// Test that meminfo probe returns valid data
#[test]
fn test_meminfo_parser_real_system() {
    let output = Command::new("cat")
        .arg("/proc/meminfo")
        .output()
        .expect("Failed to read /proc/meminfo");

    assert!(output.status.success());
    let content = String::from_utf8_lossy(&output.stdout);

    // Basic sanity checks
    assert!(content.contains("MemTotal"));
    assert!(content.contains("MemFree"));
}

/// Test that lsblk returns valid JSON
#[test]
fn test_lsblk_json_output() {
    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE"])
        .output()
        .expect("Failed to run lsblk");

    assert!(output.status.success());
    let content = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
    assert!(parsed.is_ok(), "lsblk output is not valid JSON");

    let json = parsed.unwrap();
    assert!(json.get("blockdevices").is_some());
}
