//! Local documentation search implementation (v0.0.408).
//!
//! Searches local knowledge sources:
//! - Man pages (via `man -k` / apropos)
//! - /usr/share/doc files
//! - Offline Arch Wiki mirror (if present)
//! - Anna's own docs (recipes, handbook)
//!
//! All sources are local - no internet access.

mod anna_docs;
mod arch_wiki;
mod constants;
mod local_docs;
mod man_pages;
mod utils;

use tracing::debug;

use crate::knowledge_item::{KnowledgeItem, KnowledgeQuery, KnowledgeSourceType};

// Re-export public functions
pub use anna_docs::search_anna_docs;
pub use arch_wiki::{arch_wiki_available, search_arch_wiki_local, suggest_arch_wiki_link};
pub use local_docs::search_local_docs;
pub use man_pages::{get_help_output, search_man_pages};
pub use utils::deduplicate_results;

/// Search all knowledge sources based on query
pub fn search_knowledge(query: &KnowledgeQuery) -> Vec<KnowledgeItem> {
    let mut results = vec![];
    let search_all = query.source_types.is_empty();

    // Search man pages
    if search_all || query.source_types.contains(&KnowledgeSourceType::ManPage) {
        let man_results = search_man_pages(&query.keywords, query.max_items);
        results.extend(man_results);
    }

    // Search local docs
    if search_all || query.source_types.contains(&KnowledgeSourceType::LocalDoc) {
        let doc_results = search_local_docs(&query.keywords, &query.tags, query.max_items);
        results.extend(doc_results);
    }

    // Search Arch Wiki local (if present)
    if search_all
        || query
            .source_types
            .contains(&KnowledgeSourceType::ArchWikiLocal)
    {
        let wiki_results = search_arch_wiki_local(&query.keywords, query.max_items);
        results.extend(wiki_results);
    }

    // Search Anna docs
    if search_all || query.source_types.contains(&KnowledgeSourceType::AnnaDoc) {
        let anna_results = search_anna_docs(&query.keywords, query.max_items);
        results.extend(anna_results);
    }

    // Sort by confidence (descending) and deduplicate
    results.sort_by(|a, b| b.confidence.cmp(&a.confidence));
    deduplicate_results(&mut results);

    // Limit total results
    results.truncate(query.max_items);

    debug!(
        "search_knowledge: {} results for keywords {:?}",
        results.len(),
        query.keywords
    );

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_query() {
        let query = KnowledgeQuery::new()
            .with_keywords(vec!["systemctl".to_string()])
            .with_limit(5);

        assert_eq!(query.keywords, vec!["systemctl"]);
        assert_eq!(query.max_items, 5);
    }
}
