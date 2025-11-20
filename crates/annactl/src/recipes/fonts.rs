// Beta.176: Fonts Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, PlanMeta, DetectionResults, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct FontsRecipe;

impl FontsRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        (input_lower.contains("font") || input_lower.contains("noto") || input_lower.contains("dejavu")
         || input_lower.contains("liberation") || input_lower.contains("nerd font") || input_lower.contains("ttf"))
        && (input_lower.contains("install") || input_lower.contains("setup"))
        && !input_lower.contains("change") && !input_lower.contains("configure")
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let input_lower = user_input.to_lowercase();

        let (font_name, package, description, additional_notes) = if input_lower.contains("noto") {
            ("Noto Fonts", "noto-fonts noto-fonts-cjk noto-fonts-emoji",
             "Google's Noto font families - comprehensive Unicode coverage",
             "Includes Latin, CJK (Chinese/Japanese/Korean), and emoji fonts. Recommended for international text support.")
        } else if input_lower.contains("dejavu") {
            ("DejaVu Fonts", "ttf-dejavu",
             "DejaVu font family - high quality TrueType fonts",
             "Based on Bitstream Vera fonts. Excellent for programming and general use.")
        } else if input_lower.contains("liberation") {
            ("Liberation Fonts", "ttf-liberation",
             "Liberation font family - metric-compatible with Microsoft fonts",
             "Compatible with Times New Roman, Arial, and Courier New. Ideal for document compatibility.")
        } else if input_lower.contains("nerd") || input_lower.contains("powerline") {
            ("Nerd Fonts", "ttf-nerd-fonts-symbols-mono",
             "Nerd Fonts - iconic font aggregator and patcher",
             "Adds glyphs from popular icon fonts (Font Awesome, Devicons, Octicons, etc.). Essential for terminal customization.")
        } else {
            // Default to essential fonts bundle
            ("Essential Fonts Bundle", "noto-fonts ttf-dejavu ttf-liberation",
             "Essential font collection for Linux systems",
             "Includes Noto Fonts (Unicode), DejaVu (quality), and Liberation (MS compatibility). Recommended for new installations.")
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("fonts.rs"));
        meta_other.insert("font_family".to_string(), serde_json::json!(font_name));

        Ok(ActionPlan {
            analysis: format!("Installing {} - {}", font_name, description),
            goals: vec![format!("Install {}", font_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "install-fonts".to_string(),
                    description: format!("Install {} packages", font_name),
                    command: format!("sudo pacman -S --needed --noconfirm {}", package),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-fonts".to_string()),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "rebuild-font-cache".to_string(),
                    description: "Rebuild font cache".to_string(),
                    command: "fc-cache -fv".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-fonts".to_string(),
                    description: format!("Remove {}", font_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package),
                },
            ],
            notes_for_user: format!(
                "{} installed successfully.\n\n{}\n\nFont cache rebuilt. Applications may need restart to detect new fonts.\n\nView installed fonts: fc-list\nSearch fonts: fc-list | grep <name>",
                font_name, additional_notes
            ),
            meta: PlanMeta {
                detection_results: DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("fonts_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_fonts() {
        assert!(FontsRecipe::matches_request("install noto fonts"));
        assert!(FontsRecipe::matches_request("install dejavu font"));
        assert!(FontsRecipe::matches_request("install liberation fonts"));
        assert!(FontsRecipe::matches_request("install nerd fonts"));
        assert!(FontsRecipe::matches_request("setup ttf fonts"));
        assert!(!FontsRecipe::matches_request("what are fonts"));
        assert!(!FontsRecipe::matches_request("change font"));
        assert!(!FontsRecipe::matches_request("configure fonts"));
    }

    #[test]
    fn test_build_plan_noto() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install noto fonts".to_string());

        let plan = FontsRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("Noto"));
        assert_eq!(plan.command_plan.len(), 2);
        assert!(plan.command_plan[0].command.contains("noto-fonts"));
        assert!(plan.command_plan[1].command.contains("fc-cache"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_build_plan_default_bundle() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install fonts".to_string());

        let plan = FontsRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("Essential"));
        assert!(plan.command_plan[0].command.contains("noto-fonts"));
        assert!(plan.command_plan[0].command.contains("ttf-dejavu"));
        assert!(plan.command_plan[0].command.contains("ttf-liberation"));
    }

    #[test]
    fn test_build_plan_nerd_fonts() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install nerd fonts".to_string());

        let plan = FontsRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("Nerd"));
        assert!(plan.command_plan[0].command.contains("ttf-nerd-fonts-symbols-mono"));
    }
}
