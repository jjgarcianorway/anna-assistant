//! Docker Installation Recipe
//!
//! Beta.151: Deterministic recipe for installing and enabling Docker
//!
//! This module generates a predictable, safe ActionPlan for:
//! - Installing Docker from official Arch repos
//! - Enabling and starting the Docker service
//! - Adding user to docker group (optional)
//! - Verifying installation

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use std::collections::HashMap;

/// Docker installation scenario detector
pub struct DockerRecipe;

impl DockerRecipe {
    /// Check if user request matches docker installation
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Match various phrasings
        (input_lower.contains("docker") || input_lower.contains("container"))
            && (input_lower.contains("install")
                || input_lower.contains("setup")
                || input_lower.contains("enable")
                || input_lower.contains("get"))
    }

    /// Generate docker installation ActionPlan
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        // Extract relevant telemetry
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let disk_space_gb = telemetry
            .get("disk_free_gb")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let user = telemetry.get("user").map(|s| s.as_str()).unwrap_or("user");

        // Necessary checks
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-internet".to_string(),
                description: "Verify internet connectivity".to_string(),
                command: "ping -c 1 archlinux.org".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-disk-space".to_string(),
                description: "Verify sufficient disk space (need ~1GB)".to_string(),
                command: "df -h /".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-docker-installed".to_string(),
                description: "Check if Docker is already installed".to_string(),
                command: "pacman -Q docker".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        // Command plan
        let mut command_plan = vec![
            CommandStep {
                id: "install-docker".to_string(),
                description: "Install Docker from official Arch repositories".to_string(),
                command: "sudo pacman -S --noconfirm docker".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("remove-docker".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "enable-docker-service".to_string(),
                description: "Enable Docker service to start on boot".to_string(),
                command: "sudo systemctl enable docker".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("disable-docker-service".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "start-docker-service".to_string(),
                description: "Start Docker service immediately".to_string(),
                command: "sudo systemctl start docker".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("stop-docker-service".to_string()),
                requires_confirmation: true,
            },
        ];

        // Optionally add user to docker group
        command_plan.push(CommandStep {
            id: "add-user-to-docker-group".to_string(),
            description: format!(
                "Add user '{}' to docker group (allows running docker without sudo)",
                user
            ),
            command: format!("sudo usermod -aG docker {}", user),
            risk_level: RiskLevel::Medium,
            rollback_id: Some("remove-user-from-docker-group".to_string()),
            requires_confirmation: true,
        });

        // Verification step
        command_plan.push(CommandStep {
            id: "verify-docker".to_string(),
            description: "Verify Docker installation and service status".to_string(),
            command: "docker --version && sudo systemctl status docker".to_string(),
            risk_level: RiskLevel::Info,
            rollback_id: None,
            requires_confirmation: false,
        });

        // Rollback plan
        let rollback_plan = vec![
            RollbackStep {
                id: "remove-user-from-docker-group".to_string(),
                description: format!("Remove user '{}' from docker group", user),
                command: format!("sudo gpasswd -d {} docker", user),
            },
            RollbackStep {
                id: "stop-docker-service".to_string(),
                description: "Stop Docker service".to_string(),
                command: "sudo systemctl stop docker".to_string(),
            },
            RollbackStep {
                id: "disable-docker-service".to_string(),
                description: "Disable Docker service from starting on boot".to_string(),
                command: "sudo systemctl disable docker".to_string(),
            },
            RollbackStep {
                id: "remove-docker".to_string(),
                description: "Uninstall Docker package".to_string(),
                command: "sudo pacman -Rns --noconfirm docker".to_string(),
            },
        ];

        // Build analysis
        let mut analysis_parts = vec![
            "User requests Docker installation and setup.".to_string(),
            format!("System has {:.1} GB free disk space.", disk_space_gb),
        ];

        if !has_internet {
            analysis_parts.push(
                "⚠️ WARNING: Internet connectivity not confirmed. Installation will fail without network access."
                    .to_string(),
            );
        }

        if disk_space_gb < 1.0 {
            analysis_parts.push(
                "⚠️ WARNING: Low disk space. Docker requires ~1GB for installation.".to_string(),
            );
        }

        let analysis = analysis_parts.join(" ");

        // Goals
        let goals = vec![
            "Install Docker container runtime from Arch repositories".to_string(),
            "Enable Docker service to start automatically on boot".to_string(),
            "Start Docker service immediately".to_string(),
            format!("Add user '{}' to docker group for non-root access", user),
            "Verify successful installation and service status".to_string(),
        ];

        // Notes for user
        let notes_for_user = format!(
            "This will install Docker and configure it for your user account. \
             After installation, you may need to log out and back in for group \
             membership to take effect. Run 'docker ps' to test non-root access.\n\n\
             Estimated disk usage: ~500-800 MB\n\
             Estimated time: 1-3 minutes"
        );

        // Metadata (Beta.151: Use PlanMeta for recipe tracking)
        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("docker.rs".to_string()),
        );
        other.insert(
            "disk_space_gb".to_string(),
            serde_json::Value::String(disk_space_gb.to_string()),
        );
        other.insert(
            "has_internet".to_string(),
            serde_json::Value::String(has_internet.to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("docker_install".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_docker_install_request() {
        assert!(DockerRecipe::matches_request("install docker"));
        assert!(DockerRecipe::matches_request("Install Docker please"));
        assert!(DockerRecipe::matches_request("setup docker and enable it"));
        assert!(DockerRecipe::matches_request("get docker running"));
        assert!(DockerRecipe::matches_request("install container runtime"));

        // Should not match
        assert!(!DockerRecipe::matches_request("what is docker"));
        assert!(!DockerRecipe::matches_request("show me docker status"));
    }

    #[test]
    fn test_build_docker_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());
        telemetry.insert("disk_free_gb".to_string(), "50.0".to_string());
        telemetry.insert("user".to_string(), "testuser".to_string());

        let plan = DockerRecipe::build_plan(&telemetry).unwrap();

        // Verify structure
        assert_eq!(plan.goals.len(), 5);
        assert_eq!(plan.necessary_checks.len(), 3);
        assert_eq!(plan.command_plan.len(), 5); // install, enable, start, add user, verify
        assert_eq!(plan.rollback_plan.len(), 4);

        // Verify commands contain expected keywords
        assert!(plan.command_plan[0].command.contains("pacman -S"));
        assert!(plan.command_plan[0].command.contains("docker"));
        assert!(plan.command_plan[1].command.contains("systemctl enable"));
        assert!(plan.command_plan[2].command.contains("systemctl start"));
        assert!(plan.command_plan[3].command.contains("usermod -aG docker"));

        // Verify metadata
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "docker_install");
        assert_eq!(plan.meta.llm_version, "deterministic_recipe_v1");
    }

    #[test]
    fn test_low_disk_space_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());
        telemetry.insert("disk_free_gb".to_string(), "0.5".to_string());
        telemetry.insert("user".to_string(), "testuser".to_string());

        let plan = DockerRecipe::build_plan(&telemetry).unwrap();

        // Should contain warning about low disk space
        assert!(plan.analysis.contains("WARNING"));
        assert!(plan.analysis.contains("Low disk space"));
    }

    #[test]
    fn test_no_internet_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "false".to_string());
        telemetry.insert("disk_free_gb".to_string(), "50.0".to_string());
        telemetry.insert("user".to_string(), "testuser".to_string());

        let plan = DockerRecipe::build_plan(&telemetry).unwrap();

        // Should contain warning about internet
        assert!(plan.analysis.contains("WARNING"));
        assert!(plan.analysis.contains("Internet connectivity"));
    }
}
