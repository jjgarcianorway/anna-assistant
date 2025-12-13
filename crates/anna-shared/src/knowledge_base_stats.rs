//! Knowledge Base Stats (Phase 75)
//!
//! Tracks and displays statistics about Anna's knowledge base,
//! including recipes, facts, and documentation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of knowledge entry
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeType {
    /// Learned recipe
    Recipe,
    /// Stored fact
    Fact,
    /// Cached wiki page
    WikiPage,
    /// Cached man page
    ManPage,
    /// Cached help output
    HelpCache,
    /// User-taught pattern
    UserTaught,
}

impl KnowledgeType {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Recipe => "Recipe",
            Self::Fact => "Fact",
            Self::WikiPage => "Wiki Page",
            Self::ManPage => "Man Page",
            Self::HelpCache => "Help Cache",
            Self::UserTaught => "User Taught",
        }
    }

    /// Plural display name
    pub fn display_plural(&self) -> &'static str {
        match self {
            Self::Recipe => "Recipes",
            Self::Fact => "Facts",
            Self::WikiPage => "Wiki Pages",
            Self::ManPage => "Man Pages",
            Self::HelpCache => "Help Caches",
            Self::UserTaught => "User Taught",
        }
    }
}

/// Source of knowledge
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Built-in seed knowledge
    Seed,
    /// Learned from specialist
    Specialist,
    /// Learned from user interaction
    User,
    /// Fetched from Arch Wiki
    ArchWiki,
    /// Fetched from man pages
    ManPages,
    /// Fetched from help commands
    HelpCommands,
    /// Imported from external source
    Imported,
}

impl KnowledgeSource {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::Seed => "Seed",
            Self::Specialist => "Specialist",
            Self::User => "User",
            Self::ArchWiki => "Arch Wiki",
            Self::ManPages => "Man Pages",
            Self::HelpCommands => "Help Commands",
            Self::Imported => "Imported",
        }
    }
}

/// Individual knowledge entry stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// Entry ID
    pub id: String,
    /// Type of knowledge
    pub knowledge_type: KnowledgeType,
    /// Source of knowledge
    pub source: KnowledgeSource,
    /// When first acquired (Unix timestamp)
    pub acquired_at: u64,
    /// When last used (Unix timestamp)
    pub last_used: u64,
    /// Number of times used
    pub use_count: u64,
    /// Topic/category
    pub topic: Option<String>,
    /// Reliability score (0-100)
    pub reliability: u8,
}

impl KnowledgeEntry {
    /// Create a new entry
    pub fn new(
        id: impl Into<String>,
        knowledge_type: KnowledgeType,
        source: KnowledgeSource,
        acquired_at: u64,
    ) -> Self {
        Self {
            id: id.into(),
            knowledge_type,
            source,
            acquired_at,
            last_used: acquired_at,
            use_count: 0,
            topic: None,
            reliability: 80,
        }
    }

    /// Record usage
    pub fn record_use(&mut self, timestamp: u64) {
        self.use_count += 1;
        self.last_used = timestamp;
    }

    /// Is this entry stale (not used in 30 days)?
    pub fn is_stale(&self, now: u64) -> bool {
        now.saturating_sub(self.last_used) > 30 * 86400
    }
}

/// Knowledge base statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeBaseStats {
    /// Entries by type
    pub by_type: HashMap<String, u64>,
    /// Entries by source
    pub by_source: HashMap<String, u64>,
    /// Entries by topic
    pub by_topic: HashMap<String, u64>,
    /// Total entries
    pub total_entries: u64,
    /// Total uses
    pub total_uses: u64,
    /// Stale entries count
    pub stale_count: u64,
    /// Recent entries (last 20)
    pub recent: Vec<KnowledgeEntry>,
    /// Most used entries (top 10)
    pub most_used: Vec<KnowledgeEntry>,
    /// Last acquisition timestamp
    pub last_acquisition: u64,
}

