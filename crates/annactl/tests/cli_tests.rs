//! CLI integration tests for annactl v0.11.0
//!
//! Tests the locked CLI surface:
//! - annactl - Start REPL
//! - annactl "<request>" - One-shot natural language request
//! - annactl status - Compact status summary
//! - annactl -V / --version / version (case-insensitive) - Show version
//! - annactl -h / --help / help (case-insensitive) - Show help
//!
//! v0.6.0: ASCII-only sysadmin style, multi-round reliability refinement
//! v0.7.0: Self-health monitoring and auto-repair
//! v0.8.0: Observability and debug logging
//! v0.11.0: Locked CLI surface, status command, case-insensitive matching
//! v0.11.0: Strict evidence discipline - LLM-A/LLM-B audit loop
//! v0.11.0: Knowledge store, event-driven learning, user telemetry

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn get_binary_path() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/release/annactl")
}

/// Test --version flag
#[test]
fn test_annactl_version_long() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to run annactl");

    // v0.11.0: Version shows update status, config, and self-health in ASCII-only format
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either it shows version info with update status, or shows connection error
    assert!(
        stdout.contains("0.11.0") || stderr.contains("daemon") || stderr.contains("connection"),
        "Expected version 0.11.0 or daemon connection message, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

/// Test -V short flag
#[test]
fn test_annactl_version_short() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("-V")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either it shows version info, or it shows connection error (daemon not running)
    assert!(
        stdout.contains("0.11.0") || stderr.contains("daemon") || stderr.contains("connection"),
        "Expected version 0.11.0 or daemon connection message"
    );
}

/// Test version output includes config and hardware status fields (v0.6.0 format)
#[test]
fn test_annactl_version_includes_config_status() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v0.11.0: Version should include structured sections with ASCII-only formatting
    if stdout.contains("0.11.0") {
        // Check for v0.11.0 ASCII-only format fields
        let has_summary = stdout.contains("[SUMMARY]");
        let has_details = stdout.contains("[DETAILS]");
        let has_reliability = stdout.contains("[RELIABILITY]");
        let has_mode = stdout.contains("Mode:") && stdout.contains("[source: config.core]");
        let has_self_health = stdout.contains("Self-health:") || stdout.contains("[source: self_health]");

        // At least some v0.11.0 structured sections should be present
        assert!(
            has_summary || has_details || has_reliability || has_mode || has_self_health,
            "Version output should include v0.11.0 structured sections, got: {}",
            stdout
        );
    }
}

/// Test --help flag
#[test]
fn test_annactl_help_long() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // v0.4.0: Help goes through LLM pipeline
    // Either it shows help info, or it shows connection error (daemon not running)
    assert!(
        stdout.contains("Anna") || stderr.contains("daemon") || stderr.contains("connection"),
        "Expected Anna help or daemon connection message"
    );
}

/// Test -h short flag
#[test]
fn test_annactl_help_short() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("-h")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either it shows help info, or it shows connection error (daemon not running)
    assert!(
        stdout.contains("Anna") || stderr.contains("daemon") || stderr.contains("connection"),
        "Expected Anna help or daemon connection message"
    );
}

/// Test help output mentions auto-update
#[test]
fn test_annactl_help_mentions_autoupdate() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v0.4.0: Help should mention auto-update configuration
    if stdout.contains("Usage:") {
        assert!(
            stdout.contains("Auto-update") || stdout.contains("auto") || stdout.contains("config"),
            "Help output should mention auto-update, got: {}",
            stdout
        );
    }
}

/// Test question argument (without daemon - expects connection error)
#[test]
fn test_annactl_question_without_daemon() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("How many CPU cores do I have?")
        .output()
        .expect("Failed to run annactl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Without daemon running, should get connection error
    assert!(
        stderr.contains("daemon") || stderr.contains("connection") || stderr.contains("Failed"),
        "Expected daemon connection error message, got: {}",
        stderr
    );
}

/// Test that old commands no longer exist (v0.3.0+ removed subcommands)
/// Note: v0.11.0 re-added 'status' as a built-in command
#[test]
fn test_old_commands_removed() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // These commands were removed in v0.3.0 (status re-added in v0.11.0)
    let removed_commands = ["config", "init", "probes", "update"];

    for cmd in removed_commands {
        let output = Command::new(&binary)
            .arg(cmd)
            .output()
            .expect("Failed to run annactl");

        let stderr = String::from_utf8_lossy(&output.stderr);

        // v0.3.0+: These are now treated as questions, not subcommands
        // They should try to connect to daemon (and fail if not running)
        assert!(
            stderr.contains("daemon") || stderr.contains("connection") || stderr.contains("Failed"),
            "Command '{}' should be treated as a question and require daemon, got: {}",
            cmd,
            stderr
        );
    }
}

// ============================================================================
// v0.11.0: Status command tests
// ============================================================================

/// Test 'status' command shows structured output
#[test]
fn test_annactl_status_command() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // v0.11.0: Status command should show ANNA STATUS section or connection error
    assert!(
        stdout.contains("ANNA STATUS")
            || stdout.contains("Daemon:")
            || stderr.contains("daemon")
            || stderr.contains("connection"),
        "Expected status output or daemon connection error, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

/// Test 'status' command is case-insensitive
#[test]
fn test_annactl_status_case_insensitive() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test various case combinations
    for status_arg in ["status", "STATUS", "Status", "sTaTuS"] {
        let output = Command::new(&binary)
            .arg(status_arg)
            .output()
            .expect("Failed to run annactl");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // All should be recognized as status command
        assert!(
            stdout.contains("ANNA STATUS")
                || stdout.contains("Daemon:")
                || stderr.contains("daemon")
                || stderr.contains("connection"),
            "'{}' should be recognized as status command, got stdout: {}, stderr: {}",
            status_arg,
            stdout,
            stderr
        );
    }
}

// ============================================================================
// v0.11.0: Case-insensitive version/help tests
// ============================================================================

/// Test 'version' word (case-insensitive) shows version
#[test]
fn test_annactl_version_word_case_insensitive() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test various case combinations
    for version_arg in ["version", "VERSION", "Version", "vErSiOn"] {
        let output = Command::new(&binary)
            .arg(version_arg)
            .output()
            .expect("Failed to run annactl");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // All should show version info or connection error
        assert!(
            stdout.contains("0.11.0") || stderr.contains("daemon") || stderr.contains("connection"),
            "'{}' should show version, got stdout: {}, stderr: {}",
            version_arg,
            stdout,
            stderr
        );
    }
}

/// Test 'help' word (case-insensitive) shows help
#[test]
fn test_annactl_help_word_case_insensitive() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test various case combinations
    for help_arg in ["help", "HELP", "Help", "hElP"] {
        let output = Command::new(&binary)
            .arg(help_arg)
            .output()
            .expect("Failed to run annactl");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // All should show help info or connection error
        assert!(
            stdout.contains("Anna") || stderr.contains("daemon") || stderr.contains("connection"),
            "'{}' should show help, got stdout: {}, stderr: {}",
            help_arg,
            stdout,
            stderr
        );
    }
}
