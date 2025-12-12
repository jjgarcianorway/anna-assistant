//! Integration between doc_engine and recipe system (v0.0.429).
//!
//! Provides:
//! - DocReference type for recipes
//! - Functions to add doc citations to recipes
//! - Functions to query docs for recipe execution

use super::{DocEngine, DocQuery, DocReference, DocResult, DocSnippet, DocSourceKind};
use crate::learning_engine::{AnswerKind, LearnedRecipe, RecipeOrigin};

/// Add documentation references to a recipe's origin
pub fn add_doc_references(recipe: &mut LearnedRecipe, refs: &[DocReference]) {
    for doc_ref in refs {
        let citation = doc_ref.citation();
        if !recipe.origin.sources.contains(&citation) {
            recipe.origin.sources.push(citation);
        }
    }
}

/// Create doc references from snippets
pub fn refs_from_snippets(snippets: &[DocSnippet]) -> Vec<DocReference> {
    snippets
        .iter()
        .map(|s| DocReference {
            source: s.source,
            name: s.name.clone(),
            section: s.section.clone(),
            reason: Some(s.summary.clone()),
        })
        .collect()
}

/// Query documentation relevant to a recipe's domain/intent
pub fn query_for_recipe(engine: &DocEngine, recipe: &LearnedRecipe) -> DocResult {
    let query_text = format!(
        "{} {}",
        recipe.pattern.intent.replace('_', " "),
        recipe.domain
    );

    // Determine preferred sources based on answer kind
    let sources = match recipe.logic.answer_kind {
        AnswerKind::Status | AnswerKind::Diagnostic => {
            // Prefer man pages for status checks
            vec![DocSourceKind::ManPage, DocSourceKind::ToolHelp]
        }
        AnswerKind::Explanation => {
            // Prefer wiki for explanations
            vec![DocSourceKind::ArchWiki, DocSourceKind::ManPage]
        }
        AnswerKind::Fix => {
            // Both for fixes
            vec![
                DocSourceKind::ArchWiki,
                DocSourceKind::ManPage,
                DocSourceKind::ToolHelp,
            ]
        }
    };

    let query = DocQuery::new(&query_text)
        .with_sources(sources)
        .with_limit(3);

    engine.query_or_fetch(query)
}

/// Get documentation for a specific command used in a recipe probe
pub fn docs_for_probe_command(engine: &DocEngine, command: &str) -> DocResult {
    // Get base command (first word)
    let base_cmd = command.split_whitespace().next().unwrap_or(command);

    // Try man page first, then help
    let man_result = engine.man_page(base_cmd);
    if !man_result.is_empty() {
        return man_result;
    }

    engine.help_output(base_cmd)
}

/// Suggested doc references based on domain
pub fn suggest_doc_refs(domain: &str) -> Vec<DocReference> {
    match domain {
        "services" | "services.systemd" => vec![
            DocReference::arch_wiki("systemd").with_reason("Service management"),
            DocReference::man_page("systemctl", "1").with_reason("Service control"),
            DocReference::man_page("journalctl", "1").with_reason("Service logs"),
        ],
        "storage" | "storage.disk" => vec![
            DocReference::arch_wiki("File_systems").with_reason("Filesystem concepts"),
            DocReference::arch_wiki("Fstab").with_reason("Mount configuration"),
            DocReference::man_page("df", "1").with_reason("Disk space"),
            DocReference::man_page("lsblk", "8").with_reason("Block devices"),
        ],
        "storage.ssd" | "storage.nvme" => vec![
            DocReference::arch_wiki("Solid_state_drive").with_reason("SSD optimization"),
            DocReference::arch_wiki("TRIM").with_reason("SSD TRIM support"),
            DocReference::man_page("fstrim", "8").with_reason("Trim command"),
        ],
        "packages" | "packages.pacman" => vec![
            DocReference::arch_wiki("pacman").with_reason("Package management"),
            DocReference::man_page("pacman", "8").with_reason("Pacman command"),
        ],
        "network" => vec![
            DocReference::arch_wiki("Network_configuration").with_reason("Network setup"),
            DocReference::man_page("ip", "8").with_reason("IP configuration"),
            DocReference::man_page("ss", "8").with_reason("Socket statistics"),
        ],
        "boot" => vec![
            DocReference::arch_wiki("Arch_boot_process").with_reason("Boot sequence"),
            DocReference::arch_wiki("Mkinitcpio").with_reason("Initramfs"),
            DocReference::man_page("bootctl", "1").with_reason("Boot manager"),
        ],
        "performance" | "performance.memory" => vec![
            DocReference::man_page("free", "1").with_reason("Memory info"),
            DocReference::man_page("top", "1").with_reason("Process monitor"),
        ],
        _ => {
            vec![DocReference::arch_wiki("General_troubleshooting").with_reason("Troubleshooting")]
        }
    }
}

/// Check if a recipe should include doc citations
pub fn recipe_needs_docs(recipe: &LearnedRecipe) -> bool {
    // Explanations always need docs
    if recipe.logic.answer_kind == AnswerKind::Explanation {
        return true;
    }

    // Fixes benefit from docs
    if recipe.logic.answer_kind == AnswerKind::Fix {
        return true;
    }

    // If recipe has no existing sources, suggest adding docs
    recipe.origin.sources.is_empty()
}

/// Format doc citations for display in answers
pub fn format_citations_for_answer(snippets: &[DocSnippet]) -> String {
    if snippets.is_empty() {
        return String::new();
    }

    let citations: Vec<String> = snippets.iter().map(|s| s.citation()).collect();

    if citations.len() == 1 {
        format!("(See {})", citations[0])
    } else {
        format!("(See {}, {})", citations[0], citations[1..].join(", "))
    }
}

/// Extract relevant snippet for a specific topic from results
pub fn find_relevant_snippet<'a>(result: &'a DocResult, topic: &str) -> Option<&'a DocSnippet> {
    let topic_lower = topic.to_lowercase();

    // First try exact name match
    if let Some(s) = result
        .snippets
        .iter()
        .find(|s| s.name.to_lowercase() == topic_lower)
    {
        return Some(s);
    }

    // Then try section match
    if let Some(s) = result.snippets.iter().find(|s| {
        s.section
            .as_ref()
            .map(|sec| sec.to_lowercase().contains(&topic_lower))
            .unwrap_or(false)
    }) {
        return Some(s);
    }

    // Fall back to content match
    result
        .snippets
        .iter()
        .find(|s| s.content.to_lowercase().contains(&topic_lower))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_doc_refs() {
        let refs = suggest_doc_refs("services.systemd");
        assert!(!refs.is_empty());
        assert!(refs
            .iter()
            .any(|r| r.name.contains("systemd") || r.name.contains("systemctl")));
    }

    #[test]
    fn test_format_citations() {
        let snippets = vec![DocSnippet::new(
            DocSourceKind::ManPage,
            "systemctl",
            Some("1"),
            "test",
            "content",
        )];
        let formatted = format_citations_for_answer(&snippets);
        assert!(formatted.contains("systemctl(1)"));
    }

    #[test]
    fn test_refs_from_snippets() {
        let snippets = vec![DocSnippet::new(
            DocSourceKind::ArchWiki,
            "systemd",
            None,
            "Systemd info",
            "...",
        )];
        let refs = refs_from_snippets(&snippets);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].source, DocSourceKind::ArchWiki);
    }
}
