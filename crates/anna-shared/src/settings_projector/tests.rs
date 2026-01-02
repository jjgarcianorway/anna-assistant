// v0.0.672: Settings Projector Tests (Phase 248)
// Unit tests for settings projector

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_projection_type_display() {
        assert_eq!(format!("{}", ProjectionType::Include), "include");
        assert_eq!(format!("{}", ProjectionType::Exclude), "exclude");
    }

    #[test]
    fn test_mapping_include() {
        let m = FieldMapping::include("field");
        assert_eq!(m.source, "field");
        assert!(m.target.is_none());
    }

    #[test]
    fn test_mapping_rename() {
        let m = FieldMapping::rename("old", "new");
        assert_eq!(m.source, "old");
        assert_eq!(m.target, Some("new".to_string()));
    }

    #[test]
    fn test_config_new() {
        let c = ProjectorConfig::new(ProjectionType::Include);
        assert!(c.preserve_order);
    }

    #[test]
    fn test_config_builder() {
        let c = ProjectorConfig::new(ProjectionType::Exclude)
            .include_unmatched(true);
        assert!(c.include_unmatched);
    }

    #[test]
    fn test_result_success() {
        let r = ProjectionResult::success(HashMap::new());
        assert!(r.success);
    }

    #[test]
    fn test_result_with_counts() {
        let r = ProjectionResult::success(HashMap::new())
            .with_counts(5, 3, 2);
        assert_eq!(r.total_changes(), 10);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ProjectorStats::default();
        let mut settings = HashMap::new();
        settings.insert("k".to_string(), "v".to_string());
        let r = ProjectionResult::success(settings);
        s.record(&r, ProjectionType::Include);
        assert_eq!(s.total_projections, 1);
    }

    #[test]
    fn test_projector_new() {
        let p = SettingsProjector::new(ProjectorConfig::default());
        assert_eq!(p.mapping_count(), 0);
    }

    #[test]
    fn test_projector_include() {
        let mut p = SettingsProjector::new(ProjectorConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());
        settings.insert("c".to_string(), "3".to_string());

        let result = p.project_include(&settings, &["a", "b"]);
        assert_eq!(result.settings.len(), 2);
        assert!(result.settings.contains_key("a"));
    }

    #[test]
    fn test_projector_exclude() {
        let mut p = SettingsProjector::new(ProjectorConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());
        settings.insert("c".to_string(), "3".to_string());

        let result = p.project_exclude(&settings, &["c"]);
        assert_eq!(result.settings.len(), 2);
        assert!(!result.settings.contains_key("c"));
    }

    #[test]
    fn test_projector_with_mappings() {
        let mut p = SettingsProjector::new(ProjectorConfig::default());
        p.add_mapping(FieldMapping::rename("old_name", "new_name"));

        let mut settings = HashMap::new();
        settings.insert("old_name".to_string(), "value".to_string());

        let result = p.project(&settings);
        assert!(result.settings.contains_key("new_name"));
    }

    #[test]
    fn test_registry_new() {
        let r = ProjectorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ProjectorRegistry::new();
        r.register("p1", SettingsProjector::new(ProjectorConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_projector_query() {
        assert!(is_projector_query("project settings"));
        assert!(!is_projector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = projector_fun_fact();
        assert!(fact.contains("projector"));
    }
}
