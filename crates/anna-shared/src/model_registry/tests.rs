//! Tests for model_registry module (v0.0.201).

#[cfg(test)]
mod tests {
    use crate::model_registry::{
        recommended_model_for_tier, HardwareTier, ModelRegistry, ModelSpec, ModelState,
    };
    use crate::specialists::SpecialistRole;
    use crate::teams::Team;

    #[test]
    fn test_model_spec_new() {
        let spec = ModelSpec::new("llama3.2:3b")
            .with_size(2.0)
            .with_quant("Q4_K_M");
        assert_eq!(spec.name, "llama3.2:3b");
        assert_eq!(spec.size_hint_gb, Some(2.0));
        assert_eq!(spec.quant, Some("Q4_K_M".to_string()));
    }

    #[test]
    fn test_hardware_tier_from_specs() {
        assert_eq!(HardwareTier::from_specs(2.0, 2, false), HardwareTier::Low);
        assert_eq!(
            HardwareTier::from_specs(4.0, 4, false),
            HardwareTier::Medium
        );
        assert_eq!(HardwareTier::from_specs(8.0, 8, false), HardwareTier::High);
        assert_eq!(
            HardwareTier::from_specs(16.0, 8, true),
            HardwareTier::VeryHigh
        );
    }

    #[test]
    fn test_recommended_model_for_tier() {
        let low = recommended_model_for_tier(HardwareTier::Low);
        assert!(low.name.contains("0.5b"));

        let high = recommended_model_for_tier(HardwareTier::High);
        assert!(high.name.contains("3b"));
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = ModelRegistry::with_defaults(HardwareTier::Medium);

        // 8 teams × 3 roles = 24 bindings
        assert_eq!(registry.bindings.len(), 24);
        assert_eq!(registry.hardware_tier, Some(HardwareTier::Medium));

        // All bindings should use the same model
        let model_name = &registry.bindings[0].model.name;
        assert!(registry
            .bindings
            .iter()
            .all(|b| &b.model.name == model_name));
    }

    #[test]
    fn test_registry_get_binding() {
        let registry = ModelRegistry::with_defaults(HardwareTier::High);
        let binding = registry
            .get_binding(Team::Storage, SpecialistRole::Junior)
            .unwrap();
        assert_eq!(binding.team, Team::Storage);
        assert_eq!(binding.role, SpecialistRole::Junior);
    }

    #[test]
    fn test_registry_model_presence() {
        let mut registry = ModelRegistry::with_defaults(HardwareTier::Medium);

        // Initially no states
        assert!(registry.missing_models().len() > 0);
        assert!(!registry.all_models_present());

        // Add state for the model
        let model_name = registry.bindings[0].model.name.clone();
        registry.update_state(
            &model_name,
            ModelState {
                present: true,
                digest: None,
                last_seen_ts: None,
                size_bytes: None,
            },
        );

        assert!(registry.is_model_present(&model_name));
        assert!(registry.all_models_present());
    }
}
