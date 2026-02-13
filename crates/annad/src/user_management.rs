//! User account management — create, delete, modify users.
//!
//! All operations are HIGH risk and always require explicit confirmation.
//! Created accounts are tracked in ArtifactRegistry for later removal.
//!
//! Uses pkexec for privilege escalation.

use anyhow::{anyhow, Result};
use tracing::info;

/// Operation requested by user.
#[derive(Debug, Clone)]
pub enum UserOp {
    Create { username: String, groups: Vec<String> },
    Delete { username: String },
    ChangePassword { username: String },
    AddToGroup { username: String, group: String },
    ListUsers,
}

/// Parse user operation from a natural language query (LLM-assisted).
pub async fn parse_user_op(model: &str, question: &str) -> Result<UserOp> {
    let prompt = format!(
        "Parse this user management request into a structured operation.\n\
        Request: {question}\n\
        \n\
        Output EXACTLY one of these formats:\n\
        CREATE_USER: <username> GROUPS: <group1,group2 or empty>\n\
        DELETE_USER: <username>\n\
        CHANGE_PASSWORD: <username>\n\
        ADD_TO_GROUP: <username> GROUP: <groupname>\n\
        LIST_USERS\n\
        \n\
        Output ONLY the format, nothing else."
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 20).await
        .map_err(|e| anyhow!("LLM parse error: {}", e))?;
    let response = response.trim();

    if response.starts_with("CREATE_USER:") {
        let rest = response.trim_start_matches("CREATE_USER:").trim();
        let (username, groups) = if let Some(g_idx) = rest.find("GROUPS:") {
            let uname = rest[..g_idx].trim().to_string();
            let grp_str = rest[g_idx..].trim_start_matches("GROUPS:").trim();
            let groups: Vec<String> = grp_str.split(',')
                .map(|g| g.trim().to_string())
                .filter(|g| !g.is_empty())
                .collect();
            (uname, groups)
        } else {
            (rest.trim().to_string(), vec![])
        };
        return Ok(UserOp::Create { username, groups });
    }

    if response.starts_with("DELETE_USER:") {
        let username = response.trim_start_matches("DELETE_USER:").trim().to_string();
        return Ok(UserOp::Delete { username });
    }

    if response.starts_with("CHANGE_PASSWORD:") {
        let username = response.trim_start_matches("CHANGE_PASSWORD:").trim().to_string();
        return Ok(UserOp::ChangePassword { username });
    }

    if response.starts_with("ADD_TO_GROUP:") {
        let rest = response.trim_start_matches("ADD_TO_GROUP:").trim();
        if let Some(g_idx) = rest.find("GROUP:") {
            let username = rest[..g_idx].trim().to_string();
            let group = rest[g_idx..].trim_start_matches("GROUP:").trim().to_string();
            return Ok(UserOp::AddToGroup { username, group });
        }
    }

    if response.contains("LIST_USERS") {
        return Ok(UserOp::ListUsers);
    }

    Err(anyhow!("Could not parse user management operation from: {}", question))
}

/// List all non-system users (UID >= 1000).
pub fn list_users() -> String {
    let output = std::process::Command::new("getent")
        .args(["passwd"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let users: Vec<String> = output.lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 4 {
                let uid: u32 = fields[2].parse().unwrap_or(0);
                if uid >= 1000 && uid < 65534 {
                    return Some(format!("  {} (uid={})", fields[0], uid));
                }
            }
            None
        })
        .collect();

    if users.is_empty() {
        "No non-system users found.".to_string()
    } else {
        format!("Non-system users:\n{}", users.join("\n"))
    }
}

/// Validate a username (alphanumeric + _ + -, 1-32 chars, not root).
fn validate_username(username: &str) -> Result<()> {
    if username == "root" {
        return Err(anyhow!("Cannot manage the root account via this command"));
    }
    if username.is_empty() || username.len() > 32 {
        return Err(anyhow!("Username must be 1-32 characters"));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(anyhow!("Username must contain only letters, numbers, underscores, and hyphens"));
    }
    Ok(())
}

