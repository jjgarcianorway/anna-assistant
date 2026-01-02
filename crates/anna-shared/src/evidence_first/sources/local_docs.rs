//! Local documentation source (v0.0.435).

use serde::{Deserialize, Serialize};
use std::process::Command;

use super::error::SourceError;

/// Local documentation source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDocsSource {
    /// Path to documentation.
    pub path: String,
    /// Package name if known.
    pub package: Option<String>,
    /// Content (if small enough to load).
    pub content: Option<String>,
}

impl LocalDocsSource {
    /// Search /usr/share/doc for a package.
    pub fn find_for_package(package: &str) -> Option<Self> {
        let doc_path = format!("/usr/share/doc/{}", package);
        if std::path::Path::new(&doc_path).exists() {
            return Some(Self {
                path: doc_path,
                package: Some(package.to_string()),
                content: None,
            });
        }
        None
    }

    /// Search /usr/share/doc for files matching a query.
    pub fn search_docs(query: &str) -> Vec<Self> {
        let mut results = Vec::new();

        // Search with ripgrep if available
        let output = Command::new("rg")
            .args(["-l", "-i", query, "/usr/share/doc"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines().take(10) {
                    results.push(Self {
                        path: line.to_string(),
                        package: extract_package_from_path(line),
                        content: None,
                    });
                }
            }
        }

        results
    }

    /// Load content from file.
    pub fn load_content(&mut self) -> Result<(), SourceError> {
        let path = std::path::Path::new(&self.path);

        if path.is_file() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| SourceError::ReadFailed(e.to_string()))?;

            // Limit content size
            if content.len() > 100_000 {
                self.content = Some(content[..100_000].to_string());
            } else {
                self.content = Some(content);
            }
        }

        Ok(())
    }
}

/// Extract package name from doc path.
fn extract_package_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() > 4 && parts[1] == "usr" && parts[2] == "share" && parts[3] == "doc" {
        return Some(parts[4].to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_from_path() {
        let path = "/usr/share/doc/systemd/README";
        assert_eq!(extract_package_from_path(path), Some("systemd".to_string()));

        let path = "/some/other/path";
        assert_eq!(extract_package_from_path(path), None);
    }
}
