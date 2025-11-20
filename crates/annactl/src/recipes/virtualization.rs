// Beta.163: Virtualization Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct VirtualizationRecipe;

#[derive(Debug, PartialEq)]
enum VirtualizationOperation {
    Install,
    CheckStatus,
    EnableKvm,
    ListTools,
}

impl VirtualizationOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("enable kvm") || input_lower.contains("setup kvm") {
            VirtualizationOperation::EnableKvm
        } else if input_lower.contains("check") || input_lower.contains("status") {
            VirtualizationOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            VirtualizationOperation::ListTools
        } else {
            VirtualizationOperation::Install
        }
    }
}

impl VirtualizationRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("virtualbox") || input_lower.contains("qemu")
            || input_lower.contains("kvm") || input_lower.contains("virtualization")
            || input_lower.contains("virtual machine") || input_lower.contains("vm");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("enable");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = VirtualizationOperation::detect(user_input);
        match operation {
            VirtualizationOperation::Install => Self::build_install_plan(telemetry),
            VirtualizationOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            VirtualizationOperation::EnableKvm => Self::build_enable_kvm_plan(telemetry),
            VirtualizationOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("virtualbox") || input_lower.contains("vbox") { "virtualbox" }
        else if input_lower.contains("qemu") || input_lower.contains("kvm") { "qemu" }
        else { "qemu" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, packages) = match tool {
            "virtualbox" => ("VirtualBox", vec!["virtualbox", "virtualbox-host-modules-arch"]),
            "qemu" => ("QEMU/KVM", vec!["qemu-full", "libvirt", "virt-manager"]),
            _ => ("QEMU/KVM", vec!["qemu-full", "libvirt", "virt-manager"]),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("virtualization.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = format!("sudo pacman -S --needed --noconfirm {}", packages.join(" "));
        let notes = if tool == "qemu" {
            "QEMU/KVM installed. Enable libvirt service: sudo systemctl enable --now libvirtd. Add user to libvirt group: sudo usermod -aG libvirt $USER".to_string()
        } else {
            "VirtualBox installed. Add user to vboxusers group: sudo usermod -aG vboxusers $USER, then reboot.".to_string()
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} virtualization", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-cpu-virt".to_string(),
                    description: "Check CPU virtualization support".to_string(),
                    command: "grep -E '(vmx|svm)' /proc/cpuinfo && echo 'Virtualization supported' || echo 'Virtualization NOT supported'".to_string(),
                    risk_level: RiskLevel::Info,
                    required: false,
                },
            ],
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
                    command: format!("sudo pacman -Rns --noconfirm {}", packages.join(" ")),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("virtualization_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("virtualization.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking virtualization tools".to_string(),
            goals: vec!["List installed virtualization tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "check-virt-support".to_string(),
                    description: "Check virtualization support".to_string(),
                    command: "grep -E '(vmx|svm)' /proc/cpuinfo && echo 'CPU supports virtualization' || echo 'No virtualization support'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "list-virt-tools".to_string(),
                    description: "List virtualization tools".to_string(),
                    command: "pacman -Q virtualbox qemu-full libvirt 2>/dev/null || echo 'No virtualization tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows virtualization support and installed tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("virtualization_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_enable_kvm_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("virtualization.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("EnableKvm"));

        Ok(ActionPlan {
            analysis: "Enabling KVM virtualization".to_string(),
            goals: vec![
                "Load KVM kernel modules".to_string(),
                "Add user to kvm group".to_string(),
            ],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-virt-support".to_string(),
                    description: "Verify CPU virtualization support".to_string(),
                    command: "grep -E '(vmx|svm)' /proc/cpuinfo".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "load-kvm-module".to_string(),
                    description: "Load KVM kernel modules".to_string(),
                    command: "sudo modprobe kvm && sudo modprobe kvm_intel || sudo modprobe kvm_amd".to_string(),
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some("unload-kvm-module".to_string()),
                    requires_confirmation: true,
                },
                CommandStep {
                    id: "add-user-kvm".to_string(),
                    description: "Add user to kvm group".to_string(),
                    command: "sudo usermod -aG kvm $USER".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "unload-kvm-module".to_string(),
                    description: "Unload KVM kernel modules".to_string(),
                    command: "sudo modprobe -r kvm_intel kvm_amd kvm".to_string(),
                },
            ],
            notes_for_user: "KVM enabled. Log out and back in for group changes to take effect.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("virtualization_enable_kvm".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("virtualization.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available virtualization tools".to_string(),
            goals: vec!["List available tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Available:\n- QEMU/KVM (official) - Linux native virtualization\n- VirtualBox (official) - User-friendly VM manager\n- virt-manager (official) - GUI for libvirt/KVM'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Virtualization tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("virtualization_list_tools".to_string()),
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
        assert!(VirtualizationRecipe::matches_request("install virtualbox"));
        assert!(VirtualizationRecipe::matches_request("setup qemu"));
        assert!(VirtualizationRecipe::matches_request("enable kvm"));
        assert!(!VirtualizationRecipe::matches_request("what is kvm"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install qemu".to_string());
        let plan = VirtualizationRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
