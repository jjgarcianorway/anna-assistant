//! Doc Fetchers - Local documentation sources (v0.0.410).
//!
//! Priority order:
//! 1. Local Arch Wiki cache
//! 2. Man pages
//! 3. --help output
//! 4. /usr/share/doc files
//!
//! All sources are local - no internet access.

mod arch_wiki;
mod help_output;
mod local_doc;
mod man_page;
pub(crate) mod utils;

use crate::evidence_engine::DocSnippet;

// Re-export public types and functions
pub use arch_wiki::{fetch_arch_wiki, wiki_cache_available, wiki_cache_stats, WikiCacheStats};
pub use help_output::fetch_help_output;
pub use local_doc::fetch_local_doc;
pub use man_page::fetch_man_page;

/// Fetch documentation for a list of tags
pub fn fetch_docs(tags: &[String], max_docs: usize) -> Vec<DocSnippet> {
    let mut docs = vec![];

    for tag in tags {
        // Try each source in priority order
        if let Some(doc) = fetch_arch_wiki(tag) {
            docs.push(doc);
        }
        if let Some(doc) = fetch_man_page(tag) {
            docs.push(doc);
        }
        if let Some(doc) = fetch_help_output(tag) {
            docs.push(doc);
        }

        if docs.len() >= max_docs {
            break;
        }
    }

    // Sort by relevance and deduplicate
    docs.sort_by(|a, b| b.relevance.cmp(&a.relevance));
    utils::dedup_docs(&mut docs);
    docs.truncate(max_docs);

    docs
}
