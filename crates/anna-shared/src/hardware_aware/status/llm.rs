//! LLM section for status display (v0.0.434).

use super::super::model_config::ModelConfig;
use super::super::model_health::{InstalledBy, ModelHealth};
use super::super::model_plan::ModelPlan;
use serde::{Deserialize, Serialize};

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
                let installed_by =
                    health
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
        let all_ok = models
            .iter()
            .all(|m| m.status == "OK" || m.status == "UNVERIFIED");
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
