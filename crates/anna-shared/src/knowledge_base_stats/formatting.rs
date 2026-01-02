//! Formatting functions for knowledge base stats

use super::stats::KnowledgeBaseStats;

/// Format knowledge base stats as full display
pub fn format_knowledge_stats(stats: &KnowledgeBaseStats) -> String {
    let mut lines = Vec::new();

    lines.push("=== Knowledge Base Statistics ===".to_string());
    lines.push(String::new());

    // Overview
    lines.push(format!("Total Entries: {}", stats.total_entries));
    lines.push(format!("Total Uses: {}", stats.total_uses));
    lines.push(format!("Avg Uses/Entry: {:.1}", stats.avg_uses()));
    lines.push(format!("Learned: {}", stats.learned_count()));

    if stats.stale_count > 0 {
        lines.push(format!("Stale Entries: {} ({:.0}%)", stats.stale_count, stats.stale_percent()));
    }

    lines.push(String::new());

    // By type
    lines.push("--- By Type ---".to_string());
    lines.push(format!("  Recipes: {}", stats.recipe_count()));
    lines.push(format!("  Facts: {}", stats.fact_count()));
    lines.push(format!("  Cached Docs: {}", stats.doc_cache_count()));
    lines.push(format!("  User Taught: {}", stats.user_taught_count()));

    // By source
    if !stats.by_source.is_empty() {
        lines.push(String::new());
        lines.push("--- By Source ---".to_string());
        for (source, count) in &stats.by_source {
            lines.push(format!("  {}: {}", source, count));
        }
    }

    // Top topics
    if !stats.by_topic.is_empty() {
        lines.push(String::new());
        lines.push("--- Top Topics ---".to_string());
        let mut topics: Vec<_> = stats.by_topic.iter().collect();
        topics.sort_by(|a, b| b.1.cmp(a.1));
        for (topic, count) in topics.iter().take(5) {
            lines.push(format!("  {}: {}", topic, count));
        }
    }

    // Most used
    if !stats.most_used.is_empty() {
        lines.push(String::new());
        lines.push("--- Most Used ---".to_string());
        for entry in stats.most_used.iter().take(5) {
            lines.push(format!("  {} ({} uses)", entry.id, entry.use_count));
        }
    }

    lines.join("\n")
}

/// Format knowledge base stats compact
pub fn format_knowledge_stats_compact(stats: &KnowledgeBaseStats) -> String {
    format!(
        "Knowledge: {} entries ({} recipes, {} facts, {} learned)",
        stats.total_entries,
        stats.recipe_count(),
        stats.fact_count(),
        stats.learned_count()
    )
}

/// Format knowledge base stats one-line
pub fn format_knowledge_stats_oneline(stats: &KnowledgeBaseStats) -> String {
    format!(
        "KB: {} total, {} recipes, {:.1} avg uses",
        stats.total_entries,
        stats.recipe_count(),
        stats.avg_uses()
    )
}
