//! Unit tests for learned recipes.

#[cfg(test)]
mod tests {
    use super::super::storage::RecipeStore;
    use super::super::types::{
        AnswerTemplate, CompareOp, LearnedRecipe, RecipeStats,
    };
    use crate::canonical_intents::CanonicalIntent;
    use std::collections::HashMap;

    #[test]
    fn test_compare_op() {
        assert!(CompareOp::Gt.eval(10.0, 5.0));
        assert!(!CompareOp::Lt.eval(10.0, 5.0));
        assert!(CompareOp::Ge.eval(10.0, 10.0));
    }

    #[test]
    fn test_template_render() {
        let template = AnswerTemplate {
            summary: "RAM: {used} / {total} GiB ({percent}% used)".to_string(),
            details: vec![],
            evidence: vec!["memory_info".to_string()],
        };

        let mut values = HashMap::new();
        values.insert("used".to_string(), "8".to_string());
        values.insert("total".to_string(), "16".to_string());
        values.insert("percent".to_string(), "50".to_string());

        let rendered = template.render(&values);
        assert_eq!(rendered.summary, "RAM: 8 / 16 GiB (50% used)");
    }

    #[test]
    fn test_recipe_stats() {
        let mut stats = RecipeStats::default();
        stats.record_success(0.9);
        stats.record_success(0.85);
        stats.record_failure();

        assert_eq!(stats.uses, 3);
        assert_eq!(stats.successes, 2);
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_recipe_store() {
        let mut store = RecipeStore::default();

        let recipe = LearnedRecipe {
            id: "test-recipe".to_string(),
            name: "Test Recipe".to_string(),
            version: 1,
            intent: CanonicalIntent::CheckDiskUsage,
            domain: "storage".to_string(),
            required_probes: vec!["disk_usage".to_string()],
            optional_probes: vec![],
            steps: vec![],
            answer_ok: AnswerTemplate {
                summary: "Disk is OK".to_string(),
                details: vec![],
                evidence: vec![],
            },
            answer_critical: None,
            answer_partial: None,
            knowledge_topics: vec![],
            source_tickets: vec![],
            stats: RecipeStats::default(),
            created_at: 0,
            last_used_at: 0,
            deprecated: false,
        };

        store.upsert(recipe);
        assert!(store
            .find_for_intent(CanonicalIntent::CheckDiskUsage)
            .is_some());
    }
}
