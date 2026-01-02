// v0.0.541: Tips System Tests (Phase 117)

#[cfg(test)]
mod tests {
    use crate::tips_system::manager::TipsSystem;
    use crate::tips_system::types::{Tip, TipCategory, TipPriority};
    use crate::tips_system::utils::{is_tips_query, tips_fun_fact};

    #[test]
    fn test_tip_category_default() {
        let cat = TipCategory::default();
        assert_eq!(cat, TipCategory::Feature);
    }

    #[test]
    fn test_tip_priority_default() {
        let priority = TipPriority::default();
        assert_eq!(priority, TipPriority::Normal);
    }

    #[test]
    fn test_tips_system_creation() {
        let system = TipsSystem::new();
        assert!(system.total() > 0);
    }

    #[test]
    fn test_add_tip() {
        let mut system = TipsSystem::new();
        let initial = system.total();
        system.add_tip(Tip::new("test", "Test Tip", "Test content"));
        assert_eq!(system.total(), initial + 1);
    }

    #[test]
    fn test_next_tip() {
        let mut system = TipsSystem::new();
        let tip = system.next_tip();
        assert!(tip.is_some());
    }

    #[test]
    fn test_mark_shown() {
        let mut tip = Tip::new("test", "Test", "Content");
        assert_eq!(tip.shown_count, 0);
        tip.mark_shown();
        assert_eq!(tip.shown_count, 1);
        assert!(tip.last_shown.is_some());
    }

    #[test]
    fn test_category_stats() {
        let system = TipsSystem::new();
        let stats = system.category_stats();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_remaining_today() {
        let system = TipsSystem::new();
        assert_eq!(system.remaining_today(), system.max_daily_tips);
    }

    #[test]
    fn test_is_tips_query() {
        assert!(is_tips_query("Show me a tip"));
        assert!(is_tips_query("Any hints?"));
        assert!(!is_tips_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = tips_fun_fact();
        assert!(fact.contains("tip") || fact.contains("configuration"));
    }
}
