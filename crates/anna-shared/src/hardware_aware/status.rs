//! Status and stats display (v0.0.434).
//!
//! Honest reflection of hardware, models, and helpers in annactl status/stats.

use super::helpers::{HelperCatalog, HelperManager, HelperInstalledBy};
use super::helper_config::HelperConfig;
use super::model_config::ModelConfig;
use super::model_health::{ModelHealth, ModelStatus, InstalledBy};
use super::model_plan::ModelPlan;
use super::profile::HardwareProfile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete hardware-aware status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareStatus {
    /// System profile section.
    pub profile: SystemProfileSection,
    /// LLM section.
    pub llm: LlmSection,
    /// Helpers section.
    pub helpers: HelperStatusSection,
}

impl HardwareStatus {
    /// Build from components.
    pub fn build(
        profile: &HardwareProfile,
        plan: &ModelPlan,
        health: &ModelHealth,
        model_config: &ModelConfig,
        helper_manager: &HelperManager,
        helper_catalog: &HelperCatalog,
        helper_config: &HelperConfig,
    ) -> Self {
        Self {
            profile: SystemProfileSection::from_profile(profile),
            llm: LlmSection::build(plan, health, model_config),
            helpers: HelperStatusSection::build(helper_manager, helper_catalog, helper_config),
        }
    }

    /// Format for annactl status display.
    pub fn format(&self) -> String {
        let mut lines = Vec::new();

        // System profile section
        lines.push("[system_profile]".to_string());
        lines.push(format!("  ram_total            {:.1} GB", self.profile.ram_total_gb));
        lines.push(format!(
            "  cpu                  {}, {} cores{}",
            self.profile.cpu_model,
            self.profile.cpu_cores,
            if self.profile.avx2 { ", AVX2" } else { "" }
        ));
        if let Some(gpu) = &self.profile.gpu {
            lines.push(format!("  gpu                  {}", gpu));
        }
        lines.push(format!("  tier                 {}", self.profile.tier));
        lines.push(format!("  last_profiled        {}", self.profile.last_profiled));
        lines.push(String::new());

        // LLM section
        lines.push("[llm]".to_string());
        lines.push(format!("  provider             {}", self.llm.provider));
        lines.push(format!("  state                {}", self.llm.state));
        lines.push(format!(
            "  model_plan_catalog   v{}",
            self.llm.catalog_version
        ));
        lines.push(format!(
            "  model_plan_profile   v{} ({})",
            self.llm.profile_version, self.llm.tier
        ));
        lines.push("  models".to_string());
        for model in &self.llm.models {
            lines.push(format!(
                "    {:<18} {:<20} [{}{}]",
                model.role,
                model.name,
                model.status,
                model
                    .installed_by
                    .as_ref()
                    .map(|s| format!(", installed_by={}", s))
                    .unwrap_or_default()
            ));
        }
        lines.push(String::new());

        // Helpers section
        lines.push("[helpers]".to_string());
        lines.push(format!(
            "  installed_by_anna    {}",
            self.helpers.anna_installed.len()
        ));
        for helper in &self.helpers.anna_installed {
            lines.push(format!("    {}   ({})", helper.id, helper.purpose));
        }
        lines.push(format!(
            "  installed_by_user    {}",
            self.helpers.user_installed.len()
        ));
        for helper in &self.helpers.user_installed {
            lines.push(format!("    {}", helper.id));
        }
        lines.push(format!("  helper_policy        {}", self.helpers.policy));

        lines.join("\n")
    }
}

/// System profile section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemProfileSection {
    /// Total RAM in GB.
    pub ram_total_gb: f32,
    /// CPU model name.
    pub cpu_model: String,
    /// CPU core count.
    pub cpu_cores: u32,
    /// AVX2 supported.
    pub avx2: bool,
    /// GPU description (if any).
    pub gpu: Option<String>,
    /// Capability tier.
    pub tier: String,
    /// Last profiled timestamp.
    pub last_profiled: String,
}

