//! RAG-first answerer - answers queries from knowledge store (v0.0.32R).
//!
//! Attempts to answer RAG-first QueryClasses without LLM calls.
//! Falls back to collectors if knowledge is stale or missing.

use anna_shared::knowledge::{
    KnowledgeDoc, KnowledgeSource, KnowledgeStore, KnowledgeStoreTrait, RetrievalQuery,
};
use tracing::info;

use crate::collectors;
use crate::router::QueryClass;

/// Result from RAG answerer
#[derive(Debug)]
pub struct RagAnswerResult {
    /// The answer text
    pub answer: String,
    /// Whether we had to collect fresh data
    pub collected_fresh: bool,
    /// Source documents used
    pub sources: Vec<String>,
    /// Whether answer is complete (vs needs specialist)
    pub is_complete: bool,
}

impl RagAnswerResult {
    pub fn complete(answer: String, sources: Vec<String>) -> Self {
        Self {
            answer,
            collected_fresh: false,
            sources,
            is_complete: true,
        }
    }

    pub fn with_fresh_collection(mut self) -> Self {
        self.collected_fresh = true;
        self
    }

    pub fn incomplete(reason: &str) -> Self {
        Self {
            answer: reason.to_string(),
            collected_fresh: false,
            sources: vec![],
            is_complete: false,
        }
    }
}

/// Try to answer a RAG-first query from knowledge store
pub fn try_rag_answer(
    class: QueryClass,
    query: &str,
    store: &mut KnowledgeStore,
) -> Option<RagAnswerResult> {
    match class {
        QueryClass::BootTimeStatus => answer_boot_time(store),
        QueryClass::InstalledPackagesOverview => answer_packages_overview(store),
        QueryClass::AppAlternatives => answer_app_alternatives(query, store),
        _ => None, // Not a RAG-first class
    }
}

/// Answer boot time query from knowledge store
fn answer_boot_time(store: &mut KnowledgeStore) -> Option<RagAnswerResult> {
    // Query for boot time docs
    let query = RetrievalQuery::new("boot time startup")
        .with_sources(vec![KnowledgeSource::SystemFact])
        .with_limit(1)
        .with_min_confidence(80);

    let hits = store.query(&query);

    if let Some(hit) = hits.first() {
        if let Some(doc) = store.get(&hit.doc_id) {
            let answer = format_boot_time_answer(doc);
            return Some(RagAnswerResult::complete(answer, vec![hit.doc_id.clone()]));
        }
    }

    // No cached data - collect fresh
    info!("No boot time in knowledge store, collecting fresh");
    let result = collectors::collect_boot_time();

    if let Some(doc) = result.docs.into_iter().next() {
        let answer = format_boot_time_answer(&doc);
        let doc_id = doc.id.clone();

        // Store for future queries
        if let Err(e) = store.upsert(doc) {
            info!("Failed to store boot time doc: {}", e);
        }

        return Some(RagAnswerResult::complete(answer, vec![doc_id]).with_fresh_collection());
    }

    // Collection failed
    if !result.errors.is_empty() {
        return Some(RagAnswerResult::incomplete(&format!(
            "Could not determine boot time: {}",
            result.errors.join(", ")
        )));
    }

    None
}

/// Format boot time answer from document
fn format_boot_time_answer(doc: &KnowledgeDoc) -> String {
    format!(
        "**{}**\n\n{}",
        doc.title,
        doc.body.lines().next().unwrap_or(&doc.body)
    )
}

/// Answer packages overview from knowledge store
fn answer_packages_overview(store: &mut KnowledgeStore) -> Option<RagAnswerResult> {
    // Query for package docs
    let query = RetrievalQuery::new("packages installed total")
        .with_sources(vec![KnowledgeSource::PackageFact])
        .with_limit(1)
        .with_min_confidence(80);

    let hits = store.query(&query);

    if let Some(hit) = hits.first() {
        if let Some(doc) = store.get(&hit.doc_id) {
            let answer = format_packages_answer(doc);
            return Some(RagAnswerResult::complete(answer, vec![hit.doc_id.clone()]));
        }
    }

    // No cached data - collect fresh
    info!("No packages info in knowledge store, collecting fresh");
    let result = collectors::collect_packages();

    if let Some(doc) = result.docs.into_iter().next() {
        let answer = format_packages_answer(&doc);
        let doc_id = doc.id.clone();

        // Store for future queries
        if let Err(e) = store.upsert(doc) {
            info!("Failed to store packages doc: {}", e);
        }

        return Some(RagAnswerResult::complete(answer, vec![doc_id]).with_fresh_collection());
    }

    // Collection failed
    if !result.errors.is_empty() {
        return Some(RagAnswerResult::incomplete(&format!(
            "Could not determine installed packages: {}",
            result.errors.join(", ")
        )));
    }

    None
}

