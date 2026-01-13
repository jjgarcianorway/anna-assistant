//! Local documentation corpus for ClaimGate citations.
//!
//! v0.3.26: Provides trusted documentation sources:
//! - Arch Wiki (local mirror)
//! - Man pages
//! - --help output caching
//!
//! All citations are deterministic and locally stored.

pub mod man_pages;
pub mod help_cache;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::anna_data_dir;

/// A citation from local documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocCitation {
    /// Type of documentation source
    pub source: DocSource,
    /// Title or command name
    pub title: String,
    /// Section heading (if applicable)
    pub section: Option<String>,
    /// Relevant excerpt
    pub excerpt: String,
    /// Local file path or identifier
    pub local_path: String,
    /// Line range in source (start, end)
    pub line_range: Option<(usize, usize)>,
}

impl DocCitation {
    /// Format citation for display (normal mode)
    pub fn format_short(&self) -> String {
        match &self.source {
            DocSource::ArchWiki => {
                if let Some(ref section) = self.section {
                    format!("[Arch Wiki: {} - {}]", self.title, section)
                } else {
                    format!("[Arch Wiki: {}]", self.title)
                }
            }
            DocSource::ManPage { section } => {
                format!("[man {}({})]", self.title, section)
            }
            DocSource::HelpOutput { version, .. } => {
                if let Some(ver) = version {
                    format!("[{} --help ({})]", self.title, ver)
                } else {
                    format!("[{} --help]", self.title)
                }
            }
        }
    }

    /// Format citation for display (debug mode)
    pub fn format_full(&self) -> String {
        let base = self.format_short();
        let mut parts = vec![base];

        parts.push(format!("  path: {}", self.local_path));

        if let Some((start, end)) = self.line_range {
            parts.push(format!("  lines: {}-{}", start, end));
        }

        if !self.excerpt.is_empty() {
            let excerpt_preview = if self.excerpt.len() > 100 {
                format!("{}...", &self.excerpt[..100])
            } else {
                self.excerpt.clone()
            };
            parts.push(format!("  excerpt: \"{}\"", excerpt_preview.replace('\n', " ")));
        }

        parts.join("\n")
    }
}

/// Type of documentation source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocSource {
    /// Arch Wiki article
    ArchWiki,
    /// Man page
    ManPage {
        /// Man section number (1-8)
        section: u8,
    },
    /// Command --help output
    HelpOutput {
        /// Command version if available
        version: Option<String>,
        /// Capture timestamp
        captured_at: String,
    },
}

/// Get docs cache directory
pub fn docs_cache_dir() -> PathBuf {
    anna_data_dir().join("docs")
}

/// Get man pages cache directory
pub fn man_cache_dir() -> PathBuf {
    docs_cache_dir().join("man")
}

/// Get help output cache directory
pub fn help_cache_dir() -> PathBuf {
    docs_cache_dir().join("help")
}

/// Initialize docs cache directories
pub fn init_docs_cache() -> std::io::Result<()> {
    std::fs::create_dir_all(man_cache_dir())?;
    std::fs::create_dir_all(help_cache_dir())?;
    Ok(())
}

/// Search all local documentation for a topic
pub fn search_docs(query: &str) -> Vec<DocCitation> {
    let mut citations = Vec::new();

    // Search Arch Wiki
    if let Some(wiki_citation) = search_wiki(query) {
        citations.push(wiki_citation);
    }

    // Search man pages for command-related queries
    let words: Vec<&str> = query.split_whitespace().collect();
    for word in &words {
        // Skip common words, look for command-like tokens
        if word.len() >= 2 && word.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            if let Some(man_citation) = man_pages::search_man(word, query) {
                citations.push(man_citation);
                break; // One man page citation is enough
            }
        }
    }

    // Search help cache
    for word in &words {
        if let Some(help_citation) = help_cache::search_help(word, query) {
            citations.push(help_citation);
            break;
        }
    }

    citations
}

