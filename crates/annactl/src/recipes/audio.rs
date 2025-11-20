// Beta.164: Audio System Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct AudioRecipe;

#[derive(Debug, PartialEq)]
enum AudioOperation {
    Install,
    CheckStatus,
    SwitchToPipewire,
    ListSystems,
}

impl AudioOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("switch") || input_lower.contains("migrate") {
            AudioOperation::SwitchToPipewire
        } else if input_lower.contains("check") || input_lower.contains("status") {
            AudioOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            AudioOperation::ListSystems
        } else {
            AudioOperation::Install
        }
    }
}

impl AudioRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("pipewire") || input_lower.contains("pulseaudio")
            || input_lower.contains("alsa") || input_lower.contains("audio system")
            || input_lower.contains("sound system");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("switch") || input_lower.contains("migrate");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = AudioOperation::detect(user_input);
        match operation {
            AudioOperation::Install => Self::build_install_plan(telemetry),
            AudioOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            AudioOperation::SwitchToPipewire => Self::build_switch_pipewire_plan(telemetry),
            AudioOperation::ListSystems => Self::build_list_systems_plan(telemetry),
        }
    }

    fn detect_system(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("pipewire") { "pipewire" }
        else if input_lower.contains("pulseaudio") || input_lower.contains("pulse") { "pulseaudio" }
        else { "pipewire" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let system = Self::detect_system(user_input);
        let (system_name, packages) = match system {
            "pipewire" => ("PipeWire", vec!["pipewire", "pipewire-pulse", "pipewire-alsa", "wireplumber"]),
            "pulseaudio" => ("PulseAudio", vec!["pulseaudio", "pulseaudio-alsa"]),
            _ => ("PipeWire", vec!["pipewire", "pipewire-pulse", "pipewire-alsa", "wireplumber"]),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("audio.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("system".to_string(), serde_json::json!(system_name));

        let install_cmd = format!("sudo pacman -S --needed --noconfirm {}", packages.join(" "));

        Ok(ActionPlan {
            analysis: format!("Installing {} audio system", system_name),
            goals: vec![format!("Install {}", system_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", system),
                    description: format!("Install {}", system_name),
                    command: install_cmd,
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", system)),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "enable-user-service".to_string(),
                    description: format!("Enable {} user service", system_name),
                    command: if system == "pipewire" {
                        "systemctl --user enable --now pipewire pipewire-pulse wireplumber".to_string()
                    } else {
                        "systemctl --user enable --now pulseaudio".to_string()
                    },
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", system),
                    description: format!("Remove {}", system_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", packages.join(" ")),
                },
            ],
            notes_for_user: format!("{} installed. Restart or log out for changes to take effect.", system_name),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("audio_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("audio.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking audio system".to_string(),
            goals: vec!["Check installed audio systems and status".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "check-audio-systems".to_string(),
                    description: "Check audio systems".to_string(),
                    command: "pacman -Q pipewire pulseaudio 2>/dev/null || echo 'No audio system installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "check-audio-status".to_string(),
                    description: "Check audio service status".to_string(),
                    command: "systemctl --user is-active pipewire pulseaudio 2>/dev/null || echo 'No audio services running'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed audio systems and their status".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("audio_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_switch_pipewire_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("audio.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("SwitchToPipewire"));

        Ok(ActionPlan {
            analysis: "Switching from PulseAudio to PipeWire".to_string(),
            goals: vec![
                "Stop PulseAudio".to_string(),
                "Install PipeWire".to_string(),
                "Start PipeWire services".to_string(),
            ],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-pulseaudio".to_string(),
                    description: "Verify PulseAudio is installed".to_string(),
                    command: "pacman -Q pulseaudio".to_string(),
                    risk_level: RiskLevel::Info,
                    required: false,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "stop-pulseaudio".to_string(),
                    description: "Stop PulseAudio".to_string(),
                    command: "systemctl --user stop pulseaudio || true".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "install-pipewire".to_string(),
                    description: "Install PipeWire".to_string(),
                    command: "sudo pacman -S --needed --noconfirm pipewire pipewire-pulse pipewire-alsa wireplumber".to_string(),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some("remove-pipewire".to_string()),
                    requires_confirmation: true,
                },
                CommandStep {
                    id: "start-pipewire".to_string(),
                    description: "Start PipeWire services".to_string(),
                    command: "systemctl --user enable --now pipewire pipewire-pulse wireplumber".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-pipewire".to_string(),
                    description: "Remove PipeWire".to_string(),
                    command: "sudo pacman -Rns --noconfirm pipewire pipewire-pulse pipewire-alsa wireplumber".to_string(),
                },
            ],
            notes_for_user: "PipeWire installed. Restart or log out for full migration. PulseAudio apps will work through pipewire-pulse.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("audio_switch_pipewire".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_systems_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("audio.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListSystems"));

        Ok(ActionPlan {
            analysis: "Showing available audio systems".to_string(),
            goals: vec!["List available audio systems".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-systems".to_string(),
                    description: "Show available systems".to_string(),
                    command: r"echo 'Available:\n- PipeWire (official) - Modern audio/video server\n- PulseAudio (official) - Traditional Linux audio server\n- ALSA (kernel) - Low-level audio interface'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Audio systems for Arch Linux. PipeWire is recommended for modern systems.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("audio_list_systems".to_string()),
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
        assert!(AudioRecipe::matches_request("install pipewire"));
        assert!(AudioRecipe::matches_request("setup audio system"));
        assert!(AudioRecipe::matches_request("switch to pipewire"));
        assert!(!AudioRecipe::matches_request("what is pipewire"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install pipewire".to_string());
        let plan = AudioRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
