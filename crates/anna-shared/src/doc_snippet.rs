//! Documentation Snippet - Integration with Arch Wiki, man pages, help (v0.0.412).
//!
//! Anna treats documentation as the primary technical authority:
//! - Arch Wiki pages
//! - Man pages
//! - Command help output
//! - Info pages
//! - Local config files

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Documentation source kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocSourceKind {
    /// Arch Wiki page
    ArchWiki,
    /// Man page
    ManPage,
    /// Command --help output
    HelpFlag,
    /// Info page
    Info,
    /// Local file (config, etc.)
    LocalFile,
    /// Built-in knowledge
    Builtin,
}

impl std::fmt::Display for DocSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchWiki => write!(f, "arch_wiki"),
            Self::ManPage => write!(f, "man"),
            Self::HelpFlag => write!(f, "help"),
            Self::Info => write!(f, "info"),
            Self::LocalFile => write!(f, "file"),
            Self::Builtin => write!(f, "builtin"),
        }
    }
}

/// A documentation snippet used to support a recipe/answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSnippet {
    /// Unique ID
    pub id: String,
    /// Source kind
    pub kind: DocSourceKind,
    /// Reference (man page name, wiki URL, file path)
    pub reference: String,
    /// Section/heading within the doc
    pub section: Option<String>,
    /// Short excerpt used
    pub excerpt: String,
    /// When this was retrieved
    pub retrieved_at: u64,
    /// Relevance score (0.0-1.0)
    pub relevance: f32,
}

impl DocSnippet {
    /// Create a new doc snippet
    pub fn new(kind: DocSourceKind, reference: &str, excerpt: &str) -> Self {
        Self {
            id: compute_snippet_id(kind, reference),
            kind,
            reference: reference.to_string(),
            section: None,
            excerpt: excerpt.to_string(),
            retrieved_at: current_secs(),
            relevance: 0.8,
        }
    }

    /// Create from man page
    pub fn from_man(page: &str, excerpt: &str) -> Self {
        Self::new(DocSourceKind::ManPage, &format!("man:{}", page), excerpt)
    }

    /// Create from Arch Wiki
    pub fn from_wiki(title: &str, excerpt: &str) -> Self {
        Self::new(
            DocSourceKind::ArchWiki,
            &format!(
                "https://wiki.archlinux.org/title/{}",
                title.replace(' ', "_")
            ),
            excerpt,
        )
    }

    /// Create from help flag
    pub fn from_help(command: &str, excerpt: &str) -> Self {
        Self::new(
            DocSourceKind::HelpFlag,
            &format!("{} --help", command),
            excerpt,
        )
    }

    /// Set section
    pub fn with_section(mut self, section: &str) -> Self {
        self.section = Some(section.to_string());
        self
    }

    /// Set relevance
    pub fn with_relevance(mut self, relevance: f32) -> Self {
        self.relevance = relevance;
        self
    }

    /// Format as citation string
    pub fn citation(&self) -> String {
        match self.kind {
            DocSourceKind::ManPage => format!("man:{}", self.reference.trim_start_matches("man:")),
            DocSourceKind::ArchWiki => {
                if let Some(title) = self.reference.split("/title/").nth(1) {
                    format!("Arch Wiki: {}", title.replace('_', " "))
                } else {
                    format!("Arch Wiki: {}", self.reference)
                }
            }
            DocSourceKind::HelpFlag => self.reference.clone(),
            DocSourceKind::Info => format!("info:{}", self.reference),
            DocSourceKind::LocalFile => format!("file:{}", self.reference),
            DocSourceKind::Builtin => "builtin".to_string(),
        }
    }
}

/// Documentation cache for frequently used docs
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DocCache {
    /// Cached snippets by ID
    pub snippets: HashMap<String, DocSnippet>,
    /// Index: keyword -> snippet IDs
    pub keyword_index: HashMap<String, Vec<String>>,
    /// Cache metadata
    pub last_cleanup: u64,
}