/// Create a new user with home directory.
pub fn create_user(username: &str, groups: &[String]) -> Result<String> {
    validate_username(username)?;

    // Check if user already exists
    let exists = std::process::Command::new("id")
        .arg(username)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if exists {
        return Err(anyhow!("User '{}' already exists", username));
    }

    // Build useradd command
    let mut args = vec!["useradd".to_string(), "-m".to_string()];
    if !groups.is_empty() {
        args.push("-G".to_string());
        args.push(groups.join(","));
    }
    args.push(username.to_string());

    let status = std::process::Command::new("pkexec")
        .args(&args)
        .status()
        .map_err(|e| anyhow!("pkexec failed: {}", e))?;

    if !status.success() {
        return Err(anyhow!("useradd failed for '{}'", username));
    }

    info!("Created user: {}", username);

    // Register in artifact registry
    let mut registry = crate::artifact_registry::ArtifactRegistry::load();
    let artifact = crate::artifact_registry::CreatedArtifact::new(
        crate::artifact_registry::ArtifactKind::UserAccount,
        format!("user account: {}", username),
        &format!("User account '{}' with home directory", username),
        vec![format!("/home/{}", username)],
        vec![
            format!("pkexec userdel -r {}", username),
        ],
    );
    registry.add(artifact);

    let group_info = if groups.is_empty() {
        String::new()
    } else {
        format!(" (groups: {})", groups.join(", "))
    };

    Ok(format!(
        "User '{}' created successfully{}.\n\
        Home directory: /home/{}\n\
        Set a password with: passwd {}",
        username, group_info, username, username
    ))
}

/// Delete a user and their home directory (always requires confirmation upstream).
pub fn delete_user(username: &str) -> Result<String> {
    validate_username(username)?;

    // Check if user exists
    let exists = std::process::Command::new("id")
        .arg(username)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !exists {
        return Err(anyhow!("User '{}' does not exist", username));
    }

    let status = std::process::Command::new("pkexec")
        .args(["userdel", "-r", username])
        .status()
        .map_err(|e| anyhow!("pkexec failed: {}", e))?;

    if !status.success() {
        return Err(anyhow!("userdel failed for '{}' (user may be logged in)", username));
    }

    info!("Deleted user: {}", username);

    // Update registry
    let mut registry = crate::artifact_registry::ArtifactRegistry::load();
    registry.remove_by_name(&format!("user account: {}", username));

    Ok(format!("User '{}' and home directory deleted.", username))
}

/// Add a user to a group.
pub fn add_user_to_group(username: &str, group: &str) -> Result<String> {
    validate_username(username)?;

    let status = std::process::Command::new("pkexec")
        .args(["usermod", "-aG", group, username])
        .status()
        .map_err(|e| anyhow!("pkexec failed: {}", e))?;

    if !status.success() {
        return Err(anyhow!("usermod failed: check that group '{}' exists", group));
    }

    Ok(format!("Added '{}' to group '{}'. Changes take effect on next login.", username, group))
}

/// Build a plan summary string for user operations (shown BEFORE execution).
pub fn plan_summary(op: &UserOp) -> String {
    match op {
        UserOp::Create { username, groups } => {
            let grp = if groups.is_empty() {
                "no additional groups".into()
            } else {
                format!("groups: {}", groups.join(", "))
            };
            format!("Create user '{}' with home directory ({})", username, grp)
        }
        UserOp::Delete { username } => {
            format!("DELETE user '{}' and their home directory /home/{} — THIS CANNOT BE UNDONE", username, username)
        }
        UserOp::ChangePassword { username } => {
            format!("Change password for user '{}'", username)
        }
        UserOp::AddToGroup { username, group } => {
            format!("Add user '{}' to group '{}'", username, group)
        }
        UserOp::ListUsers => "List all non-system users".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        assert!(validate_username("alice").is_ok());
        assert!(validate_username("bob_test").is_ok());
        assert!(validate_username("user-123").is_ok());
    }

    #[test]
    fn test_validate_username_invalid() {
        assert!(validate_username("root").is_err());
        assert!(validate_username("").is_err());
        assert!(validate_username("user with spaces").is_err());
        assert!(validate_username("user@domain").is_err());
    }

    #[test]
    fn test_plan_summary_delete() {
        let op = UserOp::Delete { username: "alice".into() };
        let summary = plan_summary(&op);
        assert!(summary.contains("CANNOT BE UNDONE"));
    }
}
