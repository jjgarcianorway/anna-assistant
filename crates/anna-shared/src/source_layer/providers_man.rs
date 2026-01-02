//! Man Page Provider - v0.0.443.
//!
//! Provides access to man pages via `man -P cat <cmd>`.

/// Man page provider.
pub struct ManProvider;

impl ManProvider {
    /// Fetch man page content.
    pub fn fetch(page: &str) -> Result<String, String> {
        // Use man -P cat to get plain text
        let output = std::process::Command::new("man")
            .args(["-P", "cat", page])
            .output()
            .map_err(|e| format!("Failed to run man: {}", e))?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            // Clean up formatting
            Ok(Self::clean_man_output(&content))
        } else {
            Err(format!("man {} not found", page))
        }
    }

    /// Clean man page output.
    fn clean_man_output(content: &str) -> String {
        // Remove backspace sequences used for bold/underline
        let mut result = String::new();
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\x08' {
                // Backspace - skip previous and next char
                result.pop();
                chars.next();
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Extract section from man page.
    pub fn extract_section(content: &str, section: &str) -> Option<String> {
        let section_upper = section.to_uppercase();
        let mut in_section = false;
        let mut result = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for section header (all caps, at start of line)
            if trimmed
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_whitespace())
                && !trimmed.is_empty()
            {
                if trimmed.contains(&section_upper) {
                    in_section = true;
                    result.push(line.to_string());
                } else if in_section {
                    // New section started, stop
                    break;
                }
            } else if in_section {
                result.push(line.to_string());
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result.join("\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_man_section_extraction() {
        let content = "NAME\n       pacman - package manager\n\nSYNOPSIS\n       pacman <operation>\n\nDESCRIPTION\n       pacman is a package manager.";
        let section = ManProvider::extract_section(content, "SYNOPSIS");
        assert!(section.is_some());
        assert!(section.unwrap().contains("operation"));
    }
}
