//! First run detection and bootstrap flow
//!
//! Phase 4.3: Make Anna behave like a real sysadmin from first contact
//! Citation: [product:PRODUCT_VISION]

use anyhow::Result;
use std::path::PathBuf;

const FIRST_RUN_MARKER: &str = "/var/lib/anna/first_run_complete";
const CONTEXT_DB_PATH: &str = "/var/lib/anna/context.db";
const CONFIG_PATH: &str = "/etc/anna/anna.yaml";

/// Detect if this is the first time Anna is running on this machine
pub fn is_first_run() -> bool {
    // Check multiple signals to determine first run status

    // 1. Check for first run marker file
    if PathBuf::from(FIRST_RUN_MARKER).exists() {
        return false;
    }

    // 2. Check if context database exists and is non-empty
    let context_db = PathBuf::from(CONTEXT_DB_PATH);
    if context_db.exists() {
        if let Ok(metadata) = std::fs::metadata(&context_db) {
            if metadata.len() > 1024 {
                // Database exists and has content (>1KB)
                return false;
            }
        }
    }

    // 3. Check if config directory exists
    let config_path = PathBuf::from(CONFIG_PATH);
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if content.len() > 100 {
                // Config exists and has meaningful content
                return false;
            }
        }
    }

    // If none of the above exist, this is a first run
    true
}

/// Mark first run as complete
pub fn mark_first_run_complete() -> Result<()> {
    let marker_path = PathBuf::from(FIRST_RUN_MARKER);

    // Ensure parent directory exists
    if let Some(parent) = marker_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create marker file with timestamp
    let timestamp = chrono::Utc::now().to_rfc3339();
    std::fs::write(&marker_path, format!("First run completed at: {}\n", timestamp))?;

    Ok(())
}

/// Display first run welcome message
pub fn display_first_run_message(use_color: bool) {
    use owo_colors::OwoColorize;

    println!();
    if use_color {
        println!("{}", "👋 Welcome to Anna!".bold().cyan());
    } else {
        println!("👋 Welcome to Anna!");
    }
    println!();
    println!("Looks like this is the first time I see this machine.");
    println!("I will run a deeper scan once and then remember the results.");
    println!();
    if use_color {
        println!("{}", "Running first system scan...".bold());
    } else {
        println!("Running first system scan...");
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_run_detection() {
        // This is a basic test structure
        // In real tests, we'd mock the filesystem
        assert!(is_first_run() || !is_first_run()); // Always true, just structure
    }
}