/// Format packages answer from document
fn format_packages_answer(doc: &KnowledgeDoc) -> String {
    format!("**{}**\n\n{}", doc.title, doc.body)
}

/// Answer app alternatives from knowledge store
fn answer_app_alternatives(query: &str, store: &KnowledgeStore) -> Option<RagAnswerResult> {
    // Extract the app name from the query
    let app_name = extract_app_name(query)?;

    // Query for recipes and wiki docs mentioning this app
    let search_query = RetrievalQuery::new(&format!("{} alternative", app_name))
        .with_sources(vec![
            KnowledgeSource::Recipe,
            KnowledgeSource::ArchWiki,
            KnowledgeSource::AUR,
        ])
        .with_limit(5);

    let hits = store.query(&search_query);

    if hits.is_empty() {
        // No knowledge available - suggest enabling wiki/aur import
        return Some(RagAnswerResult::incomplete(&format!(
            "I don't have information about alternatives to '{}' in my knowledge store. \
             To enable app alternatives, import Arch Wiki or AUR data with `annactl import`.",
            app_name
        )));
    }

    // Build answer from hits
    let mut answer = format!("**Alternatives to {}:**\n\n", app_name);
    let sources: Vec<String> = hits.iter().map(|h| h.doc_id.clone()).collect();

    for hit in &hits {
        answer.push_str(&format!("- **{}** ({})\n", hit.title, hit.source));
        answer.push_str(&format!("  {}\n\n", hit.snippet));
    }

    Some(RagAnswerResult::complete(answer, sources))
}

/// Extract app name from query like "alternative to vim"
fn extract_app_name(query: &str) -> Option<String> {
    let q = query.to_lowercase();

    // Try patterns like "alternative to X", "instead of X"
    let patterns = [
        "alternative to ",
        "alternatives to ",
        "instead of ",
        "replacement for ",
        "similar to ",
    ];

    for pattern in patterns {
        if let Some(idx) = q.find(pattern) {
            let rest = &q[idx + pattern.len()..];
            let app = rest
                .split_whitespace()
                .next()
                .map(|s| s.trim_end_matches(|c: char| !c.is_alphanumeric()));
            if let Some(app) = app {
                if !app.is_empty() {
                    return Some(app.to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_app_name() {
        assert_eq!(extract_app_name("alternative to vim"), Some("vim".to_string()));
        assert_eq!(extract_app_name("alternatives to vim?"), Some("vim".to_string()));
        assert_eq!(extract_app_name("instead of firefox"), Some("firefox".to_string()));
        assert_eq!(extract_app_name("replacement for vscode"), Some("vscode".to_string()));
    }

    #[test]
    fn test_extract_app_name_no_match() {
        assert_eq!(extract_app_name("show me vim"), None);
        assert_eq!(extract_app_name("how to install"), None);
    }

    #[test]
    fn test_format_boot_time_answer() {
        let doc = KnowledgeDoc::new(
            KnowledgeSource::SystemFact,
            "Boot Time: 7.5s",
            "Boot time: 7.5s\nKernel: 2s, Userspace: 5.5s",
            vec![],
            anna_shared::knowledge::Provenance::computed("test", 100),
        );

        let answer = format_boot_time_answer(&doc);
        assert!(answer.contains("Boot Time: 7.5s"));
    }

    #[test]
    fn test_rag_answer_result_complete() {
        let result = RagAnswerResult::complete(
            "Test answer".to_string(),
            vec!["doc1".to_string()],
        );
        assert!(result.is_complete);
        assert!(!result.collected_fresh);
    }

    #[test]
    fn test_rag_answer_result_with_fresh() {
        let result = RagAnswerResult::complete(
            "Test answer".to_string(),
            vec!["doc1".to_string()],
        ).with_fresh_collection();
        assert!(result.is_complete);
        assert!(result.collected_fresh);
    }
}
