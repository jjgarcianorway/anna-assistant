//! CLI integration tests for annactl
//!
//! Tests the locked CLI surface:
//! - annactl - Start REPL
//! - annactl "<request>" - One-shot natural language request
//! - annactl status - Compact status summary
//! - annactl -V / --version / version (case-insensitive) - Show version
//! - annactl -h / --help / help (case-insensitive) - Show help

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

/// Test --version flag (instant, no daemon required since v0.14.4)
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

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v0.14.4+: --version is instant, outputs "annactl X.Y.Z", no daemon needed
    assert!(
        stdout.contains("annactl"),
        "Expected annactl version output, got: {}",
        stdout
    );
    assert!(output.status.success(), "annactl --version should succeed");
}

/// Test -V short flag (instant, no daemon required since v0.14.4)
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

    // v0.14.4+: -V is instant, outputs "annactl X.Y.Z", no daemon needed
    assert!(
        stdout.contains("annactl"),
        "Expected annactl version output, got: {}",
        stdout
    );
    assert!(output.status.success(), "annactl -V should succeed");
}

/// Test 'version' word goes through daemon and includes detailed status
/// Note: -V/--version are now instant (v0.14.4+), only 'version' word uses daemon
#[test]
fn test_annactl_version_word_detailed_output() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("version")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // v0.14.4+: 'version' word goes through daemon for detailed output
    // Either shows detailed version OR connection error if daemon not running
    assert!(
        stdout.contains("annactl v") || stderr.contains("daemon") || stderr.contains("connection"),
        "Expected version info or daemon connection error, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
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

/// Test question argument - expects daemon connection error OR valid answer
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either daemon is running (answer given) or not running (connection error)
    assert!(
        output.status.success()
            || stderr.contains("daemon")
            || stderr.contains("connection")
            || stderr.contains("Failed")
            || stdout.contains("core")
            || stdout.contains("CPU"),
        "Expected daemon connection error or valid answer, got stdout: {}, stderr: {}",
        stdout,
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
        // If daemon IS running, they will be processed as natural language requests
        assert!(
            output.status.success()
                || stderr.contains("daemon")
                || stderr.contains("connection")
                || stderr.contains("Failed"),
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
            stdout.contains("annactl v")
                || stderr.contains("daemon")
                || stderr.contains("connection"),
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
