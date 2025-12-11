//! Knowledge Query Executor (v0.0.414).
//!
//! Executes knowledge queries against all configured sources.
//! This is the main entry point for doc-first reasoning.
//!
//! Query flow:
//! 1. Check built-in pack (fast, curated)
//! 2. Check man pages
//! 3. Check --help output
//! 4. Check Arch Wiki cache (if enabled)
//! 5. Check knowledge store (learned recipes, cached docs)

use crate::doc_brain::search_docs;
use crate::doc_fetcher::{fetch_arch_wiki, fetch_help_output, fetch_man_page, wiki_cache_available};
use crate::knowledge::search_builtin_pack;
use crate::knowledge_config::KnowledgeConfig;
use crate::knowledge_query::{KnowledgeHit, KnowledgeQuery, KnowledgeResult, KnowledgeSourceKind};
use std::time::Instant;
use tracing::{debug, info};

/// Execute a knowledge query against all sources
pub fn query_knowledge(query: &KnowledgeQuery) -> KnowledgeResult {
    let start = Instant::now();
    let config = KnowledgeConfig::load();
    let mut result = KnowledgeResult::empty();

    debug!(
        "Querying knowledge: domain={}, topic={}, commands={:?}",
        query.domain, query.topic, query.related_commands
    );

    // Track which sources we searched
    let mut sources_searched = Vec::new();

    // 1. Built-in pack (always available, fast)
    sources_searched.push(KnowledgeSourceKind::BuiltIn);
    let builtin_hits = search_builtin(&query.topic, query.max_results / 2);
    result.hits.extend(builtin_hits);

    // 2. Man pages for related commands
    sources_searched.push(KnowledgeSourceKind::ManPage);
    for cmd in &query.related_commands {
        if let Some(hit) = search_man_page(cmd, &query.topic) {
            result.hits.push(hit);
        }
    }
    // Also try topic-derived commands
    for tag in query.search_tags().iter().take(3) {
        if looks_like_command(tag) {
            if let Some(hit) = search_man_page(tag, &query.topic) {
                if !result.hits.iter().any(|h| h.doc_id == hit.doc_id) {
                    result.hits.push(hit);
                }
            }
        }
    }

    // 3. --help output for commands
    sources_searched.push(KnowledgeSourceKind::CliHelp);
    for cmd in &query.related_commands {
        if let Some(hit) = search_help(cmd, &query.topic) {
            if !result.hits.iter().any(|h| h.doc_id == hit.doc_id) {
                result.hits.push(hit);
            }
        }
    }

    // 4. Arch Wiki (if enabled and available)
    result.wiki_available = config.wiki_available();
    if config.arch_wiki_enabled && result.wiki_available {
        sources_searched.push(KnowledgeSourceKind::ArchWikiPage);
        let wiki_hits = search_arch_wiki(&query.topic, &query.domain, query.max_results / 2);
        for hit in wiki_hits {
            if !result.hits.iter().any(|h| h.doc_id == hit.doc_id) {
                result.hits.push(hit);
            }
        }
    }

    // 5. Knowledge store (cached docs, learned recipes)
    sources_searched.push(KnowledgeSourceKind::LearnedRecipe);
    let store_hits = search_docs(&query.topic, query.max_results);
    for store_hit in store_hits {
        let hit = KnowledgeHit {
            doc_id: store_hit.doc_id,
            kind: KnowledgeSourceKind::from_legacy(&store_hit.source),
            title: store_hit.title,
            origin: format_store_origin(&store_hit.source),
            excerpt: store_hit.snippet,
            relevance: store_hit.confidence,
            path: None,
        };
        if !result.hits.iter().any(|h| h.doc_id == hit.doc_id) {
            result.hits.push(hit);
        }
    }

    // Sort by relevance and priority, then truncate
    result.hits.sort_by(|a, b| {
        // First by relevance (descending)
        let rel_cmp = b.relevance.cmp(&a.relevance);
        if rel_cmp != std::cmp::Ordering::Equal {
            return rel_cmp;
        }
        // Then by source priority (ascending = higher priority)
        a.kind.priority().cmp(&b.kind.priority())
    });

    // Filter by minimum relevance
    result.hits.retain(|h| h.relevance >= query.min_relevance);

    // Truncate to max results
    result.hits.truncate(query.max_results);

    result.sources_searched = sources_searched;
    result.query_time_ms = start.elapsed().as_millis() as u64;

    info!(
        "Knowledge query complete: {} hits in {}ms",
        result.hits.len(),
        result.query_time_ms
    );

    result
}

