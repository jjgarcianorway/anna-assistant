//! Knowledge sources with strict priority (v0.0.432).
//!
//! Trust hierarchy: Probes > LocalDocs > CachedWiki > Remote

use serde::{Deserialize, Serialize};
use std::fmt;

/// Source priority levels (lower = higher trust).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SourcePriority {
    /// Live system probes - highest trust (0).
    Probe = 0,
    /// Local documentation - high trust (1).
    LocalDoc = 1,
    /// Cached wiki content - medium trust (2).
    CachedWiki = 2,
    /// Remote sources - lowest trust, disabled by default (3).
    Remote = 3,
}

impl SourcePriority {
    /// Trust score (1.0 = full trust, 0.0 = no trust).
    pub fn trust_score(&self) -> f32 {
        match self {
            Self::Probe => 1.0,
            Self::LocalDoc => 0.95,
            Self::CachedWiki => 0.8,
            Self::Remote => 0.6,
        }
    }

    /// Whether this source requires verification.
    pub fn requires_verification(&self) -> bool {
        matches!(self, Self::CachedWiki | Self::Remote)
    }
}

impl fmt::Display for SourcePriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Probe => write!(f, "probe"),
            Self::LocalDoc => write!(f, "local_doc"),
            Self::CachedWiki => write!(f, "wiki_cache"),
            Self::Remote => write!(f, "remote"),
        }
    }
}

/// Knowledge source types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// System probe (e.g., /proc/meminfo, systemd-analyze).
    Probe { name: String, command: Option<String> },
    /// Man page.
    ManPage { name: String, section: Option<u8> },
    /// Command help output (--help).
    HelpOutput { command: String },
    /// File in /usr/share/doc or similar.
    DocFile { path: String },
    /// Arch Wiki article (cached).
    ArchWiki { article: String, cached: bool },
    /// Generic wiki source.
    Wiki { name: String, article: String },
    /// Remote URL (disabled by default).
    RemoteUrl { url: String },
}

impl KnowledgeSource {
    /// Get the priority for this source.
    pub fn priority(&self) -> SourcePriority {
        match self {
            Self::Probe { .. } => SourcePriority::Probe,
            Self::ManPage { .. } | Self::HelpOutput { .. } | Self::DocFile { .. } => {
                SourcePriority::LocalDoc
            }
            Self::ArchWiki { cached: true, .. } | Self::Wiki { .. } => SourcePriority::CachedWiki,
            Self::ArchWiki { cached: false, .. } | Self::RemoteUrl { .. } => SourcePriority::Remote,
        }
    }

    /// Human-readable source description.
    pub fn description(&self) -> String {
        match self {
            Self::Probe { name, .. } => format!("probe:{}", name),
            Self::ManPage { name, section } => match section {
                Some(s) => format!("man {}({})", name, s),
                None => format!("man {}", name),
            },
            Self::HelpOutput { command } => format!("{} --help", command),
            Self::DocFile { path } => format!("doc:{}", path),
            Self::ArchWiki { article, cached } => {
                if *cached {
                    format!("wiki:arch:{} (cached)", article)
                } else {
                    format!("wiki:arch:{}", article)
                }
            }
            Self::Wiki { name, article } => format!("wiki:{}:{}", name, article),
            Self::RemoteUrl { url } => format!("remote:{}", url),
        }
    }

    /// Create a probe source.
    pub fn probe(name: &str) -> Self {
        Self::Probe {
            name: name.to_string(),
            command: None,
        }
    }

    /// Create a probe source with command.
    pub fn probe_with_cmd(name: &str, cmd: &str) -> Self {
        Self::Probe {
            name: name.to_string(),
            command: Some(cmd.to_string()),
        }
    }

    /// Create a man page source.
    pub fn man(name: &str) -> Self {
        Self::ManPage {
            name: name.to_string(),
            section: None,
        }
    }

    /// Create a man page source with section.
    pub fn man_section(name: &str, section: u8) -> Self {
        Self::ManPage {
            name: name.to_string(),
            section: Some(section),
        }
    }

    /// Create a help output source.
    pub fn help(command: &str) -> Self {
        Self::HelpOutput {
            command: command.to_string(),
        }
    }

    /// Create a cached Arch Wiki source.
    pub fn arch_wiki(article: &str) -> Self {
        Self::ArchWiki {
            article: article.to_string(),
            cached: true,
        }
    }
}

/// Result from a knowledge source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceResult {
    /// The source that provided this result.
    pub source: KnowledgeSource,
    /// The content retrieved.
    pub content: String,
    /// Relevance score (0.0 to 1.0).
    pub relevance: f32,
    /// When this was retrieved (unix timestamp).
    pub retrieved_at: u64,
    /// Additional metadata.
    pub metadata: std::collections::HashMap<String, String>,
}

impl SourceResult {
    /// Create a new source result.
    pub fn new(source: KnowledgeSource, content: String, relevance: f32) -> Self {
        Self {
            source,
            content,
            relevance,
            retrieved_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Combined trust score (priority trust * relevance).
    pub fn trust_score(&self) -> f32 {
        self.source.priority().trust_score() * self.relevance
    }
}

/// Citation for an answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Source that was cited.
    pub source: KnowledgeSource,
    /// Specific excerpt or reference.
    pub excerpt: Option<String>,
    /// Line numbers if applicable.
    pub lines: Option<(usize, usize)>,
    /// Confidence in this citation.
    pub confidence: f32,
}

impl Citation {
    /// Create a new citation.
    pub fn new(source: KnowledgeSource, confidence: f32) -> Self {
        Self {
            source,
            excerpt: None,
            lines: None,
            confidence,
        }
    }

    /// Create a citation with excerpt.
    pub fn with_excerpt(source: KnowledgeSource, excerpt: &str, confidence: f32) -> Self {
        Self {
            source,
            excerpt: Some(excerpt.to_string()),
            lines: None,
            confidence,
        }
    }

    /// Format as a citation string.
    pub fn format(&self) -> String {
        let desc = self.source.description();
        match (&self.excerpt, &self.lines) {
            (Some(exc), Some((start, end))) => {
                format!("{} (lines {}-{}): \"{}\"", desc, start, end, exc)
            }
            (Some(exc), None) => format!("{}: \"{}\"", desc, exc),
            (None, Some((start, end))) => format!("{} (lines {}-{})", desc, start, end),
            (None, None) => desc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(SourcePriority::Probe < SourcePriority::LocalDoc);
        assert!(SourcePriority::LocalDoc < SourcePriority::CachedWiki);
        assert!(SourcePriority::CachedWiki < SourcePriority::Remote);
    }

    #[test]
    fn test_source_priorities() {
        assert_eq!(KnowledgeSource::probe("mem").priority(), SourcePriority::Probe);
        assert_eq!(KnowledgeSource::man("ls").priority(), SourcePriority::LocalDoc);
        assert_eq!(KnowledgeSource::arch_wiki("Systemd").priority(), SourcePriority::CachedWiki);
    }

    #[test]
    fn test_trust_scores() {
        assert_eq!(SourcePriority::Probe.trust_score(), 1.0);
        assert!(SourcePriority::LocalDoc.trust_score() > SourcePriority::CachedWiki.trust_score());
    }
}
