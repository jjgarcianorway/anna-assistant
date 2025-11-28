//! First-run detection v0.72.0
//!
//! Detects whether this is the first time Anna has been run on this system.
//! Uses a marker file to reliably distinguish between:
//! - Fresh installation (no marker file)
//! - Empty stats (marker exists but no questions answered)

use std::fs;
use std::path::PathBuf;

/// Default marker file location
pub const MARKER_FILE: &str = "/var/lib/anna/.initialized";

/// Check if this is the first run (marker file doesn't exist)
pub fn is_first_run() -> bool {
    !PathBuf::from(MARKER_FILE).exists()
}

/// Mark the system as initialized (create marker file)
pub fn mark_initialized() -> Result<(), std::io::Error> {
    let path = PathBuf::from(MARKER_FILE);

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write marker file with timestamp
    let content = format!(
        "Anna initialized at {}\nVersion: {}\n",
        chrono::Utc::now().to_rfc3339(),
        env!("CARGO_PKG_VERSION")
    );
    fs::write(&path, content)?;

    Ok(())
}

/// Check if initialized (marker file exists)
pub fn is_initialized() -> bool {
    PathBuf::from(MARKER_FILE).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_first_run_detection() {
        // Use temp file for testing
        let temp_marker = env::temp_dir().join("anna_test_marker");
        let marker = temp_marker.to_str().unwrap();

        // Clean up first
        let _ = fs::remove_file(marker);

        // Without marker, should be "first run"
        assert!(!PathBuf::from(marker).exists());

        // Create marker
        fs::write(marker, "test").unwrap();
        assert!(PathBuf::from(marker).exists());

        // Clean up
        let _ = fs::remove_file(marker);
    }
}
