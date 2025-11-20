// Beta.163: Virtual Network Configuration Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct VirtNetworkRecipe;

#[derive(Debug, PartialEq)]
enum VirtNetworkOperation {
    Install,
    CheckStatus,
    CreateNetwork,
    StartNetwork,
}

impl VirtNetworkOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("create") || input_lower.contains("new network") {
            VirtNetworkOperation::CreateNetwork
        } else if input_lower.contains("start") || input_lower.contains("enable network") {
            VirtNetworkOperation::StartNetwork
        } else if input_lower.contains("check") || input_lower.contains("status") {
            VirtNetworkOperation::CheckStatus
        } else {
            VirtNetworkOperation::Install
        }
    }
}

impl VirtNetworkRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("libvirt") && input_lower.contains("network")
            || input_lower.contains("virtual network") || input_lower.contains("virt-manager network");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("create")
            || input_lower.contains("start");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = VirtNetworkOperation::detect(user_input);
        match operation {
            VirtNetworkOperation::Install => Self::build_install_plan(telemetry),
            VirtNetworkOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            VirtNetworkOperation::CreateNetwork => Self::build_create_network_plan(telemetry),
            VirtNetworkOperation::StartNetwork => Self::build_start_network_plan(telemetry),
        }
    }

    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("virt_network.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));

        Ok(ActionPlan {
            analysis: "Setting up libvirt networking".to_string(),
            goals: vec![
                "Enable libvirtd service".to_string(),
                "Start default network".to_string(),
            ],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-libvirt".to_string(),
                    description: "Verify libvirt is installed".to_string(),
                    command: "pacman -Q libvirt".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "enable-libvirtd".to_string(),
                    description: "Enable libvirtd service".to_string(),
                    command: "sudo systemctl enable --now libvirtd".to_string(),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some("disable-libvirtd".to_string()),
                    requires_confirmation: true,
                },
                CommandStep {
                    id: "start-default-network".to_string(),
                    description: "Start default virtual network".to_string(),
                    command: "sudo virsh net-start default || echo 'Network already started or needs configuration'".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "disable-libvirtd".to_string(),
                    description: "Disable libvirtd service".to_string(),
                    command: "sudo systemctl disable --now libvirtd".to_string(),
                },
            ],
            notes_for_user: "Libvirt networking configured. List networks: sudo virsh net-list --all".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("virt_network_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("virt_network.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking virtual networks".to_string(),
            goals: vec!["List all virtual networks".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-networks".to_string(),
                    description: "List virtual networks".to_string(),
                    command: "sudo virsh net-list --all 2>/dev/null || echo 'libvirt not running'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows all virtual networks and their status".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("virt_network_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_create_network_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("virt_network.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CreateNetwork"));

        Ok(ActionPlan {
            analysis: "Creating custom virtual network".to_string(),
            goals: vec!["Define new NAT network 'custom-net'".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-libvirtd".to_string(),
                    description: "Verify libvirtd is running".to_string(),
                    command: "systemctl is-active libvirtd".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "create-network-xml".to_string(),
                    description: "Create network definition".to_string(),
                    command: r#"cat << 'NETEOF' | sudo tee /tmp/custom-net.xml
<network>
  <name>custom-net</name>
  <forward mode='nat'/>
  <bridge name='virbr1' stp='on' delay='0'/>
  <ip address='192.168.100.1' netmask='255.255.255.0'>
    <dhcp>
      <range start='192.168.100.2' end='192.168.100.254'/>
    </dhcp>
  </ip>
</network>
NETEOF"#.to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "define-network".to_string(),
                    description: "Define network in libvirt".to_string(),
                    command: "sudo virsh net-define /tmp/custom-net.xml && rm /tmp/custom-net.xml".to_string(),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some("undefine-network".to_string()),
                    requires_confirmation: true,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "undefine-network".to_string(),
                    description: "Remove network definition".to_string(),
                    command: "sudo virsh net-undefine custom-net".to_string(),
                },
            ],
            notes_for_user: "Network 'custom-net' created. Start with: sudo virsh net-start custom-net".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("virt_network_create_network".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_start_network_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("virt_network.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("StartNetwork"));

        Ok(ActionPlan {
            analysis: "Starting virtual network".to_string(),
            goals: vec!["Start default network and set autostart".to_string()],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-default-network".to_string(),
                    description: "Verify default network exists".to_string(),
                    command: "sudo virsh net-info default".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "start-network".to_string(),
                    description: "Start default network".to_string(),
                    command: "sudo virsh net-start default || echo 'Network already running'".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("stop-network".to_string()),
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "autostart-network".to_string(),
                    description: "Enable network autostart".to_string(),
                    command: "sudo virsh net-autostart default".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "stop-network".to_string(),
                    description: "Stop default network".to_string(),
                    command: "sudo virsh net-destroy default".to_string(),
                },
            ],
            notes_for_user: "Default network started and set to autostart. VMs can now use networking.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("virt_network_start_network".to_string()),
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
        assert!(VirtNetworkRecipe::matches_request("setup libvirt network"));
        assert!(VirtNetworkRecipe::matches_request("create virtual network"));
        assert!(VirtNetworkRecipe::matches_request("start libvirt network"));
        assert!(!VirtNetworkRecipe::matches_request("what is libvirt"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "setup libvirt network".to_string());
        let plan = VirtNetworkRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
