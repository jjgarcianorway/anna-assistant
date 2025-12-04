//! Debug Toggle v0.0.82
//!
//! Natural language debug mode toggle.
//! "enable debug" -> transcript_mode=debug, debug_level=2
//! "disable debug" -> transcript_mode=human, debug_level=0
//!
//! The toggle is handled via ANNA_DEBUG_TRANSCRIPT env var for session-based
//! changes, or by writing to config.toml for persistent changes.

use std::env;
use std::fs;
use std::path::Path;

/// Debug mode state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugMode {
    /// Human mode - clean output, no internals
    Human,
    /// Debug mode - full internal details
    Debug,
}

impl DebugMode {
    /// Human-readable name
    pub fn as_str(&self) -> &'static str {
        match self {
            DebugMode::Human => "human",
            DebugMode::Debug => "debug",
        }
    }

    /// Current mode based on env vars and config
    pub fn current() -> Self {
        // Check env vars first (highest priority)
        if env::var("ANNA_DEBUG_TRANSCRIPT").map_or(false, |v| v == "1") {
            return DebugMode::Debug;
        }
        if env::var("ANNA_UI_TRANSCRIPT_MODE").map_or(false, |v| v == "debug") {
            return DebugMode::Debug;
        }

        // Check config file
        let config = crate::config::AnnaConfig::load();
        if config.ui.transcript_mode == "debug" || config.ui.debug_level >= 2 {
            return DebugMode::Debug;
        }

        DebugMode::Human
    }

    /// Is debug mode currently enabled?
    pub fn is_debug_enabled() -> bool {
        Self::current() == DebugMode::Debug
    }
}

/// Result of toggle operation
#[derive(Debug, Clone)]
pub struct ToggleResult {
    /// Whether the toggle succeeded
    pub success: bool,
    /// New mode after toggle
    pub new_mode: DebugMode,
    /// Human-readable message
    pub message: String,
    /// Whether this is a session-only change (env var)
    pub session_only: bool,
}

/// Parse debug toggle request to determine enable/disable
pub fn parse_debug_toggle_request(request: &str) -> Option<bool> {
    let lower = request.to_lowercase();

    // Enable patterns
    let enable_patterns = [
        "enable debug",
        "debug on",
        "turn debug on",
        "debug mode on",
        "enable debug mode",
        "start debug",
        "show debug",
        "verbose mode",
        "verbose on",
    ];

    for pattern in enable_patterns {
        if lower.contains(pattern) {
            return Some(true);
        }
    }

    // Disable patterns
    let disable_patterns = [
        "disable debug",
        "debug off",
        "turn debug off",
        "debug mode off",
        "disable debug mode",
        "stop debug",
        "hide debug",
        "normal mode",
        "verbose off",
    ];

    for pattern in disable_patterns {
        if lower.contains(pattern) {
            return Some(false);
        }
    }

    // Toggle pattern (toggle current state)
    if lower.contains("toggle debug") {
        return Some(!DebugMode::is_debug_enabled());
    }

    None
}

/// Toggle debug mode for this session (via env var)
pub fn toggle_debug_session(enable: bool) -> ToggleResult {
    if enable {
        env::set_var("ANNA_DEBUG_TRANSCRIPT", "1");
        ToggleResult {
            success: true,
            new_mode: DebugMode::Debug,
            message: "Debug mode enabled for this session. You'll see full internal details."
                .to_string(),
            session_only: true,
        }
    } else {
        env::remove_var("ANNA_DEBUG_TRANSCRIPT");
        env::remove_var("ANNA_UI_TRANSCRIPT_MODE");
        ToggleResult {
            success: true,
            new_mode: DebugMode::Human,
            message: "Debug mode disabled. Back to clean output.".to_string(),
            session_only: true,
        }
    }
}

/// Toggle debug mode persistently (writes to config file)
pub fn toggle_debug_persistent(enable: bool) -> ToggleResult {
    let config_path = Path::new("/etc/anna/config.toml");

    // Check if we can write to config
    if !config_path.exists() {
        // Create minimal config
        let mode = if enable { "debug" } else { "human" };
        let level = if enable { 2 } else { 0 };
        let content = format!(
            r#"# Anna Configuration
[ui]
transcript_mode = "{}"
debug_level = {}
"#,
            mode, level
        );

        match fs::write(config_path, content) {
            Ok(_) => {
                // Also set env for immediate effect
                if enable {
                    env::set_var("ANNA_DEBUG_TRANSCRIPT", "1");
                } else {
                    env::remove_var("ANNA_DEBUG_TRANSCRIPT");
                }

                return ToggleResult {
                    success: true,
                    new_mode: if enable {
                        DebugMode::Debug
                    } else {
                        DebugMode::Human
                    },
                    message: format!(
                        "Debug mode {} and saved to config.",
                        if enable { "enabled" } else { "disabled" }
                    ),
                    session_only: false,
                };
            }
            Err(e) => {
                // Fall back to session-only
                return toggle_debug_session_with_warning(
                    enable,
                    &format!("Cannot write config ({}), session-only", e),
                );
            }
        }
    }

    // Read existing config
    let content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            return toggle_debug_session_with_warning(
                enable,
                &format!("Cannot read config ({}), session-only", e),
            );
        }
    };

    // Update transcript_mode and debug_level
    let mode = if enable { "debug" } else { "human" };
    let level = if enable { "2" } else { "0" };

    let new_content = update_config_value(&content, "transcript_mode", &format!("\"{}\"", mode));
    let new_content = update_config_value(&new_content, "debug_level", level);

    // Write back
    match fs::write(config_path, new_content) {
        Ok(_) => {
            // Also set env for immediate effect
            if enable {
                env::set_var("ANNA_DEBUG_TRANSCRIPT", "1");
            } else {
                env::remove_var("ANNA_DEBUG_TRANSCRIPT");
            }

            ToggleResult {
                success: true,
                new_mode: if enable {
                    DebugMode::Debug
                } else {
                    DebugMode::Human
                },
                message: format!(
                    "Debug mode {} and saved to config.",
                    if enable { "enabled" } else { "disabled" }
                ),
                session_only: false,
            }
        }
        Err(e) => toggle_debug_session_with_warning(
            enable,
            &format!("Cannot write config ({}), session-only", e),
        ),
    }
}

