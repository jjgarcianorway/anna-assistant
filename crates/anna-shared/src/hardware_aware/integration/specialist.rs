//! Specialist helper integration (v0.0.434).
//!
//! Connects hardware-aware system to specialist responses.

use super::super::catalog::ModelRole;
use super::super::helper_entry::HelperCatalog;
use super::super::model_health::{ModelHealth, ModelStatus};
use super::super::model_plan::ModelPlan;
use serde::{Deserialize, Serialize};

/// Specialist helper integration.
#[derive(Debug, Clone)]
pub struct SpecialistHelper {
    catalog: HelperCatalog,
}

impl SpecialistHelper {
    /// Create with default catalog.
    pub fn new() -> Self {
        Self {
            catalog: HelperCatalog::default_catalog(),
        }
    }

    /// Suggest a helper installation step for a specialist result.
    pub fn suggest_helper_step(
        &self,
        helper_id: &str,
        reason: &str,
        distro: &str,
    ) -> Option<HelperSuggestion> {
        let helper = self.catalog.get(helper_id)?;
        let packages = helper.packages_for_distro(distro);

        if packages.is_empty() {
            return None;
        }

        let package_manager = if distro.to_lowercase().contains("arch") {
            "pacman -S"
        } else if distro.to_lowercase().contains("debian")
            || distro.to_lowercase().contains("ubuntu")
        {
            "apt install"
        } else if distro.to_lowercase().contains("fedora") {
            "dnf install"
        } else {
            "package-manager install"
        };

        Some(HelperSuggestion {
            helper_id: helper_id.to_string(),
            helper_name: helper.name.clone(),
            purpose: helper.purpose.clone(),
            reason: reason.to_string(),
            install_command: format!("sudo {} {}", package_manager, packages.join(" ")),
            packages: packages.to_vec(),
        })
    }

    /// Check if a model is available for a role.
    pub fn check_model_availability(
        &self,
        role: ModelRole,
        plan: &ModelPlan,
        health: &ModelHealth,
    ) -> ModelAvailability {
        let model_name = match role {
            ModelRole::Translator => &plan.translator_model,
            ModelRole::Junior => &plan.junior_model,
            ModelRole::Senior => &plan.senior_model,
        };

        let status = health.status(model_name);

        match status {
            ModelStatus::Ok | ModelStatus::Unverified => ModelAvailability::Available {
                model: model_name.clone(),
            },
            ModelStatus::Missing => ModelAvailability::Missing {
                model: model_name.clone(),
                fallback: self.find_fallback(role, plan, health),
            },
            ModelStatus::Broken => ModelAvailability::Broken {
                model: model_name.clone(),
                error: health
                    .models
                    .get(model_name)
                    .and_then(|r| r.last_error.clone()),
                fallback: self.find_fallback(role, plan, health),
            },
            ModelStatus::Installing => ModelAvailability::Installing {
                model: model_name.clone(),
            },
        }
    }

    /// Find a fallback model for a role.
    fn find_fallback(
        &self,
        role: ModelRole,
        plan: &ModelPlan,
        health: &ModelHealth,
    ) -> Option<String> {
        // For senior, try junior; for junior, try translator
        let fallback_model = match role {
            ModelRole::Senior => Some(&plan.junior_model),
            ModelRole::Junior => Some(&plan.translator_model),
            ModelRole::Translator => None,
        };

        fallback_model.and_then(|model| {
            if health.status(model).is_usable() {
                Some(model.clone())
            } else {
                None
            }
        })
    }
}

impl Default for SpecialistHelper {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper installation suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperSuggestion {
    /// Helper ID.
    pub helper_id: String,
    /// Helper name.
    pub helper_name: String,
    /// Purpose of the helper.
    pub purpose: String,
    /// Why it's being suggested.
    pub reason: String,
    /// Command to install.
    pub install_command: String,
    /// Packages to install.
    pub packages: Vec<String>,
}

impl HelperSuggestion {
    /// Format for specialist response step.
    pub fn format_step(&self) -> String {
        format!(
            "Install {} to {}. Command: {}",
            self.helper_name,
            self.purpose.to_lowercase(),
            self.install_command
        )
    }
}

/// Model availability status.
#[derive(Debug, Clone)]
pub enum ModelAvailability {
    /// Model is available.
    Available { model: String },
    /// Model is missing.
    Missing {
        model: String,
        fallback: Option<String>,
    },
    /// Model is broken.
    Broken {
        model: String,
        error: Option<String>,
        fallback: Option<String>,
    },
    /// Model is being installed.
    Installing { model: String },
}

impl ModelAvailability {
    /// Whether the model (or fallback) can be used.
    pub fn can_proceed(&self) -> bool {
        match self {
            Self::Available { .. } => true,
            Self::Missing { fallback, .. } => fallback.is_some(),
            Self::Broken { fallback, .. } => fallback.is_some(),
            Self::Installing { .. } => false,
        }
    }

    /// Get the model to use (original or fallback).
    pub fn usable_model(&self) -> Option<&str> {
        match self {
            Self::Available { model } => Some(model),
            Self::Missing { fallback, .. } => fallback.as_deref(),
            Self::Broken { fallback, .. } => fallback.as_deref(),
            Self::Installing { .. } => None,
        }
    }

    /// Whether using a fallback.
    pub fn is_fallback(&self) -> bool {
        matches!(
            self,
            Self::Missing {
                fallback: Some(_),
                ..
            } | Self::Broken {
                fallback: Some(_),
                ..
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_helper_suggestion() {
        let helper = SpecialistHelper::new();

        let suggestion = helper.suggest_helper_step(
            "lm_sensors",
            "To get accurate CPU temperatures",
            "Arch Linux",
        );

        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert!(s.install_command.contains("pacman"));
        assert!(s.packages.contains(&"lm_sensors".to_string()));
    }

    #[test]
    fn test_model_availability_available() {
        let avail = ModelAvailability::Available {
            model: "qwen3:4b".to_string(),
        };

        assert!(avail.can_proceed());
        assert_eq!(avail.usable_model(), Some("qwen3:4b"));
        assert!(!avail.is_fallback());
    }

    #[test]
    fn test_model_availability_missing_with_fallback() {
        let avail = ModelAvailability::Missing {
            model: "qwen2.5:14b".to_string(),
            fallback: Some("qwen2.5:7b".to_string()),
        };

        assert!(avail.can_proceed());
        assert_eq!(avail.usable_model(), Some("qwen2.5:7b"));
        assert!(avail.is_fallback());
    }

    #[test]
    fn test_model_availability_missing_no_fallback() {
        let avail = ModelAvailability::Missing {
            model: "qwen3:0.6b".to_string(),
            fallback: None,
        };

        assert!(!avail.can_proceed());
        assert!(avail.usable_model().is_none());
    }
}
