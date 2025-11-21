//! Web Server Recipe - Nginx Installation and Configuration
//!
//! Beta.156: Nginx web server installation, site configuration, and SSL setup
//!
//! This recipe handles Nginx installation on Arch Linux, including:
//! - Installation from official repos
//! - Service management
//! - Virtual host configuration
//! - SSL/TLS setup guidance
//!
//! Operations:
//! - Install: Install Nginx and configure basic setup
//! - CheckStatus: Verify Nginx installation and service status
//! - CreateSite: Create a new virtual host configuration
//! - EnableSsl: Configure SSL/TLS with Let's Encrypt guidance

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RollbackStep, RiskLevel};
use anyhow::Result;
use std::collections::HashMap;

pub struct WebServerRecipe;

#[derive(Debug, PartialEq)]
enum WebServerOperation {
    Install,
    CheckStatus,
    CreateSite,
    EnableSsl,
}

impl WebServerRecipe {
    /// Check if user request matches web server patterns
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Web server related keywords
        let has_webserver_context = input_lower.contains("nginx")
            || input_lower.contains("web server")
            || input_lower.contains("webserver")
            || input_lower.contains("http server")
            || (input_lower.contains("server") && input_lower.contains("web"));

        // Action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("configure")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("create")
            || input_lower.contains("add")
            || input_lower.contains("enable")
            || input_lower.contains("ssl")
            || input_lower.contains("https")
            || input_lower.contains("site")
            || input_lower.contains("vhost")
            || input_lower.contains("virtual host")
            || input_lower.contains("running");

        // Exclude informational-only queries
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;