impl SystemProfileSection {
    /// Build from hardware profile.
    pub fn from_profile(profile: &HardwareProfile) -> Self {
        let gpu = if profile.gpu.discrete {
            Some(format!(
                "{} {}",
                profile.gpu.vendor.label(),
                profile
                    .gpu
                    .model_name
                    .clone()
                    .unwrap_or_else(|| "GPU".to_string())
            ))
        } else if profile.gpu.vendor != super::profile::GpuVendor::None {
            Some(format!("{} (integrated)", profile.gpu.vendor.label()))
        } else {
            None
        };

        Self {
            ram_total_gb: profile.ram_total_gb,
            cpu_model: profile.cpu.model_name.clone(),
            cpu_cores: profile.cpu.core_count,
            avx2: profile.cpu.avx2_supported,
            gpu,
            tier: profile.tier.label().to_string(),
            last_profiled: profile.last_profiled_at.clone(),
        }
    }
}

/// LLM section for status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSection {
    /// Provider name.
    pub provider: String,
    /// Overall state.
    pub state: String,
    /// Catalog version.
    pub catalog_version: u32,
    /// Profile version.
    pub profile_version: u32,
    /// Tier.
    pub tier: String,
    /// Model entries.
    pub models: Vec<ModelStatusEntry>,
    /// Config summary.
    pub config: String,
}

impl LlmSection {
    /// Build from plan and health.
    pub fn build(plan: &ModelPlan, health: &ModelHealth, config: &ModelConfig) -> Self {
        let mut models = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Add each role's model (deduplicated)
        for (role, model_name) in [
            ("translator", &plan.translator_model),
            ("junior", &plan.junior_model),
            ("senior", &plan.senior_model),
        ] {
            if !seen.contains(model_name) {
                seen.insert(model_name.clone());
                let status = health.status(model_name);
                let installed_by = health
                    .models
                    .get(model_name)
                    .map(|r| r.installed_by)
                    .map(|ib| match ib {
                        InstalledBy::Anna => "anna",
                        InstalledBy::User => "user",
                        InstalledBy::Unknown => "unknown",
                    });

                models.push(ModelStatusEntry {
                    role: role.to_string(),
                    name: model_name.clone(),
                    status: status.label().to_string(),
                    installed_by: installed_by.map(|s| s.to_string()),
                });
            } else {
                // Same model used for multiple roles
                models.push(ModelStatusEntry {
                    role: role.to_string(),
                    name: format!("{} (shared)", model_name),
                    status: health.status(model_name).label().to_string(),
                    installed_by: None,
                });
            }
        }

        // Determine overall state
        let all_ok = models.iter().all(|m| m.status == "OK" || m.status == "UNVERIFIED");
        let any_missing = models.iter().any(|m| m.status == "MISSING");
        let state = if all_ok {
            "READY"
        } else if any_missing {
            "DEGRADED"
        } else {
            "ERROR"
        };

        Self {
            provider: "ollama".to_string(),
            state: state.to_string(),
            catalog_version: plan.catalog_version,
            profile_version: plan.profile_version,
            tier: plan.tier.label().to_string(),
            models,
            config: config.format_summary(),
        }
    }
}

/// Single model status entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatusEntry {
    /// Role (translator, junior, senior).
    pub role: String,
    /// Model name.
    pub name: String,
    /// Status (OK, MISSING, BROKEN, etc.).
    pub status: String,
    /// Who installed it.
    pub installed_by: Option<String>,
}

/// Helper status section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperStatusSection {
    /// Helpers installed by Anna.
    pub anna_installed: Vec<HelperStatusEntry>,
    /// Helpers installed by user.
    pub user_installed: Vec<HelperStatusEntry>,
    /// Policy summary.
    pub policy: String,
}

impl HelperStatusSection {
    /// Build from manager and config.
    pub fn build(
        manager: &HelperManager,
        catalog: &HelperCatalog,
        config: &HelperConfig,
    ) -> Self {
        let mut anna_installed = Vec::new();
        let mut user_installed = Vec::new();

        for (id, state) in &manager.helpers {
            let purpose = catalog
                .get(id)
                .map(|h| h.purpose.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let entry = HelperStatusEntry {
                id: id.clone(),
                purpose,
                use_count: state.use_count,
                last_used: state.last_used.clone(),
            };

            match state.installed_by {
                HelperInstalledBy::Anna => anna_installed.push(entry),
                HelperInstalledBy::User => user_installed.push(entry),
            }
        }

        Self {
            anna_installed,
            user_installed,
            policy: config.format_summary(),
        }
    }
}

/// Single helper status entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperStatusEntry {
    /// Helper ID.
    pub id: String,
    /// Purpose.
    pub purpose: String,
    /// Usage count.
    pub use_count: u64,
    /// Last used timestamp.
    pub last_used: Option<String>,
}

