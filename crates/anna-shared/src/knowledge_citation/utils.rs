// v0.0.530: Knowledge Citation Tracker - Utilities
// Utility functions for citation queries and fun facts

/// Check if query is citation-related
pub fn is_citation_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("citation")
        || lower.contains("source")
        || lower.contains("reference")
        || lower.contains("wiki")
        || lower.contains("man page")
        || lower.contains("documentation")
}

/// Fun fact about citations
pub fn citation_fun_fact() -> &'static str {
    "The Arch Wiki is one of the most comprehensive Linux documentation sources, with over 13,000 articles - Anna considers it her bible!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_citation_query() {
        assert!(is_citation_query("What's the source for this?"));
        assert!(is_citation_query("Show me the wiki page"));
        assert!(is_citation_query("Check man page for ls"));
        assert!(!is_citation_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = citation_fun_fact();
        assert!(fact.contains("Arch Wiki") || fact.contains("13,000"));
    }
}
