//! Integration tests for health CLI commands
//!
//! Phase 0.5c: Test exit codes, report generation, logging, and permissions
//! Citation: [archwiki:System_maintenance]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Test helper to run annactl command
fn run_annactl(args: &[&str]) -> std::process::Output {
    Command::new("./target/release/annactl")
        .args(args)
        .output()
        .expect("Failed to execute annactl")
}

/// Test helper to check if daemon is running
fn is_daemon_running() -> bool {
    std::path::Path::new("/run/anna/anna.sock").exists()
}

/// Test 1: health command with all ok should exit 0
#[test]
fn test_health_all_ok() {
    if !is_daemon_running() {
        eprintln!("Skipping test: daemon not running");
        return;
    }

    // Set environment to force all probes to return ok
    let output = Command::new("./target/release/annactl")
        .args(&["health"])
        .env("TEST_PROBE_STATUS_DISK_SPACE", "ok")
        .env("TEST_PROBE_STATUS_PACMAN_DB", "ok")
        .env("TEST_PROBE_STATUS_SYSTEMD_UNITS", "ok")
        .env("TEST_PROBE_STATUS_JOURNAL_ERRORS", "ok")
        .env("TEST_PROBE_STATUS_SERVICES_FAILED", "ok")
        .env("TEST_PROBE_STATUS_FIRMWARE_MICROCODE", "ok")
        .output()
        .expect("Failed to execute annactl health");

    assert_eq!(output.status.code(), Some(0), "Expected exit code 0 for all ok");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok="), "Expected health summary in output");
}

/// Test 2: health command with one warn should exit 2
#[test]
fn test_health_with_warn() {
    if !is_daemon_running() {
        eprintln!("Skipping test: daemon not running");
        return;
    }

    let output = Command::new("./target/release/annactl")
        .args(&["health"])
        .env("TEST_PROBE_STATUS_DISK_SPACE", "warn")
        .env("TEST_PROBE_STATUS_PACMAN_DB", "ok")
        .env("TEST_PROBE_STATUS_SYSTEMD_UNITS", "ok")
        .env("TEST_PROBE_STATUS_JOURNAL_ERRORS", "ok")
        .env("TEST_PROBE_STATUS_SERVICES_FAILED", "ok")
        .env("TEST_PROBE_STATUS_FIRMWARE_MICROCODE", "ok")
        .output()
        .expect("Failed to execute annactl health");

    assert_eq!(output.status.code(), Some(2), "Expected exit code 2 for warn");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("warn="), "Expected health summary with warnings");
}

/// Test 3: health command with one fail should exit 1
#[test]
fn test_health_with_fail() {
    if !is_daemon_running() {
        eprintln!("Skipping test: daemon not running");
        return;
    }

    let output = Command::new("./target/release/annactl")
        .args(&["health"])
        .env("TEST_PROBE_STATUS_DISK_SPACE", "fail")
        .env("TEST_PROBE_STATUS_PACMAN_DB", "ok")
        .env("TEST_PROBE_STATUS_SYSTEMD_UNITS", "ok")
        .env("TEST_PROBE_STATUS_JOURNAL_ERRORS", "ok")
        .env("TEST_PROBE_STATUS_SERVICES_FAILED", "ok")
        .env("TEST_PROBE_STATUS_FIRMWARE_MICROCODE", "ok")
        .output()
        .expect("Failed to execute annactl health");

    assert_eq!(output.status.code(), Some(1), "Expected exit code 1 for fail");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fail="), "Expected health summary with failures");
}

/// Test 4: health command should generate report
#[test]
fn test_health_report_generation() {
    if !is_daemon_running() {
        eprintln!("Skipping test: daemon not running");
        return;
    }

    let output = Command::new("./target/release/annactl")
        .args(&["health"])
        .output()
        .expect("Failed to execute annactl health");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Details saved:"), "Expected report path in output");

    // Check that reports directory exists
    let reports_dir = PathBuf::from("/var/lib/anna/reports");
    if reports_dir.exists() {
        // Find the most recent health report
        let mut reports: Vec<_> = fs::read_dir(&reports_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("health-"))
            .collect();

        reports.sort_by_key(|e| e.metadata().unwrap().modified().unwrap());

        if let Some(latest) = reports.last() {
            let content = fs::read_to_string(latest.path()).unwrap();
            assert!(content.contains("\"state\""), "Expected state field in report");
            assert!(content.contains("\"summary\""), "Expected summary field in report");

            // Check permissions (0600)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = latest.metadata().unwrap().permissions();
                assert_eq!(perms.mode() & 0o777, 0o600, "Expected 0600 permissions on report");
            }
        }
    }
}

