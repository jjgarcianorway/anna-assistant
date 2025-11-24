//! PostgreSQL Recipe - Database Installation and Management
//!
//! Beta.156: PostgreSQL database server installation, initialization, and configuration
//!
//! This recipe handles PostgreSQL installation on Arch Linux, including:
//! - Installation from official repos
//! - Database cluster initialization
//! - Service management
//! - User and database creation
//!
//! Operations:
//! - Install: Install PostgreSQL and initialize database cluster
//! - CheckStatus: Verify PostgreSQL installation and service status
//! - CreateDatabase: Create a new database and user
//! - ConfigureSecurity: Configure authentication and security settings

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RollbackStep, RiskLevel};
use anyhow::Result;
use std::collections::HashMap;

pub struct PostgresqlRecipe;

#[derive(Debug, PartialEq)]
enum PostgresqlOperation {
    Install,
    CheckStatus,
    CreateDatabase,
    ConfigureSecurity,
}

impl PostgresqlRecipe {
    /// Check if user request matches PostgreSQL patterns
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // PostgreSQL related keywords
        let has_postgres_context = input_lower.contains("postgres")
            || input_lower.contains("postgresql")
            || input_lower.contains("psql")
            || (input_lower.contains("database") &&
                (input_lower.contains("sql") || input_lower.contains("relational")));

        // Action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("configure")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("create")
            || input_lower.contains("init")
            || input_lower.contains("start")
            || input_lower.contains("enable")
            || input_lower.contains("secure")
            || input_lower.contains("running")
            || input_lower.contains("new")
            || input_lower.contains("add")
            || input_lower.contains("set");

        // Exclude informational-only queries
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !has_action;

