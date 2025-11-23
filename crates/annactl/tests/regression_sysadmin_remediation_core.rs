//! Regression tests for core system remediation (Beta.269)
//!
//! Tests cover:
//! - Services (failed/degraded)
//! - Disk space (critical/warning)
//! - Logs (critical issues)
//! - CPU (overload/high load)
//! - Memory (pressure critical/warning, with/without swap)
//! - Processes (runaway)
//! - Healthy system (no remediation)
//! - Canonical format validation
//! - Safety checks (no Rust types, no enum names)

use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData};
use annactl::sysadmin_answers::{
    compose_services_fix_answer,
    compose_disk_fix_answer,
    compose_logs_fix_answer,
    compose_cpu_fix_answer,
    compose_memory_fix_answer,
    compose_process_fix_answer,
    route_system_remediation,
};

/// Helper: Create brain analysis with specific insight
fn create_brain_with_insight(
    rule_id: &str,
    severity: &str,
    summary: &str,
    details: &str,
    evidence: &str,
) -> BrainAnalysisData {
    BrainAnalysisData {
        insights: vec![DiagnosticInsightData {
            rule_id: rule_id.to_string(),
            severity: severity.to_string(),
            summary: summary.to_string(),
            details: details.to_string(),
            commands: vec![],
            citations: vec![],
            evidence: evidence.to_string(),
        }],
        timestamp: chrono::Utc::now().to_rfc3339(),
        formatted_output: String::new(),
        critical_count: if severity == "critical" { 1 } else { 0 },
        warning_count: if severity == "warning" { 1 } else { 0 },
    }
}

#[test]
fn test_failed_service_routes_to_services_composer() {
    let brain = create_brain_with_insight(
        "failed_services",
        "critical",
        "2 critical systemd services have failed",
        "Failed services:\n• sshd.service - OpenSSH Daemon\n• nginx.service - Web Server",
        "sshd.service, nginx.service",
    );

    let result = route_system_remediation(&brain);
    assert!(result.is_some(), "Should route failed services to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("systemd service"), "Should mention systemd services");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("Failed systemd services"), "Should explain failure");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("systemctl --failed"), "Should have systemctl command");
    assert!(answer.contains("journalctl"), "Should have log inspection command");
}

#[test]
fn test_degraded_service_routes_to_services_composer() {
    let brain = create_brain_with_insight(
        "degraded_services",
        "warning",
        "1 systemd service is degraded",
        "• NetworkManager.service - Network Manager",
        "NetworkManager.service",
    );

    let result = route_system_remediation(&brain);
    assert!(result.is_some(), "Should route degraded services to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("degraded"), "Should mention degraded state");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("Degraded services"), "Should explain degraded state");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
}

#[test]
fn test_disk_critical_routes_to_disk_composer() {
    let brain = create_brain_with_insight(
        "disk_space_critical",
        "critical",
        "Disk usage on / is 92%",
        "Root filesystem is critically full. This will cause system instability.",
        "92% used on /",
    );

    let result = route_system_remediation(&brain);
    assert!(result.is_some(), "Should route disk critical to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("Disk usage on /"), "Should mention mountpoint");
    assert!(answer.contains("92%"), "Should show percentage");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("High disk usage"), "Should explain impact");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("df -h"), "Should have df command");
    assert!(answer.contains("du -h"), "Should have du command");
    assert!(answer.contains("pacman -Sc"), "Should suggest package cache cleanup");
}

#[test]
fn test_disk_warning_routes_to_disk_composer() {
    let brain = create_brain_with_insight(
        "disk_space_warning",
        "warning",
        "Disk usage on /home is 85%",
        "Home directory is approaching capacity.",
        "85% used on /home",
    );

    let result = route_system_remediation(&brain);
    assert!(result.is_some(), "Should route disk warning to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("/home"), "Should mention /home");
    assert!(answer.contains("85%"), "Should show percentage");
    assert!(answer.contains("approaching capacity"), "Should indicate warning level");
}

#[test]
fn test_critical_log_issues_routes_to_logs_composer() {
    let brain = create_brain_with_insight(
        "critical_log_issues",
        "critical",
        "5 critical log issues detected",
        "Multiple services are logging errors repeatedly.",
        "5 unique error patterns",
    );

    let result = route_system_remediation(&brain);
    assert!(result.is_some(), "Should route log issues to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("5 critical log issues"), "Should show count");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("journalctl -p err"), "Should have journalctl error command");
}

#[test]
fn test_cpu_overload_routes_to_cpu_composer() {
    let brain = create_brain_with_insight(
        "cpu_overload_critical",
        "critical",
        "CPU usage sustained at 98.5%",
        "CPU load has been critical for extended period.",
        "98.5% CPU usage",
    );

    let result = route_system_remediation(&brain);
    assert!(result.is_some(), "Should route CPU overload to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("CPU usage sustained"), "Should mention sustained load");
    assert!(answer.contains("98"), "Should show percentage");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("Sustained high CPU usage"), "Should explain impact");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("top"), "Should suggest top");
    assert!(answer.contains("ps aux"), "Should have ps command");
}

#[test]
fn test_single_process_cpu_hog_routes_to_process_composer() {
    // Note: In current architecture, process-specific insights would be CPU or memory insights
    // We test the process composer directly
    let answer = compose_process_fix_answer(Some("chromium"), "cpu");

    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("chromium"), "Should mention process name");
    assert!(answer.contains("CPU"), "Should mention CPU");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("WARNING"), "Should warn about killing processes");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("pgrep chromium"), "Should have pgrep command");
    assert!(answer.contains("systemctl status chromium"), "Should check if it's a service");
}

