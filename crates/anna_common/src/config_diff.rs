//! Config Diff - Generate unified diffs for configuration files
//!
//! v6.51.0: Show "what changed" for file modifications with backups

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Generate a unified diff between a backup and current file
///
/// Returns None if either file doesn't exist or can't be read
pub fn generate_diff(backup_path: &str, current_path: &str) -> Result<Option<String>> {
    let backup_path = Path::new(backup_path);
    let current_path = Path::new(current_path);

    // Check if both files exist
    if !backup_path.exists() {
        return Ok(None);
    }
    if !current_path.exists() {
        return Ok(Some(format!(
            "File {} was deleted",
            current_path.display()
        )));
    }

    // Read both files
    let backup_content = fs::read_to_string(backup_path)
        .context(format!("Failed to read backup: {}", backup_path.display()))?;
    let current_content = fs::read_to_string(current_path)
        .context(format!("Failed to read current: {}", current_path.display()))?;

    // Generate diff
    let diff = unified_diff(&backup_content, &current_content, backup_path, current_path);

    if diff.is_empty() {
        Ok(Some("No changes detected".to_string()))
    } else {
        Ok(Some(diff))
    }
}

/// Generate a unified diff between two strings
fn unified_diff(old: &str, new: &str, old_path: &Path, new_path: &Path) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // Check if content is identical
    if old == new {
        return String::new();
    }

    // Simple line-by-line diff (not a full LCS algorithm, but sufficient)
    let mut output = String::new();

    // Header
    output.push_str(&format!("--- {}\n", old_path.display()));
    output.push_str(&format!("+++ {}\n", new_path.display()));

    // Find differences
    let mut i = 0;
    let mut j = 0;

    while i < old_lines.len() || j < new_lines.len() {
        if i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
            // Lines match, skip
            i += 1;
            j += 1;
        } else {
            // Found a difference - collect context and show hunk
            let hunk = generate_hunk(&old_lines, &new_lines, &mut i, &mut j);
            if !hunk.is_empty() {
                output.push_str(&hunk);
            }
        }
    }

    output
}

/// Generate a diff hunk starting at the current position
fn generate_hunk(
    old_lines: &[&str],
    new_lines: &[&str],
    i: &mut usize,
    j: &mut usize,
) -> String {
    let mut hunk = String::new();

    // Collect context before
    let context_before = 3;
    let start_i = i.saturating_sub(context_before);
    let start_j = j.saturating_sub(context_before);

    // Hunk header
    hunk.push_str(&format!("@@ -{},{} +{},{} @@\n", start_i + 1, old_lines.len() - start_i, start_j + 1, new_lines.len() - start_j));

    // Show context before
    for k in start_i..*i {
        if k < old_lines.len() {
            hunk.push_str(&format!(" {}\n", old_lines[k]));
        }
    }

    // Show differences
    let mut changes = 0;
    while *i < old_lines.len() && *j < new_lines.len() && old_lines[*i] != new_lines[*j] {
        // Removed lines
        if *i < old_lines.len() {
            hunk.push_str(&format!("-{}\n", old_lines[*i]));
            *i += 1;
            changes += 1;
        }
        // Added lines
        if *j < new_lines.len() {
            hunk.push_str(&format!("+{}\n", new_lines[*j]));
            *j += 1;
            changes += 1;
        }

        // Limit hunk size
        if changes > 20 {
            break;
        }
    }

    // Handle remaining deletions
    while *i < old_lines.len()
        && (*j >= new_lines.len() || old_lines[*i] != new_lines[*j])
        && changes < 20
    {
        hunk.push_str(&format!("-{}\n", old_lines[*i]));
        *i += 1;
        changes += 1;
    }

    // Handle remaining additions
    while *j < new_lines.len()
        && (*i >= old_lines.len() || old_lines[*i] != new_lines[*j])
        && changes < 20
    {
        hunk.push_str(&format!("+{}\n", new_lines[*j]));
        *j += 1;
        changes += 1;
    }

    // Show context after (up to 3 lines)
    let context_after = 3;
    let mut ctx_count = 0;
    while ctx_count < context_after && *i < old_lines.len() && *j < new_lines.len() {
        if old_lines[*i] == new_lines[*j] {
            hunk.push_str(&format!(" {}\n", old_lines[*i]));
            *i += 1;
            *j += 1;
            ctx_count += 1;
        } else {
            break;
        }
    }

    hunk
}

