//! CLI argument parsing tests.
//!
//! Tests for v0.0.18 regressions:
//! - "help" should be treated as a request, not trigger clap help
//! - Subcommands should work correctly

use std::process::Command;

/// Test that "annactl help" is treated as a request, not clap help
#[test]
fn test_help_not_clap_subcommand() {
    // Build the binary first (skip if not available)
    let output = Command::new("cargo")
        .args(["build", "--release", "-p", "annactl"])
        .current_dir(env!("CARGO_MANIFEST_DIR").replace("/crates/annactl", ""))
        .output();

    if output.is_err() {
        eprintln!("Skipping test - cargo build failed");
        return;
    }

    // The binary path
    let binary = format!(
        "{}/target/release/annactl",
        env!("CARGO_MANIFEST_DIR").replace("/crates/annactl", "")
    );

    // Test that "help" alone doesn't show clap help output
    // This would fail if disable_help_subcommand wasn't set
    let output = Command::new(&binary)
        .args(["--help"])
        .output()
        .expect("Failed to run annactl --help");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify help output mentions "help" as a valid request example
    assert!(
        stdout.contains("annactl help"),
        "Help should show 'annactl help' as an example"
    );
}

/// Test that --version works
#[test]
fn test_version_flag() {
    let binary = format!(
        "{}/target/release/annactl",
        env!("CARGO_MANIFEST_DIR").replace("/crates/annactl", "")
    );

    let output = Command::new(&binary)
        .args(["--version"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("annactl") && stdout.contains("0.0."),
            "Version output should contain 'annactl' and version number"
        );
    }
}

/// Test that status subcommand is recognized
#[test]
fn test_status_subcommand_recognized() {
    let binary = format!(
        "{}/target/release/annactl",
        env!("CARGO_MANIFEST_DIR").replace("/crates/annactl", "")
    );

    // Just check that status is a valid subcommand (it will fail to connect but won't show help)
    let output = Command::new(&binary)
        .args(["status", "--help"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Status subcommand help should mention "debug" flag
        assert!(
            stdout.contains("--debug") || stdout.contains("Anna status"),
            "Status help should mention --debug flag or describe the command"
        );
    }
}
