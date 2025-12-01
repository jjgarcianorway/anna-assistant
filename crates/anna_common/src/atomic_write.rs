//! Atomic File Write v5.5.2
//!
//! Provides atomic file write operations to prevent data corruption.
//! Uses the write-to-temp-then-rename pattern:
//! 1. Write to temporary file in same directory
//! 2. fsync the file
//! 3. Rename over target (atomic on POSIX)
//!
//! This ensures that readers always see either the old or new complete file,
//! never a partial write.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

/// Atomically write content to a file
///
/// # Arguments
/// * `path` - Target file path
/// * `content` - Content to write
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(io::Error)` on failure
///
/// # Example
/// ```ignore
/// atomic_write("/var/lib/anna/knowledge/knowledge_v5.json", json_content)?;
/// ```
pub fn atomic_write(path: &str, content: &str) -> io::Result<()> {
    let path = Path::new(path);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create temp file in same directory (ensures same filesystem for atomic rename)
    let dir = path.parent().unwrap_or(Path::new("."));
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("temp");
    let temp_path = dir.join(format!(".{}.tmp.{}", filename, std::process::id()));

    // Write to temp file with restricted permissions
    {
        #[cfg(unix)]
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o640)  // rw-r----- (owner read/write, group read)
            .open(&temp_path)?;

        #[cfg(not(unix))]
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;

        file.write_all(content.as_bytes())?;

        // Ensure data is flushed to disk
        file.sync_all()?;
    }

    // Atomic rename
    fs::rename(&temp_path, path)?;

    // Sync parent directory to ensure rename is durable
    #[cfg(unix)]
    if let Ok(dir_file) = File::open(dir) {
        let _ = dir_file.sync_all();
    }

    Ok(())
}

/// Atomically write bytes to a file
pub fn atomic_write_bytes(path: &str, content: &[u8]) -> io::Result<()> {
    let path = Path::new(path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let dir = path.parent().unwrap_or(Path::new("."));
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("temp");
    let temp_path = dir.join(format!(".{}.tmp.{}", filename, std::process::id()));

    {
        #[cfg(unix)]
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o640)
            .open(&temp_path)?;

        #[cfg(not(unix))]
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;

        file.write_all(content)?;
        file.sync_all()?;
    }

    fs::rename(&temp_path, path)?;

    #[cfg(unix)]
    if let Ok(dir_file) = File::open(dir) {
        let _ = dir_file.sync_all();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_atomic_write() {
        let temp_dir = env::temp_dir();
        let test_path = temp_dir.join("anna_atomic_test.json");
        let path_str = test_path.to_str().unwrap();

        // Write some content
        atomic_write(path_str, r#"{"test": "value"}"#).unwrap();

        // Verify content
        let content = fs::read_to_string(&test_path).unwrap();
        assert_eq!(content, r#"{"test": "value"}"#);

        // Clean up
        let _ = fs::remove_file(&test_path);
    }

    #[test]
    fn test_atomic_overwrite() {
        let temp_dir = env::temp_dir();
        let test_path = temp_dir.join("anna_atomic_overwrite.json");
        let path_str = test_path.to_str().unwrap();

        // Write initial content
        atomic_write(path_str, "initial").unwrap();

        // Overwrite with new content
        atomic_write(path_str, "updated").unwrap();

        // Verify new content
        let content = fs::read_to_string(&test_path).unwrap();
        assert_eq!(content, "updated");

        // Clean up
        let _ = fs::remove_file(&test_path);
    }
}
