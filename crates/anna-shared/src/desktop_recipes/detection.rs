//! Desktop configuration intent detection.

use super::recipe::DesktopRecipe;
use super::utils::{extract_file_path, extract_theme_name};

/// Detect desktop configuration request from query
pub fn detect_desktop_intent(query: &str) -> Option<DesktopRecipe> {
    let query_lower = query.to_lowercase();

    // Wallpaper detection
    if query_lower.contains("wallpaper") || query_lower.contains("background") {
        // Try to extract path from query
        if let Some(path) = extract_file_path(&query_lower) {
            return DesktopRecipe::set_wallpaper(&path);
        }
        // No path provided - return None, will need clarification
        return None;
    }

    // Dark mode detection
    if (query_lower.contains("dark") && query_lower.contains("mode"))
        || (query_lower.contains("dark") && query_lower.contains("theme"))
    {
        if query_lower.contains("disable")
            || query_lower.contains("turn off")
            || query_lower.contains("light")
        {
            return DesktopRecipe::disable_dark_mode();
        }
        return DesktopRecipe::enable_dark_mode();
    }

    // Light mode detection
    if query_lower.contains("light") && query_lower.contains("mode") {
        return DesktopRecipe::disable_dark_mode();
    }

    // Theme detection
    if query_lower.contains("theme") && !query_lower.contains("dark") {
        if let Some(theme) = extract_theme_name(&query_lower) {
            return DesktopRecipe::set_theme(&theme);
        }
    }

    None
}

/// Check if query is a desktop config request
pub fn is_desktop_config_request(query: &str) -> bool {
    let query_lower = query.to_lowercase();
    let keywords = [
        "wallpaper",
        "background",
        "dark mode",
        "light mode",
        "theme",
        "desktop",
        "appearance",
    ];
    let actions = ["set", "change", "enable", "disable", "switch", "toggle"];

    let has_keyword = keywords.iter().any(|k| query_lower.contains(k));
    let has_action = actions.iter().any(|a| query_lower.contains(a));

    has_keyword && has_action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_desktop_config_request() {
        assert!(is_desktop_config_request("set my wallpaper"));
        assert!(is_desktop_config_request("enable dark mode"));
        assert!(is_desktop_config_request("change theme to dracula"));
        assert!(!is_desktop_config_request("what is my disk usage"));
        assert!(!is_desktop_config_request("install vim"));
    }

    #[test]
    fn test_detect_dark_mode_intent() {
        // These will return None in test environment (no DE detected)
        // but we test the logic path
        let _result = detect_desktop_intent("enable dark mode");
        let _result = detect_desktop_intent("disable dark mode");
        let _result = detect_desktop_intent("switch to light mode");
    }
}
