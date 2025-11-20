//! SSH Configuration Recipe
//!
//! Beta.153: Deterministic recipe for SSH setup and configuration
//!
//! This module generates safe ActionPlans for:
//! - Installing OpenSSH server and client
//! - Enabling and starting sshd service
//! - Generating SSH keys (client-side)
//! - Basic SSH configuration guidance
//! - Showing connection instructions

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use std::collections::HashMap;

/// SSH configuration scenario detector
pub struct SshRecipe;

/// SSH operation types
#[derive(Debug, Clone, PartialEq)]
enum SshOperation {
    InstallServer,    // "install ssh server" / "setup sshd"
    GenerateKeys,     // "generate ssh keys" / "create ssh key"
    CheckStatus,      // "is ssh running" / "ssh status"
    ShowConfig,       // "show ssh configuration"
}

impl SshRecipe {
    /// Check if user request matches SSH operations
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Exclude informational queries
        if input_lower.contains("what is ssh") || input_lower.contains("explain ssh") {
            return false;
        }

        // Must mention SSH
        let has_ssh_context = input_lower.contains("ssh")
            || input_lower.contains("sshd")
            || input_lower.contains("openssh");

        // Need action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("enable")
            || input_lower.contains("start")
            || input_lower.contains("generate")
            || input_lower.contains("create")
            || input_lower.contains("configure")
            || input_lower.contains("status")
            || input_lower.contains("check")
            || input_lower.contains("show")
            || input_lower.contains("is")
            || input_lower.contains("running");

