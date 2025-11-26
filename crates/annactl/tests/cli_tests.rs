//! CLI integration tests for annactl

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

/// Test binary execution
#[test]
fn test_annactl_version() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to run annactl");

    assert!(output.status.success());
    let version = String::from_utf8_lossy(&output.stdout);
    assert!(version.contains("0.2.0"));
}

/// Test annactl help
#[test]
fn test_annactl_help() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to run annactl");

    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("Anna"));
    assert!(help.contains("ask"));
    assert!(help.contains("status"));
    assert!(help.contains("probes"));
}

/// Test annactl config (doesn't need daemon)
#[test]
fn test_annactl_config() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("config")
        .output()
        .expect("Failed to run annactl config");

    assert!(output.status.success());
    let config = String::from_utf8_lossy(&output.stdout);
    assert!(config.contains("0.2.0"));
    assert!(config.contains("Orchestrator"));
    assert!(config.contains("Expert"));
}

/// Test annactl init (doesn't need daemon)
#[test]
fn test_annactl_init() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("init")
        .output()
        .expect("Failed to run annactl init");

    assert!(output.status.success());
    let init = String::from_utf8_lossy(&output.stdout);
    assert!(init.contains("Detecting hardware"));
    assert!(init.contains("RAM"));
    assert!(init.contains("CPU"));
}
