//! Tests for knowledge base stats module

#[cfg(test)]
mod tests {
    use crate::knowledge_base_stats::*;

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
