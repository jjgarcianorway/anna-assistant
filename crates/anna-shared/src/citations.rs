//! v0.0.67: Local citations system.
//!
//! Provides citations from local sources:
//! - man pages
//! - --help output
//! - Arch Wiki snapshots (pre-fetched)
//!
//! No web browsing - only local files.

use std::fs;
use std::path::{Path, PathBuf};

/// Citation source type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationSource {
    ManPage { command: String },
    HelpOutput { command: String },
    ArchWiki { slug: String },
    LocalFile { path: String },
}

impl CitationSource {
    /// Format for display (no URLs)
    pub fn format_label(&self) -> String {
        match self {
            Self::ManPage { command } => format!("man {}", command),
            Self::HelpOutput { command } => format!("{} --help", command),
            Self::ArchWiki { slug } => format!("archwiki {}", slug),
            Self::LocalFile { path } => format!("file {}", path),
        }
    }
}

/// A citation reference
#[derive(Debug, Clone)]
pub struct Citation {
    pub source: CitationSource,
    pub excerpt: Option<String>,
    pub path: PathBuf,
}

impl Citation {
    /// Format for inline display: [source: X]
    pub fn format_inline(&self) -> String {
        format!("[source: {}]", self.source.format_label())
    }
}

/// Knowledge cache directory structure
pub struct KnowledgeCache {
    base_dir: PathBuf,
}

impl KnowledgeCache {
    /// Create with base directory
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Default location
    pub fn default_location() -> Self {
        Self::new("/var/lib/anna/knowledge")
    }

