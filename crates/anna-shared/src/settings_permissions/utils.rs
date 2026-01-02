// v0.0.582: Settings Permissions (Phase 158)
// Utility functions for permissions

use super::manager::PermissionManager;

/// Format permissions for display
pub fn format_permissions(manager: &PermissionManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Permissions ===\n\n");

    if let Some(role) = manager.active_role() {
        output.push_str(&format!("Active Role: {}\n", role.name));
        output.push_str(&format!("Description: {}\n\n", role.description));
    }

    output.push_str("--- Available Roles ---\n");
    for role in manager.roles() {
        let marker = if manager.active_role().map(|r| &r.name) == Some(&role.name) {
            "*"
        } else {
            " "
        };
        output.push_str(&format!(
            "{} {} - {} ({})\n",
            marker, role.name, role.description, role.default_level
        ));
    }

    output
}

/// Check if query is about permissions
pub fn is_permissions_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("permission")
        || lower.contains("access control")
        || lower.contains("role")
}

/// Fun fact about permissions
pub fn settings_permissions_fun_fact() -> &'static str {
    "Anna uses role-based permissions to control who can modify settings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_permissions() {
        let mgr = PermissionManager::new();
        let output = format_permissions(&mgr);
        assert!(output.contains("Permissions"));
    }

    #[test]
    fn test_is_permissions_query() {
        assert!(is_permissions_query("show permissions"));
        assert!(is_permissions_query("access control settings"));
        assert!(!is_permissions_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_permissions_fun_fact();
        assert!(fact.contains("permission"));
    }
}
