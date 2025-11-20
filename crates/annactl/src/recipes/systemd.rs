//! Systemd Service Management Recipe
//!
//! Beta.152: Deterministic recipe for managing systemd services
//!
//! This module generates safe ActionPlans for:
//! - Enabling/disabling services
//! - Starting/stopping/restarting services
//! - Checking service status
//! - Viewing service logs

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use std::collections::HashMap;

/// Systemd service management scenario detector
pub struct SystemdRecipe;

/// Service action types
#[derive(Debug, Clone, PartialEq)]
enum ServiceAction {
    Enable,
    Disable,
    Start,
    Stop,
    Restart,
    Status,
    Logs,
}

impl SystemdRecipe {
    /// Check if user request matches systemd service management
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Must mention a service-related action
        let has_action = input_lower.contains("enable")
            || input_lower.contains("disable")
            || input_lower.contains("start")
            || input_lower.contains("stop")
            || input_lower.contains("restart")
            || input_lower.contains("status")
            || input_lower.contains("log");

        // And must mention service or daemon or systemd or specific service names
        let has_service_context = input_lower.contains("service")
            || input_lower.contains("daemon")
            || input_lower.contains("systemd")
            || input_lower.contains("systemctl")
            // Common services
            || input_lower.contains("network")
            || input_lower.contains("bluetooth")
            || input_lower.contains("sshd")
            || input_lower.contains("docker")
            || input_lower.contains("nginx")
            || input_lower.contains("apache")
            || input_lower.contains("mysql")
            || input_lower.contains("postgresql")
            || input_lower.contains("firewall");

