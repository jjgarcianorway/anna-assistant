//! Docker Compose Recipe - Container Orchestration
//!
//! Beta.156: Docker Compose installation and project management
//!
//! This recipe handles Docker Compose installation (both standalone and Docker plugin),
//! project initialization, configuration validation, and common orchestration tasks.
//!
//! Operations:
//! - Install: Install docker-compose (standalone or plugin)
//! - CheckStatus: Verify Docker Compose is installed and working
//! - InitProject: Create a basic docker-compose.yml template
//! - ValidateConfig: Check if docker-compose.yml is valid

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RollbackStep, RiskLevel};
use anyhow::Result;
use std::collections::HashMap;

pub struct DockerComposeRecipe;

#[derive(Debug, PartialEq)]
enum DockerComposeOperation {
    Install,
    CheckStatus,
    InitProject,
    ValidateConfig,
}

impl DockerComposeRecipe {
    /// Check if user request matches Docker Compose patterns
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Docker Compose related keywords
        let has_compose_context = input_lower.contains("docker-compose")
            || input_lower.contains("docker compose")
            || input_lower.contains("compose")
            || (input_lower.contains("container") && input_lower.contains("orchestrat"));

        // Action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("configure")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("init")
            || input_lower.contains("create")
            || input_lower.contains("validate")
            || input_lower.contains("verify")
            || input_lower.contains("start")
            || input_lower.contains("template")
            || input_lower.contains("generate")
            || input_lower.contains("new");

        // Exclude informational-only queries
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;

