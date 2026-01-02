//! Documentation Snippet - Integration with Arch Wiki, man pages, help (v0.0.412).
//!
//! Anna treats documentation as the primary technical authority:
//! - Arch Wiki pages
//! - Man pages
//! - Command help output
//! - Info pages
//! - Local config files

mod cache;
mod providers;
mod types;
mod utils;

// Re-export public API
pub use cache::DocCache;
pub use providers::{DocProvider, HelpProvider, ManPageProvider};
pub use types::{format_sources, DocSnippet, DocSourceKind};

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