/// Search built-in knowledge pack
fn search_builtin(topic: &str, limit: usize) -> Vec<KnowledgeHit> {
    search_builtin_pack(topic, limit)
        .into_iter()
        .map(|(score, entry)| KnowledgeHit {
            doc_id: format!("builtin:{}", entry.id),
            kind: KnowledgeSourceKind::BuiltIn,
            title: entry.title.to_string(),
            origin: format!("Built-in: {}", entry.title),
            excerpt: truncate(entry.body, 300),
            relevance: score_to_relevance(score),
            path: None,
        })
        .collect()
}

/// Search man page for a command
fn search_man_page(command: &str, topic: &str) -> Option<KnowledgeHit> {
    let snippet = fetch_man_page(command)?;
    let relevance = calculate_relevance(&snippet.snippet, topic);

    Some(KnowledgeHit {
        doc_id: format!("man:{}", command),
        kind: KnowledgeSourceKind::ManPage,
        title: snippet.title,
        origin: format!("man {}", command),
        excerpt: snippet.snippet,
        relevance,
        path: Some(snippet.location),
    })
}

/// Search --help output for a command
fn search_help(command: &str, topic: &str) -> Option<KnowledgeHit> {
    let snippet = fetch_help_output(command)?;
    let relevance = calculate_relevance(&snippet.snippet, topic);

    Some(KnowledgeHit {
        doc_id: format!("help:{}", command),
        kind: KnowledgeSourceKind::CliHelp,
        title: snippet.title,
        origin: format!("{} --help", command),
        excerpt: snippet.snippet,
        relevance,
        path: None,
    })
}

/// Search Arch Wiki cache
fn search_arch_wiki(topic: &str, domain: &str, limit: usize) -> Vec<KnowledgeHit> {
    let mut hits = Vec::new();

    // Try direct topic match
    if let Some(snippet) = fetch_arch_wiki(topic) {
        hits.push(KnowledgeHit {
            doc_id: format!("wiki:{}", topic.to_lowercase().replace(' ', "_")),
            kind: KnowledgeSourceKind::ArchWikiPage,
            title: snippet.title,
            origin: format!("Arch Wiki: {}", topic),
            excerpt: snippet.snippet,
            relevance: snippet.relevance,
            path: Some(snippet.location),
        });
    }

    // Try domain-related pages
    let domain_pages = wiki_pages_for_domain(domain);
    for page in domain_pages.into_iter().take(limit) {
        if let Some(snippet) = fetch_arch_wiki(page) {
            let doc_id = format!("wiki:{}", page.to_lowercase().replace(' ', "_"));
            if !hits.iter().any(|h| h.doc_id == doc_id) {
                let relevance = calculate_relevance(&snippet.snippet, topic);
                hits.push(KnowledgeHit {
                    doc_id,
                    kind: KnowledgeSourceKind::ArchWikiPage,
                    title: snippet.title,
                    origin: format!("Arch Wiki: {}", page),
                    excerpt: snippet.snippet,
                    relevance,
                    path: Some(snippet.location),
                });
            }
        }
    }

    hits
}

/// Get wiki pages related to a domain
fn wiki_pages_for_domain(domain: &str) -> Vec<&'static str> {
    match domain {
        "services" | "systemd" => vec!["systemd", "systemd-service", "systemd-analyze"],
        "network" => vec!["Network_configuration", "NetworkManager", "systemd-networkd"],
        "storage" => vec!["Fstab", "Partitioning", "Btrfs", "LVM"],
        "boot" => vec!["Arch_boot_process", "systemd-boot", "GRUB"],
        "audio" => vec!["PipeWire", "PulseAudio", "ALSA"],
        "desktop" => vec!["Hyprland", "Sway", "KDE", "GNOME"],
        "packages" => vec!["Pacman", "AUR", "makepkg"],
        "security" => vec!["Security", "UFW", "iptables", "SSH"],
        _ => vec![],
    }
}

