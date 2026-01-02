//! Anna documentation search (recipes, handbook).

use std::path::PathBuf;

use crate::knowledge_item::{KnowledgeItem, KnowledgeSourceType};

use super::constants::ANNA_DOCS_PATH;
use super::utils::grep_directory;

/// Search Anna's own documentation
pub fn search_anna_docs(keywords: &[String], limit: usize) -> Vec<KnowledgeItem> {
    let anna_path = std::path::Path::new(ANNA_DOCS_PATH);
    let home_anna = dirs::home_dir()
        .map(|h| h.join(".anna").join("docs"))
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    let mut results = vec![];

    // Search both system and user anna docs
    for doc_path in [anna_path.to_path_buf(), home_anna] {
        if !doc_path.exists() {
            continue;
        }

        if keywords.is_empty() {
            continue;
        }

        let pattern = keywords.join("|");
        let grep_results = grep_directory(doc_path.to_str().unwrap_or(""), &pattern, limit);

        for (path, snippet) in grep_results {
            let title = path
                .file_stem()
                .map(|n| format!("Anna: {}", n.to_string_lossy()))
                .unwrap_or_else(|| "Anna doc".to_string());

            let item = KnowledgeItem::from_path(KnowledgeSourceType::AnnaDoc, path, title, snippet);

            results.push(item);
        }
    }

    results.truncate(limit);
    results
}
