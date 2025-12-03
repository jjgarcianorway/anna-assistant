//! CLI integration tests for annactl v0.0.2 - Strict CLI Surface
//!
//! Public CLI surface (strict):
//! - annactl                  REPL mode (interactive)
//! - annactl <request...>     one-shot natural language request
//! - annactl status           self-status
//! - annactl --version        version (also: -V)
//!
//! All other commands (sw, hw, etc.) route through natural language processing.

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
// v0.0.2: Help command tests
// ============================================================================

/// Test --help shows only the strict CLI surface
#[test]
fn test_annactl_help_strict_surface() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v0.0.2: Help shows only strict surface
    assert!(
        stdout.contains("Anna Assistant"),
        "Expected 'Anna Assistant' header, got: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl status"),
        "Help should mention status command"
    );
    assert!(
        stdout.contains("annactl --version"),
        "Help should mention --version"
    );
    assert!(
        stdout.contains("annactl <request>"),
        "Help should mention natural language request"
    );

    // v0.0.2: Help must NOT advertise sw/hw commands
    assert!(
        !stdout.contains("annactl sw"),
        "Help should NOT mention sw command (removed from public surface)"
    );
    assert!(
        !stdout.contains("annactl hw"),
        "Help should NOT mention hw command (removed from public surface)"
    );

    assert!(output.status.success(), "annactl --help should succeed");
}

/// Test -h also shows help
#[test]
fn test_annactl_h_flag_shows_help() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("-h")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Anna Assistant"),
        "Expected help output with -h flag"
    );
    assert!(output.status.success(), "annactl -h should succeed");
}

// ============================================================================
// v0.0.2: Version command tests
// ============================================================================

/// Test --version prints exact format "annactl vX.Y.Z"
#[test]
fn test_annactl_version_format() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout.trim();

    // Must match exact format: "annactl vX.Y.Z"
    assert!(
        stdout.starts_with("annactl v"),
        "Version should start with 'annactl v', got: {}",
        stdout
    );
    assert!(
        stdout.contains("0.0.2"),
        "Version should contain 0.0.2, got: {}",
        stdout
    );
    assert!(output.status.success(), "annactl --version should succeed");
}

/// Test -V also prints version
#[test]
fn test_annactl_v_flag_prints_version() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("-V")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout.trim();

    assert!(
        stdout.starts_with("annactl v"),
        "Version should start with 'annactl v', got: {}",
        stdout
    );
    assert!(output.status.success(), "annactl -V should succeed");
}

// ============================================================================
// v0.0.2: Status command tests
// ============================================================================

/// Test 'status' command exits 0 and shows status header
#[test]
fn test_annactl_status_exits_zero() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Status command shows Anna Status header
    assert!(
        stdout.contains("Anna Status") || stdout.contains("[VERSION]"),
        "Expected status output, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl status should exit 0");
}

// ============================================================================
// v0.0.2: Legacy command routing tests
// ============================================================================

/// Test 'sw' routes as natural language request (no custom error message)
#[test]
fn test_annactl_sw_routes_as_request() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("sw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v0.0.2: 'sw' should be treated as a natural language request
    // It should show the dialogue format, not a custom error
    assert!(
        stdout.contains("[you] to [anna]") || stdout.contains("sw"),
        "Expected 'sw' to be processed as request, got: {}",
        stdout
    );

    // Must NOT show custom "unrecognized command" error
    assert!(
        !stdout.contains("is not a recognized command"),
        "'sw' should not trigger custom error message"
    );

    // The command should succeed (it's a valid request, just not implemented yet)
    assert!(output.status.success(), "annactl sw should succeed as a request");
}

/// Test 'hw' routes as natural language request
#[test]
fn test_annactl_hw_routes_as_request() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v0.0.2: 'hw' should be treated as a natural language request
    assert!(
        stdout.contains("[you] to [anna]") || stdout.contains("hw"),
        "Expected 'hw' to be processed as request, got: {}",
        stdout
    );

    // Must NOT show custom error
    assert!(
        !stdout.contains("is not a recognized command"),
        "'hw' should not trigger custom error message"
    );

    assert!(output.status.success(), "annactl hw should succeed as a request");
}

/// Test natural language request format
#[test]
fn test_annactl_natural_language_request() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("what CPU do I have?")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show dialogue format
    assert!(
        stdout.contains("[you] to [anna]"),
        "Expected dialogue format, got: {}",
        stdout
    );
    assert!(
        stdout.contains("[anna] to [you]"),
        "Expected anna response, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Reliability:"),
        "Expected reliability score, got: {}",
        stdout
    );

    assert!(output.status.success(), "natural language request should succeed");
}

/// Test multi-word request
#[test]
fn test_annactl_multiword_request() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["show", "me", "disk", "usage"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should join args and process as single request
    assert!(
        stdout.contains("show me disk usage") || stdout.contains("[you] to [anna]"),
        "Expected multi-word request to be processed, got: {}",
        stdout
    );

    assert!(output.status.success(), "multi-word request should succeed");
}
