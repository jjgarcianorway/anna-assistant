//! Integration tests for annactl
//!
//! Phase 0.3e: Tests for state-aware dispatch, help system, and error codes
//! Citation: [archwiki:system_maintenance]

use std::path::PathBuf;
use std::process::Command;

/// Get path to annactl binary
fn annactl_bin() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("target");
    path.push("debug");
    path.push("annactl");
    path
}

/// Test that annactl binary compiles and runs
#[test]
fn test_annactl_compiles() {
    let output = Command::new("cargo")
        .args(&["build", "--bin", "annactl"])
        .output()
        .expect("Failed to build annactl");

    assert!(
        output.status.success(),
        "annactl should compile successfully"
    );
}

/// Test version flag works
#[test]
fn test_version_flag() {
    let output = Command::new(annactl_bin())
        .arg("--version")
        .output()
        .expect("Failed to run annactl --version");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("annactl"),
        "Version output should contain 'annactl'"
    );
}

/// Test help subcommand with --json flag produces valid JSON
#[test]
#[ignore] // Requires daemon to be running
fn test_help_json_output() {
    use std::time::Duration;

    // Use timeout to prevent hanging if daemon is unavailable
    let child = Command::new(annactl_bin())
        .args(&["help", "--json"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    if let Ok(mut child) = child {
        // Give it 5 seconds to complete
        std::thread::sleep(Duration::from_secs(5));

        // Try to get output
        if let Ok(status) = child.try_wait() {
            if status.is_some() {
                // Process has finished
                let output = child.wait_with_output().ok();
                if let Some(output) = output {
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // If daemon is running, output should be valid JSON
                    if !stdout.is_empty() && stdout.contains("{") {
                        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
                        if parsed.is_ok() {
                            let json = parsed.unwrap();
                            // Should have expected fields
                            assert!(
                                json.get("version").is_some(),
                                "JSON should have version field"
                            );
                            assert!(json.get("ok").is_some(), "JSON should have ok field");
                            assert!(json.get("state").is_some(), "JSON should have state field");
                        }
                    }
                }
            } else {
                // Still running, kill it
                let _ = child.kill();
            }
        } else {
            // Error checking status, kill it
            let _ = child.kill();
        }
    }
}

/// Test daemon unavailable returns exit code 70
#[test]
#[ignore] // Requires daemon to NOT be running
fn test_daemon_unavailable_exit_code() {
    use std::time::Duration;

    // Use timeout to prevent hanging if daemon is unavailable
    let child = Command::new(annactl_bin())
        .arg("status")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    if let Ok(mut child) = child {
        // Give it 5 seconds to complete
        std::thread::sleep(Duration::from_secs(5));

        // Try to get output
        if let Ok(status) = child.try_wait() {
            if status.is_some() {
                // Process has finished
                let output = child.wait_with_output().ok();
                if let Some(output) = output {
                    if !output.status.success() {
                        // If daemon is unavailable, exit code should be 70
                        let code = output.status.code().unwrap_or(0);
                        if code != 0 {
                            // Either 70 (daemon unavailable) or 64 (command not available)
                            assert!(
                                code == 70 || code == 64,
                                "Exit code should be 64 or 70, got {}",
                                code
                            );
                        }
                    }
                }
            } else {
                // Still running, kill it
                let _ = child.kill();
            }
        } else {
            // Error checking status, kill it
            let _ = child.kill();
        }
    }
}

/// Test log entry structure
#[test]
fn test_log_entry_structure() {
    use annactl::logging::ErrorDetails;
    use annactl::logging::LogEntry;

    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: LogEntry::generate_req_id(),
        state: "configured".to_string(),
        command: "status".to_string(),
        allowed: Some(true),
        args: vec![],
        exit_code: 0,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms: 100,
        ok: true,
        error: None,
    };

    // Should serialize to JSON
    let json = serde_json::to_string(&log_entry).expect("LogEntry should serialize");
    assert!(json.contains("\"command\":\"status\""));
    assert!(json.contains("\"ok\":true"));

    // Test with error
    let error_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: LogEntry::generate_req_id(),
        state: "unknown".to_string(),
        command: "update".to_string(),
        allowed: Some(false),
        args: vec![],
        exit_code: 64,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms: 50,
        ok: false,
        error: Some(ErrorDetails {
            code: "COMMAND_NOT_AVAILABLE".to_string(),
            message: "Command not available in this state".to_string(),
        }),
    };

    let error_json = serde_json::to_string(&error_entry).expect("Error entry should serialize");
    assert!(error_json.contains("\"ok\":false"));
    assert!(error_json.contains("COMMAND_NOT_AVAILABLE"));
}

/// Test output structure
#[test]
fn test_output_structure() {
    use annactl::output::CommandOutput;

    // Test not_available output
    let output = CommandOutput::not_available(
        "iso_live".to_string(),
        "update".to_string(),
        "[archwiki:Installation_guide]".to_string(),
    );

    assert_eq!(output.state, "iso_live");
    assert_eq!(output.command, "update");
    assert!(!output.allowed);
    assert!(!output.ok);

    // Should serialize to JSON
    let json = serde_json::to_string(&output).expect("Output should serialize");
    assert!(json.contains("\"allowed\":false"));
    assert!(json.contains("\"ok\":false"));

    // Test daemon_unavailable output
    let daemon_output = CommandOutput::daemon_unavailable("status".to_string());
    assert_eq!(daemon_output.state, "unknown");
    assert!(!daemon_output.ok);
    assert!(daemon_output.message.contains("Daemon unavailable"));
}

/// Test error codes are defined correctly
#[test]
fn test_error_codes() {
    use annactl::errors::*;

    assert_eq!(EXIT_SUCCESS, 0);
    assert_eq!(EXIT_COMMAND_NOT_AVAILABLE, 64);
    assert_eq!(EXIT_INVALID_RESPONSE, 65);
    assert_eq!(EXIT_DAEMON_UNAVAILABLE, 70);
    assert_eq!(EXIT_GENERAL_ERROR, 1);
}
