// firewall.rs - UFW firewall management recipe
// Beta.153: Firewall configuration and management

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use serde_json;
use std::collections::HashMap;

pub struct FirewallRecipe;

#[derive(Debug, PartialEq)]
enum FirewallOperation {
    Install,
    Enable,
    Disable,
    AddRule,
    RemoveRule,
    Status,
    ListRules,
}

impl FirewallRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Exclude informational queries
        if input_lower.contains("what is")
            || input_lower.contains("tell me about")
            || input_lower.contains("explain")
        {
            return false;
        }

        // Firewall keywords
        let has_firewall = input_lower.contains("firewall")
            || input_lower.contains("ufw")
            || input_lower.contains("iptables");

        // Action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("enable")
            || input_lower.contains("disable")
            || input_lower.contains("allow")
            || input_lower.contains("deny")
            || input_lower.contains("block")
            || input_lower.contains("open")
            || input_lower.contains("close")
            || input_lower.contains("status")
            || input_lower.contains("check")
            || input_lower.contains("show")
            || input_lower.contains("list")
            || input_lower.contains("remove")
            || input_lower.contains("delete")
            || input_lower.contains("configure");

        // Port-related queries
        let has_port_context = input_lower.contains("port")
            || input_lower.contains("ssh")
            || input_lower.contains("http")
            || input_lower.contains("https");

        // Match if we have firewall context and either action or port context
        has_firewall && (has_action || has_port_context)
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_request = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = Self::detect_operation(user_request);

        match operation {
            FirewallOperation::Install => Self::build_install_plan(telemetry),
            FirewallOperation::Enable => Self::build_enable_plan(telemetry),
            FirewallOperation::Disable => Self::build_disable_plan(telemetry),
            FirewallOperation::AddRule => Self::build_add_rule_plan(user_request, telemetry),
            FirewallOperation::RemoveRule => Self::build_remove_rule_plan(user_request, telemetry),
            FirewallOperation::Status => Self::build_status_plan(telemetry),
            FirewallOperation::ListRules => Self::build_list_rules_plan(telemetry),
        }
    }

    fn detect_operation(user_input: &str) -> FirewallOperation {
        let input_lower = user_input.to_lowercase();

        // Check for install first
        if (input_lower.contains("install") || input_lower.contains("setup"))
            && input_lower.contains("firewall")
        {
            return FirewallOperation::Install;
        }

        // Check for enable/disable
        if input_lower.contains("enable") || input_lower.contains("turn on") {
            return FirewallOperation::Enable;
        }
        if input_lower.contains("disable") || input_lower.contains("turn off") {
            return FirewallOperation::Disable;
        }

        // Check for rule removal
        if input_lower.contains("remove")
            || input_lower.contains("delete")
            || input_lower.contains("close port")
            || input_lower.contains("block")
        {
            return FirewallOperation::RemoveRule;
        }

        // Check for adding rules
        if input_lower.contains("allow")
            || input_lower.contains("open")
            || input_lower.contains("permit")
            || input_lower.contains("add")
        {
            return FirewallOperation::AddRule;
        }

        // Check for listing rules
        if input_lower.contains("list") || input_lower.contains("show rules") {
            return FirewallOperation::ListRules;
        }

        // Default to status
        FirewallOperation::Status
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let mut warning = String::new();
        if !has_internet {
            warning = "⚠️ No internet detected. Package installation requires internet.\n\n"
                .to_string();
        }

        let necessary_checks = vec![NecessaryCheck {
            id: "check-ufw-installed".to_string(),
            description: "Check if UFW is already installed".to_string(),
            command: "pacman -Q ufw".to_string(),
            risk_level: RiskLevel::Info,
            required: false,
        }];

        let command_plan = vec![
            CommandStep {
                id: "install-ufw".to_string(),
                description: "Install UFW (Uncomplicated Firewall)".to_string(),
                command: "sudo pacman -S --noconfirm ufw".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-ufw".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-ufw".to_string(),
                description: "Verify UFW installation".to_string(),
                command: "ufw version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-status".to_string(),
                description: "Show initial UFW status".to_string(),
                command: "sudo ufw status verbose".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "uninstall-ufw".to_string(),
            description: "Uninstall UFW if not needed".to_string(),
            command: "sudo pacman -Rns ufw".to_string(),
        }];

        let notes_for_user = format!(
            "{}📦 Installing UFW Firewall\n\n\
             UFW (Uncomplicated Firewall) provides a simple interface for managing iptables firewall rules.\n\n\
             ⚠️ IMPORTANT: After installation, the firewall is DISABLED by default.\n\
             You must explicitly enable it and configure rules before it protects your system.\n\n\
             Common first steps:\n\
             1. Allow SSH before enabling: `annactl \"allow SSH through firewall\"`\n\
             2. Enable firewall: `annactl \"enable firewall\"`\n\
             3. Check status: `annactl \"firewall status\"`\n\n\
             Default policy: DENY incoming, ALLOW outgoing",
            warning
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("firewall.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("firewall_install".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to install UFW firewall for network protection. UFW provides a simple interface for managing firewall rules on Arch Linux.".to_string(),
            goals: vec![
                "Install UFW package".to_string(),
                "Verify installation".to_string(),
                "Show initial configuration".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_enable_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-ufw-installed".to_string(),
                description: "Check if UFW is installed".to_string(),
                command: "pacman -Q ufw".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-ssh-rule".to_string(),
                description: "Check if SSH is allowed in firewall".to_string(),
                command: "sudo ufw status | grep -i ssh".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "enable-firewall".to_string(),
                description: "Enable UFW firewall".to_string(),
                command: "sudo ufw --force enable".to_string(),
                risk_level: RiskLevel::High,
                rollback_id: Some("disable-firewall".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "enable-systemd-service".to_string(),
                description: "Enable UFW service to start on boot".to_string(),
                command: "sudo systemctl enable ufw".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("disable-systemd-service".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "show-status".to_string(),
                description: "Show firewall status after enabling".to_string(),
                command: "sudo ufw status verbose".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "disable-firewall".to_string(),
                description: "Disable UFW firewall".to_string(),
                command: "sudo ufw disable".to_string(),
            },
            RollbackStep {
                id: "disable-systemd-service".to_string(),
                description: "Disable UFW systemd service".to_string(),
                command: "sudo systemctl disable ufw".to_string(),
            },
        ];

        let notes_for_user = "⚠️ CRITICAL WARNING: Enabling Firewall\n\n\
             Enabling the firewall will activate all configured rules immediately.\n\n\
             🚨 SSH ACCESS WARNING:\n\
             If you are connected via SSH and SSH is NOT allowed in the firewall rules,\n\
             YOU WILL BE LOCKED OUT when the firewall enables!\n\n\
             Before enabling, ensure:\n\
             1. You have SSH rule configured: `sudo ufw allow ssh`\n\
             2. You have physical/console access in case of lockout\n\
             3. You understand your current firewall rules\n\n\
             Check current rules: `annactl \"list firewall rules\"`\n\
             Add SSH rule first: `annactl \"allow SSH through firewall\"`\n\n\
             If locked out:\n\
             1. Access system via physical console or recovery mode\n\
             2. Disable firewall: `sudo ufw disable`\n\
             3. Reconfigure rules properly\n\
             4. Re-enable firewall".to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("firewall.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("firewall_enable".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to enable UFW firewall. This will activate all configured rules and start protecting the system. High risk due to potential SSH lockout.".to_string(),
            goals: vec![
                "Enable UFW firewall".to_string(),
                "Enable UFW service on boot".to_string(),
                "Verify firewall is active".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_disable_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![NecessaryCheck {
            id: "check-ufw-status".to_string(),
            description: "Check current UFW status".to_string(),
            command: "sudo ufw status".to_string(),
            risk_level: RiskLevel::Info,
            required: false,
        }];

        let command_plan = vec![
            CommandStep {
                id: "disable-firewall".to_string(),
                description: "Disable UFW firewall".to_string(),
                command: "sudo ufw disable".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("re-enable-firewall".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "show-status".to_string(),
                description: "Show firewall status after disabling".to_string(),
                command: "sudo ufw status".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "re-enable-firewall".to_string(),
            description: "Re-enable UFW firewall".to_string(),
            command: "sudo ufw enable".to_string(),
        }];

        let notes_for_user = "⚠️ WARNING: Disabling Firewall\n\n\
             Disabling the firewall removes all active protection from your network interfaces.\n\
             Your system will accept all incoming connections until the firewall is re-enabled.\n\n\
             This does NOT delete your firewall rules - they remain configured and will\n\
             be reactivated when you enable the firewall again.\n\n\
             To re-enable: `annactl \"enable firewall\"`\n\
             To check status: `annactl \"firewall status\"`".to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("firewall.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("firewall_disable".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to disable UFW firewall. This will deactivate all firewall rules but preserve the configuration.".to_string(),
            goals: vec![
                "Disable UFW firewall".to_string(),
                "Verify firewall is inactive".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_add_rule_plan(
        user_request: &str,
        _telemetry: &HashMap<String, String>,
    ) -> Result<ActionPlan> {
        let input_lower = user_request.to_lowercase();

        // Try to detect what the user wants to allow
        let (service_name, port, protocol) = if input_lower.contains("ssh") {
            ("SSH", "22", "tcp")
        } else if input_lower.contains("http") && input_lower.contains("https") {
            ("HTTP and HTTPS", "80,443", "tcp")
        } else if input_lower.contains("https") {
            ("HTTPS", "443", "tcp")
        } else if input_lower.contains("http") {
            ("HTTP", "80", "tcp")
        } else {
            // Generic rule - extract port if mentioned
            let port_str = Self::extract_port(&input_lower).unwrap_or("SPECIFY_PORT");
            ("specified service", port_str, "tcp")
        };

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-ufw-installed".to_string(),
                description: "Check if UFW is installed".to_string(),
                command: "pacman -Q ufw".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-current-rules".to_string(),
                description: "Show current firewall rules".to_string(),
                command: "sudo ufw status numbered".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command = if port == "SPECIFY_PORT" {
            "sudo ufw allow SPECIFY_PORT/tcp".to_string()
        } else if port.contains(',') {
            // Multiple ports
            let ports: Vec<&str> = port.split(',').collect();
            ports
                .iter()
                .map(|p| format!("sudo ufw allow {}/{}", p, protocol))
                .collect::<Vec<_>>()
                .join(" && ")
        } else {
            format!("sudo ufw allow {}/{}", port, protocol)
        };

        let command_plan = vec![
            CommandStep {
                id: "add-rule".to_string(),
                description: format!("Allow {} through firewall", service_name),
                command: command.clone(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("remove-rule".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "show-rules".to_string(),
                description: "Show updated firewall rules".to_string(),
                command: "sudo ufw status numbered".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let remove_command = if port == "SPECIFY_PORT" {
            "sudo ufw delete allow SPECIFY_PORT/tcp".to_string()
        } else if port.contains(',') {
            let ports: Vec<&str> = port.split(',').collect();
            ports
                .iter()
                .map(|p| format!("sudo ufw delete allow {}/{}", p, protocol))
                .collect::<Vec<_>>()
                .join(" && ")
        } else {
            format!("sudo ufw delete allow {}/{}", port, protocol)
        };

        let rollback_plan = vec![RollbackStep {
            id: "remove-rule".to_string(),
            description: format!("Remove {} rule from firewall", service_name),
            command: remove_command,
        }];

        let notes_for_user = format!(
            "🔓 Adding Firewall Rule\n\n\
             This will allow incoming connections to {} (port {}).\n\n\
             The rule will be added to UFW configuration but will only take effect\n\
             if the firewall is enabled.\n\n\
             To enable firewall: `annactl \"enable firewall\"`\n\
             To see all rules: `annactl \"list firewall rules\"`\n\
             To remove this rule: `annactl \"remove port {} from firewall\"`\n\n\
             Common firewall commands:\n\
             - Allow SSH: `annactl \"allow SSH through firewall\"`\n\
             - Allow HTTP: `annactl \"allow HTTP through firewall\"`\n\
             - Allow HTTPS: `annactl \"allow HTTPS through firewall\"`",
            service_name, port, port
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("firewall.rs".to_string()),
        );
        other.insert(
            "detected_service".to_string(),
            serde_json::Value::String(service_name.to_string()),
        );
        other.insert(
            "detected_port".to_string(),
            serde_json::Value::String(port.to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("firewall_add_rule".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: format!(
                "User wants to allow {} (port {}) through the firewall. This will add a rule to permit incoming connections on this port.",
                service_name, port
            ),
            goals: vec![
                format!("Add firewall rule for {}", service_name),
                "Verify rule was added successfully".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_remove_rule_plan(
        user_request: &str,
        _telemetry: &HashMap<String, String>,
    ) -> Result<ActionPlan> {
        let input_lower = user_request.to_lowercase();

        // Try to detect what the user wants to remove
        let (service_name, port, protocol) = if input_lower.contains("ssh") {
            ("SSH", "22", "tcp")
        } else if input_lower.contains("https") {
            ("HTTPS", "443", "tcp")
        } else if input_lower.contains("http") {
            ("HTTP", "80", "tcp")
        } else {
            let port_str = Self::extract_port(&input_lower).unwrap_or("SPECIFY_PORT");
            ("specified service", port_str, "tcp")
        };

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-current-rules".to_string(),
                description: "Show current firewall rules".to_string(),
                command: "sudo ufw status numbered".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command = if port == "SPECIFY_PORT" {
            "sudo ufw delete allow SPECIFY_PORT/tcp".to_string()
        } else {
            format!("sudo ufw delete allow {}/{}", port, protocol)
        };

        let command_plan = vec![
            CommandStep {
                id: "remove-rule".to_string(),
                description: format!("Remove {} rule from firewall", service_name),
                command: command.clone(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("re-add-rule".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "show-rules".to_string(),
                description: "Show updated firewall rules".to_string(),
                command: "sudo ufw status numbered".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let re_add_command = if port == "SPECIFY_PORT" {
            "sudo ufw allow SPECIFY_PORT/tcp".to_string()
        } else {
            format!("sudo ufw allow {}/{}", port, protocol)
        };

        let rollback_plan = vec![RollbackStep {
            id: "re-add-rule".to_string(),
            description: format!("Re-add {} rule to firewall", service_name),
            command: re_add_command,
        }];

        let ssh_warning = if service_name == "SSH" {
            "\n\n🚨 SSH WARNING:\n\
             You are about to remove the SSH rule from the firewall!\n\
             If you are connected via SSH and the firewall is enabled,\n\
             you may lose access to this system!\n\n\
             Only proceed if:\n\
             1. You have physical/console access, OR\n\
             2. You plan to disable the firewall, OR\n\
             3. You are accessing via another method\n\n"
        } else {
            ""
        };

        let notes_for_user = format!(
            "🔒 Removing Firewall Rule{}\n\
             This will remove the rule allowing incoming connections to {} (port {}).\n\n\
             The rule will be removed from UFW configuration. If the firewall is enabled,\n\
             the change takes effect immediately.\n\n\
             To re-add this rule: `annactl \"allow {} through firewall\"`\n\
             To see all rules: `annactl \"list firewall rules\"`",
            ssh_warning, service_name, port, service_name
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("firewall.rs".to_string()),
        );
        other.insert(
            "detected_service".to_string(),
            serde_json::Value::String(service_name.to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("firewall_remove_rule".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: format!(
                "User wants to remove the {} rule from the firewall. This will block incoming connections on port {}.",
                service_name, port
            ),
            goals: vec![
                format!("Remove firewall rule for {}", service_name),
                "Verify rule was removed successfully".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "check-ufw-installed".to_string(),
                description: "Check if UFW is installed".to_string(),
                command: "pacman -Q ufw 2>/dev/null || echo 'UFW not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "ufw-status".to_string(),
                description: "Show UFW firewall status".to_string(),
                command: "sudo ufw status verbose 2>/dev/null || echo 'UFW not available'"
                    .to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "ufw-service-status".to_string(),
                description: "Check UFW systemd service status".to_string(),
                command: "systemctl is-enabled ufw 2>/dev/null || echo 'Service not enabled'"
                    .to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let notes_for_user = "ℹ️ Firewall Status Check\n\n\
             This shows the current state of the UFW firewall:\n\
             - Installation status\n\
             - Active/inactive state\n\
             - Current rules and policies\n\
             - Systemd service status\n\n\
             Common next steps:\n\
             - Install firewall: `annactl \"install firewall\"`\n\
             - Enable firewall: `annactl \"enable firewall\"`\n\
             - Add SSH rule: `annactl \"allow SSH through firewall\"`\n\
             - List all rules: `annactl \"list firewall rules\"`"
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("firewall.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("firewall_status".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to check firewall status. This is a read-only operation showing UFW installation, activation state, and configuration.".to_string(),
            goals: vec![
                "Check if UFW is installed".to_string(),
                "Show firewall active/inactive state".to_string(),
                "Display current rules and policies".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_list_rules_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![NecessaryCheck {
            id: "check-ufw-installed".to_string(),
            description: "Check if UFW is installed".to_string(),
            command: "pacman -Q ufw".to_string(),
            risk_level: RiskLevel::Info,
            required: true,
        }];

        let command_plan = vec![
            CommandStep {
                id: "list-rules-numbered".to_string(),
                description: "List all firewall rules with numbers".to_string(),
                command: "sudo ufw status numbered".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-verbose".to_string(),
                description: "Show detailed firewall status".to_string(),
                command: "sudo ufw status verbose".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let notes_for_user = "📋 Firewall Rules List\n\n\
             This shows all configured UFW rules with rule numbers for easy reference.\n\n\
             Rule numbers can be used to delete specific rules:\n\
             `sudo ufw delete [number]`\n\n\
             Common rule management:\n\
             - Add SSH: `annactl \"allow SSH through firewall\"`\n\
             - Remove rule: `annactl \"remove SSH from firewall\"`\n\
             - Check status: `annactl \"firewall status\"`"
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("firewall.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("firewall_list_rules".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to see all configured firewall rules. This is a read-only operation showing numbered rules for easy reference.".to_string(),
            goals: vec![
                "List all firewall rules with numbers".to_string(),
                "Show detailed firewall status".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn extract_port(input: &str) -> Option<&str> {
        // Simple port extraction - look for numbers
        for word in input.split_whitespace() {
            if word.chars().all(|c| c.is_ascii_digit()) {
                let port_num: u16 = word.parse().ok()?;
                if port_num > 0 && port_num <= 65535 {
                    return Some(word);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_firewall_requests() {
        // Should match
        assert!(FirewallRecipe::matches_request("install firewall"));
        assert!(FirewallRecipe::matches_request("setup ufw"));
        assert!(FirewallRecipe::matches_request("enable firewall"));
        assert!(FirewallRecipe::matches_request("disable ufw"));
        assert!(FirewallRecipe::matches_request("allow SSH through firewall"));
        assert!(FirewallRecipe::matches_request("open port 80 in firewall"));
        assert!(FirewallRecipe::matches_request("firewall status"));
        assert!(FirewallRecipe::matches_request("list firewall rules"));
        assert!(FirewallRecipe::matches_request("remove port 443 from firewall"));

        // Should NOT match
        assert!(!FirewallRecipe::matches_request("what is a firewall"));
        assert!(!FirewallRecipe::matches_request("tell me about ufw"));
        assert!(!FirewallRecipe::matches_request("install docker"));
        assert!(!FirewallRecipe::matches_request("how much RAM do I have"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            FirewallRecipe::detect_operation("install firewall"),
            FirewallOperation::Install
        );
        assert_eq!(
            FirewallRecipe::detect_operation("enable ufw"),
            FirewallOperation::Enable
        );
        assert_eq!(
            FirewallRecipe::detect_operation("disable firewall"),
            FirewallOperation::Disable
        );
        assert_eq!(
            FirewallRecipe::detect_operation("allow SSH through firewall"),
            FirewallOperation::AddRule
        );
        assert_eq!(
            FirewallRecipe::detect_operation("remove port 80 from firewall"),
            FirewallOperation::RemoveRule
        );
        assert_eq!(
            FirewallRecipe::detect_operation("firewall status"),
            FirewallOperation::Status
        );
        assert_eq!(
            FirewallRecipe::detect_operation("list firewall rules"),
            FirewallOperation::ListRules
        );
    }

    #[test]
    fn test_install_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = FirewallRecipe::build_install_plan(&telemetry).unwrap();

        assert_eq!(plan.goals.len(), 3);
        assert!(plan.analysis.contains("UFW"));
        assert!(plan.command_plan.len() >= 2);
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Medium);
        assert!(plan.notes_for_user.contains("DISABLED by default"));
    }

    #[test]
    fn test_enable_plan_high_risk() {
        let telemetry = HashMap::new();
        let plan = FirewallRecipe::build_enable_plan(&telemetry).unwrap();

        // Enable should be HIGH risk due to lockout potential
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::High);
        assert!(plan.command_plan[0].requires_confirmation);
        assert!(plan.notes_for_user.contains("SSH ACCESS WARNING"));
        assert!(plan.notes_for_user.contains("LOCKED OUT"));
    }

    #[test]
    fn test_add_ssh_rule() {
        let telemetry = HashMap::new();
        let plan = FirewallRecipe::build_add_rule_plan("allow SSH through firewall", &telemetry)
            .unwrap();

        assert!(plan.analysis.contains("SSH"));
        assert!(plan.command_plan[0].command.contains("22"));
        assert!(plan.command_plan[0].command.contains("tcp"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_remove_ssh_rule_warning() {
        let telemetry = HashMap::new();
        let plan =
            FirewallRecipe::build_remove_rule_plan("remove SSH from firewall", &telemetry)
                .unwrap();

        assert!(plan.notes_for_user.contains("SSH WARNING"));
        assert!(plan.notes_for_user.contains("lose access"));
    }

    #[test]
    fn test_status_plan() {
        let telemetry = HashMap::new();
        let plan = FirewallRecipe::build_status_plan(&telemetry).unwrap();

        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Info);
        assert!(!plan.command_plan[0].requires_confirmation);
        assert!(plan.notes_for_user.contains("Status Check"));
    }

    #[test]
    fn test_list_rules_plan() {
        let telemetry = HashMap::new();
        let plan = FirewallRecipe::build_list_rules_plan(&telemetry).unwrap();

        assert!(plan.command_plan[0].command.contains("numbered"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Info);
    }

    #[test]
    fn test_extract_port() {
        assert_eq!(FirewallRecipe::extract_port("open port 8080"), Some("8080"));
        assert_eq!(FirewallRecipe::extract_port("allow 443 through"), Some("443"));
        assert_eq!(FirewallRecipe::extract_port("no port here"), None);
        assert_eq!(FirewallRecipe::extract_port("invalid 99999"), None);
    }
}