        has_compose_context && has_action && !is_info_only
    }

    /// Detect specific operation from user request
    fn detect_operation(user_input: &str) -> DockerComposeOperation {
        let input_lower = user_input.to_lowercase();

        // CheckStatus operation
        if input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("verify")
            || input_lower.contains("is") && input_lower.contains("install")
        {
            return DockerComposeOperation::CheckStatus;
        }

        // InitProject operation
        if input_lower.contains("init")
            || input_lower.contains("create") && (input_lower.contains("project") || input_lower.contains("template"))
            || input_lower.contains("generate") && input_lower.contains("compose")
            || input_lower.contains("new") && input_lower.contains("compose")
        {
            return DockerComposeOperation::InitProject;
        }

        // ValidateConfig operation
        if input_lower.contains("validate")
            || input_lower.contains("check") && input_lower.contains("config")
            || input_lower.contains("verify") && input_lower.contains("compose.yml")
        {
            return DockerComposeOperation::ValidateConfig;
        }

        // Default to Install
        DockerComposeOperation::Install
    }

    /// Build ActionPlan based on detected operation
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_request = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = Self::detect_operation(user_request);

        match operation {
            DockerComposeOperation::Install => Self::build_install_plan(telemetry),
            DockerComposeOperation::CheckStatus => Self::build_status_plan(telemetry),
            DockerComposeOperation::InitProject => Self::build_init_project_plan(telemetry),
            DockerComposeOperation::ValidateConfig => Self::build_validate_plan(telemetry),
        }
    }

    /// Install Docker Compose
    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "check-docker-installed".to_string(),
                description: "Check if Docker is installed (prerequisite)".to_string(),
                command: "which docker".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-docker-compose-plugin".to_string(),
                description: "Check if docker-compose plugin is available".to_string(),
                command: "docker compose version 2>/dev/null || echo 'Not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-docker-compose".to_string(),
                description: "Install docker-compose package from official Arch repos".to_string(),
                command: "sudo pacman -S --needed --noconfirm docker-compose".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-docker-compose-installation".to_string(),
                description: "Verify docker-compose installation".to_string(),
                command: "docker-compose --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-docker-compose-plugin".to_string(),
                description: "Verify docker compose plugin (v2)".to_string(),
                command: "docker compose version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-docker-installation".to_string(),
                description: "Check Docker installation".to_string(),
                command: "which docker".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-internet-connectivity".to_string(),
                description: "Check internet connectivity".to_string(),
                command: "ping -c 1 archlinux.org".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "remove-docker-compose".to_string(),
                description: "Remove docker-compose if installation causes issues".to_string(),
                command: "sudo pacman -Rns docker-compose".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("docker_compose.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("install"));

        Ok(ActionPlan {
            analysis: "Docker Compose installation on Arch Linux. Docker Compose is available as:\n\
                      1. Standalone binary (docker-compose) - v1 legacy\n\
                      2. Docker plugin (docker compose) - v2 modern\n\
                      \n\
                      The docker-compose package from official repos provides both interfaces. \
                      Requires Docker to be installed and running.".to_string(),
            goals: vec![
                "Install docker-compose from official Arch repos".to_string(),
                "Verify both standalone and plugin interfaces work".to_string(),
                "Ensure Docker daemon is available".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Docker Compose will be installed from official Arch repositories. \
                            Both the legacy 'docker-compose' command and modern 'docker compose' \
                            plugin will be available. Docker daemon must be running to use Compose.\n\
                            \n\
                            Make sure Docker is installed first (see: annactl 'install docker')."
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("docker_compose_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    /// Check Docker Compose status
    fn build_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "check-docker-compose-v1".to_string(),
                description: "Check if docker-compose (v1 standalone) is installed".to_string(),
                command: "which docker-compose".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-docker-compose-version".to_string(),
                description: "Check docker-compose version".to_string(),
                command: "docker-compose --version 2>/dev/null || echo 'Not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-docker-compose-v2-availability".to_string(),
                description: "Check docker compose plugin (v2) availability".to_string(),
                command: "docker compose version 2>/dev/null || echo 'Plugin not available'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-docker-daemon-status".to_string(),
                description: "Check Docker daemon status".to_string(),
                command: "systemctl is-active docker".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-docker-group-membership".to_string(),
                description: "Check current user's Docker group membership".to_string(),
                command: "groups | grep -q docker && echo 'User in docker group' || echo 'User NOT in docker group'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("docker_compose.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("check_status"));

        Ok(ActionPlan {
            analysis: "Checking Docker Compose installation status. Will verify both v1 (docker-compose) \
                      and v2 (docker compose plugin) interfaces, Docker daemon status, and user permissions."
                .to_string(),
            goals: vec![
                "Verify Docker Compose installation (v1 and/or v2)".to_string(),
                "Check Docker daemon status".to_string(),
                "Verify user has Docker permissions".to_string(),
            ],
            necessary_checks: vec![],
            command_plan,
            rollback_plan: vec![],
            notes_for_user: "This will check your Docker Compose setup. Both v1 (docker-compose) \
                            and v2 (docker compose) interfaces will be verified."
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("docker_compose_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    /// Initialize a new Docker Compose project
    fn build_init_project_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "check-docker-compose-file-exists".to_string(),
                description: "Check if docker-compose.yml already exists".to_string(),
                command: "test -f docker-compose.yml && echo 'File exists' || echo 'No file'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "create-docker-compose-template".to_string(),
                description: "Create basic docker-compose.yml template".to_string(),
                command: r#"cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  # Example service - replace with your application
  web:
    image: nginx:alpine
    ports:
      - "8080:80"
    volumes:
      - ./html:/usr/share/nginx/html:ro
    restart: unless-stopped

  # Example database service (uncomment to use)
  # db:
  #   image: postgres:15-alpine
  #   environment:
  #     POSTGRES_PASSWORD: changeme
  #   volumes:
  #     - db-data:/var/lib/postgresql/data
  #   restart: unless-stopped

# volumes:
#   db-data:
EOF
"#.to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "create-html-directory".to_string(),
                description: "Create example html directory".to_string(),
                command: "mkdir -p html && echo '<h1>Welcome to Docker Compose</h1>' > html/index.html".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "validate-docker-compose-file".to_string(),
                description: "Validate the generated docker-compose.yml".to_string(),
                command: "docker compose config --quiet".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-docker-compose-installed".to_string(),
                description: "Check Docker Compose is installed".to_string(),
                command: "which docker-compose || docker compose version".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "remove-docker-compose-file".to_string(),
                description: "Remove generated docker-compose.yml if needed".to_string(),
                command: "rm -f docker-compose.yml".to_string(),
            },
            RollbackStep {
                id: "remove-html-directory".to_string(),
                description: "Remove html directory".to_string(),
                command: "rm -rf html/".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("docker_compose.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("init_project"));

        Ok(ActionPlan {
            analysis: "Creating a basic Docker Compose project template with docker-compose.yml. \
                      The template includes:\n\
                      - Example web service (nginx)\n\
                      - Commented database service (PostgreSQL)\n\
                      - Volume configurations\n\
                      - Port mappings\n\
                      \n\
                      This provides a starting point that can be customized for your application."
                .to_string(),
            goals: vec![
                "Create docker-compose.yml template in current directory".to_string(),
                "Add example service configurations".to_string(),
                "Validate configuration syntax".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "A docker-compose.yml template will be created in the current directory. \
                            This includes a basic nginx web server and commented PostgreSQL example. \
                            Edit the file to customize for your application.\n\
                            \n\
                            To start services: docker compose up -d\n\
                            To stop services: docker compose down"
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("docker_compose_init".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    /// Validate docker-compose.yml configuration
    fn build_validate_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "check-docker-compose-file-exists-validation".to_string(),
                description: "Check if docker-compose.yml exists".to_string(),
                command: "test -f docker-compose.yml && echo 'File found' || echo 'No docker-compose.yml in current directory'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "validate-docker-compose-syntax".to_string(),
                description: "Validate docker-compose.yml syntax and configuration".to_string(),
                command: "docker compose config --quiet && echo 'Configuration is valid' || echo 'Configuration has errors'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-docker-compose-config".to_string(),
                description: "Show parsed configuration (with interpolated variables)".to_string(),
                command: "docker compose config".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "list-docker-compose-services".to_string(),
                description: "List services defined in docker-compose.yml".to_string(),
                command: "docker compose config --services".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "list-docker-compose-volumes".to_string(),
                description: "List volumes defined in docker-compose.yml".to_string(),
                command: "docker compose config --volumes".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-docker-compose-installed-validation".to_string(),
                description: "Check Docker Compose is installed".to_string(),
                command: "which docker-compose || docker compose version".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("docker_compose.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("validate_config"));

        Ok(ActionPlan {
            analysis: "Validating docker-compose.yml configuration file. This will:\n\
                      - Check file exists\n\
                      - Validate YAML syntax\n\
                      - Verify Docker Compose schema compliance\n\
                      - Show parsed configuration with variable interpolation\n\
                      - List all services and volumes"
                .to_string(),
            goals: vec![
                "Verify docker-compose.yml exists".to_string(),
                "Validate configuration syntax and schema".to_string(),
                "Display parsed configuration".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan: vec![],
            notes_for_user: "This will validate your docker-compose.yml file without starting any containers. \
                            Any syntax errors or configuration issues will be reported."
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("docker_compose_validate".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_docker_compose_requests() {
        // Install queries
        assert!(DockerComposeRecipe::matches_request("install docker-compose"));
        assert!(DockerComposeRecipe::matches_request("setup docker compose"));
        assert!(DockerComposeRecipe::matches_request("install compose"));

        // Status queries
        assert!(DockerComposeRecipe::matches_request("check docker-compose status"));
        assert!(DockerComposeRecipe::matches_request("is docker compose installed"));
        assert!(DockerComposeRecipe::matches_request("verify compose installation"));

        // Init project queries
        assert!(DockerComposeRecipe::matches_request("init docker-compose project"));
        assert!(DockerComposeRecipe::matches_request("create compose template"));
        assert!(DockerComposeRecipe::matches_request("generate docker-compose.yml"));
        assert!(DockerComposeRecipe::matches_request("new compose project"));

        // Validate queries
        assert!(DockerComposeRecipe::matches_request("validate docker-compose.yml"));
        assert!(DockerComposeRecipe::matches_request("check compose config"));
        assert!(DockerComposeRecipe::matches_request("verify compose configuration"));

        // Should NOT match pure informational queries
        assert!(!DockerComposeRecipe::matches_request("what is docker-compose"));
        assert!(!DockerComposeRecipe::matches_request("tell me about compose"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            DockerComposeRecipe::detect_operation("install docker-compose"),
            DockerComposeOperation::Install
        );
        assert_eq!(
            DockerComposeRecipe::detect_operation("check docker compose status"),
            DockerComposeOperation::CheckStatus
        );
        assert_eq!(
            DockerComposeRecipe::detect_operation("init compose project"),
            DockerComposeOperation::InitProject
        );
        assert_eq!(
            DockerComposeRecipe::detect_operation("validate docker-compose.yml"),
            DockerComposeOperation::ValidateConfig
        );
    }

    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install docker-compose".to_string());

        let plan = DockerComposeRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.analysis.contains("Docker Compose"));
        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("install")));
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_status_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check docker-compose status".to_string());

        let plan = DockerComposeRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.command_plan.iter().all(|cmd| cmd.risk_level == RiskLevel::Info));
        assert!(plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_init_project_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "init docker compose project".to_string());

        let plan = DockerComposeRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.analysis.contains("template"));
        assert!(plan.goals.iter().any(|g| g.contains("docker-compose.yml")));
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_validate_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "validate docker-compose.yml".to_string());

        let plan = DockerComposeRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.command_plan.iter().all(|cmd| cmd.risk_level == RiskLevel::Info));
        assert!(plan.analysis.contains("Validating"));
    }

    #[test]
    fn test_recipe_metadata() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install docker-compose".to_string());

        let plan = DockerComposeRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.meta.detection_results.other.contains_key("recipe_module"));
    }
}
