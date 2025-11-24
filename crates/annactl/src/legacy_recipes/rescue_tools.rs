// Beta.176: Rescue and Recovery Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, PlanMeta, DetectionResults, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct RescueToolsRecipe;

impl RescueToolsRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        (input_lower.contains("testdisk") || input_lower.contains("photorec")
         || input_lower.contains("ddrescue") || input_lower.contains("gpart")
         || input_lower.contains("data recovery") || input_lower.contains("disk recovery")
         || input_lower.contains("rescue") && (input_lower.contains("tool") || input_lower.contains("disk") || input_lower.contains("partition")))
        && (input_lower.contains("install") || input_lower.contains("setup"))
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let input_lower = user_input.to_lowercase();

        let (tool, package, description, notes) = if input_lower.contains("testdisk") || input_lower.contains("photorec") {
            ("TestDisk/PhotoRec", "testdisk",
             "Data recovery software - recover lost partitions and files",
             "TestDisk recovers lost partitions. PhotoRec recovers files from any media. Run with: sudo testdisk or sudo photorec")
        } else if input_lower.contains("ddrescue") {
            ("GNU ddrescue", "ddrescue",
             "Data recovery tool for disk imaging and cloning",
             "Use ddrescue to create disk images from failing drives: sudo ddrescue /dev/sdX output.img logfile")
        } else if input_lower.contains("gpart") {
            ("gpart", "gpart",
             "Partition table recovery tool",
             "Recovers lost partition tables: sudo gpart /dev/sdX")
        } else {
            // Default to testdisk (most common use case)
            ("TestDisk/PhotoRec", "testdisk",
             "Data recovery software - recover lost partitions and files",
             "TestDisk recovers lost partitions. PhotoRec recovers files. Includes both tools in one package.")
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("rescue_tools.rs"));
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
            notes_for_user: format!("{} installed successfully.\n\n{}\n\n⚠️ WARNING: Data recovery tools should be used on unmounted drives. Always work on disk images when possible.", tool, notes),
            meta: PlanMeta {
                detection_results: DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("rescue_tools_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_rescue_tools() {
        assert!(RescueToolsRecipe::matches_request("install testdisk"));
        assert!(RescueToolsRecipe::matches_request("install photorec"));
        assert!(RescueToolsRecipe::matches_request("install ddrescue"));
        assert!(RescueToolsRecipe::matches_request("install data recovery tool"));
        assert!(RescueToolsRecipe::matches_request("setup disk recovery"));
        assert!(!RescueToolsRecipe::matches_request("what is testdisk"));
        assert!(!RescueToolsRecipe::matches_request("how to recover files"));
    }

    #[test]
    fn test_build_plan_testdisk() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install testdisk".to_string());

        let plan = RescueToolsRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("TestDisk"));
        assert_eq!(plan.command_plan.len(), 1);
        assert!(plan.command_plan[0].command.contains("testdisk"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_build_plan_ddrescue() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install ddrescue".to_string());

        let plan = RescueToolsRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("ddrescue"));
        assert!(plan.command_plan[0].command.contains("ddrescue"));
    }
}