/// Test 5: doctor command should produce diagnostic report
#[test]
fn test_doctor_report() {
    if !is_daemon_running() {
        eprintln!("Skipping test: daemon not running");
        return;
    }

    let output = Command::new("./target/release/annactl")
        .args(&["doctor"])
        .output()
        .expect("Failed to execute annactl doctor");

    assert!(output.status.code().unwrap() <= 2, "Expected exit code 0, 1, or 2");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Doctor report"), "Expected doctor report header");
    assert!(stdout.contains("Failed probes:"), "Expected failed probes section");
    assert!(stdout.contains("Citations:"), "Expected citations section");
    assert!(stdout.contains("[archwiki:System_maintenance]"), "Expected Arch Wiki citation");
}

/// Test 6: doctor command with --json should output valid JSON
#[test]
fn test_doctor_json_output() {
    if !is_daemon_running() {
        eprintln!("Skipping test: daemon not running");
        return;
    }

    let output = Command::new("./target/release/annactl")
        .args(&["doctor", "--json"])
        .output()
        .expect("Failed to execute annactl doctor --json");

    assert!(output.status.code().unwrap() <= 2, "Expected exit code 0, 1, or 2");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Expected valid JSON output");

    assert!(json.get("version").is_some(), "Expected version field");
    assert!(json.get("ok").is_some(), "Expected ok field");
    assert!(json.get("state").is_some(), "Expected state field");
    assert!(json.get("summary").is_some(), "Expected summary field");
    assert!(json.get("citation").is_some(), "Expected citation field");
    assert!(json.get("probes").is_some(), "Expected probes field");
}

/// Test 7: rescue list should display recovery plans
#[test]
fn test_rescue_list() {
    if !is_daemon_running() {
        eprintln!("Skipping test: daemon not running");
        return;
    }

    let output = Command::new("./target/release/annactl")
        .args(&["rescue", "list"])
        .output()
        .expect("Failed to execute annactl rescue list");

    assert_eq!(output.status.code(), Some(0), "Expected exit code 0");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should list 5 recovery plans
    assert!(stdout.contains("bootloader"), "Expected bootloader plan");
    assert!(stdout.contains("initramfs"), "Expected initramfs plan");
    assert!(stdout.contains("pacman-db"), "Expected pacman-db plan");
    assert!(stdout.contains("fstab"), "Expected fstab plan");
    assert!(stdout.contains("systemd"), "Expected systemd plan");
}

/// Test 8: daemon unavailable should exit 70
#[test]
fn test_daemon_unavailable_exit_code() {
    // Stop daemon temporarily or skip if daemon is running
    if is_daemon_running() {
        eprintln!("Skipping test: daemon is running (cannot test unavailable state)");
        return;
    }

    let output = Command::new("./target/release/annactl")
        .args(&["health"])
        .output()
        .expect("Failed to execute annactl health");

    assert_eq!(output.status.code(), Some(70), "Expected exit code 70 for daemon unavailable");
}

/// Test 9: health command should write to ctl.jsonl log
#[test]
fn test_health_logging() {
    if !is_daemon_running() {
        eprintln!("Skipping test: daemon not running");
        return;
    }

    // Get current log size
    let log_path = PathBuf::from("/var/log/anna/ctl.jsonl");
    let initial_size = if log_path.exists() {
        fs::metadata(&log_path).unwrap().len()
    } else {
        0
    };

    // Run health command
    let _output = Command::new("./target/release/annactl")
        .args(&["health"])
        .output()
        .expect("Failed to execute annactl health");

    // Check that log grew
    if log_path.exists() {
        let new_size = fs::metadata(&log_path).unwrap().len();
        assert!(new_size > initial_size, "Expected log to grow after health command");

        // Read last line and verify structure
        let content = fs::read_to_string(&log_path).unwrap();
        let last_line = content.lines().last().unwrap();
        let log_entry: serde_json::Value = serde_json::from_str(last_line)
            .expect("Expected valid JSON in log");

        assert!(log_entry.get("ts").is_some(), "Expected ts field");
        assert!(log_entry.get("req_id").is_some(), "Expected req_id field");
        assert!(log_entry.get("state").is_some(), "Expected state field");
        assert!(log_entry.get("command").is_some(), "Expected command field");
        assert_eq!(log_entry.get("command").unwrap().as_str().unwrap(), "health");
        assert!(log_entry.get("exit_code").is_some(), "Expected exit_code field");
        assert!(log_entry.get("citation").is_some(), "Expected citation field");
        assert!(log_entry.get("duration_ms").is_some(), "Expected duration_ms field");
    }
}

/// Test 10: permissions check for reports directory
#[test]
fn test_reports_directory_permissions() {
    let reports_dir = PathBuf::from("/var/lib/anna/reports");

    if reports_dir.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::metadata(&reports_dir).unwrap().permissions();
            let mode = perms.mode() & 0o777;
            assert!(mode == 0o700 || mode == 0o755, "Expected 0700 or 0755 permissions on reports directory, got {:o}", mode);
        }
    }
}

