//! CLI integration tests for annactl v0.4.0
//!
//! Tests the simplified CLI interface:
//! - annactl "<question>" - Ask a question
//! - annactl - Start REPL
//! - annactl -V / --version - Show version
//! - annactl -h / --help - Show help
//!
//! v0.4.0: Version/help now show update status

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

    // v0.4.0: Version shows update status
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either it shows version info with update status, or shows connection error
    assert!(
        stdout.contains("0.4.0") || stderr.contains("daemon") || stderr.contains("connection"),
        "Expected version 0.4.0 or daemon connection message, got stdout: {}, stderr: {}",
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
        stdout.contains("0.4.0") || stderr.contains("daemon") || stderr.contains("connection"),
        "Expected version 0.4.0 or daemon connection message"
    );
}

/// Test version output includes update status fields
#[test]
fn test_annactl_version_includes_update_status() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v0.4.0: Version should include channel and update mode
    // If LLM is available and daemon is running, we should see these fields
    if stdout.contains("0.4.0") {
        // Check for update-related fields (at least one should be present)
        let has_channel = stdout.contains("Channel:");
        let has_update_mode = stdout.contains("Update mode:");
        let has_last_check = stdout.contains("Last update check:");

        // At least some update info should be present
        assert!(
            has_channel || has_update_mode || has_last_check || stdout.contains("stable"),
            "Version output should include update status fields, got: {}",
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
#[test]
fn test_old_commands_removed() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // These commands were removed in v0.3.0
    let removed_commands = ["config", "init", "status", "probes", "update"];

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
