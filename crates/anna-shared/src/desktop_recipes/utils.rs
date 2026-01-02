//! Utility functions for desktop recipe parsing.

/// Extract file path from query (simple heuristic)
pub fn extract_file_path(query: &str) -> Option<String> {
    // Look for paths starting with / or ~
    let words: Vec<&str> = query.split_whitespace().collect();
    for word in words {
        let clean = word.trim_matches(|c| c == '\'' || c == '"');
        if clean.starts_with('/') || clean.starts_with('~') {
            // Expand ~ to $HOME
            let expanded = if clean.starts_with('~') {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                clean.replacen('~', &home, 1)
            } else {
                clean.to_string()
            };
            return Some(expanded);
        }
    }
    None
}

/// Extract theme name from query (simple heuristic)
pub fn extract_theme_name(query: &str) -> Option<String> {
    // Common theme patterns - longer names first to match "arc-dark" before "arc"
    let known_themes = [
        "adwaita-dark",
        "adwaita",
        "arc-dark",
        "arc",
        "breeze-dark",
        "breeze",
        "dracula",
        "nord",
        "gruvbox",
        "materia",
        "numix",
    ];

    let query_lower = query.to_lowercase();
    for theme in known_themes {
        if query_lower.contains(theme) {
            // Capitalize properly
            return Some(
                theme
                    .split('-')
                    .map(capitalize_first)
                    .collect::<Vec<_>>()
                    .join("-"),
            );
        }
    }
    None
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_path() {
        assert_eq!(
            extract_file_path("set wallpaper to /home/user/pic.jpg"),
            Some("/home/user/pic.jpg".to_string())
        );
        assert!(extract_file_path("set my wallpaper").is_none());
    }

    #[test]
    fn test_extract_theme_name() {
        assert_eq!(
            extract_theme_name("change theme to dracula"),
            Some("Dracula".to_string())
        );
        assert_eq!(
            extract_theme_name("set arc-dark theme"),
            Some("Arc-Dark".to_string())
        );
        assert!(extract_theme_name("change theme").is_none());
    }
}
