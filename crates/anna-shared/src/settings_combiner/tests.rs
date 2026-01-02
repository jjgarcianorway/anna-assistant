// v0.0.690: Settings Combiner Tests (Phase 266)
// Test cases for settings combiner functionality

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::settings_combiner::{
        CombineStrategy, CombineDepth, CombinerConfig, CombineConflict, CombineResult,
        CombinerStats, SettingsCombiner, CombinerRegistry, is_combiner_query, combiner_fun_fact,
    };

    #[test]
    fn test_merge_strategy_display() {
        assert_eq!(format!("{}", CombineStrategy::LeftWins), "left_wins");
        assert_eq!(format!("{}", CombineStrategy::RightWins), "right_wins");
    }

    #[test]
    fn test_merge_depth_display() {
        assert_eq!(format!("{}", CombineDepth::Shallow), "shallow");
        assert_eq!(format!("{}", CombineDepth::Deep), "deep");
    }

    #[test]
    fn test_config_new() {
        let c = CombinerConfig::new(CombineStrategy::LeftWins);
        assert_eq!(c.strategy, CombineStrategy::LeftWins);
    }

    #[test]
    fn test_config_builder() {
        let c = CombinerConfig::new(CombineStrategy::RightWins)
            .depth(CombineDepth::Deep)
            .preserve_empty(true);
        assert_eq!(c.depth, CombineDepth::Deep);
        assert!(c.preserve_empty);
    }

    #[test]
    fn test_conflict_new() {
        let c = CombineConflict::new("key", "left", "right");
        assert!(!c.is_resolved());
    }

    #[test]
    fn test_conflict_resolve() {
        let c = CombineConflict::new("key", "left", "right").resolve("final");
        assert!(c.is_resolved());
    }

    #[test]
    fn test_result_new() {
        let mut merged = HashMap::new();
        merged.insert("a".to_string(), "1".to_string());
        let r = CombineResult::new(merged, Vec::new(), 1, 0);
        assert_eq!(r.total_keys(), 1);
    }

    #[test]
    fn test_result_has_conflicts() {
        let conflicts = vec![CombineConflict::new("k", "l", "r")];
        let r = CombineResult::new(HashMap::new(), conflicts, 0, 0);
        assert!(r.has_conflicts());
    }

    #[test]
    fn test_stats_record() {
        let mut s = CombinerStats::default();
        let r = CombineResult::new(HashMap::new(), Vec::new(), 0, 0);
        s.record(&r, CombineStrategy::LeftWins);
        assert_eq!(s.total_merges, 1);
    }

    #[test]
    fn test_combiner_new() {
        let m = SettingsCombiner::new(CombinerConfig::default());
        assert_eq!(m.stats().total_merges, 0);
    }

    #[test]
    fn test_combiner_merge_no_conflict() {
        let mut m = SettingsCombiner::new(CombinerConfig::default());
        let mut left = HashMap::new();
        left.insert("a".to_string(), "1".to_string());
        let mut right = HashMap::new();
        right.insert("b".to_string(), "2".to_string());

        let result = m.merge(&left, &right);
        assert_eq!(result.total_keys(), 2);
        assert!(!result.has_conflicts());
    }

    #[test]
    fn test_combiner_merge_right_wins() {
        let mut m = SettingsCombiner::new(CombinerConfig::new(CombineStrategy::RightWins));
        let mut left = HashMap::new();
        left.insert("key".to_string(), "left".to_string());
        let mut right = HashMap::new();
        right.insert("key".to_string(), "right".to_string());

        let result = m.merge(&left, &right);
        assert_eq!(result.get("key"), Some(&"right".to_string()));
    }

    #[test]
    fn test_combiner_merge_left_wins() {
        let mut m = SettingsCombiner::new(CombinerConfig::new(CombineStrategy::LeftWins));
        let mut left = HashMap::new();
        left.insert("key".to_string(), "left".to_string());
        let mut right = HashMap::new();
        right.insert("key".to_string(), "right".to_string());

        let result = m.merge(&left, &right);
        assert_eq!(result.get("key"), Some(&"left".to_string()));
    }

    #[test]
    fn test_combiner_merge_keep_both() {
        let mut m = SettingsCombiner::new(CombinerConfig::new(CombineStrategy::KeepBoth));
        let mut left = HashMap::new();
        left.insert("key".to_string(), "left".to_string());
        let mut right = HashMap::new();
        right.insert("key".to_string(), "right".to_string());

        let result = m.merge(&left, &right);
        assert!(result.get("key").is_some());
        assert!(result.get("key_conflict").is_some());
    }

    #[test]
    fn test_combiner_merge_all() {
        let mut m = SettingsCombiner::new(CombinerConfig::default());
        let mut c1 = HashMap::new();
        c1.insert("a".to_string(), "1".to_string());
        let mut c2 = HashMap::new();
        c2.insert("b".to_string(), "2".to_string());

        let result = m.merge_all(&[c1, c2]);
        assert_eq!(result.total_keys(), 2);
    }

    #[test]
    fn test_registry_new() {
        let r = CombinerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CombinerRegistry::new();
        r.register("m1", SettingsCombiner::new(CombinerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_combiner_query() {
        assert!(is_combiner_query("combine settings"));
        assert!(!is_combiner_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = combiner_fun_fact();
        assert!(fact.contains("combiner"));
    }
}
