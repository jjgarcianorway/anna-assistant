// Beta.164: Audio Recording & Production Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct RecordingRecipe;

#[derive(Debug, PartialEq)]
enum RecordingOperation {
    Install,
    CheckStatus,
    SetupJack,
    ListTools,
}

impl RecordingOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("setup jack") || input_lower.contains("configure jack") {
            RecordingOperation::SetupJack
        } else if input_lower.contains("check") || input_lower.contains("status") {
            RecordingOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            RecordingOperation::ListTools
        } else {
            RecordingOperation::Install
        }
    }
}

impl RecordingRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("ardour") || input_lower.contains("audacity")
            || input_lower.contains("jack") || input_lower.contains("audio recording")
            || input_lower.contains("audio production") || input_lower.contains("daw");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = RecordingOperation::detect(user_input);
        match operation {
            RecordingOperation::Install => Self::build_install_plan(telemetry),
            RecordingOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            RecordingOperation::SetupJack => Self::build_setup_jack_plan(telemetry),
            RecordingOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("ardour") { "ardour" }
        else if input_lower.contains("audacity") { "audacity" }
        else if input_lower.contains("jack") { "jack" }
        else { "audacity" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, is_aur) = match tool {
            "ardour" => ("Ardour", "ardour", false),
            "audacity" => ("Audacity", "audacity", false),
            "jack" => ("JACK", "jack2", false),
            _ => ("Audacity", "audacity", false),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("recording.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = match tool {
            "jack" => "JACK installed. For low-latency audio, configure with qjackctl or start with: jackd -d alsa".to_string(),
            "ardour" => "Ardour DAW installed. Professional audio workstation for recording and mixing.".to_string(),
            _ => format!("{} installed. Launch from application menu.", tool_name),
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} audio tool", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
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
                template_used: Some("recording_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("recording.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking audio recording tools".to_string(),
            goals: vec!["List installed recording tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-recording-tools".to_string(),
                    description: "List recording tools".to_string(),
                    command: "pacman -Q ardour audacity jack2 2>/dev/null || echo 'No recording tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed audio recording and production tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("recording_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_setup_jack_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("recording.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("SetupJack"));

        Ok(ActionPlan {
            analysis: "Setting up JACK audio server".to_string(),
            goals: vec![
                "Install JACK and utilities".to_string(),
                "Install qjackctl GUI".to_string(),
            ],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "install-jack-suite".to_string(),
                    description: "Install JACK suite".to_string(),
                    command: "sudo pacman -S --needed --noconfirm jack2 qjackctl".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-jack-suite".to_string()),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "add-audio-group".to_string(),
                    description: "Add user to audio group".to_string(),
                    command: "sudo usermod -aG audio $USER".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-jack-suite".to_string(),
                    description: "Remove JACK suite".to_string(),
                    command: "sudo pacman -Rns --noconfirm jack2 qjackctl".to_string(),
                },
            ],
            notes_for_user: "JACK installed. Launch qjackctl to configure and start JACK. Log out and back in for group changes.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("recording_setup_jack".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("recording.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available recording tools".to_string(),
            goals: vec!["List available recording tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Available:\n- Audacity (official) - Multi-track audio editor\n- Ardour (official) - Professional DAW\n- JACK (official) - Low-latency audio server\n- Reaper (AUR) - Commercial DAW'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Audio recording and production tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("recording_list_tools".to_string()),
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
        assert!(RecordingRecipe::matches_request("install ardour"));
        assert!(RecordingRecipe::matches_request("setup jack"));
        assert!(RecordingRecipe::matches_request("install audio production tools"));
        assert!(!RecordingRecipe::matches_request("what is ardour"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install audacity".to_string());
        let plan = RecordingRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
