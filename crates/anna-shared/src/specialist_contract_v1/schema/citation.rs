//! Citation definitions for SRC v1.

use serde::{Deserialize, Serialize};

use super::constants::{truncate_str, MAX_SNIPPET_CHARS};
use super::types::SrcCitationSource;

/// A citation in SRC v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrcCitation {
    /// Source type.
    pub source: SrcCitationSource,
    /// Reference identifier (e.g., "systemd-analyze(1)" or "ArchWiki:Systemd").
    #[serde(rename = "ref")]
    pub reference: String,
    /// Relevant snippet, max 140 chars.
    pub snippet: String,
}

impl SrcCitation {
    /// Create a new citation.
    pub fn new(source: SrcCitationSource, reference: &str, snippet: &str) -> Self {
        Self {
            source,
            reference: reference.to_string(),
            snippet: truncate_str(snippet, MAX_SNIPPET_CHARS),
        }
    }

    /// Validate the citation.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.reference.is_empty() {
            errors.push("citation ref cannot be empty".to_string());
        }
        if self.snippet.len() > MAX_SNIPPET_CHARS {
            errors.push(format!(
                "citation snippet exceeds {} chars",
                MAX_SNIPPET_CHARS
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
