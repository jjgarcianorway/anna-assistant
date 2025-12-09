//! Helper package detection (v0.0.221).

use std::path::PathBuf;

use super::registry::HelpersRegistry;
use super::types::{HelperPackage, InstallSource};

/// Known helper packages for Anna.
/// Returns a registry with default package definitions (not detected yet).
pub fn known_helpers() -> HelpersRegistry {
    let mut registry = HelpersRegistry::new();

    // Ollama - LLM backend
    registry.register(HelperPackage::new("ollama", "Ollama").required());

    registry
}

/// Detect a helper package on the system.
/// Returns updated package with availability and path info.
pub fn detect_helper(id: &str) -> Option<HelperPackage> {
    match id {
        "ollama" => detect_ollama(),
        _ => None,
    }
}

/// Detect ollama installation.
fn detect_ollama() -> Option<HelperPackage> {
    // Check common paths
    let paths = [
        "/usr/local/bin/ollama",
        "/usr/bin/ollama",
        "/opt/homebrew/bin/ollama", // macOS
    ];

    for path in paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(
                HelperPackage::new("ollama", "Ollama")
                    .required()
                    .with_available(true)
                    .with_binary_path(p)
                    .with_source(InstallSource::User), // Assume user if found
            );
        }
    }

    // Check if ollama is in PATH (will be handled by caller via `which ollama`)
    None
}