/// Format origin for knowledge store entries
fn format_store_origin(source: &crate::knowledge::KnowledgeSource) -> String {
    match source {
        crate::knowledge::KnowledgeSource::Recipe => "Learned recipe".to_string(),
        crate::knowledge::KnowledgeSource::SystemFact => "System fact".to_string(),
        crate::knowledge::KnowledgeSource::PackageFact => "Package info".to_string(),
        crate::knowledge::KnowledgeSource::ManPage => "man page".to_string(),
        crate::knowledge::KnowledgeSource::HelpOutput => "--help output".to_string(),
        crate::knowledge::KnowledgeSource::ArchWiki => "Arch Wiki".to_string(),
        crate::knowledge::KnowledgeSource::BuiltIn => "Built-in".to_string(),
        _ => "Knowledge store".to_string(),
    }
}

/// Check if string looks like a command name
fn looks_like_command(s: &str) -> bool {
    // Commands are typically short, lowercase, no spaces
    s.len() >= 2
        && s.len() <= 20
        && !s.contains(' ')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Calculate relevance based on keyword matching
fn calculate_relevance(text: &str, topic: &str) -> u8 {
    let text_lower = text.to_lowercase();
    let topic_lower = topic.to_lowercase();
    let topic_words: Vec<&str> = topic_lower.split_whitespace().collect();

    let mut score = 50u8; // Base score

    // Exact topic match
    if text_lower.contains(&topic_lower) {
        score = score.saturating_add(30);
    }

    // Word matches
    let word_matches = topic_words.iter().filter(|w| text_lower.contains(*w)).count();
    let word_bonus = ((word_matches as f32 / topic_words.len().max(1) as f32) * 20.0) as u8;
    score = score.saturating_add(word_bonus);

    score.min(100)
}

/// Convert search score to relevance (0-100)
fn score_to_relevance(score: u32) -> u8 {
    let base = 50u8;
    let bonus = (score as u8).min(45);
    base.saturating_add(bonus)
}

/// Truncate text with ellipsis
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Build a knowledge query from ticket context
pub fn build_query_from_context(
    domain: &str,
    intent: &str,
    question: &str,
    tags: &[String],
) -> KnowledgeQuery {
    // Extract potential commands from tags
    let commands: Vec<String> = tags
        .iter()
        .filter(|t| looks_like_command(t))
        .cloned()
        .collect();

    // Add domain-specific commands
    let mut all_commands = commands;
    all_commands.extend(domain_commands(domain).into_iter().map(String::from));
    all_commands.dedup();

    KnowledgeQuery::new(domain, question)
        .with_commands(all_commands.iter().map(|s| s.as_str()).collect())
        .with_limit(8)
}

/// Get default commands for a domain
fn domain_commands(domain: &str) -> Vec<&'static str> {
    match domain {
        "services" | "systemd" => vec!["systemctl", "journalctl"],
        "network" => vec!["ip", "nmcli", "networkctl", "ss"],
        "storage" => vec!["df", "lsblk", "mount", "findmnt"],
        "packages" => vec!["pacman", "yay"],
        "boot" => vec!["systemd-analyze", "bootctl"],
        "audio" => vec!["pactl", "wpctl"],
        "security" => vec!["ufw", "ss", "iptables"],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_command() {
        assert!(looks_like_command("systemctl"));
        assert!(looks_like_command("pacman"));
        assert!(!looks_like_command("failed service")); // Has space
        assert!(!looks_like_command("x")); // Too short
    }

    #[test]
    fn test_calculate_relevance() {
        let text = "systemctl is used to introspect and control the state of the systemd system";
        assert!(calculate_relevance(text, "systemctl") > 70);
        assert!(calculate_relevance(text, "unrelated topic") < 60);
    }

    #[test]
    fn test_build_query_from_context() {
        let query = build_query_from_context(
            "services",
            "diagnose",
            "why is my service failing",
            &["nginx".to_string(), "systemd".to_string()],
        );

        assert_eq!(query.domain, "services");
        assert!(query.related_commands.contains(&"systemctl".to_string()));
    }

    #[test]
    fn test_wiki_pages_for_domain() {
        let pages = wiki_pages_for_domain("boot");
        assert!(pages.contains(&"systemd-boot"));
    }
}
