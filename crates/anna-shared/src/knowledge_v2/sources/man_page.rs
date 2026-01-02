//! Man page fetching (v0.0.422).

use std::process::Command;

use super::types::SourceFetchResult;
use super::utils::is_safe_name;

/// Fetch man page for a command
pub fn fetch_man_page(command: &str) -> Option<SourceFetchResult> {
    // Validate command name (alphanumeric, dash, underscore only)
    if !is_safe_name(command) {
        return None;
    }

    // Check if man page exists
    let check = Command::new("man").args(["-w", command]).output().ok()?;

    if !check.status.success() {
        return None;
    }

    let man_path = String::from_utf8_lossy(&check.stdout).trim().to_string();

    // Fetch man page content
    let output = Command::new("man")
        .args(["-P", "cat", command])
        .env("MANWIDTH", "80")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let content = String::from_utf8_lossy(&output.stdout);

    // Extract relevant sections (NAME, SYNOPSIS, DESCRIPTION, up to 100 lines)
    let extracted = extract_man_sections(&content);

    if extracted.is_empty() {
        return None;
    }

    Some(SourceFetchResult::new(extracted, &man_path))
}

/// Extract key sections from man page
fn extract_man_sections(content: &str) -> String {
    let mut result = String::new();
    let mut in_section = false;
    let mut current_section = String::new();
    let mut lines_collected = 0;
    const MAX_LINES: usize = 100;

    let important_sections = ["NAME", "SYNOPSIS", "DESCRIPTION", "OPTIONS", "COMMANDS"];

    for line in content.lines() {
        if lines_collected >= MAX_LINES {
            break;
        }

        // Check for section header (all caps, at start of line)
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && trimmed
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_whitespace())
            && trimmed.len() < 30
        {
            current_section = trimmed.to_string();
            in_section = important_sections.contains(&current_section.as_str());

            if in_section {
                result.push_str(&format!("\n{}\n", current_section));
                lines_collected += 1;
            }
            continue;
        }

        if in_section {
            result.push_str(line);
            result.push('\n');
            lines_collected += 1;
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_man_sections() {
        let man_content = r#"
NAME
       systemctl - Control the systemd system and service manager

SYNOPSIS
       systemctl [OPTIONS...] COMMAND [UNIT...]

DESCRIPTION
       systemctl may be used to introspect and control the state of the
       systemd system and service manager.

SEE ALSO
       systemd(1)
"#;
        let extracted = extract_man_sections(man_content);
        assert!(extracted.contains("NAME"));
        assert!(extracted.contains("SYNOPSIS"));
        assert!(extracted.contains("DESCRIPTION"));
    }
}
