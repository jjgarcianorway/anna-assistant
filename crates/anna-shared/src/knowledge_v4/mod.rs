//! Knowledge V4 - Complete Knowledge Engine (v0.0.424).
//!
//! Anna's local, citation-based knowledge brain that prefers Arch-style sources:
//! - Man pages (primary)
//! - Command help (--help, -h)
//! - Local documentation (/usr/share/doc)
//! - Arch Wiki (offline snapshot)
//!
//! Key principles:
//! - Local-first: no network calls by default
//! - Citation-based: every snippet has a traceable source
//! - Graceful degradation: missing sources are skipped, not errors
//! - Specialist-friendly: designed for integration into reasoning

pub mod config;
pub mod engine;
pub mod query;
pub mod snippet;
pub mod adapters;
pub mod citation;

// Re-exports
pub use config::KnowledgeConfig;
pub use engine::KnowledgeEngine;
pub use query::{KnowledgeQuery, KnowledgeSource};
pub use snippet::{KnowledgeSnippet, KnowledgeResult};
pub use citation::{Citation, format_citation};
pub use adapters::{ManAdapter, HelpAdapter, DocAdapter, WikiAdapter};

/// Maximum characters per snippet excerpt
pub const MAX_SNIPPET_CHARS: usize = 1500;

/// Maximum results per query
pub const MAX_RESULTS_PER_QUERY: usize = 5;

/// Command execution timeout in milliseconds
pub const COMMAND_TIMEOUT_MS: u64 = 5000;

/// Default man paths
pub const DEFAULT_MAN_PATHS: &[&str] = &[
    "/usr/share/man",
    "/usr/local/share/man",
];

/// Default doc paths
pub const DEFAULT_DOC_PATHS: &[&str] = &[
    "/usr/share/doc",
    "/usr/local/share/doc",
    "/usr/share/help",
];

/// Default wiki path (optional)
pub const DEFAULT_WIKI_PATH: &str = "/var/lib/anna/wiki/arch";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(MAX_SNIPPET_CHARS > 100);
        assert!(MAX_RESULTS_PER_QUERY > 0);
        assert!(!DEFAULT_MAN_PATHS.is_empty());
        assert!(!DEFAULT_DOC_PATHS.is_empty());
    }
}
