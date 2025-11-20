// Beta.176: Build Systems Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, PlanMeta, DetectionResults, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct BuildSystemsRecipe;

impl BuildSystemsRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        (input_lower.contains("make") || input_lower.contains("cmake")
         || input_lower.contains("build") && (input_lower.contains("system") || input_lower.contains("tool")))
        && (input_lower.contains("install") || input_lower.contains("setup"))
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let input_lower = user_input.to_lowercase();

        let (tool, package, description) = if input_lower.contains("cmake") {
            ("CMake", "cmake", "Cross-platform build system generator")
        } else if input_lower.contains("gnu make") || input_lower.contains(" make") {
            ("GNU Make", "make", "Classic build automation tool")
        } else {
            ("base-devel", "base-devel", "Essential build tools for Arch Linux")
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("build_systems.rs"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool));

        Ok(ActionPlan {
            analysis: format!("Installing {} - {}", tool, description),
            goals: vec![format!("Install {}", tool)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", package),
                    description: format!("Install {} package", tool),
                    command: format!("sudo pacman -S --needed --noconfirm {}", package),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", package)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", package),
                    description: format!("Remove {}", tool),
                    command: format!("sudo pacman -Rns --noconfirm {}", package),
                },
            ],
            notes_for_user: format!("{} installed successfully. {}", tool, description),
            meta: PlanMeta {
                detection_results: DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("build_systems_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_build_systems() {
        assert!(BuildSystemsRecipe::matches_request("install cmake"));
        assert!(BuildSystemsRecipe::matches_request("install make"));
        assert!(BuildSystemsRecipe::matches_request("install build tools"));
        assert!(!BuildSystemsRecipe::matches_request("what is cmake"));
    }
}
