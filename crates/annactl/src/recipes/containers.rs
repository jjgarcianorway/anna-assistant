// Beta.163: Container Management Recipe (Podman focus)
use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct ContainersRecipe;

#[derive(Debug, PartialEq)]
enum ContainersOperation {
    Install,
    CheckStatus,
    ConfigureRootless,
    ListTools,
}

impl ContainersOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("rootless") || input_lower.contains("configure") {
            ContainersOperation::ConfigureRootless
        } else if input_lower.contains("check") || input_lower.contains("status") {
            ContainersOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            ContainersOperation::ListTools
        } else {
            ContainersOperation::Install
        }
    }
}

impl ContainersRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("podman") || input_lower.contains("container")
            || input_lower.contains("buildah") || input_lower.contains("skopeo");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        // Exclude docker-specific queries (handled by docker.rs and docker_compose.rs)
        let is_docker = input_lower.contains("docker") && !input_lower.contains("podman");
        has_context && has_action && !is_info_only && !is_docker
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = ContainersOperation::detect(user_input);
        match operation {
            ContainersOperation::Install => Self::build_install_plan(telemetry),
            ContainersOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            ContainersOperation::ConfigureRootless => Self::build_configure_rootless_plan(telemetry),
            ContainersOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("containers.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));

        Ok(ActionPlan {
            analysis: "Installing Podman container tools".to_string(),
            goals: vec![
                "Install Podman".to_string(),
                "Install Buildah and Skopeo (build + image tools)".to_string(),
            ],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "install-podman".to_string(),
                    description: "Install Podman ecosystem".to_string(),
                    command: "sudo pacman -S --needed --noconfirm podman buildah skopeo".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("remove-podman".to_string()),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "remove-podman".to_string(),
                    description: "Remove Podman tools".to_string(),
                    command: "sudo pacman -Rns --noconfirm podman buildah skopeo".to_string(),
                },
            ],
            notes_for_user: "Podman installed. Rootless containers work out-of-the-box. Use: podman run, podman build, podman-compose".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("containers_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("containers.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking container tools".to_string(),
            goals: vec!["List installed container tools and running containers".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-container-tools".to_string(),
                    description: "List container tools".to_string(),
                    command: "pacman -Q podman buildah skopeo 2>/dev/null || echo 'No container tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "list-containers".to_string(),
                    description: "List running containers".to_string(),
                    command: "podman ps 2>/dev/null || echo 'No containers running'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed container tools and running containers".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("containers_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_configure_rootless_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("containers.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ConfigureRootless"));

        Ok(ActionPlan {
            analysis: "Configuring rootless containers".to_string(),
            goals: vec![
                "Configure user namespaces".to_string(),
                "Enable podman user service".to_string(),
            ],
            necessary_checks: vec![
                NecessaryCheck {
                    id: "check-podman".to_string(),
                    description: "Verify Podman is installed".to_string(),
                    command: "which podman".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                },
            ],
            command_plan: vec![
                CommandStep {
                    id: "configure-subuids".to_string(),
                    description: "Configure subuid/subgid ranges".to_string(),
                    command: "grep -q $USER /etc/subuid || (echo \"$USER:100000:65536\" | sudo tee -a /etc/subuid && echo \"$USER:100000:65536\" | sudo tee -a /etc/subgid)".to_string(),
                    risk_level: RiskLevel::Medium,
                    rollback_id: None,
                    requires_confirmation: true,
                },
                CommandStep {
                    id: "enable-user-service".to_string(),
                    description: "Enable podman user service".to_string(),
                    command: "systemctl --user enable --now podman.socket".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: Some("disable-user-service".to_string()),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: "disable-user-service".to_string(),
                    description: "Disable podman user service".to_string(),
                    command: "systemctl --user disable --now podman.socket".to_string(),
                },
            ],
            notes_for_user: "Rootless containers configured. Test with: podman run hello-world".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("containers_configure_rootless".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("containers.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available container tools".to_string(),
            goals: vec!["List available tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Available:\n- Podman (official) - Docker-compatible container runtime\n- Buildah (official) - Container image builder\n- Skopeo (official) - Container image operations'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Container tools for Arch Linux (rootless by default)".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("containers_list_tools".to_string()),
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
        assert!(ContainersRecipe::matches_request("install podman"));
        assert!(ContainersRecipe::matches_request("setup container tools"));
        assert!(ContainersRecipe::matches_request("configure rootless containers"));
        assert!(!ContainersRecipe::matches_request("install docker")); // Should use docker.rs
        assert!(!ContainersRecipe::matches_request("what is podman"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install podman".to_string());
        let plan = ContainersRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
