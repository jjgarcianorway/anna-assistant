//! Local documentation adapter (v0.0.424).
//!
//! Searches /usr/share/doc, /usr/local/share/doc for relevant docs.

use std::fs;
use std::path::{Path, PathBuf};

use super::{extract_context, truncate_text};
use crate::knowledge_v4::query::KnowledgeSource;
use crate::knowledge_v4::snippet::KnowledgeSnippet;

/// Local documentation adapter
pub struct DocAdapter {
    /// Paths to search
    doc_paths: Vec<PathBuf>,
    /// Maximum characters per excerpt
    max_chars: usize,
    /// Maximum files to scan
    max_files: usize,
}

impl Default for DocAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DocAdapter {
    /// Create new adapter with default paths
    pub fn new() -> Self {
        Self {
            doc_paths: vec![
                PathBuf::from("/usr/share/doc"),
                PathBuf::from("/usr/local/share/doc"),
                PathBuf::from("/usr/share/help"),
            ],
            max_chars: 1500,
            max_files: 50,
        }
    }

    /// Set doc paths
    pub fn with_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.doc_paths = paths;
        self
    }

    /// Set max chars
    pub fn with_max_chars(mut self, max: usize) -> Self {
        self.max_chars = max;
        self
    }

    /// Query local docs for a topic
    pub fn query(&self, topic: &str, entities: &[&str]) -> Option<KnowledgeSnippet> {
        // Search for matching directories/files
        let candidates = self.find_candidates(topic, entities);

        if candidates.is_empty() {
            return None;
        }

        // Try to find a README or relevant doc in the first candidate
        for (name, path) in candidates.iter().take(5) {
            if let Some(snippet) = self.extract_from_path(name, path, topic) {
                return Some(snippet);
            }
        }

        None
    }

    /// Query for a specific entity (command/package name)
    pub fn query_entity(&self, entity: &str) -> Option<KnowledgeSnippet> {
        self.query(entity, &[entity])
    }

    /// Find candidate directories/files
    fn find_candidates(&self, topic: &str, entities: &[&str]) -> Vec<(String, PathBuf)> {
        let mut candidates = vec![];
        let topic_lower = topic.to_lowercase();

        for doc_path in &self.doc_paths {
            if !doc_path.exists() {
                continue;
            }

            // Look for directories matching entities or topic
            if let Ok(entries) = fs::read_dir(doc_path) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    let path = entry.path();

                    // Match by entity
                    let matches_entity = entities.iter().any(|e| {
                        name.contains(&e.to_lowercase()) || e.to_lowercase().contains(&name)
                    });

                    // Match by topic words
                    let matches_topic = topic_lower
                        .split_whitespace()
                        .any(|w| w.len() > 2 && name.contains(w));

                    if matches_entity || matches_topic {
                        candidates.push((entry.file_name().to_string_lossy().to_string(), path));
                    }
                }
            }

            if candidates.len() >= self.max_files {
                break;
            }
        }

        // Sort by relevance (exact entity matches first)
        candidates.sort_by(|(a, _), (b, _)| {
            let a_exact = entities.iter().any(|e| a.eq_ignore_ascii_case(e));
            let b_exact = entities.iter().any(|e| b.eq_ignore_ascii_case(e));
            b_exact.cmp(&a_exact)
        });

        candidates
    }

    /// Extract snippet from a path (directory or file)
    fn extract_from_path(&self, name: &str, path: &Path, topic: &str) -> Option<KnowledgeSnippet> {
        if path.is_dir() {
            self.extract_from_dir(name, path, topic)
        } else if path.is_file() {
            self.extract_from_file(name, path, topic)
        } else {
            None
        }
    }

    /// Extract snippet from a directory (look for README, etc.)
    fn extract_from_dir(&self, name: &str, dir: &Path, topic: &str) -> Option<KnowledgeSnippet> {
        // Look for common doc files
        let doc_names = [
            "README",
            "README.md",
            "README.txt",
            "README.rst",
            "readme",
            "readme.md",
            "readme.txt",
            "INSTALL",
            "USAGE",
            "HELP",
        ];

        for doc_name in doc_names {
            let doc_path = dir.join(doc_name);
            if doc_path.exists() {
                if let Some(snippet) = self.extract_from_file(name, &doc_path, topic) {
                    return Some(snippet);
                }
            }
        }

        // Try any .txt or .md file
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "txt" || ext == "md" || ext == "rst" {
                    if let Some(snippet) = self.extract_from_file(name, &path, topic) {
                        return Some(snippet);
                    }
                }
            }
        }

        None
    }

    /// Extract snippet from a file
    fn extract_from_file(&self, name: &str, file: &Path, topic: &str) -> Option<KnowledgeSnippet> {
        // Only read text files
        if !is_text_file(file) {
            return None;
        }

        // Read content
        let content = fs::read_to_string(file).ok()?;

        if content.len() < 20 {
            return None;
        }

        // Extract relevant excerpt
        let excerpt = if let Some(ctx) = extract_context(&content, topic, 10) {
            ctx
        } else {
            // Fall back to first N lines
            content.lines().take(30).collect::<Vec<_>>().join("\n")
        };

        let truncated = truncate_text(&excerpt, self.max_chars);

        if truncated.len() < 20 {
            return None;
        }

        Some(KnowledgeSnippet::from_doc(
            name,
            &file.to_path_buf(),
            &truncated,
        ))
    }
}

/// Check if file is likely a text file
fn is_text_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Known text extensions
    let text_exts = [
        "txt", "md", "rst", "asciidoc", "adoc", "conf", "cfg", "ini", "html", "htm", "xml",
        "", // No extension (README, INSTALL, etc.)
    ];

    if text_exts.contains(&ext) {
        return true;
    }

    // Skip known binary extensions
    let binary_exts = [
        "gz", "xz", "bz2", "zst", "png", "jpg", "gif", "ico", "pdf", "ps", "dvi", "so", "o", "a",
    ];

    if binary_exts.contains(&ext) {
        return false;
    }

    // Check filename for common doc patterns
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let name_upper = name.to_uppercase();

    name_upper.starts_with("README")
        || name_upper.starts_with("INSTALL")
        || name_upper.starts_with("CHANGELOG")
        || name_upper.starts_with("NEWS")
        || name_upper.starts_with("TODO")
        || name_upper.starts_with("COPYING")
        || name_upper.starts_with("LICENSE")
        || name_upper.starts_with("AUTHORS")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_text_file() {
        assert!(is_text_file(Path::new("README")));
        assert!(is_text_file(Path::new("README.md")));
        assert!(is_text_file(Path::new("doc.txt")));
        assert!(!is_text_file(Path::new("archive.tar.gz")));
        assert!(!is_text_file(Path::new("image.png")));
    }

    #[test]
    fn test_doc_adapter_creation() {
        let adapter = DocAdapter::new();
        assert!(!adapter.doc_paths.is_empty());
    }

    #[test]
    fn test_find_candidates() {
        let adapter = DocAdapter::new();
        // This test depends on what's installed
        let _candidates = adapter.find_candidates("linux", &["linux"]);
        // Just verify it doesn't panic
    }
}