#[test]
fn test_memory_pressure_with_swap_routes_to_memory_composer() {
    let brain = create_brain_with_insight(
        "memory_pressure_critical",
        "critical",
        "Memory usage at 94.2%",
        "RAM is critically full. Swap at 65%.",
        "94.2% RAM, 65% swap",
    );

    let result = route_system_remediation(&brain);
    assert!(result.is_some(), "Should route memory pressure to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("Memory"), "Should mention memory");
    assert!(answer.contains("94"), "Should show memory percentage");
    assert!(answer.contains("[DETAILS]"), "Should have details section");
    assert!(answer.contains("[COMMANDS]"), "Should have commands section");
    assert!(answer.contains("free -h"), "Should have free command");
    assert!(answer.contains("swapon --show"), "Should check swap");
}

#[test]
fn test_memory_without_swap_routes_to_memory_composer() {
    let brain = create_brain_with_insight(
        "memory_pressure_warning",
        "warning",
        "Memory usage at 87.5%",
        "RAM is high. No swap configured.",
        "87.5% RAM, no swap",
    );

    let result = route_system_remediation(&brain);
    assert!(result.is_some(), "Should route memory warning to remediation");

    let answer = result.unwrap();
    assert!(answer.contains("[SUMMARY]"), "Should have summary section");
    assert!(answer.contains("Memory usage at 87"), "Should show percentage");
    // Should NOT mention heavy swap usage since there's no swap
    assert!(!answer.contains("heavy swap usage"), "Should not mention heavy swap when there is none");
}

#[test]
fn test_healthy_system_no_remediation() {
    let brain = BrainAnalysisData {
        insights: vec![],
        proactive_issues: vec![],
        timestamp: chrono::Utc::now().to_rfc3339(),
        formatted_output: String::new(),
        critical_count: 0,
        warning_count: 0,
        proactive_issues: vec![],
    };

    let result = route_system_remediation(&brain);
    assert!(result.is_none(), "Healthy system should not trigger remediation");
}

#[test]
fn test_canonical_format_validation_all_composers() {
    // Test all six composers produce canonical format
    let services_answer = compose_services_fix_answer(vec!["sshd.service"], true);
    let disk_answer = compose_disk_fix_answer("/", 92, true);
    let logs_answer = compose_logs_fix_answer(5, false);
    let cpu_answer = compose_cpu_fix_answer(98.5, true, Some("chromium"));
    let memory_answer = compose_memory_fix_answer(94.2, Some(65.0), true);
    let process_answer = compose_process_fix_answer(Some("firefox"), "memory");

    for (name, answer) in [
        ("services", services_answer),
        ("disk", disk_answer),
        ("logs", logs_answer),
        ("cpu", cpu_answer),
        ("memory", memory_answer),
        ("process", process_answer),
    ] {
        assert!(
            answer.starts_with("[SUMMARY]"),
            "{} answer should start with [SUMMARY]",
            name
        );
        assert!(
            answer.contains("[DETAILS]"),
            "{} answer should contain [DETAILS]",
            name
        );
        assert!(
            answer.contains("[COMMANDS]"),
            "{} answer should contain [COMMANDS]",
            name
        );

        // Commands section should have actual commands
        let commands_section = answer.split("[COMMANDS]").nth(1).expect("Commands section should exist");
        assert!(
            commands_section.contains("$") ||
            commands_section.contains("#") ||
            commands_section.len() > 50, // Has substantial content
            "{} answer should contain actual commands or substantial guidance",
            name
        );

        // No LLM-style preamble
        assert!(
            !answer.contains("I recommend") &&
            !answer.contains("You might want to") &&
            !answer.contains("Let me help"),
            "{} answer should not contain LLM-style language",
            name
        );
    }
}

#[test]
fn test_safety_no_rust_types_in_output() {
    // Test that no Rust enum names or debug syntax appears
    let services_answer = compose_services_fix_answer(vec![], true);
    let disk_answer = compose_disk_fix_answer("/var", 88, false);
    let logs_answer = compose_logs_fix_answer(3, true);
    let cpu_answer = compose_cpu_fix_answer(85.0, false, None);
    let memory_answer = compose_memory_fix_answer(82.0, None, false);
    let process_answer = compose_process_fix_answer(None, "cpu");

    for (name, answer) in [
        ("services", services_answer),
        ("disk", disk_answer),
        ("logs", logs_answer),
        ("cpu", cpu_answer),
        ("memory", memory_answer),
        ("process", process_answer),
    ] {
        // No Rust enum syntax
        assert!(
            !answer.contains("::"),
            "{} answer should not contain Rust enum syntax (::)",
            name
        );

        // No Rust type names
        assert!(
            !answer.contains("DiagnosticSeverity") &&
            !answer.contains("HealthStatus") &&
            !answer.contains("Option<") &&
            !answer.contains("Vec<"),
            "{} answer should not contain Rust type names",
            name
        );

        // No debug formatting artifacts
        assert!(
            !answer.contains("Some(") &&
            !answer.contains("None)"),
            "{} answer should not contain debug formatting",
            name
        );
    }
}
