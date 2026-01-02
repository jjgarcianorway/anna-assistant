// v0.0.598: Settings Transformer Tests (Phase 174)
// Unit tests for transformer modules

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_transform_type_display() {
        assert_eq!(format!("{}", TransformType::Trim), "trim");
        assert_eq!(format!("{}", TransformType::Upper), "upper");
    }

    #[test]
    fn test_transform_direction_display() {
        assert_eq!(format!("{}", TransformDirection::Input), "input");
        assert_eq!(format!("{}", TransformDirection::Both), "both");
    }

    #[test]
    fn test_transform_def_new() {
        let t = TransformDef::new("t1", TransformType::Trim);
        assert_eq!(t.id, "t1");
        assert!(t.enabled);
    }

    #[test]
    fn test_transform_def_builder() {
        let t = TransformDef::new("t1", TransformType::Replace)
            .name("Replacer")
            .direction(TransformDirection::Input)
            .param("from", "a")
            .param("to", "b")
            .priority(50);
        assert_eq!(t.priority, 50);
        assert_eq!(t.params.len(), 2);
    }

    #[test]
    fn test_transform_result_new() {
        let r = TransformResult::new("hello", "HELLO");
        assert!(r.was_transformed());
        assert!(r.success);
    }

    #[test]
    fn test_transform_result_not_transformed() {
        let r = TransformResult::new("same", "same");
        assert!(!r.was_transformed());
    }

    #[test]
    fn test_pipeline_new() {
        let p = TransformPipeline::new();
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_pipeline_add_remove() {
        let mut p = TransformPipeline::new();
        p.add(TransformDef::new("t1", TransformType::Trim));
        assert_eq!(p.count(), 1);
        p.remove("t1");
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn test_pipeline_enable_disable() {
        let mut p = TransformPipeline::new();
        p.add(TransformDef::new("t1", TransformType::Lower));
        p.disable("t1");
        assert_eq!(p.enabled_count(), 0);
        p.enable("t1");
        assert_eq!(p.enabled_count(), 1);
    }

    #[test]
    fn test_manager_new() {
        let m = TransformerManager::new();
        assert_eq!(m.pipeline_count(), 0);
    }

    #[test]
    fn test_manager_add_pipeline() {
        let mut m = TransformerManager::new();
        m.add_pipeline("test", TransformPipeline::new());
        assert_eq!(m.pipeline_count(), 1);
    }

    #[test]
    fn test_is_transformer_query() {
        assert!(is_transformer_query("transform settings"));
        assert!(!is_transformer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = transformer_fun_fact();
        assert!(fact.contains("transform"));
    }
}
