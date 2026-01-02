//! Core hardware status (v0.0.434).
//!
//! Honest reflection of hardware, models, and helpers in annactl status/stats.

use super::super::helper_config::HelperConfig;
use super::super::helper_entry::HelperCatalog;
use super::super::helper_manager::HelperManager;
use super::super::model_config::ModelConfig;
use super::super::model_health::ModelHealth;
use super::super::model_plan::ModelPlan;
use super::super::profile::HardwareProfile;
use super::helpers::HelperStatusSection;
use super::llm::LlmSection;
use super::system_profile::SystemProfileSection;
use serde::{Deserialize, Serialize};

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
        lines.push(format!(
            "  ram_total            {:.1} GB",
            self.profile.ram_total_gb
        ));
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
        lines.push(format!(
            "  last_profiled        {}",
            self.profile.last_profiled
        ));
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
