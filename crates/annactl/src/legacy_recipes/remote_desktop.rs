// Beta.168: Remote Desktop Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct RemoteDesktopRecipe;

#[derive(Debug, PartialEq)]
enum RemoteDesktopOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl RemoteDesktopOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            RemoteDesktopOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            RemoteDesktopOperation::ListTools
        } else {
            RemoteDesktopOperation::Install
        }
    }
}

impl RemoteDesktopRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("remmina") || input_lower.contains("rdp")
            || input_lower.contains("vnc") || input_lower.contains("remote desktop")
            || input_lower.contains("tigervnc") || input_lower.contains("anydesk");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = RemoteDesktopOperation::detect(user_input);
        match operation {
            RemoteDesktopOperation::Install => Self::build_install_plan(telemetry),
            RemoteDesktopOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            RemoteDesktopOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("remmina") { "remmina" }
        else if input_lower.contains("tigervnc") { "tigervnc" }
        else if input_lower.contains("anydesk") { "anydesk" }
        else { "remmina" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, is_aur, description) = match tool {
            "remmina" => ("Remmina", "remmina", false, "Multi-protocol remote desktop client (RDP, VNC, SSH)"),
            "tigervnc" => ("TigerVNC", "tigervnc", false, "VNC server and viewer"),
            "anydesk" => ("AnyDesk", "anydesk-bin", true, "Remote desktop software (AUR)"),
            _ => ("Remmina", "remmina", false, "Multi-protocol remote desktop client"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("remote_desktop.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = format!("{} installed. {}. Launch from application menu.", tool_name, description);

        let risk_level = if is_aur { RiskLevel::Medium } else { RiskLevel::Low };
        let requires_confirmation = is_aur;

        Ok(ActionPlan {
            analysis: format!("Installing {} remote desktop tool", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
                    risk_level,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: if is_aur {
                        format!("yay -Rns --noconfirm {} || paru -Rns --noconfirm {}", package_name, package_name)
                    } else {
                        format!("sudo pacman -Rns --noconfirm {}", package_name)
                    },
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("remote_desktop_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("remote_desktop.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking remote desktop tools".to_string(),
            goals: vec!["List installed remote desktop tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-remote-desktop-tools".to_string(),
                    description: "List remote desktop tools".to_string(),
                    command: "pacman -Q remmina tigervnc anydesk-bin 2>/dev/null || echo 'No remote desktop tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed remote desktop tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("remote_desktop_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("remote_desktop.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available remote desktop tools".to_string(),
            goals: vec!["List available remote desktop tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Remote Desktop Clients:\n- Remmina (official) - Multi-protocol (RDP, VNC, SSH, SPICE)\n- TigerVNC (official) - VNC server and viewer\n- KRDC (official) - KDE remote desktop client\n- Vinagre (official) - GNOME remote desktop viewer\n\nRemote Desktop Software:\n- AnyDesk (AUR) - Cross-platform remote desktop\n- TeamViewer (AUR) - Remote access and support\n- RustDesk (AUR) - Open-source remote desktop\n\nCommand-line Tools:\n- freerdp (official) - RDP client and server\n- x11vnc (official) - VNC server for X11'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Remote desktop tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("remote_desktop_list_tools".to_string()),
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
        assert!(RemoteDesktopRecipe::matches_request("install remmina"));
        assert!(RemoteDesktopRecipe::matches_request("install rdp client"));
        assert!(RemoteDesktopRecipe::matches_request("install remote desktop"));
        assert!(!RemoteDesktopRecipe::matches_request("what is remmina"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install tigervnc".to_string());
        let plan = RemoteDesktopRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
