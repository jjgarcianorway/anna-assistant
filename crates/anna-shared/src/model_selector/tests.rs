//! Model selector tests (v0.0.223).
//! v0.0.393: Updated tests for 3B+ translator requirement.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::model_selector::{
        model_catalog, model_matches, select_model, ModelFamily, ModelRole, ModelSelectorConfig,
    };

    #[test]
    fn test_model_catalog_not_empty() {
        let catalog = model_catalog();
        assert!(!catalog.is_empty());
    }

    #[test]
    fn test_select_model_prefers_qwen3_vl() {
        let available = vec![
            "qwen3-vl:4b".to_string(),
            "qwen2.5:3b".to_string(),
            "llama3.2:3b".to_string(),
        ];
        let config = ModelSelectorConfig::default();
        let benchmarks = HashMap::new();

        let selection = select_model(ModelRole::Specialist, &available, &config, &benchmarks);
        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.family, ModelFamily::Qwen3VL);
        assert!(sel.is_preferred);
        assert!(!sel.is_fallback);
    }

    #[test]
    fn test_select_model_fallback() {
        let available = vec!["qwen2.5:3b".to_string(), "llama3.2:3b".to_string()];
        let config = ModelSelectorConfig::default();
        let benchmarks = HashMap::new();

        let selection = select_model(ModelRole::Specialist, &available, &config, &benchmarks);
        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert!(sel.is_fallback);
    }

    #[test]
    fn test_model_matches() {
        assert!(model_matches("qwen3-vl:4b", "qwen3-vl:4b"));
        assert!(model_matches("qwen3-vl:4b", "qwen3vl:4b"));
        assert!(model_matches("qwen3-vl:4b", "qwen3-vl:4b-q4_k_m"));
        assert!(!model_matches("qwen3-vl:4b", "qwen2.5:3b"));
    }

    #[test]
    fn test_select_translator_requires_3b_plus() {
        // v0.0.393: Translator requires 3B+ for reliable JSON output
        // Small models (2b, 1b) produce malformed JSON causing routing failures
        let available = vec![
            "qwen3-vl:4b".to_string(),
            "qwen3-vl:2b".to_string(),
            "qwen3-vl:1b".to_string(),
        ];
        let config = ModelSelectorConfig::default();
        let benchmarks = HashMap::new();

        let selection = select_model(ModelRole::Translator, &available, &config, &benchmarks);
        assert!(selection.is_some());
        let sel = selection.unwrap();
        // Should select 4b (smallest valid translator - 3B+ required)
        assert!(sel.model.contains("4b"));
    }

    #[test]
    fn test_small_models_not_valid_translators() {
        // v0.0.393: Models < 3B should NOT be selected as translators
        let available = vec!["qwen3-vl:2b".to_string(), "qwen3-vl:1b".to_string()];
        let config = ModelSelectorConfig::default();
        let benchmarks = HashMap::new();

        let selection = select_model(ModelRole::Translator, &available, &config, &benchmarks);
        // No valid translators available - should return None
        assert!(selection.is_none());
    }
}