        has_webserver_context && has_action && !is_info_only
    }

    /// Detect specific operation from user request
    fn detect_operation(user_input: &str) -> WebServerOperation {
        let input_lower = user_input.to_lowercase();

        // CheckStatus operation
        if input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("verify")
            || input_lower.contains("is") && (input_lower.contains("running") || input_lower.contains("install"))
        {
            return WebServerOperation::CheckStatus;
        }

        // CreateSite operation
        if input_lower.contains("create") && (input_lower.contains("site") || input_lower.contains("vhost"))
            || input_lower.contains("add") && input_lower.contains("site")
            || input_lower.contains("new") && (input_lower.contains("site") || input_lower.contains("virtual host"))
            || input_lower.contains("configure") && input_lower.contains("site")
        {
            return WebServerOperation::CreateSite;
        }

        // EnableSsl operation
        if input_lower.contains("ssl")
            || input_lower.contains("https")
            || input_lower.contains("tls")
            || input_lower.contains("certbot")
            || input_lower.contains("letsencrypt")
            || input_lower.contains("let's encrypt")
            || input_lower.contains("certificate")
        {
            return WebServerOperation::EnableSsl;
        }

        // Default to Install
        WebServerOperation::Install
    }

    /// Build ActionPlan based on detected operation
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_request = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = Self::detect_operation(user_request);

        match operation {
            WebServerOperation::Install => Self::build_install_plan(telemetry),
            WebServerOperation::CheckStatus => Self::build_status_plan(telemetry),
            WebServerOperation::CreateSite => Self::build_create_site_plan(telemetry),
            WebServerOperation::EnableSsl => Self::build_ssl_plan(telemetry),
        }
    }

    /// Install Nginx
    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "check-nginx-installed".to_string(),
                description: "Check if Nginx is already installed".to_string(),
                command: "pacman -Qi nginx 2>/dev/null || echo 'Not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-nginx".to_string(),
                description: "Install Nginx from official Arch repos".to_string(),
                command: "sudo pacman -S --needed --noconfirm nginx".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "create-nginx-html-directory".to_string(),
                description: "Create directory for website files".to_string(),
                command: "sudo mkdir -p /usr/share/nginx/html".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "test-nginx-config-syntax".to_string(),
                description: "Test Nginx configuration syntax".to_string(),
                command: "sudo nginx -t".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "enable-nginx-service".to_string(),
                description: "Enable Nginx service to start on boot".to_string(),
                command: "sudo systemctl enable nginx".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "start-nginx-service".to_string(),
                description: "Start Nginx service".to_string(),
                command: "sudo systemctl start nginx".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-nginx-running".to_string(),
                description: "Verify Nginx is running".to_string(),
                command: "sudo systemctl status nginx".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "test-http-access".to_string(),
                description: "Test HTTP access to localhost".to_string(),
                command: "curl -I http://localhost".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-internet-connectivity-nginx".to_string(),
                description: "Check internet connectivity".to_string(),
                command: "ping -c 1 archlinux.org".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-port-80-available".to_string(),
                description: "Check if port 80 is available".to_string(),
                command: "sudo ss -tlnp | grep ':80 ' || echo 'Port 80 is available'".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "stop-nginx-service".to_string(),
                description: "Stop Nginx service".to_string(),
                command: "sudo systemctl stop nginx".to_string(),
            },
            RollbackStep {
                id: "disable-nginx-service".to_string(),
                description: "Disable Nginx service".to_string(),
                command: "sudo systemctl disable nginx".to_string(),
            },
            RollbackStep {
                id: "remove-nginx-package".to_string(),
                description: "Remove Nginx package".to_string(),
                command: "sudo pacman -Rns nginx".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("webserver.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("install"));

        Ok(ActionPlan {
            analysis: "Nginx web server installation on Arch Linux. This will:\n\
                      1. Install nginx package from official repos\n\
                      2. Configure basic directory structure\n\
                      3. Enable and start Nginx service\n\
                      4. Verify web server is accessible on port 80\n\
                      \n\
                      Nginx will serve content from /usr/share/nginx/html by default. \
                      Configuration files are in /etc/nginx/."
                .to_string(),
            goals: vec![
                "Install Nginx package".to_string(),
                "Configure directory structure".to_string(),
                "Enable and start Nginx service".to_string(),
                "Verify web server is accessible".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Nginx will be installed and started on port 80. The default page will be accessible at:\n\
                            http://localhost\n\
                            \n\
                            Next steps:\n\
                            - Add site content: /usr/share/nginx/html/\n\
                            - Configure virtual hosts: /etc/nginx/sites-available/\n\
                            - Enable SSL: see annactl 'enable nginx SSL'\n\
                            - View logs: /var/log/nginx/"
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("webserver_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    /// Check Nginx status
    fn build_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "check-nginx-package".to_string(),
                description: "Check if Nginx package is installed".to_string(),
                command: "pacman -Qi nginx".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-nginx-version".to_string(),
                description: "Check Nginx version".to_string(),
                command: "nginx -v".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-nginx-service-status".to_string(),
                description: "Check Nginx service status".to_string(),
                command: "systemctl status nginx".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-nginx-boot-enabled".to_string(),
                description: "Check if Nginx service is enabled on boot".to_string(),
                command: "systemctl is-enabled nginx".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "test-nginx-syntax".to_string(),
                description: "Test Nginx configuration syntax".to_string(),
                command: "sudo nginx -t".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "list-nginx-ports".to_string(),
                description: "List Nginx listening ports".to_string(),
                command: "sudo ss -tlnp | grep nginx".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "test-http-access-nginx".to_string(),
                description: "Test HTTP access to localhost".to_string(),
                command: "curl -I http://localhost".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-nginx-processes".to_string(),
                description: "Show Nginx worker processes".to_string(),
                command: "ps aux | grep '[n]ginx'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("webserver.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("check_status"));

        Ok(ActionPlan {
            analysis: "Checking Nginx web server installation and service status. This will verify:\n\
                      - Package installation and version\n\
                      - Service status and autostart configuration\n\
                      - Configuration syntax validity\n\
                      - Listening ports and worker processes\n\
                      - HTTP accessibility"
                .to_string(),
            goals: vec![
                "Verify Nginx is installed".to_string(),
                "Check service status".to_string(),
                "Validate configuration".to_string(),
                "Test HTTP accessibility".to_string(),
            ],
            necessary_checks: vec![],
            command_plan,
            rollback_plan: vec![],
            notes_for_user: "This will check your Nginx installation status. \
                            All commands are read-only and safe to execute."
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("webserver_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    /// Create a new site configuration
    fn build_create_site_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "create-nginx-site-directories".to_string(),
                description: "Create sites-available and sites-enabled directories".to_string(),
                command: "sudo mkdir -p /etc/nginx/sites-available /etc/nginx/sites-enabled".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "create-nginx-document-root".to_string(),
                description: "Create document root for new site".to_string(),
                command: "sudo mkdir -p /var/www/example.com/html".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "set-nginx-permissions".to_string(),
                description: "Set appropriate permissions on document root".to_string(),
                command: "sudo chown -R $USER:$USER /var/www/example.com".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "create-nginx-index-html".to_string(),
                description: "Create example index.html".to_string(),
                command: r#"cat > /var/www/example.com/html/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Welcome to example.com</title>
</head>
<body>
    <h1>Success! example.com is working!</h1>
    <p>This is a test page for your Nginx virtual host.</p>
</body>
</html>
EOF
"#.to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "create-nginx-vhost-config".to_string(),
                description: "Create Nginx virtual host configuration".to_string(),
                command: r#"sudo bash -c "cat > /etc/nginx/sites-available/example.com << 'EOF'
server {
    listen 80;
    listen [::]:80;

    server_name example.com www.example.com;
    root /var/www/example.com/html;
    index index.html index.htm;

    location / {
        try_files \$uri \$uri/ =404;
    }

    access_log /var/log/nginx/example.com.access.log;
    error_log /var/log/nginx/example.com.error.log;
}
EOF
""#.to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "enable-nginx-site".to_string(),
                description: "Enable the site by creating symlink".to_string(),
                command: "sudo ln -sf /etc/nginx/sites-available/example.com /etc/nginx/sites-enabled/".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "update-nginx-conf-includes".to_string(),
                description: "Update main nginx.conf to include sites-enabled".to_string(),
                command: r#"sudo bash -c 'grep -q "include /etc/nginx/sites-enabled" /etc/nginx/nginx.conf || sed -i "/http {/a\\    include /etc/nginx/sites-enabled/*;" /etc/nginx/nginx.conf'"#.to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "test-nginx-config".to_string(),
                description: "Test Nginx configuration syntax".to_string(),
                command: "sudo nginx -t".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "reload-nginx".to_string(),
                description: "Reload Nginx to apply changes".to_string(),
                command: "sudo systemctl reload nginx".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-nginx-running".to_string(),
                description: "Check Nginx is running".to_string(),
                command: "systemctl is-active nginx".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "disable-nginx-site".to_string(),
                description: "Disable the site".to_string(),
                command: "sudo rm -f /etc/nginx/sites-enabled/example.com".to_string(),
            },
            RollbackStep {
                id: "remove-nginx-vhost-config".to_string(),
                description: "Remove site configuration".to_string(),
                command: "sudo rm -f /etc/nginx/sites-available/example.com".to_string(),
            },
            RollbackStep {
                id: "remove-nginx-document-root".to_string(),
                description: "Remove document root".to_string(),
                command: "sudo rm -rf /var/www/example.com".to_string(),
            },
            RollbackStep {
                id: "reload-nginx-rollback".to_string(),
                description: "Reload Nginx".to_string(),
                command: "sudo systemctl reload nginx".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("webserver.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("create_site"));

        Ok(ActionPlan {
            analysis: "Creating a new Nginx virtual host (site) configuration. This example creates:\n\
                      - Domain: example.com (customize before running)\n\
                      - Document root: /var/www/example.com/html\n\
                      - Configuration: /etc/nginx/sites-available/example.com\n\
                      - Enabled via symlink in sites-enabled/\n\
                      \n\
                      The configuration follows Debian/Ubuntu style site management for easy organization."
                .to_string(),
            goals: vec![
                "Create document root directory".to_string(),
                "Create virtual host configuration".to_string(),
                "Enable the site".to_string(),
                "Reload Nginx configuration".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "⚠️  Before running, replace 'example.com' with your actual domain name in the commands.\n\
                            \n\
                            After creating the site:\n\
                            1. Point your domain's DNS to this server's IP\n\
                            2. Test: curl -H 'Host: example.com' http://localhost\n\
                            3. Add SSL: see annactl 'enable nginx SSL'\n\
                            \n\
                            Site files: /var/www/example.com/html/\n\
                            Configuration: /etc/nginx/sites-available/example.com\n\
                            Logs: /var/log/nginx/example.com.*.log"
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("webserver_create_site".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    /// Enable SSL/TLS with Let's Encrypt
    fn build_ssl_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "install-certbot".to_string(),
                description: "Install certbot and certbot-nginx plugin".to_string(),
                command: "sudo pacman -S --needed --noconfirm certbot certbot-nginx".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "check-firewall-ports".to_string(),
                description: "Check firewall allows ports 80 and 443".to_string(),
                command: "sudo ufw status | grep -E '80|443' || echo 'Firewall check (skip if not using UFW)'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-dns".to_string(),
                description: "Verify DNS points to this server".to_string(),
                command: "echo 'Run: dig example.com +short' to verify DNS before proceeding".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "test-nginx-config-ssl".to_string(),
                description: "Test Nginx configuration".to_string(),
                command: "sudo nginx -t".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "obtain-ssl-certificate-dryrun".to_string(),
                description: "Obtain SSL certificate with certbot (DRY RUN)".to_string(),
                command: "sudo certbot --nginx --dry-run -d example.com -d www.example.com".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-nginx-running-ssl".to_string(),
                description: "Check Nginx is running".to_string(),
                command: "systemctl is-active nginx".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-port-80-accessible".to_string(),
                description: "Check port 80 is accessible".to_string(),
                command: "curl -I http://localhost".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("webserver.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("enable_ssl"));

        Ok(ActionPlan {
            analysis: "Setting up SSL/TLS for Nginx using Let's Encrypt (certbot). This recipe:\n\
                      1. Installs certbot and certbot-nginx plugin\n\
                      2. Verifies prerequisites (DNS, firewall, Nginx config)\n\
                      3. Runs certbot in DRY RUN mode to test\n\
                      \n\
                      Note: The actual certificate issuance is NOT automated here to prevent \
                      rate limiting issues. After dry run succeeds, you'll need to run the \
                      real command manually."
                .to_string(),
            goals: vec![
                "Install certbot and Nginx plugin".to_string(),
                "Verify SSL prerequisites".to_string(),
                "Test certificate issuance (dry run)".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan: vec![],
            notes_for_user: "⚠️  SSL Setup Requirements:\n\
                            \n\
                            Before running:\n\
                            1. Your domain must point to this server's public IP\n\
                            2. Port 80 and 443 must be open in firewall\n\
                            3. Nginx must be serving your site on port 80\n\
                            4. Replace 'example.com' with your actual domain\n\
                            \n\
                            After dry run succeeds, run the REAL command:\n\
                            sudo certbot --nginx -d example.com -d www.example.com\n\
                            \n\
                            Certbot will:\n\
                            - Obtain SSL certificate from Let's Encrypt\n\
                            - Modify Nginx config to enable HTTPS\n\
                            - Set up automatic renewal\n\
                            \n\
                            Certificate auto-renewal: systemctl enable certbot-renew.timer"
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("webserver_ssl".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_webserver_requests() {
        // Install queries
        assert!(WebServerRecipe::matches_request("install nginx"));
        assert!(WebServerRecipe::matches_request("setup web server"));
        assert!(WebServerRecipe::matches_request("install nginx web server"));

        // Status queries
        assert!(WebServerRecipe::matches_request("check nginx status"));
        assert!(WebServerRecipe::matches_request("is nginx running"));
        assert!(WebServerRecipe::matches_request("verify web server installation"));

        // Create site queries
        assert!(WebServerRecipe::matches_request("create nginx site"));
        assert!(WebServerRecipe::matches_request("add virtual host to nginx"));
        assert!(WebServerRecipe::matches_request("configure new nginx site"));

        // SSL queries
        assert!(WebServerRecipe::matches_request("enable nginx SSL"));
        assert!(WebServerRecipe::matches_request("setup https for nginx"));
        assert!(WebServerRecipe::matches_request("install SSL certificate nginx"));
        assert!(WebServerRecipe::matches_request("setup let's encrypt nginx"));

        // Should NOT match pure informational queries
        assert!(!WebServerRecipe::matches_request("what is nginx"));
        assert!(!WebServerRecipe::matches_request("tell me about web servers"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            WebServerRecipe::detect_operation("install nginx"),
            WebServerOperation::Install
        );
        assert_eq!(
            WebServerRecipe::detect_operation("check nginx status"),
            WebServerOperation::CheckStatus
        );
        assert_eq!(
            WebServerRecipe::detect_operation("create nginx site"),
            WebServerOperation::CreateSite
        );
        assert_eq!(
            WebServerRecipe::detect_operation("enable nginx SSL"),
            WebServerOperation::EnableSsl
        );
    }

    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install nginx".to_string());

        let plan = WebServerRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.analysis.contains("Nginx"));
        assert!(plan.goals.iter().any(|g| g.contains("install") || g.contains("Install")));
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_status_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check nginx status".to_string());

        let plan = WebServerRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.command_plan.iter().all(|cmd| cmd.risk_level == RiskLevel::Info));
        assert!(plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_create_site_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "create nginx site".to_string());

        let plan = WebServerRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.analysis.contains("virtual host") || plan.analysis.contains("site"));
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_ssl_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "enable nginx SSL".to_string());

        let plan = WebServerRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.analysis.contains("SSL") || plan.analysis.contains("TLS"));
        assert!(plan.goals.iter().any(|g| g.contains("SSL") || g.contains("certbot")));
    }

    #[test]
    fn test_recipe_metadata() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install nginx".to_string());

        let plan = WebServerRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.meta.detection_results.other.contains_key("recipe_module"));
    }
}
