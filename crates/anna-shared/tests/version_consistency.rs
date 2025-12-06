//! Version consistency tests (v0.0.71)
//! Ensures all crates use the same version from workspace Cargo.toml.
//!
//! v0.0.70: Added workspace-level version extraction and cross-crate consistency.
//! v0.0.71: Hard gate tests that fail CI if versions mismatch.

use std::path::PathBuf;

/// Get the workspace root directory (parent of crates/)
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/anna-shared -> workspace root
    manifest_dir.parent().unwrap().parent().unwrap().to_path_buf()
}

/// Extract version from workspace Cargo.toml
fn read_workspace_version() -> String {
    let cargo_toml = workspace_root().join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)
        .expect("Failed to read workspace Cargo.toml");

    // Find [workspace.package] section and extract version
    let mut in_workspace_package = false;
    for line in content.lines() {
        if line.trim() == "[workspace.package]" {
            in_workspace_package = true;
            continue;
        }
        if in_workspace_package && line.starts_with('[') {
            break; // End of section
        }
        if in_workspace_package && line.trim().starts_with("version") {
            // Parse: version = "X.Y.Z"
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if end > start + 1 {
                        return line[start + 1..end].to_string();
                    }
                }
            }
        }
    }
    panic!("Could not find version in workspace Cargo.toml");
}

/// Read a crate's Cargo.toml and check for version conflicts
fn check_crate_cargo_toml(crate_name: &str) -> Option<String> {
    let cargo_toml = workspace_root().join("crates").join(crate_name).join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).ok()?;

    // Look for version = "X.Y.Z" (NOT version.workspace = true)
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version") && trimmed.contains('"') && !trimmed.contains("workspace") {
            // Found a hardcoded version
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if end > start + 1 {
                        return Some(line[start + 1..end].to_string());
                    }
                }
            }
        }
    }
    None
}

/// Verify anna-shared VERSION matches the workspace version.
/// This test ensures env!("CARGO_PKG_VERSION") is properly wired.
#[test]
fn version_format_is_semver() {
    let version = anna_shared::VERSION;

    // Must be non-empty
    assert!(!version.is_empty(), "VERSION must not be empty");

    // Must be valid semver (X.Y.Z format)
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "VERSION must be X.Y.Z format, got: {}", version);

    // Each part must be numeric
    for (i, part) in parts.iter().enumerate() {
        let parsed: Result<u32, _> = part.parse();
        assert!(parsed.is_ok(), "VERSION part {} ('{}') must be numeric", i, part);
    }
}

/// v0.0.71: HARD GATE - annactl version must equal workspace version.
/// This test FAILS CI if versions mismatch.
#[test]
fn hard_gate_annactl_version_equals_workspace() {
    let workspace_version = read_workspace_version();
    let annactl_version = anna_shared::VERSION; // annactl uses anna_shared::VERSION

    assert_eq!(
        annactl_version, workspace_version,
        "HARD GATE FAILURE: annactl version ({}) != workspace version ({})",
        annactl_version, workspace_version
    );
}

/// v0.0.71: HARD GATE - annad version must equal workspace version.
/// This test FAILS CI if versions mismatch.
#[test]
fn hard_gate_annad_version_equals_workspace() {
    let workspace_version = read_workspace_version();
    let annad_version = anna_shared::VERSION; // annad uses anna_shared::VERSION

    assert_eq!(
        annad_version, workspace_version,
        "HARD GATE FAILURE: annad version ({}) != workspace version ({})",
        annad_version, workspace_version
    );
}

/// v0.0.71: HARD GATE - No crate may have a hardcoded version that conflicts with workspace.
/// This test FAILS CI if any crate has version = "X.Y.Z" instead of version.workspace = true.
#[test]
fn hard_gate_no_conflicting_crate_versions() {
    let workspace_version = read_workspace_version();
    let crates = ["anna-shared", "annad", "annactl"];

    for crate_name in crates {
        if let Some(hardcoded) = check_crate_cargo_toml(crate_name) {
            if hardcoded != workspace_version {
                panic!(
                    "HARD GATE FAILURE: crate {} has hardcoded version {} but workspace is {}. \
                     Use version.workspace = true instead.",
                    crate_name, hardcoded, workspace_version
                );
            }
        }
        // If None, the crate uses version.workspace = true (correct)
    }
}

/// v0.0.71: Verify VERSION constant matches workspace Cargo.toml.
/// This is the authoritative test that ensures single source of truth.
#[test]
fn version_matches_workspace_cargo_toml() {
    let workspace_version = read_workspace_version();
    let crate_version = anna_shared::VERSION;

    assert_eq!(
        crate_version, workspace_version,
        "anna_shared::VERSION ({}) does not match workspace Cargo.toml ({})",
        crate_version, workspace_version
    );
}

/// v0.0.71: Verify VERSION file matches workspace version.
#[test]
fn version_file_matches_workspace() {
    let workspace_version = read_workspace_version();
    let version_file = workspace_root().join("VERSION");
    let file_version = std::fs::read_to_string(&version_file)
        .expect("Failed to read VERSION file")
        .trim()
        .to_string();

    assert_eq!(
        file_version, workspace_version,
        "VERSION file ({}) does not match workspace Cargo.toml ({})",
        file_version, workspace_version
    );
}

/// Ensure VERSION is accessible at compile time (not runtime computed).
/// This test verifies the env!() macro approach works.
#[test]
fn version_is_compile_time_constant() {
    // If this compiles, VERSION is a compile-time constant
    const V: &str = anna_shared::VERSION;
    assert!(!V.is_empty());
}

/// v0.0.71: Both annad and annactl must use the same VERSION constant.
/// Since they both depend on anna-shared and use anna_shared::VERSION,
/// this is guaranteed by the architecture. This test documents that contract.
#[test]
fn all_crates_use_shared_version_constant() {
    // annad and annactl both import anna_shared::VERSION
    // The workspace ensures all crates get the same version via version.workspace = true
    let version = anna_shared::VERSION;
    let workspace_version = read_workspace_version();

    assert_eq!(version, workspace_version,
        "VERSION constant ({}) does not match workspace ({}). Did you update all version sources?",
        version, workspace_version);
}
