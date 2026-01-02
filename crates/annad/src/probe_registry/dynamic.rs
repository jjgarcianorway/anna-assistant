//! Dynamic probe generation
//!
//! Extracted from probe_registry.rs for modularization.

use super::mappings::probe_id_to_command;

/// v0.0.797: Generate dynamic command for probe ID if not found in static registry
/// Supports command_v_<tool> probes for any tool name
pub fn probe_id_to_command_dynamic(id: &str) -> Option<String> {
    // First check static registry
    if let Some(cmd) = probe_id_to_command(id) {
        return Some(cmd.to_string());
    }

    // Dynamic command_v_<tool> probe generation
    if let Some(tool_name) = id.strip_prefix("command_v_") {
        // Validate tool name (alphanumeric, hyphen, underscore only)
        if tool_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Some(format!("sh -lc 'command -v {}'", tool_name));
        }
    }

    // Dynamic pacman_q_<package> probe generation
    if let Some(pkg_name) = id.strip_prefix("pacman_q_") {
        // Validate package name
        if pkg_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Some(format!("pacman -Q {} 2>/dev/null", pkg_name));
        }
    }

    None
}