        has_ssh_context && has_action
    }

    /// Generate SSH operation ActionPlan
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let operation = Self::detect_operation(
            telemetry
                .get("user_request")
                .map(|s| s.as_str())
                .unwrap_or(""),
        );

        match operation {
            SshOperation::InstallServer => Self::build_install_server_plan(telemetry),
            SshOperation::GenerateKeys => Self::build_generate_keys_plan(telemetry),
            SshOperation::CheckStatus => Self::build_check_status_plan(),
            SshOperation::ShowConfig => Self::build_show_config_plan(),
        }
    }

    fn detect_operation(user_input: &str) -> SshOperation {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("generate") || input_lower.contains("create")
            && (input_lower.contains("key") || input_lower.contains("keypair"))
        {
            SshOperation::GenerateKeys
        } else if input_lower.contains("status")
            || input_lower.contains("check")
            || input_lower.contains("is")
                && (input_lower.contains("running") || input_lower.contains("active"))
        {
            SshOperation::CheckStatus
        } else if input_lower.contains("show") || input_lower.contains("config") {
            SshOperation::ShowConfig
        } else {
            // Default to server installation
            SshOperation::InstallServer
        }
    }

    fn build_install_server_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let user = telemetry.get("user").map(|s| s.as_str()).unwrap_or("user");
        let hostname = telemetry.get("hostname").map(|s| s.as_str()).unwrap_or("localhost");

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-internet".to_string(),
                description: "Verify internet connectivity".to_string(),
                command: "ping -c 1 archlinux.org".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-openssh-installed".to_string(),
                description: "Check if OpenSSH is already installed".to_string(),
                command: "pacman -Q openssh".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-sshd-status".to_string(),
                description: "Check if sshd service is running".to_string(),
                command: "systemctl is-active sshd 2>&1 || echo 'inactive'".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-openssh".to_string(),
                description: "Install OpenSSH server and client".to_string(),
                command: "sudo pacman -S --needed --noconfirm openssh".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("remove-openssh".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "enable-sshd".to_string(),
                description: "Enable sshd service to start on boot".to_string(),
                command: "sudo systemctl enable sshd".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("disable-sshd".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "start-sshd".to_string(),
                description: "Start sshd service immediately".to_string(),
                command: "sudo systemctl start sshd".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("stop-sshd".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-sshd".to_string(),
                description: "Verify sshd service is running".to_string(),
                command: "systemctl status sshd --no-pager -l".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-listening-port".to_string(),
                description: "Show SSH listening port".to_string(),
                command: "ss -tlnp | grep sshd || echo 'Not listening yet'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "stop-sshd".to_string(),
                description: "Stop sshd service".to_string(),
                command: "sudo systemctl stop sshd".to_string(),
            },
            RollbackStep {
                id: "disable-sshd".to_string(),
                description: "Disable sshd service".to_string(),
                command: "sudo systemctl disable sshd".to_string(),
            },
            RollbackStep {
                id: "remove-openssh".to_string(),
                description: "Uninstall OpenSSH".to_string(),
                command: "sudo pacman -Rns --noconfirm openssh".to_string(),
            },
        ];

        let mut analysis_parts = vec![
            "User requests SSH server installation and configuration.".to_string(),
            "This will install OpenSSH and enable remote access to this system.".to_string(),
        ];

        if !has_internet {
            analysis_parts.push(
                "⚠️ WARNING: Internet connectivity not confirmed. Installation requires network access."
                    .to_string(),
            );
        }

        let analysis = analysis_parts.join(" ");

        let goals = vec![
            "Install OpenSSH server and client".to_string(),
            "Enable sshd service to start on boot".to_string(),
            "Start sshd service immediately".to_string(),
            "Verify SSH server is running and listening".to_string(),
        ];

        let notes_for_user = format!(
            "⚠️ SECURITY: SSH opens remote access to your system!\n\n\
             This will install and start the SSH server (sshd).\n\n\
             After installation:\n\
             • SSH server will be accessible on default port 22\n\
             • Password authentication is enabled by default\n\
             • Root login is typically disabled by default\n\n\
             To connect from another machine:\n\
             ssh {}@{}\n\n\
             IMPORTANT Security Hardening:\n\n\
             1. Use SSH keys instead of passwords:\n\
                ssh-keygen -t ed25519 -C \"{}@{}\"\n\
                ssh-copy-id {}@{}\n\n\
             2. Edit /etc/ssh/sshd_config for better security:\n\
                sudo nano /etc/ssh/sshd_config\n\n\
                Recommended changes:\n\
                PasswordAuthentication no          # Disable password login (use keys only)\n\
                PermitRootLogin no                 # Disable root login\n\
                Port 2222                          # Change from default port 22\n\
                AllowUsers {}                      # Only allow specific users\n\n\
             3. Restart sshd after config changes:\n\
                sudo systemctl restart sshd\n\n\
             4. Configure firewall (if using ufw):\n\
                sudo ufw allow 22/tcp              # Or your custom port\n\
                sudo ufw enable\n\n\
             5. Monitor failed login attempts:\n\
                journalctl -u sshd -f | grep -i failed\n\n\
             Risk: MEDIUM - Opens network service, potential security risk if not hardened",
            user, hostname, user, hostname, user, hostname, user
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "ssh_install_server",
        )
    }

    fn build_generate_keys_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user = telemetry.get("user").map(|s| s.as_str()).unwrap_or("user");
        let user_home = telemetry.get("home").map(|s| s.as_str()).unwrap_or("~");
        let hostname = telemetry.get("hostname").map(|s| s.as_str()).unwrap_or("localhost");

        let necessary_checks = vec![NecessaryCheck {
            id: "check-existing-keys".to_string(),
            description: "Check for existing SSH keys".to_string(),
            command: format!("ls -la {}/.ssh/id_* 2>/dev/null || echo 'No existing keys'", user_home),
            risk_level: RiskLevel::Info,
            required: false,
        }];

        let command_plan = vec![
            CommandStep {
                id: "create-ssh-dir".to_string(),
                description: "Create .ssh directory if it doesn't exist".to_string(),
                command: format!("mkdir -p {}/.ssh && chmod 700 {}/.ssh", user_home, user_home),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "generate-ed25519-key".to_string(),
                description: "Generate Ed25519 SSH key pair (recommended)".to_string(),
                command: format!(
                    "ssh-keygen -t ed25519 -C \"{}@{}\" -f {}/.ssh/id_ed25519 -N ''",
                    user, hostname, user_home
                ),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-keys".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "set-permissions".to_string(),
                description: "Set correct permissions on SSH keys".to_string(),
                command: format!(
                    "chmod 600 {}/.ssh/id_ed25519 && chmod 644 {}/.ssh/id_ed25519.pub",
                    user_home, user_home
                ),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-public-key".to_string(),
                description: "Display public key (for adding to remote servers)".to_string(),
                command: format!("cat {}/.ssh/id_ed25519.pub", user_home),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-fingerprint".to_string(),
                description: "Show key fingerprint".to_string(),
                command: format!("ssh-keygen -lf {}/.ssh/id_ed25519.pub", user_home),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "remove-keys".to_string(),
            description: "Remove generated SSH keys".to_string(),
            command: format!("rm -f {}/.ssh/id_ed25519 {}/.ssh/id_ed25519.pub", user_home, user_home),
        }];

        let analysis = "User requests SSH key generation. This creates a new Ed25519 key pair for secure authentication.".to_string();

        let goals = vec![
            "Create .ssh directory with correct permissions".to_string(),
            "Generate Ed25519 SSH key pair".to_string(),
            "Set correct permissions on keys".to_string(),
            "Display public key for remote server setup".to_string(),
        ];

        let notes_for_user = format!(
            "SSH Key Generation (Ed25519 - Recommended)\n\n\
             This generates a new SSH key pair:\n\
             • Private key: {}/.ssh/id_ed25519 (NEVER share this!)\n\
             • Public key: {}/.ssh/id_ed25519.pub (safe to share)\n\n\
             Why Ed25519?\n\
             • More secure than RSA\n\
             • Smaller key size (256-bit)\n\
             • Faster to generate and use\n\
             • Recommended by modern security standards\n\n\
             To use this key:\n\n\
             1. Copy public key to remote server:\n\
                ssh-copy-id -i {}/.ssh/id_ed25519.pub user@remote-host\n\n\
             2. Or manually add to remote server's authorized_keys:\n\
                cat {}/.ssh/id_ed25519.pub | ssh user@remote-host 'cat >> ~/.ssh/authorized_keys'\n\n\
             3. Test connection:\n\
                ssh user@remote-host\n\n\
             4. For GitHub/GitLab:\n\
                cat {}/.ssh/id_ed25519.pub\n\
                # Copy output and add to your account's SSH keys\n\n\
             Security best practices:\n\
             • ⚠️ NEVER share your private key (id_ed25519)\n\
             • Back up your private key securely\n\
             • Use different keys for different purposes\n\
             • Consider using ssh-agent to manage keys\n\n\
             If you need a key with a passphrase (more secure):\n\
             ssh-keygen -t ed25519 -C \"{}@{}\" -f {}/.ssh/id_ed25519_secure\n\
             # You'll be prompted for a passphrase\n\n\
             Risk: LOW - Creates local files only, no network exposure",
            user_home, user_home, user_home, user_home, user_home, user, hostname, user_home
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "ssh_generate_keys",
        )
    }

    fn build_check_status_plan() -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "check-sshd-installed".to_string(),
                description: "Check if OpenSSH is installed".to_string(),
                command: "pacman -Q openssh || echo 'OpenSSH not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-sshd-running".to_string(),
                description: "Check if sshd service is running".to_string(),
                command: "systemctl is-active sshd && echo 'sshd is RUNNING' || echo 'sshd is NOT running'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-sshd-enabled".to_string(),
                description: "Check if sshd is enabled on boot".to_string(),
                command: "systemctl is-enabled sshd && echo 'sshd will start on boot' || echo 'sshd will NOT start on boot'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-sshd-status".to_string(),
                description: "Show detailed sshd status".to_string(),
                command: "systemctl status sshd --no-pager -l || echo 'sshd service not found'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-listening-ports".to_string(),
                description: "Show SSH listening ports".to_string(),
                command: "ss -tlnp | grep sshd || echo 'sshd not listening'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = "User requests SSH service status check. This is a read-only operation.".to_string();

        let goals = vec![
            "Check if OpenSSH is installed".to_string(),
            "Check if sshd service is running".to_string(),
            "Check if sshd is enabled on boot".to_string(),
            "Show listening ports and connections".to_string(),
        ];

        let notes_for_user = "SSH Status Check (Read-Only)\n\n\
             This checks:\n\
             • OpenSSH package installation\n\
             • sshd service running status\n\
             • sshd boot enable status\n\
             • Listening ports (usually port 22)\n\n\
             If sshd is not installed:\n\
             sudo pacman -S openssh\n\n\
             If sshd is installed but not running:\n\
             sudo systemctl start sshd\n\n\
             If sshd is not enabled on boot:\n\
             sudo systemctl enable sshd\n\n\
             To view SSH configuration:\n\
             cat /etc/ssh/sshd_config\n\n\
             To view recent SSH login attempts:\n\
             journalctl -u sshd -n 50\n\n\
             To see active SSH connections:\n\
             ss -tn | grep :22\n\n\
             Risk: INFO - Read-only status checks".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "ssh_check_status",
        )
    }

    fn build_show_config_plan() -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "show-sshd-config".to_string(),
                description: "Show sshd server configuration".to_string(),
                command: "cat /etc/ssh/sshd_config | grep -v '^#' | grep -v '^$' || cat /etc/ssh/sshd_config".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-ssh-client-config".to_string(),
                description: "Show SSH client configuration".to_string(),
                command: "cat /etc/ssh/ssh_config | grep -v '^#' | grep -v '^$' | head -30 || echo 'Using defaults'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-authorized-keys".to_string(),
                description: "Check if authorized_keys file exists".to_string(),
                command: "ls -la ~/.ssh/authorized_keys 2>/dev/null || echo 'No authorized_keys file'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = "User requests SSH configuration display. This shows current SSH server and client settings.".to_string();

        let goals = vec![
            "Display sshd server configuration".to_string(),
            "Display SSH client configuration".to_string(),
            "Check for authorized_keys setup".to_string(),
        ];

        let notes_for_user = "SSH Configuration Files\n\n\
             Key configuration files:\n\
             • /etc/ssh/sshd_config - SSH server (daemon) configuration\n\
             • /etc/ssh/ssh_config - SSH client configuration (system-wide)\n\
             • ~/.ssh/config - SSH client configuration (user-specific)\n\
             • ~/.ssh/authorized_keys - Public keys allowed to login as this user\n\n\
             Common sshd_config settings:\n\
             Port 22                          # Listening port (change for security)\n\
             PermitRootLogin no               # Disable root login (recommended)\n\
             PasswordAuthentication no        # Require SSH keys (recommended)\n\
             PubkeyAuthentication yes         # Enable key-based auth\n\
             AllowUsers user1 user2           # Whitelist specific users\n\
             MaxAuthTries 3                   # Limit authentication attempts\n\n\
             To edit sshd configuration:\n\
             sudo nano /etc/ssh/sshd_config\n\n\
             After editing, test configuration:\n\
             sudo sshd -t\n\n\
             Then restart sshd:\n\
             sudo systemctl restart sshd\n\n\
             ⚠️ WARNING: Be careful when editing SSH config!\n\
             Test changes before disconnecting to avoid locking yourself out.\n\n\
             To create user-specific SSH config:\n\
             nano ~/.ssh/config\n\n\
             Example entry:\n\
             Host myserver\n\
                 HostName 192.168.1.100\n\
                 User admin\n\
                 Port 2222\n\
                 IdentityFile ~/.ssh/id_ed25519\n\n\
             Then connect with: ssh myserver\n\n\
             Risk: INFO - Read-only configuration display".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "ssh_show_config",
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
            serde_json::Value::String("ssh.rs".to_string()),
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
    fn test_matches_ssh_requests() {
        // Install server
        assert!(SshRecipe::matches_request("install ssh server"));
        assert!(SshRecipe::matches_request("setup sshd"));
        assert!(SshRecipe::matches_request("enable ssh"));

        // Generate keys
        assert!(SshRecipe::matches_request("generate ssh keys"));
        assert!(SshRecipe::matches_request("create ssh keypair"));

        // Status checks
        assert!(SshRecipe::matches_request("is ssh running"));
        assert!(SshRecipe::matches_request("ssh status"));
        assert!(SshRecipe::matches_request("check sshd"));

        // Config
        assert!(SshRecipe::matches_request("show ssh configuration"));

        // Should not match
        assert!(!SshRecipe::matches_request("what is ssh"));
        assert!(!SshRecipe::matches_request("explain ssh protocol"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            SshRecipe::detect_operation("install ssh server"),
            SshOperation::InstallServer
        );
        assert_eq!(
            SshRecipe::detect_operation("generate ssh keys"),
            SshOperation::GenerateKeys
        );
        assert_eq!(
            SshRecipe::detect_operation("is ssh running"),
            SshOperation::CheckStatus
        );
        assert_eq!(
            SshRecipe::detect_operation("show ssh config"),
            SshOperation::ShowConfig
        );
    }

    #[test]
    fn test_install_server_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install ssh".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());
        telemetry.insert("user".to_string(), "testuser".to_string());
        telemetry.insert("hostname".to_string(), "testhost".to_string());

        let plan = SshRecipe::build_plan(&telemetry).unwrap();

        // Verify structure
        assert_eq!(plan.necessary_checks.len(), 3);
        assert_eq!(plan.command_plan.len(), 5);
        assert_eq!(plan.rollback_plan.len(), 3);

        // Verify install command
        assert!(plan.command_plan[0].command.contains("pacman -S"));
        assert!(plan.command_plan[0].command.contains("openssh"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Medium);

        // Verify metadata
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "ssh_install_server");
        assert_eq!(plan.meta.llm_version, "deterministic_recipe_v1");
    }

    #[test]
    fn test_generate_keys_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "generate ssh keys".to_string());
        telemetry.insert("user".to_string(), "testuser".to_string());
        telemetry.insert("home".to_string(), "/home/testuser".to_string());
        telemetry.insert("hostname".to_string(), "testhost".to_string());

        let plan = SshRecipe::build_plan(&telemetry).unwrap();

        // Verify key generation command
        assert!(plan.command_plan[1].command.contains("ssh-keygen"));
        assert!(plan.command_plan[1].command.contains("ed25519"));
        assert_eq!(plan.command_plan[1].risk_level, RiskLevel::Low);

        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "ssh_generate_keys");
    }

    #[test]
    fn test_check_status_is_read_only() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "is ssh running".to_string());

        let plan = SshRecipe::build_plan(&telemetry).unwrap();

        // All commands should be INFO level
        for cmd in &plan.command_plan {
            assert_eq!(cmd.risk_level, RiskLevel::Info);
            assert!(!cmd.requires_confirmation);
        }

        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "ssh_check_status");
    }

    #[test]
    fn test_show_config_is_read_only() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "show ssh config".to_string());

        let plan = SshRecipe::build_plan(&telemetry).unwrap();

        // All commands should be INFO level
        for cmd in &plan.command_plan {
            assert_eq!(cmd.risk_level, RiskLevel::Info);
        }

        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "ssh_show_config");
    }

    #[test]
    fn test_security_warnings_present() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install ssh".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());
        telemetry.insert("user".to_string(), "testuser".to_string());
        telemetry.insert("hostname".to_string(), "testhost".to_string());

        let plan = SshRecipe::build_plan(&telemetry).unwrap();

        // Should have security warnings
        assert!(plan.notes_for_user.contains("⚠️ SECURITY"));
        assert!(plan.notes_for_user.contains("remote access"));
        assert!(plan.notes_for_user.contains("PasswordAuthentication"));
    }
}
