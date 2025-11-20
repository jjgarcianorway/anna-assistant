// Beta.166: Bluetooth Management Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct BluetoothRecipe;

#[derive(Debug, PartialEq)]
enum BluetoothOperation {
    Install,
    CheckStatus,
    Enable,
    ListDevices,
}

impl BluetoothOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("enable bluetooth") || input_lower.contains("start bluetooth") {
            BluetoothOperation::Enable
        } else if input_lower.contains("list device") || input_lower.contains("show device") {
            BluetoothOperation::ListDevices
        } else if input_lower.contains("check") || input_lower.contains("status") {
            BluetoothOperation::CheckStatus
        } else {
            BluetoothOperation::Install
        }
    }
}

impl BluetoothRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("bluetooth") || input_lower.contains("bluez")
            || input_lower.contains("blueman");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("enable") || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = BluetoothOperation::detect(user_input);
        match operation {
            BluetoothOperation::Install => Self::build_install_plan(telemetry),
            BluetoothOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            BluetoothOperation::Enable => Self::build_enable_plan(telemetry),
            BluetoothOperation::ListDevices => Self::build_list_devices_plan(telemetry),
        }
    }

    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("bluetooth.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));

        Ok(ActionPlan {
            analysis: "Installing Bluetooth stack".to_string(),
            goals: vec![
                "Install bluez and blueman".to_string(),
                "Enable Bluetooth service".to_string(),
            ],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "install-bluez".to_string(),
                    description: "Install Bluetooth stack".to_string(),
                    command: "sudo pacman -S --needed --noconfirm bluez bluez-utils blueman".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-bluez".to_string()),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "enable-bluetooth".to_string(),
                    description: "Enable Bluetooth service".to_string(),
                    command: "sudo systemctl enable --now bluetooth".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("disable-bluetooth".to_string()),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "disable-bluetooth".to_string(),
                    description: "Disable Bluetooth service".to_string(),
                    command: "sudo systemctl disable --now bluetooth".to_string(),
                },
                RollbackStep {
                    id: "remove-bluez".to_string(),
                    description: "Remove Bluetooth stack".to_string(),
                    command: "sudo pacman -Rns --noconfirm bluez bluez-utils blueman".to_string(),
                },
            ],
            notes_for_user: "Bluetooth installed. GUI manager: blueman-manager. CLI: bluetoothctl".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("bluetooth_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("bluetooth.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking Bluetooth status".to_string(),
            goals: vec!["Show Bluetooth service and adapter status".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "check-service".to_string(),
                    description: "Check Bluetooth service".to_string(),
                    command: "systemctl status bluetooth".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "check-adapter".to_string(),
                    description: "Check Bluetooth adapter".to_string(),
                    command: "bluetoothctl show 2>/dev/null || echo 'No Bluetooth adapter found'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows Bluetooth service status and adapter info".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("bluetooth_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_enable_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("bluetooth.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Enable"));

        Ok(ActionPlan {
            analysis: "Enabling Bluetooth".to_string(),
            goals: vec!["Enable and start Bluetooth service".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-installed".to_string(),
                    description: "Verify bluez is installed".to_string(),
                    command: "pacman -Q bluez".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "enable-service".to_string(),
                    description: "Enable Bluetooth service".to_string(),
                    command: "sudo systemctl enable --now bluetooth".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("disable-service".to_string()),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "power-on".to_string(),
                    description: "Power on Bluetooth adapter".to_string(),
                    command: "bluetoothctl power on".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "disable-service".to_string(),
                    description: "Disable Bluetooth service".to_string(),
                    command: "sudo systemctl disable --now bluetooth".to_string(),
                },
            ],
            notes_for_user: "Bluetooth enabled. Use 'bluetoothctl' or 'blueman-manager' to pair devices.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("bluetooth_enable".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_devices_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("bluetooth.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListDevices"));

        Ok(ActionPlan {
            analysis: "Listing Bluetooth devices".to_string(),
            goals: vec!["Show paired and available Bluetooth devices".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-paired".to_string(),
                    description: "List paired devices".to_string(),
                    command: "bluetoothctl devices 2>/dev/null || echo 'No paired devices'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "show-paired-info".to_string(),
                    description: "Show paired device info".to_string(),
                    command: "bluetoothctl paired-devices 2>/dev/null || echo 'No paired devices'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows Bluetooth devices. To pair: bluetoothctl scan on, pair <MAC>".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("bluetooth_list_devices".to_string()),
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
        assert!(BluetoothRecipe::matches_request("install bluetooth"));
        assert!(BluetoothRecipe::matches_request("setup bluez"));
        assert!(BluetoothRecipe::matches_request("enable bluetooth"));
        assert!(!BluetoothRecipe::matches_request("what is bluetooth"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install bluetooth".to_string());
        let plan = BluetoothRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
