//! Main wiki page reading logic (v0.0.429).

use super::file_ops::find_wiki_page;
use super::html::{clean_html, clean_plain_text, truncate};
use super::sections::{extract_wiki_summary, is_useful_section, split_wiki_sections};
use super::types::WikiReadError;
use crate::doc_engine::{DocSnippet, DocSourceKind, MAX_SNIPPET_SIZE};
use std::fs;
use std::path::Path;

/// Read Arch Wiki page from local cache
pub fn read_wiki_page(name: &str, cache_path: &Path) -> Result<Vec<DocSnippet>, WikiReadError> {
    let page_path = find_wiki_page(name, cache_path)?;
    let content =
        fs::read_to_string(&page_path).map_err(|e| WikiReadError::ReadFailed(e.to_string()))?;

    // Detect format and parse
    let is_html = page_path.extension().map(|e| e == "html").unwrap_or(false)
        || content.trim_start().starts_with('<');

    let clean_content = if is_html {
        clean_html(&content)
    } else {
        clean_plain_text(&content)
    };

    // Split into sections
    let sections = split_wiki_sections(&clean_content);

    let mut snippets = Vec::new();

    // Create main snippet
    let summary = extract_wiki_summary(&clean_content);
    snippets.push(DocSnippet::new(
        DocSourceKind::ArchWiki,
        name,
        None,
        &summary,
        &truncate(&clean_content, MAX_SNIPPET_SIZE),
    ));

    // Create snippets for key sections
    for (section_name, section_content) in sections {
        if section_content.len() > 50 && is_useful_section(&section_name) {
            let section_id = section_name.to_lowercase().replace(' ', "_");
            let mut snippet = DocSnippet::new(
                DocSourceKind::ArchWiki,
                name,
                Some(&section_id),
                &format!("{} - {}", name, section_name),
                &section_content,
            );
            snippet.truncate_content(MAX_SNIPPET_SIZE);
            snippets.push(snippet);
        }
    }

    if snippets.is_empty() {
        return Err(WikiReadError::NoContent(name.to_string()));
    }

    Ok(snippets)
}
