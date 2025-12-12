//! Arch Wiki adapter (v0.0.424).
//!
//! Searches offline Arch Wiki snapshot for documentation.
//! No network calls - works only with pre-synced local copy.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{extract_context, truncate_text};
use crate::knowledge_v4::query::KnowledgeSource;
use crate::knowledge_v4::snippet::KnowledgeSnippet;

/// Wiki index for offline Arch Wiki
pub struct WikiIndex {
    /// Root path of wiki snapshot
    root: PathBuf,
    /// Topic to filename mapping (lazy loaded)
    topic_map: HashMap<String, PathBuf>,
    /// Whether index has been built
    indexed: bool,
}

impl WikiIndex {
    /// Create new wiki index
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            topic_map: HashMap::new(),
            indexed: false,
        }
    }

    /// Check if wiki snapshot is available
    pub fn is_available(&self) -> bool {
        self.root.exists() && self.root.is_dir()
    }

    /// Build index of available pages
    pub fn build_index(&mut self) {
        if self.indexed || !self.is_available() {
            return;
        }

        self.scan_directory(&self.root.clone());
        self.indexed = true;
    }

    /// Scan directory for wiki pages
    fn scan_directory(&mut self, dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                if path.is_dir() {
                    self.scan_directory(&path);
                } else if is_wiki_file(&path) {
                    // Map topic to file
                    let topic = extract_topic_from_filename(&name);
                    self.topic_map.insert(topic.to_lowercase(), path);
                }
            }
        }
    }

    /// Find page by topic
    pub fn find_page(&self, topic: &str) -> Option<&PathBuf> {
        let topic_lower = topic.to_lowercase();

        // Try exact match first
        if let Some(path) = self.topic_map.get(&topic_lower) {
            return Some(path);
        }

        // Try with underscores
        let topic_underscore = topic_lower.replace(' ', "_");
        if let Some(path) = self.topic_map.get(&topic_underscore) {
            return Some(path);
        }

        // Try partial match
        for (key, path) in &self.topic_map {
            if key.contains(&topic_lower) || topic_lower.contains(key) {
                return Some(path);
            }
        }

        None
    }

    /// Get all indexed topics
    pub fn topics(&self) -> Vec<&str> {
        self.topic_map.keys().map(|s| s.as_str()).collect()
    }

    /// Get index size
    pub fn size(&self) -> usize {
        self.topic_map.len()
    }
}

/// Wiki adapter for Arch Wiki queries
pub struct WikiAdapter {
    /// Wiki index
    index: WikiIndex,
    /// Maximum characters per excerpt
    max_chars: usize,
}

impl WikiAdapter {
    /// Create new adapter
    pub fn new(wiki_path: Option<PathBuf>) -> Self {
        let root = wiki_path.unwrap_or_else(|| PathBuf::from("/var/lib/anna/wiki/arch"));

        Self {
            index: WikiIndex::new(root),
            max_chars: 1500,
        }
    }

    /// Set max chars
    pub fn with_max_chars(mut self, max: usize) -> Self {
        self.max_chars = max;
        self
    }

    /// Check if wiki is available
    pub fn is_available(&self) -> bool {
        self.index.is_available()
    }

    /// Initialize the adapter (builds index)
    pub fn init(&mut self) {
        self.index.build_index();
    }

    /// Query wiki for a topic
    pub fn query(&mut self, topic: &str, keyword: Option<&str>) -> Option<KnowledgeSnippet> {
        // Ensure index is built
        if !self.index.indexed {
            self.init();
        }

        if !self.is_available() {
            return None;
        }

        // Find matching page
        let page_path = self.index.find_page(topic)?;

        // Read and parse content
        let content = fs::read_to_string(page_path).ok()?;
        let plain_text = strip_markup(&content);

        if plain_text.len() < 50 {
            return None;
        }

        // Extract relevant excerpt
        let excerpt = if let Some(kw) = keyword {
            if let Some(ctx) = extract_context(&plain_text, kw, 10) {
                ctx
            } else {
                extract_intro(&plain_text)
            }
        } else {
            extract_intro(&plain_text)
        };

        let truncated = truncate_text(&excerpt, self.max_chars);

        // Create snippet
        let title = extract_topic_from_filename(&page_path.file_name()?.to_string_lossy());
        let mut snippet = KnowledgeSnippet::from_wiki(&title, &truncated);
        snippet = snippet.with_path(page_path.clone());

        Some(snippet)
    }

    /// Query by entities (try each as a topic)
    pub fn query_entities(
        &mut self,
        entities: &[&str],
        keyword: Option<&str>,
    ) -> Option<KnowledgeSnippet> {
        for entity in entities {
            if let Some(snippet) = self.query(entity, keyword) {
                return Some(snippet);
            }
        }
        None
    }
}

/// Check if file is a wiki page
fn is_wiki_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    ext == "html" || ext == "txt" || ext == "md" || ext == ""
}

/// Extract topic name from filename
fn extract_topic_from_filename(filename: &str) -> String {
    let name = filename
        .trim_end_matches(".html")
        .trim_end_matches(".txt")
        .trim_end_matches(".md");

    // Convert underscores to spaces
    name.replace('_', " ")
}

/// Strip HTML/wiki markup from content
fn strip_markup(content: &str) -> String {
    // Simple HTML tag stripping
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;

    for c in content.chars() {
        if c == '<' {
            in_tag = true;

            // Check for script/style tags
            let remaining = content[content.find(c).unwrap()..].to_lowercase();
            if remaining.starts_with("<script") || remaining.starts_with("<style") {
                in_script = true;
            }
            if remaining.starts_with("</script") || remaining.starts_with("</style") {
                in_script = false;
            }
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag && !in_script {
            result.push(c);
        }
    }

    // Clean up whitespace
    let lines: Vec<&str> = result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    lines.join("\n")
}

/// Extract introduction/first paragraph from wiki content
fn extract_intro(content: &str) -> String {
    let mut intro = String::new();
    let mut lines_collected = 0;
    let max_lines = 20;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip headers
        if trimmed.starts_with('#') || trimmed.starts_with('=') {
            continue;
        }

        // Skip empty lines at start
        if intro.is_empty() && trimmed.is_empty() {
            continue;
        }

        intro.push_str(trimmed);
        intro.push('\n');
        lines_collected += 1;

        if lines_collected >= max_lines {
            break;
        }
    }

    intro.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_topic_from_filename() {
        assert_eq!(extract_topic_from_filename("Systemd.html"), "Systemd");
        assert_eq!(extract_topic_from_filename("Arch_Linux.html"), "Arch Linux");
        assert_eq!(extract_topic_from_filename("Vim.txt"), "Vim");
    }

    #[test]
    fn test_strip_markup() {
        let html = "<p>Hello <b>world</b></p>";
        let plain = strip_markup(html);
        assert!(plain.contains("Hello"));
        assert!(plain.contains("world"));
        assert!(!plain.contains("<"));
    }

    #[test]
    fn test_wiki_index_unavailable() {
        let index = WikiIndex::new(PathBuf::from("/nonexistent/path"));
        assert!(!index.is_available());
    }

    #[test]
    fn test_wiki_adapter_creation() {
        let adapter = WikiAdapter::new(None);
        // Just verify it doesn't panic
        assert!(adapter.max_chars > 0);
    }

    #[test]
    fn test_extract_intro() {
        let content = "# Title\n\nFirst paragraph.\nSecond line.\n\n# Next section";
        let intro = extract_intro(content);
        assert!(intro.contains("First paragraph"));
        assert!(intro.contains("Second line"));
    }
}
