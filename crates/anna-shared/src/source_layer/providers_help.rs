//! Help Provider - v0.0.443.
//!
//! Provides access to command --help output.

/// Help provider (--help output).
pub struct HelpProvider;

impl HelpProvider {
    /// Fetch command help.
    pub fn fetch(command: &str) -> Result<String, String> {
        // Try --help first, then -h
        let output = std::process::Command::new(command).arg("--help").output();

        match output {
            Ok(out) if out.status.success() || !out.stdout.is_empty() => {
                Ok(String::from_utf8_lossy(&out.stdout).to_string())
            }
            _ => {
                // Try -h
                let output = std::process::Command::new(command)
                    .arg("-h")
                    .output()
                    .map_err(|e| format!("Failed to run {} -h: {}", command, e))?;

                if output.status.success() || !output.stdout.is_empty() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(format!("No help available for {}", command))
                }
            }
        }
    }

    /// Extract relevant lines matching a query.
    pub fn extract_relevant(content: &str, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();
        let mut relevant = Vec::new();

        for line in content.lines() {
            if line.to_lowercase().contains(&query_lower) {
                relevant.push(line.to_string());
            }
        }

        if relevant.is_empty() {
            // Return first 20 lines as fallback
            Some(content.lines().take(20).collect::<Vec<_>>().join("\n"))
        } else {
            Some(relevant.join("\n"))
        }
    }
}
