//! Arch Wiki local mirror search functionality.

use std::path::Path;
use tracing::debug;

use crate::knowledge_item::{KnowledgeItem, KnowledgeSourceType};

use super::constants::ARCH_WIKI_PATH;
use super::utils::grep_directory;

/// Search Arch Wiki local mirror (if present)
pub fn search_arch_wiki_local(keywords: &[String], limit: usize) -> Vec<KnowledgeItem> {
    let wiki_path = Path::new(ARCH_WIKI_PATH);

    if !wiki_path.exists() {
        debug!("Arch Wiki local mirror not found at {}", ARCH_WIKI_PATH);
        return vec![];
    }

    if keywords.is_empty() {
        return vec![];
    }

    let pattern = keywords.join("|");
    let grep_results = grep_directory(ARCH_WIKI_PATH, &pattern, limit);

    grep_results
        .into_iter()
        .map(|(path, snippet)| {
            let title = path
                .file_stem()
                .map(|n| format!("Arch Wiki: {}", n.to_string_lossy()))
                .unwrap_or_else(|| "Arch Wiki".to_string());

            KnowledgeItem::from_path(KnowledgeSourceType::ArchWikiLocal, path, title, snippet)
        })
        .collect()
}

/// Check if Arch Wiki local mirror is available
pub fn arch_wiki_available() -> bool {
    Path::new(ARCH_WIKI_PATH).exists()
}

/// Suggest a manual Arch Wiki link (for when local is unavailable)
pub fn suggest_arch_wiki_link(topic: &str) -> String {
    let slug = topic
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();

    format!("https://wiki.archlinux.org/title/{}", slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_arch_wiki_link() {
        let link = suggest_arch_wiki_link("systemd service");
        assert!(link.contains("wiki.archlinux.org"));
        assert!(link.contains("systemd_service"));
    }
}