/// Toggle with a warning message
fn toggle_debug_session_with_warning(enable: bool, warning: &str) -> ToggleResult {
    let mut result = toggle_debug_session(enable);
    result.message = format!("{}. {}", warning, result.message);
    result
}

/// Update a config value in TOML content
fn update_config_value(content: &str, key: &str, value: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let key_pattern = format!("{} =", key);
    let key_pattern_eq = format!("{}=", key);

    let mut found = false;
    for line in &mut lines {
        let trimmed = line.trim();
        if trimmed.starts_with(&key_pattern) || trimmed.starts_with(&key_pattern_eq) {
            *line = format!("{} = {}", key, value);
            found = true;
            break;
        }
    }

    // If not found, add it under [ui] section
    if !found {
        let mut in_ui_section = false;
        let mut insert_idx = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == "[ui]" {
                in_ui_section = true;
            } else if in_ui_section && trimmed.starts_with('[') {
                // Next section started
                insert_idx = Some(i);
                break;
            }
        }

        if let Some(idx) = insert_idx {
            lines.insert(idx, format!("{} = {}", key, value));
        } else if in_ui_section {
            // At end of file, after [ui]
            lines.push(format!("{} = {}", key, value));
        } else {
            // No [ui] section, add it
            lines.push(String::new());
            lines.push("[ui]".to_string());
            lines.push(format!("{} = {}", key, value));
        }
    }

    lines.join("\n")
}

/// Generate response for debug toggle
pub fn generate_toggle_response(result: &ToggleResult) -> String {
    if result.success {
        if result.new_mode == DebugMode::Debug {
            format!(
                "Debug mode enabled{}. You'll now see:\n\
                 - Full translator output\n\
                 - Evidence IDs and tool names\n\
                 - Junior verification details\n\
                 - Internal timing and parsing info\n\n\
                 To disable: \"disable debug\" or \"debug off\"",
                if result.session_only {
                    " (session only)"
                } else {
                    ""
                }
            )
        } else {
            format!(
                "Debug mode disabled{}. Output is now clean and professional.\n\n\
                 To enable: \"enable debug\" or \"debug on\"",
                if result.session_only {
                    " (session only)"
                } else {
                    ""
                }
            )
        }
    } else {
        result.message.clone()
    }
}

/// Check current debug status for display
pub fn get_debug_status_message() -> String {
    let mode = DebugMode::current();
    match mode {
        DebugMode::Debug => "Debug mode: ENABLED (full internal details visible)".to_string(),
        DebugMode::Human => "Debug mode: disabled (clean output)".to_string(),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_enable_patterns() {
        assert_eq!(parse_debug_toggle_request("enable debug"), Some(true));
        assert_eq!(parse_debug_toggle_request("debug on"), Some(true));
        assert_eq!(parse_debug_toggle_request("turn debug on"), Some(true));
        assert_eq!(parse_debug_toggle_request("enable debug mode"), Some(true));
        assert_eq!(parse_debug_toggle_request("verbose mode"), Some(true));
    }

    #[test]
    fn test_parse_disable_patterns() {
        assert_eq!(parse_debug_toggle_request("disable debug"), Some(false));
        assert_eq!(parse_debug_toggle_request("debug off"), Some(false));
        assert_eq!(parse_debug_toggle_request("turn debug off"), Some(false));
        assert_eq!(parse_debug_toggle_request("disable debug mode"), Some(false));
        assert_eq!(parse_debug_toggle_request("normal mode"), Some(false));
    }

    #[test]
    fn test_parse_no_match() {
        assert_eq!(parse_debug_toggle_request("what is debug"), None);
        assert_eq!(parse_debug_toggle_request("hello"), None);
        assert_eq!(parse_debug_toggle_request("check memory"), None);
    }

    #[test]
    fn test_update_config_value() {
        let content = r#"[ui]
transcript_mode = "human"
debug_level = 0
"#;
        let updated = update_config_value(content, "debug_level", "2");
        assert!(updated.contains("debug_level = 2"));
    }

    #[test]
    fn test_toggle_session() {
        // Clean env first
        env::remove_var("ANNA_DEBUG_TRANSCRIPT");

        let result = toggle_debug_session(true);
        assert!(result.success);
        assert_eq!(result.new_mode, DebugMode::Debug);
        assert!(result.session_only);

        // Check env was set
        assert_eq!(env::var("ANNA_DEBUG_TRANSCRIPT").unwrap(), "1");

        // Disable
        let result = toggle_debug_session(false);
        assert!(result.success);
        assert_eq!(result.new_mode, DebugMode::Human);

        // Clean up
        env::remove_var("ANNA_DEBUG_TRANSCRIPT");
    }
}
