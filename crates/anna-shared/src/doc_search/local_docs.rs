//! Local documentation search (/usr/share/doc).

use std::path::Path;

use crate::knowledge_item::{KnowledgeItem, KnowledgeSourceType};

use super::utils::grep_directory;

/// Search /usr/share/doc for relevant files
pub fn search_local_docs(keywords: &[String], tags: &[String], limit: usize) -> Vec<KnowledgeItem> {
    let mut results = vec![];

    if keywords.is_empty() && tags.is_empty() {
        return results;
    }

    let doc_dirs = ["/usr/share/doc", "/usr/share/help"];

    // Build grep pattern
    let pattern = if !keywords.is_empty() {
        keywords.join("|")
    } else {
        tags.join("|")
    };

    for doc_dir in &doc_dirs {
        if !Path::new(doc_dir).exists() {
            continue;
        }

        // Use grep to find matching files
        let grep_results = grep_directory(doc_dir, &pattern, limit);
        for (path, snippet) in grep_results {
            let title = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "doc".to_string());

            let item =
                KnowledgeItem::from_path(KnowledgeSourceType::LocalDoc, path, title, snippet)
                    .with_tags(tags.to_vec());

            results.push(item);
        }
    }

    results.truncate(limit);
    results
}