/// Model usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelUsageStats {
    /// Per-model usage counts.
    pub models: HashMap<String, ModelUsage>,
    /// Last error.
    pub last_error: Option<ModelError>,
}

impl ModelUsageStats {
    /// Record a model call.
    pub fn record_call(&mut self, model: &str, duration_ms: u64, success: bool) {
        let usage = self.models.entry(model.to_string()).or_default();
        usage.call_count += 1;
        usage.total_duration_ms += duration_ms;

        if !success {
            usage.error_count += 1;
        }
    }

    /// Record an error.
    pub fn record_error(&mut self, model: &str, error_type: &str) {
        self.last_error = Some(ModelError {
            model: model.to_string(),
            error_type: error_type.to_string(),
            timestamp: timestamp_now(),
        });

        if let Some(usage) = self.models.get_mut(model) {
            usage.error_count += 1;
        }
    }

    /// Format for stats display.
    pub fn format(&self) -> String {
        let mut lines = Vec::new();
        lines.push("[llm_usage]".to_string());

        for (role, model_key) in [("translator", "translator"), ("junior", "junior"), ("senior", "senior")] {
            if let Some((model, usage)) = self.models.iter().find(|(m, _)| m.contains(model_key)) {
                let avg_ms = if usage.call_count > 0 {
                    usage.total_duration_ms / usage.call_count
                } else {
                    0
                };
                lines.push(format!(
                    "  {}_calls     {} (avg {:.1}s)",
                    role,
                    usage.call_count,
                    avg_ms as f64 / 1000.0
                ));
            }
        }

        if let Some(error) = &self.last_error {
            lines.push(format!(
                "  last_model_error     {} ({}, {})",
                error.timestamp, error.error_type, error.model
            ));
        }

        lines.join("\n")
    }
}

/// Usage stats for a single model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Number of calls.
    pub call_count: u64,
    /// Total duration in ms.
    pub total_duration_ms: u64,
    /// Error count.
    pub error_count: u64,
}

impl ModelUsage {
    /// Average duration in ms.
    pub fn avg_duration_ms(&self) -> u64 {
        if self.call_count > 0 {
            self.total_duration_ms / self.call_count
        } else {
            0
        }
    }
}

/// Model error record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelError {
    /// Model name.
    pub model: String,
    /// Error type.
    pub error_type: String,
    /// Timestamp.
    pub timestamp: String,
}

/// Helper usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelperUsageStats {
    /// Per-helper usage.
    pub helpers: HashMap<String, HelperUsage>,
}

impl HelperUsageStats {
    /// Record helper usage.
    pub fn record_use(&mut self, helper_id: &str) {
        let usage = self.helpers.entry(helper_id.to_string()).or_default();
        usage.use_count += 1;
        usage.last_used = Some(timestamp_now());
    }

    /// Format for stats display.
    pub fn format(&self, catalog: &HelperCatalog) -> String {
        let mut lines = Vec::new();
        lines.push("[helper_usage]".to_string());

        for helper in &catalog.helpers {
            if let Some(usage) = self.helpers.get(&helper.id) {
                lines.push(format!(
                    "  {:<18} used {} times, last used {}",
                    helper.id,
                    usage.use_count,
                    usage.last_used.as_deref().unwrap_or("never")
                ));
            } else {
                lines.push(format!("  {:<18} not installed", helper.id));
            }
        }

        lines.join("\n")
    }
}

/// Usage stats for a single helper.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelperUsage {
    /// Number of uses.
    pub use_count: u64,
    /// Last used timestamp.
    pub last_used: Option<String>,
}

/// Get current timestamp.
fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::profile::{CapabilityTier, CpuInfo, GpuInfo, OsInfo, StorageInfo};

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
