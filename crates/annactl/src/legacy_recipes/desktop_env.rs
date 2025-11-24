// Beta.165: Desktop Environments & Window Managers Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct DesktopEnvRecipe;

#[derive(Debug, PartialEq)]
enum DesktopOperation {
    Install,
    CheckStatus,
    ListDesktops,
}

impl DesktopOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            DesktopOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            DesktopOperation::ListDesktops
        } else {
            DesktopOperation::Install
        }
    }
}

impl DesktopEnvRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("gnome") || input_lower.contains("kde")
            || input_lower.contains("xfce") || input_lower.contains("i3wm")
            || input_lower.contains("sway") || input_lower.contains("desktop environment")
            || input_lower.contains("window manager") || input_lower.contains("plasma");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = DesktopOperation::detect(user_input);
        match operation {
            DesktopOperation::Install => Self::build_install_plan(telemetry),
            DesktopOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            DesktopOperation::ListDesktops => Self::build_list_desktops_plan(telemetry),
        }
    }

    fn detect_desktop(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("gnome") { "gnome" }
        else if input_lower.contains("kde") || input_lower.contains("plasma") { "kde" }
        else if input_lower.contains("xfce") { "xfce" }
        else if input_lower.contains("i3") { "i3" }
        else if input_lower.contains("sway") { "sway" }
        else { "xfce" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let desktop = Self::detect_desktop(user_input);
        let (desktop_name, install_cmd, notes) = match desktop {
            "gnome" => (
                "GNOME",
                "sudo pacman -S --needed --noconfirm gnome gnome-extra",
                "GNOME installed. Enable GDM: sudo systemctl enable gdm. Reboot to use."
            ),
            "kde" => (
                "KDE Plasma",
                "sudo pacman -S --needed --noconfirm plasma-meta kde-applications-meta",
                "KDE Plasma installed. Enable SDDM: sudo systemctl enable sddm. Reboot to use."
            ),
            "xfce" => (
                "XFCE",
                "sudo pacman -S --needed --noconfirm xfce4 xfce4-goodies",
                "XFCE installed. Install a display manager (lightdm recommended). Reboot to use."
            ),
            "i3" => (
                "i3",
                "sudo pacman -S --needed --noconfirm i3-wm i3status i3lock dmenu",
                "i3 window manager installed. Add 'exec i3' to ~/.xinitrc and use startx."
            ),
            "sway" => (
                "Sway",
                "sudo pacman -S --needed --noconfirm sway swaylock swayidle waybar",
                "Sway compositor installed. Launch with 'sway' command."
            ),
            _ => (
                "XFCE",
                "sudo pacman -S --needed --noconfirm xfce4 xfce4-goodies",
                "XFCE installed. Install a display manager (lightdm recommended). Reboot to use."
            ),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("desktop_env.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("desktop".to_string(), serde_json::json!(desktop_name));

        Ok(ActionPlan {
            analysis: format!("Installing {} desktop environment", desktop_name),
            goals: vec![format!("Install {}", desktop_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", desktop),
                    description: format!("Install {}", desktop_name),
                    command: install_cmd.to_string(),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some(format!("remove-{}", desktop)),
                    requires_confirmation: true,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", desktop),
                    description: format!("Remove {} (manual cleanup may be needed)", desktop_name),
                    command: format!("echo 'Manual removal recommended: review installed packages'"),
                },
            ],
            notes_for_user: notes.to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("desktop_env_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("desktop_env.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking desktop environments".to_string(),
            goals: vec!["List installed desktops".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-desktops".to_string(),
                    description: "List installed desktops".to_string(),
                    command: "pacman -Q gnome plasma-meta xfce4 i3-wm sway 2>/dev/null || echo 'No desktop environments installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "check-current-session".to_string(),
                    description: "Check current session".to_string(),
                    command: "echo \"Current: $XDG_CURRENT_DESKTOP ($XDG_SESSION_TYPE)\"".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed desktop environments and current session".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("desktop_env_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_desktops_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("desktop_env.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListDesktops"));

        Ok(ActionPlan {
            analysis: "Showing available desktops".to_string(),
            goals: vec!["List available desktops".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-desktops".to_string(),
                    description: "Show available desktops".to_string(),
                    command: r"echo 'Available:\n- GNOME (official) - Modern, user-friendly desktop\n- KDE Plasma (official) - Feature-rich, customizable\n- XFCE (official) - Lightweight, traditional\n- i3 (official) - Tiling window manager\n- Sway (official) - Wayland i3 alternative'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Desktop environments and window managers for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("desktop_env_list_desktops".to_string()),
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
        assert!(DesktopEnvRecipe::matches_request("install gnome"));
        assert!(DesktopEnvRecipe::matches_request("install kde plasma"));
        assert!(DesktopEnvRecipe::matches_request("install desktop environment"));
        assert!(!DesktopEnvRecipe::matches_request("what is gnome"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install xfce".to_string());
        let plan = DesktopEnvRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