impl DocCache {
    /// Load from disk
    pub fn load() -> Self {
        let path = Self::cache_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(cache) => return cache,
                    Err(e) => warn!("Failed to parse doc cache: {}", e),
                },
                Err(e) => warn!("Failed to read doc cache: {}", e),
            }
        }
        Self::default()
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::cache_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&path, json)?;
        Ok(())
    }

    fn cache_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".anna")
            .join("doc_cache.json")
    }

    /// Add or update a snippet
    pub fn add(&mut self, snippet: DocSnippet) {
        // Index by keywords from excerpt
        let keywords = extract_doc_keywords(&snippet.excerpt);
        for kw in keywords {
            self.keyword_index
                .entry(kw)
                .or_default()
                .push(snippet.id.clone());
        }
        self.snippets.insert(snippet.id.clone(), snippet);
    }

    /// Find relevant snippets for a topic
    pub fn find(&self, topic: &str) -> Vec<&DocSnippet> {
        let keywords = extract_doc_keywords(topic);
        let mut scores: HashMap<&str, usize> = HashMap::new();

        for kw in &keywords {
            if let Some(ids) = self.keyword_index.get(kw) {
                for id in ids {
                    *scores.entry(id.as_str()).or_default() += 1;
                }
            }
        }

        let mut results: Vec<_> = scores
            .into_iter()
            .filter_map(|(id, score)| self.snippets.get(id).map(|s| (s, score)))
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.into_iter().map(|(s, _)| s).take(5).collect()
    }

    /// Get snippet by ID
    pub fn get(&self, id: &str) -> Option<&DocSnippet> {
        self.snippets.get(id)
    }

    /// Cleanup old entries
    pub fn cleanup(&mut self, max_age_days: u64) {
        let threshold = current_secs().saturating_sub(max_age_days * 24 * 3600);
        let to_remove: Vec<_> = self
            .snippets
            .iter()
            .filter(|(_, s)| s.retrieved_at < threshold)
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_remove {
            self.snippets.remove(&id);
        }

        // Rebuild keyword index
        self.keyword_index.clear();
        for (id, snippet) in &self.snippets {
            let keywords = extract_doc_keywords(&snippet.excerpt);
            for kw in keywords {
                self.keyword_index.entry(kw).or_default().push(id.clone());
            }
        }

        self.last_cleanup = current_secs();
    }
}

/// Documentation provider interface
pub trait DocProvider {
    /// Fetch documentation for a topic
    fn fetch(&self, topic: &str) -> Option<DocSnippet>;
    /// Get provider kind
    fn kind(&self) -> DocSourceKind;
}

/// Man page provider
pub struct ManPageProvider;

impl DocProvider for ManPageProvider {
    fn fetch(&self, topic: &str) -> Option<DocSnippet> {
        // Try to get man page content
        let output = std::process::Command::new("man")
            .args(["-P", "cat", topic])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let content = String::from_utf8_lossy(&output.stdout);
        let excerpt = extract_man_synopsis(&content);

        Some(DocSnippet::from_man(topic, &excerpt))
    }

    fn kind(&self) -> DocSourceKind {
        DocSourceKind::ManPage
    }
}

/// Help flag provider
pub struct HelpProvider;

impl DocProvider for HelpProvider {
    fn fetch(&self, command: &str) -> Option<DocSnippet> {
        let output = std::process::Command::new(command)
            .arg("--help")
            .output()
            .ok()?;

        let content = if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
        } else {
            String::from_utf8_lossy(&output.stderr)
        };

        if content.is_empty() {
            return None;
        }

        // Take first ~500 chars as excerpt
        let excerpt: String = content.chars().take(500).collect();
        Some(DocSnippet::from_help(command, &excerpt))
    }

    fn kind(&self) -> DocSourceKind {
        DocSourceKind::HelpFlag
    }
}

/// Extract synopsis section from man page
fn extract_man_synopsis(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_synopsis = false;
    let mut synopsis = String::new();

    for line in lines {
        if line.contains("SYNOPSIS") || line.contains("Synopsis") {
            in_synopsis = true;
            continue;
        }
        if in_synopsis {
            if line.chars().all(|c| c.is_uppercase() || c.is_whitespace()) && !line.is_empty() {
                break; // Next section
            }
            synopsis.push_str(line);
            synopsis.push('\n');
            if synopsis.len() > 500 {
                break;
            }
        }
    }

    if synopsis.is_empty() {
        // Fallback: first 500 chars
        content.chars().take(500).collect()
    } else {
        synopsis.trim().to_string()
    }
}

/// Extract keywords from doc content
fn extract_doc_keywords(text: &str) -> Vec<String> {
    let stop_words = [
        "the", "a", "an", "is", "are", "to", "of", "in", "for", "with", "on", "at",
    ];
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !stop_words.contains(w))
        .map(String::from)
        .collect()
}

fn compute_snippet_id(kind: DocSourceKind, reference: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    kind.to_string().hash(&mut hasher);
    reference.hash(&mut hasher);
    format!("doc_{:016x}", hasher.finish())
}

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Format multiple doc snippets as a sources section
pub fn format_sources(snippets: &[DocSnippet]) -> String {
    if snippets.is_empty() {
        return String::new();
    }

    let mut sources = String::from("\n**Sources:**\n");
    for snippet in snippets.iter().take(5) {
        sources.push_str(&format!("- {}\n", snippet.citation()));
    }
    sources
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_snippet_creation() {
        let snippet = DocSnippet::from_man("systemctl", "systemctl - Control the systemd system");
        assert_eq!(snippet.kind, DocSourceKind::ManPage);
        assert!(snippet.reference.contains("systemctl"));
    }

    #[test]
    fn test_wiki_citation() {
        let snippet = DocSnippet::from_wiki("Systemd", "systemd is a system and service manager");
        assert!(snippet.citation().contains("Arch Wiki"));
    }

    #[test]
    fn test_doc_cache() {
        let mut cache = DocCache::default();
        let snippet = DocSnippet::from_man("ls", "list directory contents");
        cache.add(snippet);

        let results = cache.find("directory listing");
        assert!(!results.is_empty());
    }
}
