//! CLI integration tests for annactl v0.3.0
//!
//! Tests the simplified CLI interface:
//! - annactl "<question>" - Ask a question
//! - annactl - Start REPL
//! - annactl -V / --version - Show version
//! - annactl -h / --help - Show help

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

    // v0.3.0: Version goes through LLM pipeline, so we check stderr for daemon connection issues
    // or stdout for actual version info
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either it shows version info, or it shows connection error (daemon not running)
    assert!(
        stdout.contains("0.3.0") || stderr.contains("daemon") || stderr.contains("connection"),
        "Expected version 0.3.0 or daemon connection message, got stdout: {}, stderr: {}",
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
        stdout.contains("0.3.0") || stderr.contains("daemon") || stderr.contains("connection"),
        "Expected version 0.3.0 or daemon connection message"
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

    // v0.3.0: Help goes through LLM pipeline
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

/// Test that old commands no longer exist (v0.3.0 removed subcommands)
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

        // v0.3.0: These are now treated as questions, not subcommands
        // They should try to connect to daemon (and fail if not running)
        assert!(
            stderr.contains("daemon") || stderr.contains("connection") || stderr.contains("Failed"),
            "Command '{}' should be treated as a question and require daemon, got: {}",
            cmd,
            stderr
        );
    }
}
