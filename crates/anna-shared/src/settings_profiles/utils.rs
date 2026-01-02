// v0.0.565: Settings Profiles (Phase 141)
// Utility functions for profile formatting and querying

use super::manager::ProfileManager;

/// Format profiles list for display
pub fn format_profiles_list(manager: &ProfileManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Profiles ===\n\n");

    if manager.count() == 0 {
        output.push_str("No profiles configured.\n");
        return output;
    }

    for profile in manager.list() {
        let active = if profile.meta.is_active { " [ACTIVE]" } else { "" };
        let default = if Some(&profile.id) == manager.default_id.as_ref() {
            " (default)"
        } else {
            ""
        };

        output.push_str(&format!(
            "• {}{}{}\n  {}\n  Created: {}\n",
            profile.meta.name,
            active,
            default,
            profile.meta.description,
            profile.meta.created.format("%Y-%m-%d %H:%M")
        ));

        if !profile.meta.tags.is_empty() {
            output.push_str(&format!("  Tags: {}\n", profile.meta.tags.join(", ")));
        }
        output.push('\n');
    }

    output
}

/// Check if query is about profiles
pub fn is_profile_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("profile")
        || lower.contains("configuration")
        || lower.contains("switch settings")
        || lower.contains("create profile")
        || lower.contains("delete profile")
}

/// Fun fact about profiles
pub fn profiles_fun_fact() -> &'static str {
    "You can create multiple named settings profiles for different use cases!"
}