        has_action && has_service_context
    }

    /// Generate systemd service management ActionPlan
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        // For now, generate a template plan that requires service name substitution
        // In practice, the planner would extract the service name from the user request
        // and pass it through telemetry or as a parameter

        let service_name = telemetry
            .get("service_name")
            .map(|s| s.as_str())
            .unwrap_or("<service-name>");

        let action = Self::detect_action(
            telemetry
                .get("user_request")
                .map(|s| s.as_str())
                .unwrap_or(""),
        );

        match action {
            ServiceAction::Enable => Self::build_enable_plan(service_name),
            ServiceAction::Disable => Self::build_disable_plan(service_name),
            ServiceAction::Start => Self::build_start_plan(service_name),
            ServiceAction::Stop => Self::build_stop_plan(service_name),
            ServiceAction::Restart => Self::build_restart_plan(service_name),
            ServiceAction::Status => Self::build_status_plan(service_name),
            ServiceAction::Logs => Self::build_logs_plan(service_name),
        }
    }

    fn detect_action(user_input: &str) -> ServiceAction {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("enable") {
            ServiceAction::Enable
        } else if input_lower.contains("disable") {
            ServiceAction::Disable
        } else if input_lower.contains("restart") {
            ServiceAction::Restart
        } else if input_lower.contains("start") {
            ServiceAction::Start
        } else if input_lower.contains("stop") {
            ServiceAction::Stop
        } else if input_lower.contains("log") {
            ServiceAction::Logs
        } else if input_lower.contains("status") {
            ServiceAction::Status
        } else {
            // Default to status for informational queries
            ServiceAction::Status
        }
    }

    fn build_enable_plan(service_name: &str) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-service-exists".to_string(),
                description: format!("Verify {} service unit exists", service_name),
                command: format!("systemctl list-unit-files | grep -w '{}.service'", service_name),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-current-status".to_string(),
                description: format!("Check current status of {}", service_name),
                command: format!("systemctl is-enabled {} 2>&1 || echo 'disabled'", service_name),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "enable-service".to_string(),
                description: format!("Enable {} service to start automatically on boot", service_name),
                command: format!("sudo systemctl enable {}", service_name),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("disable-service".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-enabled".to_string(),
                description: format!("Verify {} service is enabled", service_name),
                command: format!("systemctl is-enabled {}", service_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "disable-service".to_string(),
            description: format!("Disable {} service", service_name),
            command: format!("sudo systemctl disable {}", service_name),
        }];

        let analysis = format!(
            "User requests to enable {} service. This will configure the service to start \
             automatically on system boot, but does not start it immediately.",
            service_name
        );

        let goals = vec![
            format!("Enable {} service to start on boot", service_name),
            format!("Verify {} service is properly enabled", service_name),
        ];

        let notes_for_user = format!(
            "This will enable {} to start automatically on system boot.\n\n\
             To also start the service immediately, run:\n\
             sudo systemctl start {}\n\n\
             To enable AND start in one command:\n\
             sudo systemctl enable --now {}\n\n\
             To check service status:\n\
             systemctl status {}\n\n\
             Risk: MEDIUM - Modifies system boot behavior",
            service_name, service_name, service_name, service_name
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "service_enable",
        )
    }

    fn build_disable_plan(service_name: &str) -> Result<ActionPlan> {
        let necessary_checks = vec![NecessaryCheck {
            id: "check-current-status".to_string(),
            description: format!("Check if {} service is currently enabled", service_name),
            command: format!("systemctl is-enabled {} 2>&1", service_name),
            risk_level: RiskLevel::Info,
            required: true,
        }];

        let command_plan = vec![
            CommandStep {
                id: "disable-service".to_string(),
                description: format!("Disable {} service from starting on boot", service_name),
                command: format!("sudo systemctl disable {}", service_name),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("enable-service".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-disabled".to_string(),
                description: format!("Verify {} service is disabled", service_name),
                command: format!("systemctl is-enabled {} 2>&1 || echo 'Successfully disabled'", service_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "enable-service".to_string(),
            description: format!("Re-enable {} service", service_name),
            command: format!("sudo systemctl enable {}", service_name),
        }];

        let analysis = format!(
            "User requests to disable {} service. This prevents the service from starting \
             automatically on boot, but does not stop it if currently running.",
            service_name
        );

        let goals = vec![
            format!("Disable {} service from auto-starting on boot", service_name),
            format!("Verify {} service is properly disabled", service_name),
        ];

        let notes_for_user = format!(
            "This will prevent {} from starting automatically on system boot.\n\n\
             Note: This does not stop the service if it's currently running.\n\
             To also stop the service immediately, run:\n\
             sudo systemctl stop {}\n\n\
             To disable AND stop in one command:\n\
             sudo systemctl disable --now {}\n\n\
             Risk: MEDIUM - Modifies system boot behavior",
            service_name, service_name, service_name
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "service_disable",
        )
    }

    fn build_start_plan(service_name: &str) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-service-exists".to_string(),
                description: format!("Verify {} service unit exists", service_name),
                command: format!("systemctl list-unit-files | grep -w '{}.service'", service_name),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-running-status".to_string(),
                description: format!("Check if {} service is already running", service_name),
                command: format!("systemctl is-active {} 2>&1 || echo 'inactive'", service_name),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "start-service".to_string(),
                description: format!("Start {} service immediately", service_name),
                command: format!("sudo systemctl start {}", service_name),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("stop-service".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-running".to_string(),
                description: format!("Verify {} service is running", service_name),
                command: format!("systemctl is-active {} && systemctl status {} --no-pager -l", service_name, service_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "stop-service".to_string(),
            description: format!("Stop {} service", service_name),
            command: format!("sudo systemctl stop {}", service_name),
        }];

        let analysis = format!(
            "User requests to start {} service. This starts the service immediately, \
             but does not enable it to start automatically on boot.",
            service_name
        );

        let goals = vec![
            format!("Start {} service immediately", service_name),
            format!("Verify {} service is running properly", service_name),
        ];

        let notes_for_user = format!(
            "This will start {} service immediately.\n\n\
             Note: This does not enable the service to start on boot.\n\
             To also enable on boot, run:\n\
             sudo systemctl enable {}\n\n\
             To start AND enable in one command:\n\
             sudo systemctl enable --now {}\n\n\
             To check service logs:\n\
             journalctl -u {} -n 50\n\n\
             Risk: MEDIUM - Starts system service immediately",
            service_name, service_name, service_name, service_name
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "service_start",
        )
    }

    fn build_stop_plan(service_name: &str) -> Result<ActionPlan> {
        let necessary_checks = vec![NecessaryCheck {
            id: "check-running-status".to_string(),
            description: format!("Check if {} service is currently running", service_name),
            command: format!("systemctl is-active {} 2>&1", service_name),
            risk_level: RiskLevel::Info,
            required: true,
        }];

        let command_plan = vec![
            CommandStep {
                id: "stop-service".to_string(),
                description: format!("Stop {} service immediately", service_name),
                command: format!("sudo systemctl stop {}", service_name),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("start-service".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-stopped".to_string(),
                description: format!("Verify {} service is stopped", service_name),
                command: format!("systemctl is-active {} 2>&1 || echo 'Successfully stopped'", service_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "start-service".to_string(),
            description: format!("Restart {} service", service_name),
            command: format!("sudo systemctl start {}", service_name),
        }];

        let analysis = format!(
            "User requests to stop {} service. This stops the service immediately, \
             but does not disable it from starting on boot.",
            service_name
        );

        let goals = vec![
            format!("Stop {} service immediately", service_name),
            format!("Verify {} service is no longer running", service_name),
        ];

        let notes_for_user = format!(
            "This will stop {} service immediately.\n\n\
             Note: This does not disable the service from starting on boot.\n\
             To also disable on boot, run:\n\
             sudo systemctl disable {}\n\n\
             To stop AND disable in one command:\n\
             sudo systemctl disable --now {}\n\n\
             ⚠️ WARNING: Stopping critical system services may impact system functionality.\n\n\
             Risk: MEDIUM - Stops system service immediately",
            service_name, service_name, service_name
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "service_stop",
        )
    }

    fn build_restart_plan(service_name: &str) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-service-exists".to_string(),
                description: format!("Verify {} service unit exists", service_name),
                command: format!("systemctl list-unit-files | grep -w '{}.service'", service_name),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-current-status".to_string(),
                description: format!("Check current status of {}", service_name),
                command: format!("systemctl status {} --no-pager -l || true", service_name),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "restart-service".to_string(),
                description: format!("Restart {} service (stop then start)", service_name),
                command: format!("sudo systemctl restart {}", service_name),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-running".to_string(),
                description: format!("Verify {} service restarted successfully", service_name),
                command: format!("systemctl is-active {} && systemctl status {} --no-pager -l", service_name, service_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "restore-service".to_string(),
            description: format!("If restart fails, check service logs"),
            command: format!("journalctl -u {} -n 50 --no-pager", service_name),
        }];

        let analysis = format!(
            "User requests to restart {} service. This will stop and start the service, \
             applying any configuration changes.",
            service_name
        );

        let goals = vec![
            format!("Restart {} service to apply configuration changes", service_name),
            format!("Verify {} service is running after restart", service_name),
        ];

        let notes_for_user = format!(
            "This will restart {} service (stop then start).\n\n\
             Use cases for restarting:\n\
             • After modifying configuration files\n\
             • To clear service state/cache\n\
             • To fix service issues\n\n\
             To reload configuration without stopping (if supported):\n\
             sudo systemctl reload {}\n\n\
             To check service logs after restart:\n\
             journalctl -u {} -n 50 -f\n\n\
             Risk: MEDIUM - Temporarily interrupts service",
            service_name, service_name, service_name
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "service_restart",
        )
    }

    fn build_status_plan(service_name: &str) -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "check-service-status".to_string(),
                description: format!("Show detailed status of {} service", service_name),
                command: format!("systemctl status {} --no-pager -l", service_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-enabled-status".to_string(),
                description: format!("Check if {} service is enabled on boot", service_name),
                command: format!("systemctl is-enabled {} 2>&1 || echo 'not enabled'", service_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-active-status".to_string(),
                description: format!("Check if {} service is currently active", service_name),
                command: format!("systemctl is-active {} 2>&1 || echo 'not active'", service_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = format!(
            "User requests status information for {} service. This is a read-only operation.",
            service_name
        );

        let goals = vec![format!(
            "Display current status and state of {} service",
            service_name
        )];

        let notes_for_user = format!(
            "This will show the current status of {} service.\n\n\
             Status information includes:\n\
             • Active state (running/stopped/failed)\n\
             • Enabled state (starts on boot or not)\n\
             • Main PID and memory usage\n\
             • Recent log entries\n\n\
             To view full logs:\n\
             journalctl -u {} -n 100\n\n\
             To follow logs in real-time:\n\
             journalctl -u {} -f\n\n\
             Risk: INFO - Read-only, no system changes",
            service_name, service_name, service_name
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "service_status",
        )
    }

    fn build_logs_plan(service_name: &str) -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "show-recent-logs".to_string(),
                description: format!("Show recent logs for {} service", service_name),
                command: format!("journalctl -u {} -n 50 --no-pager", service_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-errors".to_string(),
                description: format!("Check for errors in {} service logs", service_name),
                command: format!(
                    "journalctl -u {} -p err -n 20 --no-pager || echo 'No errors found'",
                    service_name
                ),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = format!(
            "User requests logs for {} service. This is a read-only operation showing recent log entries.",
            service_name
        );

        let goals = vec![
            format!("Display recent log entries for {} service", service_name),
            format!("Check for errors in {} service logs", service_name),
        ];

        let notes_for_user = format!(
            "This will show recent logs for {} service.\n\n\
             Additional log commands:\n\n\
             Show last 100 lines:\n\
             journalctl -u {} -n 100\n\n\
             Follow logs in real-time:\n\
             journalctl -u {} -f\n\n\
             Show logs from today:\n\
             journalctl -u {} --since today\n\n\
             Show logs from last hour:\n\
             journalctl -u {} --since '1 hour ago'\n\n\
             Show only errors:\n\
             journalctl -u {} -p err\n\n\
             Risk: INFO - Read-only, no system changes",
            service_name, service_name, service_name, service_name, service_name, service_name
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "service_logs",
        )
    }

    fn build_action_plan(
        analysis: String,
        goals: Vec<String>,
        necessary_checks: Vec<NecessaryCheck>,
        command_plan: Vec<CommandStep>,
        rollback_plan: Vec<RollbackStep>,
        notes_for_user: String,
        template_name: &str,
    ) -> Result<ActionPlan> {
        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("systemd.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some(template_name.to_string()),
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
    fn test_matches_systemd_requests() {
        // Enable service
        assert!(SystemdRecipe::matches_request("enable NetworkManager service"));
        assert!(SystemdRecipe::matches_request("enable bluetooth"));

        // Start service
        assert!(SystemdRecipe::matches_request("start sshd service"));
        assert!(SystemdRecipe::matches_request("start docker daemon"));

        // Restart service
        assert!(SystemdRecipe::matches_request("restart nginx"));
        assert!(SystemdRecipe::matches_request("restart network service"));

        // Status checks
        assert!(SystemdRecipe::matches_request("status of bluetooth service"));
        assert!(SystemdRecipe::matches_request("check mysql daemon status"));

        // Stop service
        assert!(SystemdRecipe::matches_request("stop apache service"));
        assert!(SystemdRecipe::matches_request("disable firewall"));

        // Should not match
        assert!(!SystemdRecipe::matches_request("what is systemd"));
        assert!(!SystemdRecipe::matches_request("install a service"));
        assert!(!SystemdRecipe::matches_request("how do I create a service"));
    }

    #[test]
    fn test_action_detection() {
        assert_eq!(
            SystemdRecipe::detect_action("enable NetworkManager"),
            ServiceAction::Enable
        );
        assert_eq!(
            SystemdRecipe::detect_action("start sshd service"),
            ServiceAction::Start
        );
        assert_eq!(
            SystemdRecipe::detect_action("restart bluetooth"),
            ServiceAction::Restart
        );
        assert_eq!(
            SystemdRecipe::detect_action("stop nginx"),
            ServiceAction::Stop
        );
        assert_eq!(
            SystemdRecipe::detect_action("disable firewall"),
            ServiceAction::Disable
        );
        assert_eq!(
            SystemdRecipe::detect_action("status of docker"),
            ServiceAction::Status
        );
        assert_eq!(
            SystemdRecipe::detect_action("show logs for sshd"),
            ServiceAction::Logs
        );
    }

    #[test]
    fn test_build_enable_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("service_name".to_string(), "sshd".to_string());
        telemetry.insert("user_request".to_string(), "enable sshd".to_string());

        let plan = SystemdRecipe::build_plan(&telemetry).unwrap();

        // Verify structure
        assert_eq!(plan.necessary_checks.len(), 2);
        assert_eq!(plan.command_plan.len(), 2);
        assert_eq!(plan.rollback_plan.len(), 1);

        // Verify commands
        assert!(plan.command_plan[0].command.contains("systemctl enable"));
        assert!(plan.command_plan[0].command.contains("sshd"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Medium);
        assert!(plan.command_plan[0].requires_confirmation);

        // Verify metadata
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "service_enable");
        assert_eq!(plan.meta.llm_version, "deterministic_recipe_v1");
    }

    #[test]
    fn test_build_start_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("service_name".to_string(), "nginx".to_string());
        telemetry.insert("user_request".to_string(), "start nginx".to_string());

        let plan = SystemdRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.command_plan[0].command.contains("systemctl start"));
        assert!(plan.command_plan[0].command.contains("nginx"));
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "service_start");
    }

    #[test]
    fn test_build_status_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("service_name".to_string(), "bluetooth".to_string());
        telemetry.insert("user_request".to_string(), "status bluetooth".to_string());

        let plan = SystemdRecipe::build_plan(&telemetry).unwrap();

        // Status is read-only, so INFO risk
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Info);
        assert!(!plan.command_plan[0].requires_confirmation);
        assert!(plan.command_plan[0].command.contains("systemctl status"));
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "service_status");
    }

    #[test]
    fn test_restart_plan_has_proper_risk() {
        let mut telemetry = HashMap::new();
        telemetry.insert("service_name".to_string(), "docker".to_string());
        telemetry.insert("user_request".to_string(), "restart docker".to_string());

        let plan = SystemdRecipe::build_plan(&telemetry).unwrap();

        // Restart requires confirmation
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Medium);
        assert!(plan.command_plan[0].requires_confirmation);
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "service_restart");
    }
}
