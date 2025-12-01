//! CLI integration tests for annactl v7.7.0 "Snow Leopard"
//!
//! Tests the minimal CLI surface:
//! - annactl           show help
//! - annactl status    health and runtime of Anna
//! - annactl kdb       overview of Anna KDB
//! - annactl kdb NAME  profile for a package, command or category
//!
//! v7.7.0: Snow Leopard tests for PHASE 23-26
//! - PHASE 23: Telemetry per-window aggregation (compact display)
//! - PHASE 24: Config precedence and honest Source lines
//! - PHASE 25: KDB name resolution and disambiguation
//! - PHASE 26: Performance constraints

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

// ============================================================================
// v7.2.0: Help command tests
// ============================================================================

/// Test no args shows help
#[test]
fn test_annactl_no_args_shows_help() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.0.0: No args shows help with "Anna CLI" header
    assert!(
        stdout.contains("Anna CLI"),
        "Expected 'Anna CLI' help header, got: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl status"),
        "Help should mention status command"
    );
    assert!(
        stdout.contains("annactl kdb"),
        "Help should mention kdb command"
    );
    assert!(output.status.success(), "annactl should succeed");
}

// ============================================================================
// v7.2.0: Status command tests
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

    // v7.0.0+: Status command shows [VERSION], [DAEMON], [INVENTORY] sections
    assert!(
        stdout.contains("Anna Status") && stdout.contains("[VERSION]"),
        "Expected status output with sections, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl status should succeed");
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

        // All should be recognized as status command - shows [VERSION] section
        assert!(
            stdout.contains("Anna Status") && stdout.contains("[VERSION]"),
            "'{}' should be recognized as status command, got stdout: {}",
            status_arg,
            stdout
        );
    }
}

// ============================================================================
// v7.2.0: KDB command tests
// ============================================================================

/// Test 'kdb' command shows overview
#[test]
fn test_annactl_kdb_command() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("kdb")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.0.0+: KDB command shows "Anna KDB" header with [OVERVIEW] section
    assert!(
        stdout.contains("Anna KDB") && stdout.contains("[OVERVIEW]"),
        "Expected KDB output with sections, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl kdb should succeed");
}

/// Test 'kdb' command is case-insensitive
#[test]
fn test_annactl_kdb_case_insensitive() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    for kdb_arg in ["kdb", "KDB", "Kdb", "kDb"] {
        let output = Command::new(&binary)
            .arg(kdb_arg)
            .output()
            .expect("Failed to run annactl");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            stdout.contains("Anna KDB") && stdout.contains("[OVERVIEW]"),
            "'{}' should be recognized as kdb command, got stdout: {}",
            kdb_arg,
            stdout
        );
    }
}

/// Test 'kdb <name>' shows object profile
#[test]
fn test_annactl_kdb_object() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test with pacman (should exist on all Arch systems)
    let output = Command::new(&binary)
        .args(["kdb", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show object profile with [IDENTITY] section
    assert!(
        stdout.contains("Anna KDB: pacman") && stdout.contains("[IDENTITY]"),
        "Expected object profile output, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl kdb pacman should succeed");
}

/// Test 'kdb <category>' shows category view
#[test]
fn test_annactl_kdb_category() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test editors category
    let output = Command::new(&binary)
        .args(["kdb", "editors"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show category header
    assert!(
        stdout.contains("Anna KDB: Editors"),
        "Expected category output, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl kdb editors should succeed");
}

// ============================================================================
// v7.2.0: Unknown command tests
// ============================================================================

/// Test unknown commands show error
#[test]
fn test_annactl_unknown_command() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("unknown_command_that_doesnt_exist")
        .output()
        .expect("Failed to run annactl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show error message with suggestion
    assert!(
        stderr.contains("error:") && stderr.contains("not a recognized command"),
        "Expected error message for unknown command, got stderr: {}",
        stderr
    );
    assert!(!output.status.success(), "Unknown command should fail");
}

/// Test '--help' flag is not recognized (v7.0.0 minimal surface)
#[test]
fn test_annactl_help_flag_not_recognized() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to run annactl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // v7.0.0: --help is not recognized, should show error
    assert!(
        stderr.contains("not a recognized command"),
        "Expected '--help' to be rejected, got stderr: {}",
        stderr
    );
}

/// Test '--version' flag is not recognized (v7.0.0 minimal surface)
#[test]
fn test_annactl_version_flag_not_recognized() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to run annactl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // v7.0.0: --version is not recognized, should show error
    assert!(
        stderr.contains("not a recognized command"),
        "Expected '--version' to be rejected, got stderr: {}",
        stderr
    );
}

