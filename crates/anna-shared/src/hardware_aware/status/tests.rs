//! Tests for status module (v0.0.434).

#[cfg(test)]
mod tests {
    use super::super::super::helper_config::HelperConfig;
    use super::super::super::helper_entry::HelperCatalog;
    use super::super::super::helper_manager::HelperManager;
    use super::super::super::model_config::ModelConfig;
    use super::super::super::model_health::ModelHealth;
    use super::super::super::model_plan::ModelPlan;
    use super::super::super::profile::{
        CapabilityTier, CpuInfo, GpuInfo, HardwareProfile, OsInfo, StorageInfo,
    };
    use super::super::core::HardwareStatus;
    use super::super::helper_usage::HelperUsageStats;
    use super::super::llm::LlmSection;
    use super::super::model_usage::ModelUsageStats;
    use super::super::system_profile::SystemProfileSection;

    fn mock_profile() -> HardwareProfile {
        HardwareProfile {
            profile_version: 1,
            last_profiled_at: "1234567890".to_string(),
            cpu: CpuInfo {
                model_name: "Intel Core i9".to_string(),
                core_count: 24,
                thread_count: 32,
                avx2_supported: true,
            },
            ram_total_gb: 32.0,
            ram_free_gb: 16.0,
            gpu: GpuInfo::default(),
            storage: StorageInfo::default(),
            os: OsInfo::default(),
            tier: CapabilityTier::Medium,
        }
    }

    fn mock_plan() -> ModelPlan {
        ModelPlan {
            catalog_version: 1,
            profile_version: 1,
            tier: CapabilityTier::Medium,
            translator_model: "qwen3:0.6b".to_string(),
            junior_model: "qwen3:4b".to_string(),
            senior_model: "qwen2.5:7b-instruct".to_string(),
            prefer_small: false,
            estimated_disk_gb: 10,
            created_at: "0".to_string(),
            rationale: "Test".to_string(),
        }
    }

    #[test]
    fn test_system_profile_section() {
        let profile = mock_profile();
        let section = SystemProfileSection::from_profile(&profile);

        assert_eq!(section.ram_total_gb, 32.0);
        assert_eq!(section.cpu_cores, 24);
        assert!(section.avx2);
        assert_eq!(section.tier, "Medium");
    }

    #[test]
    fn test_llm_section_build() {
        let plan = mock_plan();
        let health = ModelHealth::new();
        let config = ModelConfig::new();

        let section = LlmSection::build(&plan, &health, &config);

        assert_eq!(section.provider, "ollama");
        assert_eq!(section.models.len(), 3);
        // All missing since health is empty
        assert!(section.models.iter().all(|m| m.status == "MISSING"));
    }

    #[test]
    fn test_model_usage_stats() {
        let mut stats = ModelUsageStats::default();

        stats.record_call("qwen3:4b", 500, true);
        stats.record_call("qwen3:4b", 700, true);
        stats.record_error("qwen3:4b", "ParseError");

        let usage = stats.models.get("qwen3:4b").unwrap();
        assert_eq!(usage.call_count, 2);
        assert_eq!(usage.avg_duration_ms(), 600);
        assert_eq!(usage.error_count, 1);

        assert!(stats.last_error.is_some());
    }

    #[test]
    fn test_helper_usage_stats() {
        let mut stats = HelperUsageStats::default();

        stats.record_use("lm_sensors");
        stats.record_use("lm_sensors");

        let usage = stats.helpers.get("lm_sensors").unwrap();
        assert_eq!(usage.use_count, 2);
    }

    #[test]
    fn test_hardware_status_format() {
        let profile = mock_profile();
        let plan = mock_plan();
        let health = ModelHealth::new();
        let model_config = ModelConfig::new();
        let helper_manager = HelperManager::new();
        let helper_catalog = HelperCatalog::default_catalog();
        let helper_config = HelperConfig::new();

        let status = HardwareStatus::build(
            &profile,
            &plan,
            &health,
            &model_config,
            &helper_manager,
            &helper_catalog,
            &helper_config,
        );

        let formatted = status.format();
        assert!(formatted.contains("[system_profile]"));
        assert!(formatted.contains("[llm]"));
        assert!(formatted.contains("[helpers]"));
        assert!(formatted.contains("32.0 GB"));
    }
}