/// Check if a file path looks like a configuration file
pub fn is_config_file(path: &str) -> bool {
    let path_lower = path.to_lowercase();

    path_lower.contains(".config/")
        || path_lower.contains(".conf")
        || path_lower.contains("config")
        || path_lower.ends_with("rc")
        || path_lower.contains("/etc/")
        || path_lower.ends_with(".ini")
        || path_lower.ends_with(".toml")
        || path_lower.ends_with(".yaml")
        || path_lower.ends_with(".yml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_unified_diff_simple() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nline2 modified\nline3\n";

        let diff = unified_diff(old, new, Path::new("old.txt"), Path::new("new.txt"));

        assert!(diff.contains("--- old.txt"));
        assert!(diff.contains("+++ new.txt"));
        assert!(diff.contains("-line2"));
        assert!(diff.contains("+line2 modified"));
    }

    #[test]
    fn test_unified_diff_addition() {
        let old = "line1\nline2\n";
        let new = "line1\nline2\nline3\n";

        let diff = unified_diff(old, new, Path::new("old.txt"), Path::new("new.txt"));

        assert!(diff.contains("+line3"));
    }

    #[test]
    fn test_unified_diff_deletion() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nline3\n";

        let diff = unified_diff(old, new, Path::new("old.txt"), Path::new("new.txt"));

        assert!(diff.contains("-line2"));
    }

    #[test]
    fn test_generate_diff_with_files() {
        // Create temporary files
        let mut old_file = NamedTempFile::new().unwrap();
        let mut new_file = NamedTempFile::new().unwrap();

        writeln!(old_file, "line1").unwrap();
        writeln!(old_file, "line2").unwrap();
        old_file.flush().unwrap();

        writeln!(new_file, "line1").unwrap();
        writeln!(new_file, "line2 modified").unwrap();
        new_file.flush().unwrap();

        let diff = generate_diff(
            old_file.path().to_str().unwrap(),
            new_file.path().to_str().unwrap(),
        )
        .unwrap();

        assert!(diff.is_some());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("-line2"));
        assert!(diff_text.contains("+line2 modified"));
    }

    #[test]
    fn test_generate_diff_missing_backup() {
        let mut new_file = NamedTempFile::new().unwrap();
        writeln!(new_file, "content").unwrap();
        new_file.flush().unwrap();

        let diff = generate_diff(
            "/nonexistent/backup.txt",
            new_file.path().to_str().unwrap(),
        )
        .unwrap();

        assert!(diff.is_none());
    }

    #[test]
    fn test_generate_diff_deleted_file() {
        let mut old_file = NamedTempFile::new().unwrap();
        writeln!(old_file, "content").unwrap();
        old_file.flush().unwrap();

        let diff = generate_diff(
            old_file.path().to_str().unwrap(),
            "/nonexistent/current.txt",
        )
        .unwrap();

        assert!(diff.is_some());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("was deleted"));
    }

    #[test]
    fn test_is_config_file() {
        assert!(is_config_file("/home/user/.config/nvim/init.vim"));
        assert!(is_config_file("/etc/ssh/sshd_config"));
        assert!(is_config_file("/home/user/.vimrc"));
        assert!(is_config_file("/etc/pacman.conf"));
        assert!(is_config_file("/home/user/.config/app/settings.toml"));
        assert!(!is_config_file("/tmp/data.txt"));
        assert!(!is_config_file("/home/user/document.pdf"));
    }

    #[test]
    fn test_unified_diff_no_changes() {
        let content = "line1\nline2\nline3\n";
        let diff = unified_diff(content, content, Path::new("old.txt"), Path::new("new.txt"));
        assert!(diff.is_empty());
    }
}
