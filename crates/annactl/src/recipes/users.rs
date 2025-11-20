// users.rs - User and group management recipe
// Beta.153: User account and group administration

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use serde_json;
use std::collections::HashMap;

pub struct UsersRecipe;

#[derive(Debug, PartialEq)]
enum UserOperation {
    AddUser,
    RemoveUser,
    AddToGroup,
    ListUsers,
    ShowUserInfo,
    ChangeShell,
}

impl UsersRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Exclude informational queries
        if input_lower.contains("what is")
            || input_lower.contains("tell me about")
            || input_lower.contains("explain")
        {
            return false;
        }

        // User-related keywords (including shell, which is user configuration)
        let has_user_context = input_lower.contains("user")
            || input_lower.contains("account")
            || input_lower.contains("group")
            || input_lower.contains("username")
            || input_lower.contains("shell");

        // Action keywords
        let has_action = input_lower.contains("add")
            || input_lower.contains("create")
            || input_lower.contains("remove")
            || input_lower.contains("delete")
            || input_lower.contains("list")
            || input_lower.contains("show")
            || input_lower.contains("change")
            || input_lower.contains("modify")
            || input_lower.contains("set");

        has_user_context && has_action
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_request = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = Self::detect_operation(user_request);

        match operation {
            UserOperation::AddUser => Self::build_add_user_plan(user_request, telemetry),
            UserOperation::RemoveUser => Self::build_remove_user_plan(user_request, telemetry),
            UserOperation::AddToGroup => Self::build_add_to_group_plan(user_request, telemetry),
            UserOperation::ListUsers => Self::build_list_users_plan(telemetry),
            UserOperation::ShowUserInfo => Self::build_show_user_info_plan(user_request, telemetry),
            UserOperation::ChangeShell => Self::build_change_shell_plan(user_request, telemetry),
        }
    }

    fn detect_operation(user_input: &str) -> UserOperation {
        let input_lower = user_input.to_lowercase();

        // Check for list first (most specific)
        if input_lower.contains("list") && input_lower.contains("user") {
            return UserOperation::ListUsers;
        }

        // Check for show/info
        if (input_lower.contains("show") || input_lower.contains("info"))
            && input_lower.contains("user")
        {
            return UserOperation::ShowUserInfo;
        }

        // Check for shell changes
        if input_lower.contains("shell") && (input_lower.contains("change") || input_lower.contains("set")) {
            return UserOperation::ChangeShell;
        }

        // Check for group operations
        if input_lower.contains("add")
            && (input_lower.contains("group") || input_lower.contains("to docker") || input_lower.contains("to wheel"))
        {
            return UserOperation::AddToGroup;
        }

        // Check for user removal
        if (input_lower.contains("remove") || input_lower.contains("delete"))
            && input_lower.contains("user")
        {
            return UserOperation::RemoveUser;
        }

        // Default to add user
        UserOperation::AddUser
    }

    fn build_add_user_plan(
        user_request: &str,
        _telemetry: &HashMap<String, String>,
    ) -> Result<ActionPlan> {
        let username = Self::extract_username(user_request).unwrap_or("SPECIFY_USERNAME");

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-user-exists".to_string(),
                description: format!("Check if user '{}' already exists", username),
                command: format!("id {} 2>/dev/null", username),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "create-user".to_string(),
                description: format!("Create user account '{}'", username),
                command: format!(
                    "sudo useradd -m -G wheel -s /bin/bash {}",
                    username
                ),
                risk_level: RiskLevel::High,
                rollback_id: Some("remove-user".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "set-password".to_string(),
                description: format!("Set password for user '{}'", username),
                command: format!("sudo passwd {}", username),
                risk_level: RiskLevel::High,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-user".to_string(),
                description: "Verify user creation".to_string(),
                command: format!("id {}", username),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-home".to_string(),
                description: "Show user home directory".to_string(),
                command: format!("ls -la /home/{}", username),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "remove-user".to_string(),
            description: format!("Remove user '{}' and home directory", username),
            command: format!("sudo userdel -r {}", username),
        }];

        let notes_for_user = format!(
            "👤 Creating New User Account\n\n\
             Creating user: {}\n\
             Home directory: /home/{}\n\
             Default shell: /bin/bash\n\
             Groups: wheel (sudo access)\n\n\
             ⚠️ IMPORTANT:\n\
             1. You will be prompted to set a password for the new user\n\
             2. The user will be added to the 'wheel' group (sudo/admin access)\n\
             3. A home directory will be created at /home/{}\n\n\
             Security considerations:\n\
             - Choose a strong password\n\
             - The 'wheel' group grants administrative privileges\n\
             - Consider SSH key-based authentication instead of passwords\n\n\
             After creation:\n\
             - Switch to user: `su - {}`\n\
             - Remove user: `annactl \"remove user {}\"`\n\
             - Modify groups: `annactl \"add user {} to group docker\"`",
            username, username, username, username, username, username
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("users.rs".to_string()),
        );
        other.insert(
            "username".to_string(),
            serde_json::Value::String(username.to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("user_add".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: format!(
                "User wants to create a new user account '{}'. This will create a home directory and prompt for password setup.",
                username
            ),
            goals: vec![
                format!("Create user account '{}'", username),
                "Set up home directory with proper permissions".to_string(),
                "Add user to wheel group for sudo access".to_string(),
                "Set user password".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_remove_user_plan(
        user_request: &str,
        _telemetry: &HashMap<String, String>,
    ) -> Result<ActionPlan> {
        let username = Self::extract_username(user_request).unwrap_or("SPECIFY_USERNAME");

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-user-exists".to_string(),
                description: format!("Check if user '{}' exists", username),
                command: format!("id {}", username),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "show-user-processes".to_string(),
                description: format!("Show running processes for user '{}'", username),
                command: format!("ps -u {} 2>/dev/null || echo 'No processes running'", username),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "kill-user-processes".to_string(),
                description: format!("Kill all processes running as '{}'", username),
                command: format!("sudo pkill -u {}", username),
                risk_level: RiskLevel::High,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "remove-user".to_string(),
                description: format!("Remove user '{}' and home directory", username),
                command: format!("sudo userdel -r {}", username),
                risk_level: RiskLevel::High,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-removal".to_string(),
                description: "Verify user removal".to_string(),
                command: format!("id {} 2>&1 || echo 'User successfully removed'", username),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "note-irreversible".to_string(),
            description: "User deletion is irreversible - restore from backup if needed".to_string(),
            command: "echo 'No automatic rollback available - restore from backup'".to_string(),
        }];

        let current_user_warning = if username == "SPECIFY_USERNAME" {
            ""
        } else {
            "\n\n🚨 CRITICAL WARNING:\n\
             DO NOT delete the user account you are currently logged in as!\n\
             This will cause system instability and potential data loss.\n\n"
        };

        let notes_for_user = format!(
            "⚠️ DANGEROUS: Removing User Account{}\n\
             Removing user: {}\n\n\
             This operation will:\n\
             1. Kill all processes running as this user\n\
             2. Delete the user account\n\
             3. Remove the home directory (/home/{})\n\
             4. Delete all user data permanently\n\n\
             ⚠️ THIS IS IRREVERSIBLE!\n\
             - All files in /home/{} will be PERMANENTLY DELETED\n\
             - The user account cannot be recovered\n\
             - Make sure you have backups of important data\n\n\
             Before proceeding:\n\
             1. Check who you're logged in as: `whoami`\n\
             2. Ensure you're not deleting your own account\n\
             3. Back up any important data from /home/{}\n\
             4. Verify this is the correct user: `id {}`",
            current_user_warning, username, username, username, username, username
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("users.rs".to_string()),
        );
        other.insert(
            "username".to_string(),
            serde_json::Value::String(username.to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("user_remove".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: format!(
                "User wants to delete the user account '{}'. This is a destructive operation that permanently removes the user and all their data.",
                username
            ),
            goals: vec![
                "Kill all processes running as the target user".to_string(),
                format!("Delete user account '{}'", username),
                "Remove home directory and all user data".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_add_to_group_plan(
        user_request: &str,
        telemetry: &HashMap<String, String>,
    ) -> Result<ActionPlan> {
        let username = Self::extract_username(user_request)
            .or_else(|| telemetry.get("hostname").map(|s| s.as_str()))
            .unwrap_or("SPECIFY_USERNAME");

        let group = Self::extract_group(user_request).unwrap_or("SPECIFY_GROUP");

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-user-exists".to_string(),
                description: format!("Check if user '{}' exists", username),
                command: format!("id {}", username),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-group-exists".to_string(),
                description: format!("Check if group '{}' exists", group),
                command: format!("getent group {}", group),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "show-current-groups".to_string(),
                description: format!("Show current groups for '{}'", username),
                command: format!("groups {}", username),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "add-to-group".to_string(),
                description: format!("Add user '{}' to group '{}'", username, group),
                command: format!("sudo usermod -aG {} {}", group, username),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("remove-from-group".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-membership".to_string(),
                description: "Verify group membership".to_string(),
                command: format!("groups {}", username),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "remove-from-group".to_string(),
            description: format!("Remove user '{}' from group '{}'", username, group),
            command: format!("sudo gpasswd -d {} {}", username, group),
        }];

        let group_context = match group {
            "docker" => "\n\nDocker group context:\n\
                - Allows running docker commands without sudo\n\
                - ⚠️ Security: Equivalent to root access! Docker containers can escape to host\n\
                - User must log out and back in for changes to take effect",
            "wheel" => "\n\nWheel group context:\n\
                - Grants sudo/administrative privileges\n\
                - ⚠️ Security: User will have full system control\n\
                - Ensure this user is trustworthy",
            _ => "",
        };

        let notes_for_user = format!(
            "👥 Adding User to Group\n\n\
             User: {}\n\
             Group: {}{}\n\n\
             ⚠️ IMPORTANT:\n\
             Group changes require the user to LOG OUT and log back in to take effect.\n\
             Existing sessions will NOT have the new group membership.\n\n\
             To verify after re-login:\n\
             - Check groups: `groups`\n\
             - Check specific group: `groups {} | grep {}`\n\n\
             To remove from group later:\n\
             `sudo gpasswd -d {} {}`",
            username, group, group_context, username, group, username, group
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("users.rs".to_string()),
        );
        other.insert(
            "username".to_string(),
            serde_json::Value::String(username.to_string()),
        );
        other.insert(
            "group".to_string(),
            serde_json::Value::String(group.to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("user_add_to_group".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: format!(
                "User wants to add '{}' to the '{}' group. This will grant additional permissions associated with that group.",
                username, group
            ),
            goals: vec![
                format!("Add user '{}' to group '{}'", username, group),
                "Verify group membership was updated".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_list_users_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "list-human-users".to_string(),
                description: "List all human users (UID >= 1000)".to_string(),
                command: "awk -F: '$3 >= 1000 && $1 != \"nobody\" {print $1 \": \" $5 \" (\" $6 \")\"}' /etc/passwd".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-logged-in".to_string(),
                description: "Show currently logged in users".to_string(),
                command: "who".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-last-logins".to_string(),
                description: "Show recent login history".to_string(),
                command: "last -n 10".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let notes_for_user = "📋 User Account Listing\n\n\
             This shows:\n\
             1. All human user accounts (UID >= 1000)\n\
             2. Currently logged in users\n\
             3. Recent login history\n\n\
             Common user management tasks:\n\
             - Create user: `annactl \"add user USERNAME\"`\n\
             - Remove user: `annactl \"remove user USERNAME\"`\n\
             - Add to group: `annactl \"add user USERNAME to group docker\"`\n\
             - User info: `annactl \"show info for user USERNAME\"`\n\n\
             System users (UID < 1000) are not shown - these are service accounts."
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("users.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("user_list".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to see all user accounts on the system. This is a read-only operation showing human users, logged-in users, and recent login history.".to_string(),
            goals: vec![
                "List all human user accounts".to_string(),
                "Show currently logged in users".to_string(),
                "Display recent login history".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_show_user_info_plan(
        user_request: &str,
        _telemetry: &HashMap<String, String>,
    ) -> Result<ActionPlan> {
        let username = Self::extract_username(user_request).unwrap_or("SPECIFY_USERNAME");

        let necessary_checks = vec![NecessaryCheck {
            id: "check-user-exists".to_string(),
            description: format!("Check if user '{}' exists", username),
            command: format!("id {}", username),
            risk_level: RiskLevel::Info,
            required: true,
        }];

        let command_plan = vec![
            CommandStep {
                id: "show-id".to_string(),
                description: format!("Show user ID and groups for '{}'", username),
                command: format!("id {}", username),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-passwd-entry".to_string(),
                description: "Show user account details".to_string(),
                command: format!("getent passwd {}", username),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-groups".to_string(),
                description: "Show all groups".to_string(),
                command: format!("groups {}", username),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-home-dir".to_string(),
                description: "Show home directory".to_string(),
                command: format!("ls -la /home/{} 2>/dev/null | head -20 || echo 'Home directory not found'", username),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-last-login".to_string(),
                description: "Show last login".to_string(),
                command: format!("last {} -n 5", username),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let notes_for_user = format!(
            "ℹ️ User Account Information\n\n\
             Showing detailed information for user: {}\n\n\
             This includes:\n\
             - User ID (UID) and group ID (GID)\n\
             - All group memberships\n\
             - Home directory location and contents\n\
             - Default shell\n\
             - Recent login history\n\n\
             Modify user:\n\
             - Add to group: `annactl \"add user {} to group docker\"`\n\
             - Change shell: `annactl \"change shell for user {} to zsh\"`\n\
             - Remove user: `annactl \"remove user {}\"`",
            username, username, username, username
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("users.rs".to_string()),
        );
        other.insert(
            "username".to_string(),
            serde_json::Value::String(username.to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("user_show_info".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: format!(
                "User wants to see detailed information about user '{}'. This is a read-only operation.",
                username
            ),
            goals: vec![
                format!("Show user ID and groups for '{}'", username),
                "Display account details and home directory".to_string(),
                "Show login history".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_change_shell_plan(
        user_request: &str,
        _telemetry: &HashMap<String, String>,
    ) -> Result<ActionPlan> {
        let username = Self::extract_username(user_request).unwrap_or("SPECIFY_USERNAME");
        let shell = Self::extract_shell(user_request).unwrap_or("bash");
        let shell_path = format!("/bin/{}", shell);

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-user-exists".to_string(),
                description: format!("Check if user '{}' exists", username),
                command: format!("id {}", username),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-shell-exists".to_string(),
                description: format!("Check if shell '{}' is installed", shell_path),
                command: format!("which {} || pacman -Q {}", shell_path, shell),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "show-current-shell".to_string(),
                description: "Show current shell".to_string(),
                command: format!("getent passwd {} | cut -d: -f7", username),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "change-shell".to_string(),
                description: format!("Change shell to '{}' for user '{}'", shell_path, username),
                command: format!("sudo chsh -s {} {}", shell_path, username),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("restore-shell".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-shell".to_string(),
                description: "Verify shell change".to_string(),
                command: format!("getent passwd {} | cut -d: -f7", username),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "restore-shell".to_string(),
            description: "Restore to /bin/bash".to_string(),
            command: format!("sudo chsh -s /bin/bash {}", username),
        }];

        let notes_for_user = format!(
            "🐚 Changing Default Shell\n\n\
             User: {}\n\
             New shell: {}\n\n\
             ⚠️ IMPORTANT:\n\
             - Shell change takes effect on NEXT LOGIN\n\
             - Current sessions will continue using the old shell\n\
             - If the new shell is not installed, login may fail\n\n\
             If you're changing your own shell:\n\
             - Test the new shell first: `{}`\n\
             - Verify it's in /etc/shells: `cat /etc/shells`\n\
             - Log out and back in for changes to take effect\n\n\
             Common shells:\n\
             - bash (default, most compatible)\n\
             - zsh (feature-rich, requires zsh package)\n\
             - fish (user-friendly, requires fish package)\n\n\
             To restore to bash:\n\
             `sudo chsh -s /bin/bash {}`",
            username, shell_path, shell_path, username
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("users.rs".to_string()),
        );
        other.insert(
            "username".to_string(),
            serde_json::Value::String(username.to_string()),
        );
        other.insert(
            "shell".to_string(),
            serde_json::Value::String(shell_path.clone()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("user_change_shell".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: format!(
                "User wants to change the default shell for '{}' to '{}'. This will take effect on next login.",
                username, shell_path
            ),
            goals: vec![
                format!("Change default shell to '{}'", shell_path),
                "Verify shell change was successful".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn extract_username(input: &str) -> Option<&str> {
        let words: Vec<&str> = input.split_whitespace().collect();

        // Look for patterns like "user USERNAME" or "account USERNAME"
        for i in 0..words.len() {
            if (words[i] == "user" || words[i] == "account") && i + 1 < words.len() {
                let candidate = words[i + 1];
                if Self::is_valid_username(candidate) {
                    return Some(candidate);
                }
            }
        }

        // Look for standalone valid usernames, excluding keywords
        let keywords = [
            "user", "users", "account", "group", "groups", "add", "create", "remove", "delete",
            "list", "show", "change", "modify", "set", "to", "from", "into",
            "all", "the", "for",
        ];
        for word in words.iter() {
            if Self::is_valid_username(word) && !keywords.contains(word) {
                return Some(word);
            }
        }

        None
    }

    fn extract_group(input: &str) -> Option<&str> {
        let input_lower = input.to_lowercase();

        // Common groups
        if input_lower.contains("docker") {
            return Some("docker");
        }
        if input_lower.contains("wheel") {
            return Some("wheel");
        }
        if input_lower.contains("audio") {
            return Some("audio");
        }
        if input_lower.contains("video") {
            return Some("video");
        }

        // Pattern: "to GROUP" or "group GROUP"
        let words: Vec<&str> = input.split_whitespace().collect();
        for i in 0..words.len() {
            if (words[i] == "to" || words[i] == "group") && i + 1 < words.len() {
                return Some(words[i + 1]);
            }
        }

        None
    }

    fn extract_shell(input: &str) -> Option<&str> {
        let input_lower = input.to_lowercase();

        if input_lower.contains("bash") {
            Some("bash")
        } else if input_lower.contains("zsh") {
            Some("zsh")
        } else if input_lower.contains("fish") {
            Some("fish")
        } else if input_lower.contains("sh") && !input_lower.contains("shell") {
            Some("sh")
        } else {
            None
        }
    }

    fn is_valid_username(s: &str) -> bool {
        !s.is_empty()
            && s.len() <= 32
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            && s.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_user_requests() {
        // Should match
        assert!(UsersRecipe::matches_request("add user john"));
        assert!(UsersRecipe::matches_request("create account testuser"));
        assert!(UsersRecipe::matches_request("remove user oldaccount"));
        assert!(UsersRecipe::matches_request("add user to docker group"));
        assert!(UsersRecipe::matches_request("list users"));
        assert!(UsersRecipe::matches_request("show info for user bob"));
        assert!(UsersRecipe::matches_request("change shell to zsh"));

        // Should NOT match
        assert!(!UsersRecipe::matches_request("what is a user account"));
        assert!(!UsersRecipe::matches_request("tell me about groups"));
        assert!(!UsersRecipe::matches_request("install docker"));
        assert!(!UsersRecipe::matches_request("how much RAM do I have"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            UsersRecipe::detect_operation("add user john"),
            UserOperation::AddUser
        );
        assert_eq!(
            UsersRecipe::detect_operation("remove user john"),
            UserOperation::RemoveUser
        );
        assert_eq!(
            UsersRecipe::detect_operation("add user to docker group"),
            UserOperation::AddToGroup
        );
        assert_eq!(
            UsersRecipe::detect_operation("list users"),
            UserOperation::ListUsers
        );
        assert_eq!(
            UsersRecipe::detect_operation("show info for user bob"),
            UserOperation::ShowUserInfo
        );
        assert_eq!(
            UsersRecipe::detect_operation("change shell to zsh"),
            UserOperation::ChangeShell
        );
    }

    #[test]
    fn test_add_user_plan() {
        let telemetry = HashMap::new();
        let plan = UsersRecipe::build_add_user_plan("add user testuser", &telemetry).unwrap();

        assert!(plan.analysis.contains("testuser"));
        assert_eq!(plan.goals.len(), 4);
        assert!(plan.command_plan[0].command.contains("useradd"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::High);
        assert!(plan.notes_for_user.contains("wheel"));
    }

    #[test]
    fn test_remove_user_plan_warnings() {
        let telemetry = HashMap::new();
        let plan =
            UsersRecipe::build_remove_user_plan("remove user olduser", &telemetry).unwrap();

        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::High);
        assert!(plan.notes_for_user.contains("IRREVERSIBLE"));
        assert!(plan.notes_for_user.contains("PERMANENTLY DELETED"));
        assert!(plan.command_plan[0].command.contains("pkill"));
    }

    #[test]
    fn test_add_to_docker_group() {
        let telemetry = HashMap::new();
        let plan =
            UsersRecipe::build_add_to_group_plan("add user bob to docker group", &telemetry)
                .unwrap();

        assert!(plan.command_plan[0].command.contains("usermod -aG docker"));
        assert!(plan.notes_for_user.contains("Docker group context"));
        assert!(plan.notes_for_user.contains("root access"));
    }

    #[test]
    fn test_list_users_plan() {
        let telemetry = HashMap::new();
        let plan = UsersRecipe::build_list_users_plan(&telemetry).unwrap();

        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Info);
        assert!(plan.command_plan[0].command.contains("1000"));
        assert!(!plan.command_plan[0].requires_confirmation);
    }

    #[test]
    fn test_show_user_info_plan() {
        let telemetry = HashMap::new();
        let plan =
            UsersRecipe::build_show_user_info_plan("show info for user alice", &telemetry)
                .unwrap();

        assert!(plan.analysis.contains("alice"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Info);
        assert!(plan.command_plan[0].command.contains("id alice"));
    }

    #[test]
    fn test_change_shell_plan() {
        let telemetry = HashMap::new();
        let plan =
            UsersRecipe::build_change_shell_plan("change shell for user bob to zsh", &telemetry)
                .unwrap();

        assert!(plan.command_plan[0].command.contains("chsh"));
        assert!(plan.command_plan[0].command.contains("/bin/zsh"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Medium);
        assert!(plan.notes_for_user.contains("NEXT LOGIN"));
    }

    #[test]
    fn test_extract_username() {
        assert_eq!(
            UsersRecipe::extract_username("add user john"),
            Some("john")
        );
        assert_eq!(
            UsersRecipe::extract_username("remove user test_user"),
            Some("test_user")
        );
        assert_eq!(
            UsersRecipe::extract_username("create account bob123"),
            Some("bob123")
        );
        assert_eq!(UsersRecipe::extract_username("list all users"), None);
    }

    #[test]
    fn test_extract_group() {
        assert_eq!(
            UsersRecipe::extract_group("add to docker group"),
            Some("docker")
        );
        assert_eq!(
            UsersRecipe::extract_group("add user to wheel"),
            Some("wheel")
        );
        assert_eq!(
            UsersRecipe::extract_group("join the audio group"),
            Some("audio")
        );
    }

    #[test]
    fn test_extract_shell() {
        assert_eq!(UsersRecipe::extract_shell("change shell to zsh"), Some("zsh"));
        assert_eq!(UsersRecipe::extract_shell("set shell to bash"), Some("bash"));
        assert_eq!(UsersRecipe::extract_shell("use fish shell"), Some("fish"));
        assert_eq!(UsersRecipe::extract_shell("change my shell"), None);
    }

    #[test]
    fn test_is_valid_username() {
        assert!(UsersRecipe::is_valid_username("john"));
        assert!(UsersRecipe::is_valid_username("test_user"));
        assert!(UsersRecipe::is_valid_username("user-123"));

        assert!(!UsersRecipe::is_valid_username("John")); // Must start lowercase
        assert!(!UsersRecipe::is_valid_username("user@host")); // No @ allowed
        assert!(!UsersRecipe::is_valid_username("")); // Empty
    }
}