        has_postgres_context && has_action && !is_info_only
    }

    /// Detect specific operation from user request
    fn detect_operation(user_input: &str) -> PostgresqlOperation {
        let input_lower = user_input.to_lowercase();

        // CheckStatus operation
        if input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("verify")
            || input_lower.contains("is") && input_lower.contains("running")
        {
            return PostgresqlOperation::CheckStatus;
        }

        // CreateDatabase operation
        if input_lower.contains("create") && (input_lower.contains("database") || input_lower.contains("db"))
            || input_lower.contains("new") && input_lower.contains("database")
            || input_lower.contains("add") && input_lower.contains("database")
        {
            return PostgresqlOperation::CreateDatabase;
        }

        // ConfigureSecurity operation
        if input_lower.contains("secure")
            || input_lower.contains("security")
            || input_lower.contains("harden")
            || input_lower.contains("password")
            || input_lower.contains("auth")
        {
            return PostgresqlOperation::ConfigureSecurity;
        }

        // Default to Install
        PostgresqlOperation::Install
    }

    /// Build ActionPlan based on detected operation
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_request = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = Self::detect_operation(user_request);

        match operation {
            PostgresqlOperation::Install => Self::build_install_plan(telemetry),
            PostgresqlOperation::CheckStatus => Self::build_status_plan(telemetry),
            PostgresqlOperation::CreateDatabase => Self::build_create_db_plan(telemetry),
            PostgresqlOperation::ConfigureSecurity => Self::build_security_plan(telemetry),
        }
    }

    /// Install PostgreSQL
    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "check-postgresql-installed".to_string(),
                description: "Check if PostgreSQL is already installed".to_string(),
                command: "pacman -Qi postgresql 2>/dev/null || echo 'Not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-postgresql".to_string(),
                description: "Install PostgreSQL from official Arch repos".to_string(),
                command: "sudo pacman -S --needed --noconfirm postgresql".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "initialize-postgresql-cluster".to_string(),
                description: "Switch to postgres user and initialize database cluster".to_string(),
                command: "sudo -u postgres initdb -D /var/lib/postgres/data".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "enable-postgresql-service".to_string(),
                description: "Enable PostgreSQL service to start on boot".to_string(),
                command: "sudo systemctl enable postgresql".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "start-postgresql-service".to_string(),
                description: "Start PostgreSQL service".to_string(),
                command: "sudo systemctl start postgresql".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-postgresql-running".to_string(),
                description: "Verify PostgreSQL is running".to_string(),
                command: "sudo systemctl status postgresql".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "test-postgresql-connection".to_string(),
                description: "Test PostgreSQL connection".to_string(),
                command: "sudo -u postgres psql -c 'SELECT version();'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-internet-connectivity-postgresql".to_string(),
                description: "Check internet connectivity".to_string(),
                command: "ping -c 1 archlinux.org".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-disk-space-postgresql".to_string(),
                description: "Check available disk space (PostgreSQL needs ~200MB)".to_string(),
                command: "df -h /var/lib".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "stop-postgresql-service".to_string(),
                description: "Stop PostgreSQL service".to_string(),
                command: "sudo systemctl stop postgresql".to_string(),
            },
            RollbackStep {
                id: "disable-postgresql-service".to_string(),
                description: "Disable PostgreSQL service".to_string(),
                command: "sudo systemctl disable postgresql".to_string(),
            },
            RollbackStep {
                id: "remove-postgresql-package".to_string(),
                description: "Remove PostgreSQL package".to_string(),
                command: "sudo pacman -Rns postgresql".to_string(),
            },
            RollbackStep {
                id: "remove-postgresql-data".to_string(),
                description: "Remove database cluster (WARNING: deletes all data)".to_string(),
                command: "sudo rm -rf /var/lib/postgres/data".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("postgresql.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("install"));

        Ok(ActionPlan {
            analysis: "PostgreSQL installation on Arch Linux. This will:\n\
                      1. Install postgresql package from official repos\n\
                      2. Initialize database cluster in /var/lib/postgres/data\n\
                      3. Configure systemd service\n\
                      4. Start PostgreSQL server\n\
                      \n\
                      The default superuser 'postgres' will be created with trust authentication \
                      for local connections. You should configure security settings after installation."
                .to_string(),
            goals: vec![
                "Install PostgreSQL package".to_string(),
                "Initialize database cluster".to_string(),
                "Enable and start PostgreSQL service".to_string(),
                "Verify database server is running".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "PostgreSQL will be installed and configured with default settings. \
                            The 'postgres' superuser will be created for administration.\n\
                            \n\
                            Next steps:\n\
                            - Create databases: sudo -u postgres createdb mydb\n\
                            - Create users: sudo -u postgres createuser myuser\n\
                            - Configure security: see annactl 'configure postgresql security'\n\
                            - Connect: sudo -u postgres psql"
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("postgresql_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    /// Check PostgreSQL status
    fn build_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "check-postgresql-package".to_string(),
                description: "Check if PostgreSQL package is installed".to_string(),
                command: "pacman -Qi postgresql".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-postgresql-version".to_string(),
                description: "Check PostgreSQL version".to_string(),
                command: "sudo -u postgres psql --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-postgresql-service-status".to_string(),
                description: "Check PostgreSQL service status".to_string(),
                command: "systemctl status postgresql".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-postgresql-boot-enabled".to_string(),
                description: "Check if PostgreSQL service is enabled on boot".to_string(),
                command: "systemctl is-enabled postgresql".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "list-postgresql-databases".to_string(),
                description: "List PostgreSQL databases".to_string(),
                command: "sudo -u postgres psql -c '\\l'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "list-postgresql-users".to_string(),
                description: "List PostgreSQL users".to_string(),
                command: "sudo -u postgres psql -c '\\du'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-postgresql-data-directory".to_string(),
                description: "Check PostgreSQL data directory".to_string(),
                command: "sudo ls -lh /var/lib/postgres/data/".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-postgresql-connections".to_string(),
                description: "Show active PostgreSQL connections".to_string(),
                command: "sudo -u postgres psql -c 'SELECT count(*) as active_connections FROM pg_stat_activity;'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("postgresql.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("check_status"));

        Ok(ActionPlan {
            analysis: "Checking PostgreSQL installation and service status. This will verify:\n\
                      - Package installation\n\
                      - Service status and autostart configuration\n\
                      - Database cluster health\n\
                      - Existing databases and users\n\
                      - Active connections"
                .to_string(),
            goals: vec![
                "Verify PostgreSQL is installed".to_string(),
                "Check service status".to_string(),
                "List databases and users".to_string(),
                "Show active connections".to_string(),
            ],
            necessary_checks: vec![],
            command_plan,
            rollback_plan: vec![],
            notes_for_user: "This will check your PostgreSQL installation status. \
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
                template_used: Some("postgresql_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    /// Create new database and user
    fn build_create_db_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "create-postgresql-database".to_string(),
                description: "Create new PostgreSQL database 'myapp'".to_string(),
                command: "sudo -u postgres createdb myapp".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "create-postgresql-user".to_string(),
                description: "Create new PostgreSQL user 'myapp_user'".to_string(),
                command: "sudo -u postgres createuser --pwprompt myapp_user".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "grant-postgresql-privileges".to_string(),
                description: "Grant all privileges on 'myapp' database to 'myapp_user'".to_string(),
                command: "sudo -u postgres psql -c \"GRANT ALL PRIVILEGES ON DATABASE myapp TO myapp_user;\"".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-postgresql-database-creation".to_string(),
                description: "Verify database creation".to_string(),
                command: "sudo -u postgres psql -c '\\l myapp'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-postgresql-user-creation".to_string(),
                description: "Verify user creation".to_string(),
                command: "sudo -u postgres psql -c '\\du myapp_user'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-postgresql-running".to_string(),
                description: "Check PostgreSQL is running".to_string(),
                command: "systemctl is-active postgresql".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "revoke-postgresql-privileges".to_string(),
                description: "Revoke privileges from user".to_string(),
                command: "sudo -u postgres psql -c \"REVOKE ALL PRIVILEGES ON DATABASE myapp FROM myapp_user;\"".to_string(),
            },
            RollbackStep {
                id: "drop-postgresql-database".to_string(),
                description: "Drop database 'myapp' (WARNING: deletes all data)".to_string(),
                command: "sudo -u postgres dropdb myapp".to_string(),
            },
            RollbackStep {
                id: "drop-postgresql-user".to_string(),
                description: "Drop user 'myapp_user'".to_string(),
                command: "sudo -u postgres dropuser myapp_user".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("postgresql.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("create_database"));

        Ok(ActionPlan {
            analysis: "Creating a new PostgreSQL database with dedicated user. This example creates:\n\
                      - Database: 'myapp'\n\
                      - User: 'myapp_user' (with password prompt)\n\
                      - Grants: Full privileges on myapp database\n\
                      \n\
                      You can customize the database and user names by editing the commands."
                .to_string(),
            goals: vec![
                "Create new database 'myapp'".to_string(),
                "Create new user 'myapp_user' with password".to_string(),
                "Grant database privileges to user".to_string(),
                "Verify creation successful".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "This will create a new database and user. The createuser command will \
                            prompt you to set a password for the new user.\n\
                            \n\
                            To connect to the database:\n\
                            psql -U myapp_user -d myapp -h localhost\n\
                            \n\
                            Customize database and user names by editing the commands before approval."
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("postgresql_create_db".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    /// Configure PostgreSQL security
    fn build_security_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let command_plan = vec![
            CommandStep {
                id: "backup-postgresql-config".to_string(),
                description: "Backup current pg_hba.conf".to_string(),
                command: "sudo cp /var/lib/postgres/data/pg_hba.conf /var/lib/postgres/data/pg_hba.conf.backup".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-postgresql-auth-settings".to_string(),
                description: "Show current pg_hba.conf authentication settings".to_string(),
                command: "sudo cat /var/lib/postgres/data/pg_hba.conf | grep -v '^#' | grep -v '^$'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "set-postgresql-password".to_string(),
                description: "Set password for postgres superuser".to_string(),
                command: "sudo -u postgres psql -c \"ALTER USER postgres WITH PASSWORD 'CHANGE_ME_PLEASE';\"".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "configure-postgresql-auth".to_string(),
                description: "Configure pg_hba.conf for password authentication".to_string(),
                command: r#"sudo bash -c "cat > /var/lib/postgres/data/pg_hba.conf << 'EOF'
# TYPE  DATABASE        USER            ADDRESS                 METHOD

# Local connections use peer authentication (Unix socket)
local   all             postgres                                peer
local   all             all                                     md5

# IPv4 local connections use password authentication
host    all             all             127.0.0.1/32            md5

# IPv6 local connections
host    all             all             ::1/128                 md5

# Reject all other connections
host    all             all             0.0.0.0/0               reject
EOF
""#.to_string(),
                risk_level: RiskLevel::High,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "reload-postgresql-config".to_string(),
                description: "Reload PostgreSQL configuration".to_string(),
                command: "sudo systemctl reload postgresql".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-postgresql-config-reload".to_string(),
                description: "Verify configuration reload".to_string(),
                command: "sudo systemctl status postgresql".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-postgresql-running-security".to_string(),
                description: "Check PostgreSQL is running".to_string(),
                command: "systemctl is-active postgresql".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "restore-postgresql-config".to_string(),
                description: "Restore original pg_hba.conf from backup".to_string(),
                command: "sudo cp /var/lib/postgres/data/pg_hba.conf.backup /var/lib/postgres/data/pg_hba.conf".to_string(),
            },
            RollbackStep {
                id: "reload-postgresql-config-rollback".to_string(),
                description: "Reload PostgreSQL configuration".to_string(),
                command: "sudo systemctl reload postgresql".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("postgresql.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("configure_security"));

        Ok(ActionPlan {
            analysis: "Configuring PostgreSQL security settings. This will:\n\
                      1. Backup current pg_hba.conf\n\
                      2. Set password for postgres superuser (CHANGE_ME_PLEASE - edit before running!)\n\
                      3. Configure pg_hba.conf for password authentication:\n\
                         - Local Unix socket: peer auth for postgres user, md5 for others\n\
                         - TCP/IP localhost: md5 password authentication\n\
                         - Remote: reject all connections\n\
                      4. Reload configuration\n\
                      \n\
                      WARNING: After this change, you'll need passwords to connect."
                .to_string(),
            goals: vec![
                "Set postgres superuser password".to_string(),
                "Configure secure authentication methods".to_string(),
                "Restrict remote connections".to_string(),
                "Reload PostgreSQL configuration".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "⚠️  IMPORTANT: This will require password authentication for PostgreSQL connections.\n\
                            \n\
                            Before running:\n\
                            1. Change 'CHANGE_ME_PLEASE' to a strong password\n\
                            2. Save the password securely\n\
                            \n\
                            After running:\n\
                            - Local postgres user: sudo -u postgres psql (peer auth)\n\
                            - Other users: psql -U username -d database (will prompt for password)\n\
                            \n\
                            Backup is created at: /var/lib/postgres/data/pg_hba.conf.backup"
                .to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("postgresql_security".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_postgresql_requests() {
        // Install queries
        assert!(PostgresqlRecipe::matches_request("install postgresql"));
        assert!(PostgresqlRecipe::matches_request("setup postgres"));
        assert!(PostgresqlRecipe::matches_request("install postgres database"));

        // Status queries
        assert!(PostgresqlRecipe::matches_request("check postgresql status"));
        assert!(PostgresqlRecipe::matches_request("is postgres running"));
        assert!(PostgresqlRecipe::matches_request("verify postgresql installation"));

        // Create database queries
        assert!(PostgresqlRecipe::matches_request("create postgres database"));
        assert!(PostgresqlRecipe::matches_request("new postgresql db"));
        assert!(PostgresqlRecipe::matches_request("add database to postgres"));

        // Security queries
        assert!(PostgresqlRecipe::matches_request("configure postgresql security"));
        assert!(PostgresqlRecipe::matches_request("secure postgres"));
        assert!(PostgresqlRecipe::matches_request("set postgres password"));

        // Should NOT match pure informational queries
        assert!(!PostgresqlRecipe::matches_request("what is postgresql"));
        assert!(!PostgresqlRecipe::matches_request("tell me about postgres"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            PostgresqlRecipe::detect_operation("install postgresql"),
            PostgresqlOperation::Install
        );
        assert_eq!(
            PostgresqlRecipe::detect_operation("check postgres status"),
            PostgresqlOperation::CheckStatus
        );
        assert_eq!(
            PostgresqlRecipe::detect_operation("create postgres database"),
            PostgresqlOperation::CreateDatabase
        );
        assert_eq!(
            PostgresqlRecipe::detect_operation("configure postgresql security"),
            PostgresqlOperation::ConfigureSecurity
        );
    }

    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install postgresql".to_string());

        let plan = PostgresqlRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.analysis.contains("PostgreSQL"));
        assert!(plan.goals.iter().any(|g| g.contains("install") || g.contains("Install")));
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_status_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check postgresql status".to_string());

        let plan = PostgresqlRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.command_plan.iter().all(|cmd| cmd.risk_level == RiskLevel::Info));
        assert!(plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_create_db_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "create postgres database".to_string());

        let plan = PostgresqlRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.analysis.contains("database"));
        assert!(plan.goals.iter().any(|g| g.contains("database")));
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_security_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "configure postgresql security".to_string());

        let plan = PostgresqlRecipe::build_plan(&telemetry).unwrap();

        assert!(!plan.command_plan.is_empty());
        assert!(plan.analysis.contains("security"));
        assert!(plan.goals.iter().any(|g| g.contains("password") || g.contains("auth")));
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_recipe_metadata() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install postgresql".to_string());

        let plan = PostgresqlRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.meta.detection_results.other.contains_key("recipe_module"));
    }
}
