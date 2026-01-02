//! Source Provider Types - v0.0.443.
//!
//! Core types for source providers:
//! - SourceType: enum of available source types
//! - SourceRequest: request for a specific source
//! - SourceContent: fetched content with metadata
//! - IntentCommands: intent to commands mapping

use serde::{Deserialize, Serialize};

/// Source provider type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Man pages.
    Man,
    /// Command --help output.
    Help,
    /// Arch Wiki.
    ArchWiki,
    /// Local config files.
    LocalConfig,
    /// System probes (commands).
    Probe,
}

impl SourceType {
    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Man => "man",
            Self::Help => "help",
            Self::ArchWiki => "archwiki",
            Self::LocalConfig => "config",
            Self::Probe => "probe",
        }
    }

    /// Is this a documentation source (vs evidence)?
    pub fn is_documentation(&self) -> bool {
        matches!(self, Self::Man | Self::Help | Self::ArchWiki)
    }

    /// Is this an evidence source (from this machine)?
    pub fn is_evidence(&self) -> bool {
        matches!(self, Self::LocalConfig | Self::Probe)
    }
}

/// A source request in a research plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRequest {
    /// Source type.
    #[serde(rename = "type")]
    pub source_type: SourceType,
    /// Source identifier (e.g., "pacman(8)", "Pacman").
    pub id: String,
    /// Query/section within source.
    pub query: String,
    /// Whether this source is required.
    pub required: bool,
}

impl SourceRequest {
    /// Create man page source request.
    pub fn man(page: &str, section: &str) -> Self {
        Self {
            source_type: SourceType::Man,
            id: page.to_string(),
            query: section.to_string(),
            required: true,
        }
    }

    /// Create help source request.
    pub fn help(command: &str, flag: &str) -> Self {
        Self {
            source_type: SourceType::Help,
            id: format!("{} --help", command),
            query: flag.to_string(),
            required: true,
        }
    }

    /// Create Arch Wiki source request.
    pub fn arch_wiki(page: &str, section: &str) -> Self {
        Self {
            source_type: SourceType::ArchWiki,
            id: page.to_string(),
            query: section.to_string(),
            required: false, // Wiki is optional (offline may not have it)
        }
    }

    /// Create optional version.
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Fetched source content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceContent {
    /// Source request that produced this.
    pub request: SourceRequest,
    /// Whether fetch succeeded.
    pub success: bool,
    /// Content (if successful).
    pub content: Option<String>,
    /// Error message (if failed).
    pub error: Option<String>,
    /// Relevant excerpt (extracted section).
    pub excerpt: Option<String>,
}

impl SourceContent {
    /// Create successful content.
    pub fn success(request: SourceRequest, content: &str, excerpt: Option<&str>) -> Self {
        Self {
            request,
            success: true,
            content: Some(content.to_string()),
            error: None,
            excerpt: excerpt.map(String::from),
        }
    }

    /// Create failed content.
    pub fn failed(request: SourceRequest, error: &str) -> Self {
        Self {
            request,
            success: false,
            content: None,
            error: Some(error.to_string()),
            excerpt: None,
        }
    }

    /// Create unavailable (offline) content.
    pub fn unavailable(request: SourceRequest) -> Self {
        Self {
            request: request.clone(),
            success: false,
            content: None,
            error: Some(format!(
                "{} source '{}' not available offline",
                request.source_type.label(),
                request.id
            )),
            excerpt: None,
        }
    }
}

/// Intent to canonical commands mapping.
#[derive(Debug, Clone)]
pub struct IntentCommands {
    /// Intent name.
    pub intent: String,
    /// Canonical commands for this intent.
    pub commands: Vec<String>,
    /// Recommended wiki pages.
    pub wiki_pages: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type() {
        assert!(SourceType::Man.is_documentation());
        assert!(!SourceType::Man.is_evidence());
        assert!(SourceType::Probe.is_evidence());
    }

    #[test]
    fn test_source_request() {
        let man = SourceRequest::man("pacman(8)", "SYSTEM UPGRADE");
        assert_eq!(man.source_type, SourceType::Man);
        assert!(man.required);

        let wiki = SourceRequest::arch_wiki("Pacman", "Upgrading packages");
        assert!(!wiki.required); // Wiki is optional by default
    }
}
