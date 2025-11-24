// Beta.171: Image Viewer Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct ImageViewRecipe;

#[derive(Debug, PartialEq)]
enum ImageViewOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl ImageViewOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            ImageViewOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            ImageViewOperation::ListTools
        } else {
            ImageViewOperation::Install
        }
    }
}

impl ImageViewRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("feh") || input_lower.contains("sxiv")
            || input_lower.contains("geeqie") || input_lower.contains("imv")
            || input_lower.contains("image viewer") || input_lower.contains("photo viewer")
            || input_lower.contains("picture viewer");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = ImageViewOperation::detect(user_input);
        match operation {
            ImageViewOperation::Install => Self::build_install_plan(telemetry),
            ImageViewOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            ImageViewOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("feh") { "feh" }
        else if input_lower.contains("sxiv") { "sxiv" }
        else if input_lower.contains("geeqie") { "geeqie" }
        else if input_lower.contains("imv") { "imv" }
        else { "feh" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "feh" => ("feh", "feh", "Lightweight X11 image viewer for CLI and scripts"),
            "sxiv" => ("sxiv", "sxiv", "Simple X Image Viewer with minimal interface"),
            "geeqie" => ("Geeqie", "geeqie", "GTK-based image viewer with organization features"),
            "imv" => ("imv", "imv", "Command-line image viewer for Wayland and X11"),
            _ => ("feh", "feh", "Lightweight image viewer"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("imageview.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let notes = format!("{} installed. {}. Run {} <image> to view images.", tool_name, description, tool);

        Ok(ActionPlan {
            analysis: format!("Installing {} image viewer", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: format!("sudo pacman -S --needed --noconfirm {}", package_name),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("imageview_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("imageview.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking image viewer tools".to_string(),
            goals: vec!["List installed image viewers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-imageview-tools".to_string(),
                    description: "List image viewers".to_string(),
                    command: "pacman -Q feh sxiv geeqie imv 2>/dev/null || echo 'No image viewers installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed image viewer tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("imageview_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("imageview.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available image viewer tools".to_string(),
            goals: vec!["List available image viewers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Image Viewers:

Lightweight/CLI:
- feh (official) - Fast X11 viewer, perfect for scripts, wallpaper setting
- sxiv (official) - Simple X Image Viewer, minimal keyboard-driven interface
- imv (official) - Wayland/X11 viewer for command line, scriptable
- qiv (official) - Quick Image Viewer for X11
- fim (official) - Framebuffer/terminal image viewer

Feature-Rich GUI:
- Geeqie (official) - GTK image viewer with organization, metadata editing
- gThumb (official) - GNOME image viewer and organizer
- Gwenview (official) - KDE image viewer with editing tools
- Eye of GNOME (eog) (official) - Simple GNOME image viewer
- Ristretto (official) - Fast XFCE image viewer

Advanced/Pro:
- digiKam (official) - Professional photo management and editing
- Shotwell (official) - Photo manager for GNOME
- darktable (official) - Photography workflow and RAW developer
- RawTherapee (official) - RAW photo processing

Comparison:
- feh: Best for minimal setups, window managers, scripting
- sxiv: Keyboard-driven, fast, simple interface
- Geeqie: Best balance of features and speed
- Gwenview: KDE integration, basic editing tools

Usage:
- feh: feh image.jpg or feh --bg-scale wallpaper.jpg
- sxiv: sxiv -t *.jpg (thumbnail mode)
- Geeqie: Launch from menu, browse directories
- imv: imv image.png or imv -r directory/'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Image viewers for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("imageview_list_tools".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_matches() {
        assert!(ImageViewRecipe::matches_request("install feh"));
        assert!(ImageViewRecipe::matches_request("install image viewer"));
        assert!(ImageViewRecipe::matches_request("setup sxiv"));
        assert!(!ImageViewRecipe::matches_request("what is geeqie"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install imv".to_string());
        let plan = ImageViewRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
