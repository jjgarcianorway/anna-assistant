//! Pacman package info fetching (v0.0.422).

use std::process::Command;

use super::types::SourceFetchResult;
use super::utils::{is_safe_name, truncate_doc};

/// Fetch pacman package info
pub fn fetch_pacman_info(package: &str) -> Option<SourceFetchResult> {
    if !is_safe_name(package) {
        return None;
    }

    // Get package info
    let output = Command::new("pacman")
        .args(["-Qi", package])
        .output()
        .ok()?;

    if !output.status.success() {
        // Try searching for package
        let search = Command::new("pacman")
            .args(["-Ss", package])
            .output()
            .ok()?;

        if search.status.success() {
            let content = String::from_utf8_lossy(&search.stdout);
            let truncated = truncate_doc(&content, 20);
            return Some(SourceFetchResult::new(
                truncated,
                &format!("pacman -Ss {}", package),
            ));
        }
        return None;
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();
    Some(SourceFetchResult::new(
        content,
        &format!("pacman -Qi {}", package),
    ))
}
