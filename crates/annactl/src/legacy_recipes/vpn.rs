// Beta.162: VPN Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct VpnRecipe;

#[derive(Debug, PartialEq)]
enum VpnOperation {
    Install,
    CheckStatus,
    GenerateKeys,
    ListTools,
}

impl VpnOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("generate") || input_lower.contains("keys") {
            VpnOperation::GenerateKeys
        } else if input_lower.contains("check") || input_lower.contains("status") {
            VpnOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            VpnOperation::ListTools
        } else {
            VpnOperation::Install
        }
    }
}

impl VpnRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("wireguard") || input_lower.contains("openvpn")
            || input_lower.contains("vpn") || input_lower.contains("wg-quick");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("generate");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = VpnOperation::detect(user_input);
        match operation {
            VpnOperation::Install => Self::build_install_plan(telemetry),
            VpnOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            VpnOperation::GenerateKeys => Self::build_generate_keys_plan(telemetry),
            VpnOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("wireguard") || input_lower.contains("wg") { "wireguard" }
        else if input_lower.contains("openvpn") { "openvpn" }
        else { "wireguard" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name) = match tool {
            "wireguard" => ("WireGuard", "wireguard-tools"),
            "openvpn" => ("OpenVPN", "openvpn"),
            _ => ("WireGuard", "wireguard-tools"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("vpn.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let notes = if tool == "wireguard" {
            "WireGuard installed. Generate keys: wg genkey | tee privatekey | wg pubkey > publickey".to_string()
        } else {
            "OpenVPN installed. Configure with .ovpn files: sudo openvpn --config /path/to/config.ovpn".to_string()
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} VPN", tool_name),
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
                template_used: Some("vpn_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("vpn.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking VPN tools".to_string(),
            goals: vec!["List installed VPN tools and active connections".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-vpn-tools".to_string(),
                    description: "List VPN tools".to_string(),
                    command: "pacman -Q wireguard-tools openvpn 2>/dev/null || echo 'No VPN tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "check-wg-interfaces".to_string(),
                    description: "Check WireGuard interfaces".to_string(),
                    command: "sudo wg show 2>/dev/null || echo 'No WireGuard interfaces active'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed VPN tools and active connections".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("vpn_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_generate_keys_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("vpn.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("GenerateKeys"));

        Ok(ActionPlan {
            analysis: "Generating WireGuard keys".to_string(),
            goals: vec!["Generate WireGuard private and public keys".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-wireguard".to_string(),
                    description: "Verify WireGuard is installed".to_string(),
                    command: "which wg".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "generate-keys".to_string(),
                    description: "Generate WireGuard key pair".to_string(),
                    command: "wg genkey | tee /tmp/wg-privatekey | wg pubkey > /tmp/wg-publickey && echo 'Private key: /tmp/wg-privatekey' && echo 'Public key: /tmp/wg-publickey'".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-keys".to_string()),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-keys".to_string(),
                    description: "Remove generated keys".to_string(),
                    command: "rm -f /tmp/wg-privatekey /tmp/wg-publickey".to_string(),
                },
            ],
            notes_for_user: "WireGuard keys generated in /tmp. Move them to /etc/wireguard/ for configuration.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("vpn_generate_keys".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("vpn.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available VPN tools".to_string(),
            goals: vec!["List available VPN tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available VPN tools".to_string(),
                    command: r"echo 'Available:\n- WireGuard (official) - Modern VPN protocol\n- OpenVPN (official) - Traditional VPN solution'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "VPN tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("vpn_list_tools".to_string()),
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
        assert!(VpnRecipe::matches_request("install wireguard"));
        assert!(VpnRecipe::matches_request("setup vpn"));
        assert!(VpnRecipe::matches_request("generate wireguard keys"));
        assert!(!VpnRecipe::matches_request("what is vpn"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install wireguard".to_string());
        let plan = VpnRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
