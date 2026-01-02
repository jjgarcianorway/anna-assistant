//! Wiki cache tests.
//!
//! Tests for wiki page searching and caching.

#[cfg(test)]
mod tests {
    use crate::evidence_first::wiki_cache::WikiPage;

    /// Test 9: Wiki cache operations.
    #[test]
    fn test_wiki_cache_search() {
        let page = WikiPage::new(
            "Systemd",
            "https://wiki.archlinux.org/title/Systemd",
            "# Overview\nSystemd is the init system.\n# Services\nUse systemctl to manage services.",
        );

        let hits = page.search("systemctl");
        assert!(!hits.is_empty(), "Should find systemctl");
        assert_eq!(hits[0].page_title, "Systemd");
    }
}