/// Search Arch Wiki for a topic and return citation
pub fn search_wiki(query: &str) -> Option<DocCitation> {
    use crate::wiki::{search::keyword_search, index::WikiIndex, wiki_articles_dir};

    let index = WikiIndex::load().ok()?;
    let results = keyword_search(&index, query, 1).ok()?;

    let result = results.into_iter().next()?;

    // Find relevant section
    let section = crate::wiki::search::find_relevant_section(&result.article, query);
    let section_title = section.as_ref().and_then(|s| {
        // Extract section header if present
        s.lines()
            .find(|l| l.starts_with('#') || l.starts_with("=="))
            .map(|h| h.trim_matches(|c| c == '#' || c == '=' || c == ' ').to_string())
    });

    // Get excerpt
    let excerpt = section.as_ref()
        .map(|s| {
            let lines: Vec<&str> = s.lines().take(5).collect();
            lines.join("\n")
        })
        .unwrap_or_else(|| {
            result.article.content.lines().take(3).collect::<Vec<_>>().join("\n")
        });

    // Construct local path
    let article_path = wiki_articles_dir()
        .join(format!("{}.txt", result.article.title.replace('/', "_")));

    Some(DocCitation {
        source: DocSource::ArchWiki,
        title: result.article.title,
        section: section_title,
        excerpt,
        local_path: article_path.display().to_string(),
        line_range: None, // Could be computed if needed
    })
}

/// Check if a claim type requires documentation
pub fn requires_doc_citation(claim_type: &str) -> bool {
    let claim_lower = claim_type.to_lowercase();

    // State queries don't need docs
    let state_patterns = ["how much", "how many", "how long", "is running", "is active"];
    if state_patterns.iter().any(|p| claim_lower.contains(p)) {
        return false;
    }

    // Procedural/conceptual questions need docs
    let doc_required_patterns = [
        "how does",
        "how do i",
        "how do you",
        "how to",
        "what does",
        "what is the",
        "explain",
        "meaning",
        "purpose",
        "default",
        "configure",
        "option",
        "flag",
        "syntax",
        "format",
        "behavior",
    ];

    doc_required_patterns.iter().any(|p| claim_lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_citation_format_short() {
        let wiki_cite = DocCitation {
            source: DocSource::ArchWiki,
            title: "Systemd".to_string(),
            section: Some("User units".to_string()),
            excerpt: "User units are...".to_string(),
            local_path: "/home/user/.anna/wiki/Systemd.txt".to_string(),
            line_range: Some((10, 25)),
        };
        assert_eq!(wiki_cite.format_short(), "[Arch Wiki: Systemd - User units]");

        let man_cite = DocCitation {
            source: DocSource::ManPage { section: 1 },
            title: "systemctl".to_string(),
            section: None,
            excerpt: "systemctl - Control the systemd system".to_string(),
            local_path: "/home/user/.anna/docs/man/systemctl.1.txt".to_string(),
            line_range: None,
        };
        assert_eq!(man_cite.format_short(), "[man systemctl(1)]");

        let help_cite = DocCitation {
            source: DocSource::HelpOutput {
                version: Some("v1.0".to_string()),
                captured_at: "2026-01-13".to_string(),
            },
            title: "pacman".to_string(),
            section: None,
            excerpt: "-S, --sync".to_string(),
            local_path: "/home/user/.anna/docs/help/pacman.txt".to_string(),
            line_range: None,
        };
        assert_eq!(help_cite.format_short(), "[pacman --help (v1.0)]");
    }

    #[test]
    fn test_requires_doc_citation() {
        assert!(requires_doc_citation("how does systemctl mask work"));
        assert!(requires_doc_citation("what does the -S flag mean"));
        assert!(requires_doc_citation("explain TRIM"));
        assert!(!requires_doc_citation("is nginx running"));
        assert!(!requires_doc_citation("how much RAM"));
    }
}
