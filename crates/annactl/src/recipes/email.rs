// Beta.170: Email Client Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct EmailRecipe;

#[derive(Debug, PartialEq)]
enum EmailOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl EmailOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            EmailOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            EmailOperation::ListTools
        } else {
            EmailOperation::Install
        }
    }
}

impl EmailRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("thunderbird") || input_lower.contains("mutt")
            || input_lower.contains("neomutt") || input_lower.contains("evolution")
            || input_lower.contains("geary") || input_lower.contains("email client")
            || input_lower.contains("mail client");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = EmailOperation::detect(user_input);
        match operation {
            EmailOperation::Install => Self::build_install_plan(telemetry),
            EmailOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            EmailOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("thunderbird") { "thunderbird" }
        else if input_lower.contains("neomutt") { "neomutt" }
        else if input_lower.contains("mutt") && !input_lower.contains("neomutt") { "mutt" }
        else if input_lower.contains("evolution") { "evolution" }
        else if input_lower.contains("geary") { "geary" }
        else { "thunderbird" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "thunderbird" => ("Thunderbird", "thunderbird", "Mozilla's feature-rich email client"),
            "mutt" => ("Mutt", "mutt", "Terminal-based email client"),
            "neomutt" => ("NeoMutt", "neomutt", "Modern fork of Mutt with improved features"),
            "evolution" => ("Evolution", "evolution", "GNOME email and calendar client"),
            "geary" => ("Geary", "geary", "Lightweight email client for GNOME"),
            _ => ("Thunderbird", "thunderbird", "Feature-rich email client"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("email.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let notes = format!("{} installed. {}. Launch from app menu or run {} in terminal.", tool_name, description, tool);

        Ok(ActionPlan {
            analysis: format!("Installing {} email client", tool_name),
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
                template_used: Some("email_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("email.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking email client tools".to_string(),
            goals: vec!["List installed email clients".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-email-tools".to_string(),
                    description: "List email clients".to_string(),
                    command: "pacman -Q thunderbird mutt neomutt evolution geary 2>/dev/null || echo 'No email clients installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed email client tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("email_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("email.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available email client tools".to_string(),
            goals: vec!["List available email clients".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Email Clients:

GUI Clients:
- Thunderbird (official) - Mozilla full-featured email client with calendar, tasks, contacts
- Evolution (official) - GNOME email, calendar, contacts, and task manager
- Geary (official) - Lightweight, modern GNOME email client
- KMail (official) - KDE email client (part of KDE PIM)
- Mailspring (AUR) - Modern email client with unified inbox

Terminal Clients:
- Mutt (official) - Classic terminal email client
- NeoMutt (official) - Modern fork of Mutt with patches and improvements
- Alpine (official) - Text-based email client (successor to Pine)
- aerc (official) - Modern terminal email client with vim keybindings

Webmail Access:
- Firefox/Chrome - Use web browser for Gmail, Outlook, ProtonMail, etc.

Configuration:
- Thunderbird: Launch from app menu, auto-detects most providers
- Mutt/NeoMutt: Requires configuration in ~/.muttrc or ~/.neomuttrc
- Evolution: Launch from app menu, integrated with GNOME
- Geary: Launch from app menu, simple setup wizard'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Email clients for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("email_list_tools".to_string()),
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
        assert!(EmailRecipe::matches_request("install thunderbird"));
        assert!(EmailRecipe::matches_request("install email client"));
        assert!(EmailRecipe::matches_request("setup mutt"));
        assert!(!EmailRecipe::matches_request("what is thunderbird"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install evolution".to_string());
        let plan = EmailRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