impl KnowledgeBaseStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a knowledge entry
    pub fn record(&mut self, entry: KnowledgeEntry, now: u64) {
        // Update type counts
        *self.by_type.entry(entry.knowledge_type.display().to_string()).or_insert(0) += 1;

        // Update source counts
        *self.by_source.entry(entry.source.display().to_string()).or_insert(0) += 1;

        // Update topic counts
        if let Some(ref topic) = entry.topic {
            *self.by_topic.entry(topic.clone()).or_insert(0) += 1;
        }

        self.total_entries += 1;
        self.total_uses += entry.use_count;

        if entry.is_stale(now) {
            self.stale_count += 1;
        }

        if entry.acquired_at > self.last_acquisition {
            self.last_acquisition = entry.acquired_at;
        }

        // Add to recent
        self.recent.insert(0, entry.clone());
        if self.recent.len() > 20 {
            self.recent.truncate(20);
        }

        // Update most used
        self.most_used.push(entry);
        self.most_used.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        self.most_used.truncate(10);
    }

    /// Get recipe count
    pub fn recipe_count(&self) -> u64 {
        *self.by_type.get("Recipe").unwrap_or(&0)
    }

    /// Get fact count
    pub fn fact_count(&self) -> u64 {
        *self.by_type.get("Fact").unwrap_or(&0)
    }

    /// Get cached documentation count
    pub fn doc_cache_count(&self) -> u64 {
        let wiki = *self.by_type.get("Wiki Page").unwrap_or(&0);
        let man = *self.by_type.get("Man Page").unwrap_or(&0);
        let help = *self.by_type.get("Help Cache").unwrap_or(&0);
        wiki + man + help
    }

    /// Get user-taught count
    pub fn user_taught_count(&self) -> u64 {
        *self.by_type.get("User Taught").unwrap_or(&0)
    }

    /// Get learned (non-seed) count
    pub fn learned_count(&self) -> u64 {
        self.total_entries - *self.by_source.get("Seed").unwrap_or(&0)
    }

    /// Average uses per entry
    pub fn avg_uses(&self) -> f64 {
        if self.total_entries == 0 {
            return 0.0;
        }
        self.total_uses as f64 / self.total_entries as f64
    }

    /// Stale percentage
    pub fn stale_percent(&self) -> f64 {
        if self.total_entries == 0 {
            return 0.0;
        }
        (self.stale_count as f64 / self.total_entries as f64) * 100.0
    }

    /// Top topic
    pub fn top_topic(&self) -> Option<(&String, u64)> {
        self.by_topic.iter().max_by_key(|(_, count)| *count).map(|(k, v)| (k, *v))
    }

    /// Top source
    pub fn top_source(&self) -> Option<(&String, u64)> {
        self.by_source.iter().max_by_key(|(_, count)| *count).map(|(k, v)| (k, *v))
    }
}

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

/// Generate a knowledge insight
pub fn knowledge_insight(stats: &KnowledgeBaseStats) -> Option<String> {
    if stats.total_entries == 0 {
        return None;
    }

    // Check for interesting patterns
    if stats.learned_count() > 50 {
        return Some(format!(
            "Impressive! Anna has learned {} things beyond the initial knowledge.",
            stats.learned_count()
        ));
    }

    if stats.user_taught_count() > 10 {
        return Some(format!(
            "You've taught Anna {} things directly. Great collaboration!",
            stats.user_taught_count()
        ));
    }

    if stats.avg_uses() > 5.0 {
        return Some(format!(
            "Knowledge is being put to good use! Average {:.1} uses per entry.",
            stats.avg_uses()
        ));
    }

    if let Some((topic, count)) = stats.top_topic() {
        return Some(format!(
            "Most knowledge is about {} ({} entries).",
            topic, count
        ));
    }

    Some(format!(
        "Anna's knowledge base contains {} entries.",
        stats.total_entries
    ))
}

/// Check if query is asking about knowledge base
pub fn is_knowledge_stats_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "knowledge base",
        "how much do you know",
        "what do you know",
        "knowledge stats",
        "recipe count",
        "how many recipes",
        "learned knowledge",
        "knowledge size",
        "your knowledge",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