    /// Ensure directories exist
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        fs::create_dir_all(self.archwiki_dir())?;
        fs::create_dir_all(self.man_dir())?;
        Ok(())
    }

    fn archwiki_dir(&self) -> PathBuf {
        self.base_dir.join("archwiki")
    }

    fn man_dir(&self) -> PathBuf {
        self.base_dir.join("man")
    }

    /// Cite man page
    pub fn cite_man(&self, command: &str, topic: Option<&str>) -> Option<Citation> {
        let filename = format!("{}.txt", command);
        let path = self.man_dir().join(&filename);

        if path.exists() {
            let excerpt = topic.and_then(|t| self.find_excerpt(&path, t));
            Some(Citation {
                source: CitationSource::ManPage {
                    command: command.to_string(),
                },
                excerpt,
                path,
            })
        } else {
            None
        }
    }

    /// Cite Arch Wiki article
    pub fn cite_archwiki(&self, slug: &str, topic: Option<&str>) -> Option<Citation> {
        let filename = format!("{}.md", slug.to_lowercase().replace(' ', "_"));
        let path = self.archwiki_dir().join(&filename);

        if path.exists() {
            let excerpt = topic.and_then(|t| self.find_excerpt(&path, t));
            Some(Citation {
                source: CitationSource::ArchWiki {
                    slug: slug.to_string(),
                },
                excerpt,
                path,
            })
        } else {
            None
        }
    }

    /// Find excerpt containing topic keyword
    fn find_excerpt(&self, path: &Path, topic: &str) -> Option<String> {
        let content = fs::read_to_string(path).ok()?;
        let topic_lower = topic.to_lowercase();

        // Find first line containing the topic
        for line in content.lines() {
            if line.to_lowercase().contains(&topic_lower) {
                // Return this line and next few for context
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.chars().take(200).collect());
                }
            }
        }

        None
    }

    /// Store man page content
    pub fn store_man(&self, command: &str, content: &str) -> std::io::Result<()> {
        self.ensure_dirs()?;
        let filename = format!("{}.txt", command);
        let path = self.man_dir().join(&filename);
        fs::write(path, content)
    }

    /// Store Arch Wiki snapshot
    pub fn store_archwiki(&self, slug: &str, content: &str) -> std::io::Result<()> {
        self.ensure_dirs()?;
        let filename = format!("{}.md", slug.to_lowercase().replace(' ', "_"));
        let path = self.archwiki_dir().join(&filename);
        fs::write(path, content)
    }

    /// Check if man page exists
    pub fn has_man(&self, command: &str) -> bool {
        let filename = format!("{}.txt", command);
        self.man_dir().join(&filename).exists()
    }

    /// Check if Arch Wiki article exists
    pub fn has_archwiki(&self, slug: &str) -> bool {
        let filename = format!("{}.md", slug.to_lowercase().replace(' ', "_"));
        self.archwiki_dir().join(&filename).exists()
    }

    /// List available man pages
    pub fn list_man_pages(&self) -> Vec<String> {
        self.list_files(&self.man_dir(), ".txt")
    }

    /// List available Arch Wiki articles
    pub fn list_archwiki_articles(&self) -> Vec<String> {
        self.list_files(&self.archwiki_dir(), ".md")
    }

    fn list_files(&self, dir: &Path, extension: &str) -> Vec<String> {
        if !dir.exists() {
            return Vec::new();
        }

        fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        if name.ends_with(extension) {
                            Some(name.trim_end_matches(extension).to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Citation result for guidance
#[derive(Debug)]
pub enum GuidanceCitation {
    /// Successfully cited from source
    Cited(Citation),
    /// No citation available - creates verification ticket
    Uncited { topic: String },
}

impl GuidanceCitation {
    /// Format for display
    pub fn format_inline(&self) -> String {
        match self {
            Self::Cited(c) => c.format_inline(),
            Self::Uncited { .. } => "[uncited]".to_string(),
        }
    }

    pub fn is_cited(&self) -> bool {
        matches!(self, Self::Cited(_))
    }
}

/// Try to find citation for guidance
pub fn find_citation(
    cache: &KnowledgeCache,
    command: Option<&str>,
    archwiki_slug: Option<&str>,
    topic: Option<&str>,
) -> GuidanceCitation {
    // Try man page first
    if let Some(cmd) = command {
        if let Some(citation) = cache.cite_man(cmd, topic) {
            return GuidanceCitation::Cited(citation);
        }
    }

    // Try Arch Wiki
    if let Some(slug) = archwiki_slug {
        if let Some(citation) = cache.cite_archwiki(slug, topic) {
            return GuidanceCitation::Cited(citation);
        }
    }

    // No citation found
    GuidanceCitation::Uncited {
        topic: topic.unwrap_or("unknown").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_citation_source_format() {
        let man = CitationSource::ManPage {
            command: "vim".to_string(),
        };
        assert_eq!(man.format_label(), "man vim");

        let wiki = CitationSource::ArchWiki {
            slug: "Vim".to_string(),
        };
        assert_eq!(wiki.format_label(), "archwiki Vim");
    }

    #[test]
    fn test_knowledge_cache_store_and_cite() {
        let dir = tempdir().unwrap();
        let cache = KnowledgeCache::new(dir.path());

        // Store man page
        cache.store_man("vim", "NAME\n    vim - Vi IMproved\n\nSYNTAX\n    syntax on enables highlighting").unwrap();

        // Cite with topic
        let citation = cache.cite_man("vim", Some("syntax")).unwrap();
        assert!(matches!(citation.source, CitationSource::ManPage { .. }));
        assert!(citation.excerpt.is_some());
        // The excerpt contains the matched line, case may vary
        let excerpt = citation.excerpt.unwrap().to_lowercase();
        assert!(excerpt.contains("syntax"));
    }

    #[test]
    fn test_knowledge_cache_archwiki() {
        let dir = tempdir().unwrap();
        let cache = KnowledgeCache::new(dir.path());

        cache.store_archwiki("Vim", "# Vim\n\nVim is a text editor.\n\n## Syntax highlighting\nTo enable...").unwrap();

        let citation = cache.cite_archwiki("Vim", Some("highlighting")).unwrap();
        assert!(matches!(citation.source, CitationSource::ArchWiki { .. }));
    }

    #[test]
    fn test_uncited_fallback() {
        let dir = tempdir().unwrap();
        let cache = KnowledgeCache::new(dir.path());

        let result = find_citation(&cache, Some("nonexistent"), None, Some("topic"));
        assert!(matches!(result, GuidanceCitation::Uncited { .. }));
        assert_eq!(result.format_inline(), "[uncited]");
    }

    #[test]
    fn test_list_pages() {
        let dir = tempdir().unwrap();
        let cache = KnowledgeCache::new(dir.path());

        cache.store_man("vim", "content").unwrap();
        cache.store_man("nano", "content").unwrap();

        let pages = cache.list_man_pages();
        assert_eq!(pages.len(), 2);
        assert!(pages.contains(&"vim".to_string()));
        assert!(pages.contains(&"nano".to_string()));
    }
}
