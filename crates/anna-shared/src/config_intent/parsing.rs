//! Config hint parsing and extraction logic.

use super::types::{ConfigFeatureHint, ConfigHint};

impl ConfigHint {
    /// v0.0.264: Create a config hint from a user query.
    /// Returns None if query doesn't look like a config request.
    pub fn from_query(query: &str) -> Option<Self> {
        let q = query.to_lowercase();

        // Detect app/editor
        let app_id = if q.contains("neovim") || q.contains("nvim") {
            "neovim"
        } else if q.contains("vim") {
            "vim"
        } else if q.contains("nano") {
            "nano"
        } else if q.contains("helix") || q.contains(" hx ") {
            "helix"
        } else if q.contains("emacs") {
            "emacs"
        } else if q.contains("micro") {
            "micro"
        } else if q.contains("code") || q.contains("vscode") {
            "vscode"
        } else {
            return None; // No editor detected
        };

        // Detect feature
        let feature = ConfigFeatureHint::from_query(query);

        // Detect enable/disable
        let enable = !q.contains("disable")
            && !q.contains("turn off")
            && !q.contains("no ")
            && !q.contains("hide ")
            && !q.contains("remove ");

        // Extract optional parameter (theme name, tab width, etc.)
        let param = extract_param(&q);

        Some(ConfigHint {
            app_id: app_id.to_string(),
            feature,
            enable,
            param,
        })
    }

    /// Build specialist hint text for the query.
    /// This helps the specialist (Sofia) understand what the user wants.
    pub fn to_specialist_context(&self) -> String {
        let action = if self.enable { "enable" } else { "disable" };
        let param_text = self
            .param
            .as_ref()
            .map(|p| format!(" (value: {})", p))
            .unwrap_or_default();

        format!(
            "User wants to {} {} in {}{}. Config file: {}",
            action,
            self.feature,
            self.app_id,
            param_text,
            config_path_for_app(&self.app_id)
        )
    }
}

/// Extract parameter value from query (theme name, tab width, etc.)
fn extract_param(query: &str) -> Option<String> {
    // Look for patterns like "theme gruvbox", "4 spaces", "tab width 2"
    let words: Vec<&str> = query.split_whitespace().collect();

    // Theme names after "theme" or "colorscheme"
    for (i, word) in words.iter().enumerate() {
        if (*word == "theme" || *word == "colorscheme") && i + 1 < words.len() {
            let next = words[i + 1];
            if !["in", "for", "on", "to"].contains(&next) {
                return Some(next.to_string());
            }
        }
    }

    // Tab width patterns
    if query.contains("2 spaces") || query.contains("2-space") {
        return Some("2".to_string());
    }
    if query.contains("4 spaces") || query.contains("4-space") {
        return Some("4".to_string());
    }

    None
}

/// Get config file path for an app
fn config_path_for_app(app_id: &str) -> &'static str {
    match app_id {
        "vim" => "~/.vimrc",
        "neovim" | "nvim" => "~/.config/nvim/init.vim",
        "nano" => "~/.nanorc",
        "helix" => "~/.config/helix/config.toml",
        "emacs" => "~/.emacs",
        "micro" => "~/.config/micro/settings.json",
        "vscode" | "code" => "settings.json (via GUI)",
        _ => "unknown",
    }
}

/// Check if a query is a config edit request (quick check) (v0.0.263: expanded)
pub fn is_config_edit_request(query: &str) -> bool {
    let q = query.to_lowercase();
    // Must have both an editor and an action
    let has_editor = q.contains("vim")
        || q.contains("neovim")
        || q.contains("nvim")
        || q.contains("nano")
        || q.contains("emacs")
        || q.contains("helix")
        || q.contains("micro");
    let has_action = q.contains("enable")
        || q.contains("disable")
        || q.contains("add")
        || q.contains("set")
        || q.contains("turn on")
        || q.contains("turn off")
        || q.contains("configure")
        || q.contains("theme")
        || q.contains("colorscheme")
        || q.contains("color scheme")
        || q.contains("syntax");
    has_editor && has_action
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_types::ConfigTarget;

    #[test]
    fn test_config_hint_from_query_vim() {
        let hint = ConfigHint::from_query("enable syntax highlighting in vim").unwrap();
        assert_eq!(hint.app_id, "vim");
        assert_eq!(hint.feature, ConfigFeatureHint::Syntax);
        assert!(hint.enable);
    }

    #[test]
    fn test_config_hint_disable() {
        let hint = ConfigHint::from_query("disable line numbers in nano").unwrap();
        assert_eq!(hint.app_id, "nano");
        assert_eq!(hint.feature, ConfigFeatureHint::LineNumbers);
        assert!(!hint.enable);
    }

    #[test]
    fn test_config_hint_no_editor() {
        assert!(ConfigHint::from_query("enable syntax highlighting").is_none());
    }

    #[test]
    fn test_config_hint_specialist_context() {
        let hint = ConfigHint::from_query("enable syntax in vim").unwrap();
        let ctx = hint.to_specialist_context();
        assert!(ctx.contains("enable"));
        assert!(ctx.contains("vim"));
        assert!(ctx.contains(".vimrc"));
    }

    #[test]
    fn test_config_target_expand() {
        let target = ConfigTarget::vim();
        let path = target.expand_path();
        assert!(path.to_string_lossy().contains(".vimrc"));
        assert!(!path.to_string_lossy().contains("$HOME"));
    }

    #[test]
    fn test_is_config_edit_request() {
        assert!(is_config_edit_request("enable syntax in vim"));
        assert!(is_config_edit_request("add line numbers to vim"));
        assert!(!is_config_edit_request("what is vim"));
        assert!(!is_config_edit_request("enable something"));
    }

    #[test]
    fn test_is_config_edit_request_expanded() {
        assert!(is_config_edit_request("neovim syntax highlighting"));
        assert!(is_config_edit_request("nvim theme"));
        assert!(is_config_edit_request("helix colorscheme"));
        assert!(is_config_edit_request("disable syntax in nano"));
    }
}