/// Knowledge health assessment
pub fn knowledge_health(stats: &KnowledgeBaseStats) -> &'static str {
    if stats.total_entries == 0 {
        return "Empty - No knowledge yet";
    }

    if stats.stale_percent() > 50.0 {
        return "Needs refresh - Many stale entries";
    }

    if stats.learned_count() > 100 {
        return "Thriving - Rich learned knowledge";
    }

    if stats.avg_uses() > 3.0 {
        return "Active - Knowledge is well-used";
    }

    "Growing - Building knowledge"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_type_display() {
        assert_eq!(KnowledgeType::Recipe.display(), "Recipe");
        assert_eq!(KnowledgeType::Recipe.display_plural(), "Recipes");
    }

    #[test]
    fn test_knowledge_source_display() {
        assert_eq!(KnowledgeSource::Seed.display(), "Seed");
        assert_eq!(KnowledgeSource::ArchWiki.display(), "Arch Wiki");
    }

    #[test]
    fn test_knowledge_entry_new() {
        let entry = KnowledgeEntry::new(
            "recipe-001",
            KnowledgeType::Recipe,
            KnowledgeSource::Specialist,
            1000,
        );

        assert_eq!(entry.id, "recipe-001");
        assert_eq!(entry.use_count, 0);
        assert_eq!(entry.reliability, 80);
    }

    #[test]
    fn test_knowledge_entry_record_use() {
        let mut entry = KnowledgeEntry::new(
            "recipe-001",
            KnowledgeType::Recipe,
            KnowledgeSource::Seed,
            1000,
        );

        entry.record_use(2000);
        assert_eq!(entry.use_count, 1);
        assert_eq!(entry.last_used, 2000);
    }

    #[test]
    fn test_knowledge_entry_stale() {
        let entry = KnowledgeEntry::new(
            "recipe-001",
            KnowledgeType::Recipe,
            KnowledgeSource::Seed,
            1000,
        );

        // Not stale after 1 day
        assert!(!entry.is_stale(1000 + 86400));

        // Stale after 31 days
        assert!(entry.is_stale(1000 + 31 * 86400));
    }

    #[test]
    fn test_knowledge_base_stats_record() {
        let mut stats = KnowledgeBaseStats::new();
        let entry = KnowledgeEntry::new(
            "recipe-001",
            KnowledgeType::Recipe,
            KnowledgeSource::Seed,
            1000,
        );

        stats.record(entry, 2000);

        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.recipe_count(), 1);
    }

    #[test]
    fn test_knowledge_base_stats_counts() {
        let mut stats = KnowledgeBaseStats::new();

        // Add recipes
        for i in 0..5 {
            stats.record(
                KnowledgeEntry::new(
                    format!("recipe-{}", i),
                    KnowledgeType::Recipe,
                    KnowledgeSource::Seed,
                    1000,
                ),
                2000,
            );
        }

        // Add facts
        for i in 0..3 {
            stats.record(
                KnowledgeEntry::new(
                    format!("fact-{}", i),
                    KnowledgeType::Fact,
                    KnowledgeSource::Specialist,
                    1000,
                ),
                2000,
            );
        }

        assert_eq!(stats.recipe_count(), 5);
        assert_eq!(stats.fact_count(), 3);
        assert_eq!(stats.total_entries, 8);
    }

    #[test]
    fn test_knowledge_base_learned_count() {
        let mut stats = KnowledgeBaseStats::new();

        // Add seed recipe
        stats.record(
            KnowledgeEntry::new(
                "seed-recipe",
                KnowledgeType::Recipe,
                KnowledgeSource::Seed,
                1000,
            ),
            2000,
        );

        // Add learned recipe
        stats.record(
            KnowledgeEntry::new(
                "learned-recipe",
                KnowledgeType::Recipe,
                KnowledgeSource::Specialist,
                1000,
            ),
            2000,
        );

        assert_eq!(stats.learned_count(), 1);
    }

    #[test]
    fn test_avg_uses() {
        let mut stats = KnowledgeBaseStats::new();

        let mut entry = KnowledgeEntry::new(
            "recipe",
            KnowledgeType::Recipe,
            KnowledgeSource::Seed,
            1000,
        );
        entry.use_count = 10;
        stats.record(entry, 2000);

        assert!((stats.avg_uses() - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_format_knowledge_stats() {
        let mut stats = KnowledgeBaseStats::new();
        stats.record(
            KnowledgeEntry::new(
                "recipe",
                KnowledgeType::Recipe,
                KnowledgeSource::Seed,
                1000,
            ),
            2000,
        );

        let output = format_knowledge_stats(&stats);
        assert!(output.contains("Knowledge Base"));
        assert!(output.contains("Recipe"));
    }

    #[test]
    fn test_format_knowledge_stats_compact() {
        let mut stats = KnowledgeBaseStats::new();
        stats.record(
            KnowledgeEntry::new(
                "recipe",
                KnowledgeType::Recipe,
                KnowledgeSource::Seed,
                1000,
            ),
            2000,
        );

        let output = format_knowledge_stats_compact(&stats);
        assert!(output.contains("Knowledge:"));
        assert!(output.contains("1 entries"));
    }

    #[test]
    fn test_knowledge_insight() {
        let stats = KnowledgeBaseStats::new();
        assert!(knowledge_insight(&stats).is_none());

        let mut stats2 = KnowledgeBaseStats::new();
        stats2.total_entries = 10;
        assert!(knowledge_insight(&stats2).is_some());
    }

    #[test]
    fn test_is_knowledge_stats_query() {
        assert!(is_knowledge_stats_query("show knowledge base"));
        assert!(is_knowledge_stats_query("how many recipes do you know?"));
        assert!(is_knowledge_stats_query("what do you know?"));
        assert!(!is_knowledge_stats_query("how do I install vim?"));
    }

    #[test]
    fn test_knowledge_health() {
        let empty = KnowledgeBaseStats::new();
        assert!(knowledge_health(&empty).contains("Empty"));

        let mut growing = KnowledgeBaseStats::new();
        growing.total_entries = 10;
        assert!(knowledge_health(&growing).contains("Growing"));
    }
}