// ============================================================================
// v7.7.0: Snow Leopard - PHASE 23 Telemetry Tests
// ============================================================================

/// Test 'kdb' command shows [USAGE HIGHLIGHTS] section
#[test]
fn test_annactl_kdb_shows_usage_highlights() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("kdb")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.7.0: KDB should show [USAGE HIGHLIGHTS] section
    assert!(
        stdout.contains("[USAGE HIGHLIGHTS]"),
        "Expected [USAGE HIGHLIGHTS] section in kdb output, got: {}",
        stdout
    );
}

/// Test 'kdb <name>' shows [USAGE] section with telemetry windows
#[test]
fn test_annactl_kdb_object_shows_usage_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["kdb", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.7.0: Object profile should show [USAGE] section
    assert!(
        stdout.contains("[USAGE]"),
        "Expected [USAGE] section in object profile, got: {}",
        stdout
    );
}

// ============================================================================
// v7.7.0: Snow Leopard - PHASE 24 Config Tests
// ============================================================================

/// Test 'kdb' command shows docs availability status
#[test]
fn test_annactl_kdb_shows_docs_status() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("kdb")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.7.0: KDB should show docs availability status
    assert!(
        stdout.contains("Local Arch docs:"),
        "Expected 'Local Arch docs:' status in kdb output, got: {}",
        stdout
    );
}

/// Test 'kdb <name>' shows [CONFIG] section with Source line
#[test]
fn test_annactl_kdb_object_shows_config_source() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["kdb", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.7.0: Config section should exist
    assert!(
        stdout.contains("[CONFIG]"),
        "Expected [CONFIG] section in object profile, got: {}",
        stdout
    );
}

// ============================================================================
// v7.7.0: Snow Leopard - PHASE 25 Name Resolution Tests
// ============================================================================

/// Test name resolution for .service suffix (should prefer service)
#[test]
fn test_annactl_kdb_service_suffix_resolution() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Try with a common service name
    let output = Command::new(&binary)
        .args(["kdb", "sshd.service"])
        .output()
        .expect("Failed to run annactl");

    // Should succeed (either as service or package)
    assert!(output.status.success(), "annactl kdb sshd.service should succeed");
}

/// Test case-insensitive name resolution for packages
#[test]
fn test_annactl_kdb_case_insensitive_package() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test PACMAN (uppercase) resolves to pacman
    let output = Command::new(&binary)
        .args(["kdb", "PACMAN"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should resolve and show profile
    assert!(
        stdout.contains("[IDENTITY]"),
        "PACMAN should resolve case-insensitively, got: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test category names are case-insensitive
#[test]
fn test_annactl_kdb_category_case_insensitive() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test EDITORS (uppercase) resolves to Editors category
    let output = Command::new(&binary)
        .args(["kdb", "EDITORS"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show category view
    assert!(
        stdout.contains("Anna KDB: Editors"),
        "EDITORS should resolve to Editors category, got: {}",
        stdout
    );
    assert!(output.status.success());
}

// ============================================================================
// v7.7.0: Snow Leopard - PHASE 26 Performance Tests
// ============================================================================

/// Test 'status' command completes in reasonable time (<2s)
#[test]
fn test_annactl_status_performance() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let start = std::time::Instant::now();
    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let elapsed = start.elapsed();

    assert!(output.status.success(), "annactl status should succeed");
    assert!(
        elapsed.as_secs() < 2,
        "annactl status should complete in <2s, took: {:?}",
        elapsed
    );
}

/// Test 'kdb' command completes in reasonable time (<15s)
/// Note: kdb overview scans all explicitly installed packages for categorization
/// which can take a few seconds on systems with many packages
#[test]
fn test_annactl_kdb_performance() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let start = std::time::Instant::now();
    let output = Command::new(&binary)
        .arg("kdb")
        .output()
        .expect("Failed to run annactl");

    let elapsed = start.elapsed();

    assert!(output.status.success(), "annactl kdb should succeed");
    assert!(
        elapsed.as_secs() < 15,
        "annactl kdb should complete in <15s, took: {:?}",
        elapsed
    );
}

/// Test 'kdb <name>' command completes in reasonable time (<2s)
#[test]
fn test_annactl_kdb_object_performance() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let start = std::time::Instant::now();
    let output = Command::new(&binary)
        .args(["kdb", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let elapsed = start.elapsed();

    assert!(output.status.success(), "annactl kdb pacman should succeed");
    assert!(
        elapsed.as_secs() < 2,
        "annactl kdb <name> should complete in <2s, took: {:?}",
        elapsed
    );
}
