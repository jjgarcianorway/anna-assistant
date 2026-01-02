//! Intent detection functions using seed recipes.

use crate::config_seed_recipes::find_seed_recipe;
use crate::config_types::ConfigIntent;

/// Detect if a query is requesting a vim config change.
/// v0.0.264: Now uses seed recipes (bootstrap, lower confidence).
pub fn detect_vim_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    // Must mention vim (but not neovim)
    if !query_lower.contains("vim")
        || query_lower.contains("neovim")
        || query_lower.contains("nvim")
    {
        return None;
    }

    find_seed_recipe(query, "vim")
}

/// v0.0.264: Detect neovim config intent using seed recipes
pub fn detect_neovim_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("neovim") && !query_lower.contains("nvim") {
        return None;
    }

    find_seed_recipe(query, "neovim")
}

/// v0.0.264: Detect nano config intent using seed recipes
pub fn detect_nano_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("nano") {
        return None;
    }

    find_seed_recipe(query, "nano")
}

/// v0.0.264: Detect helix config intent using seed recipes
pub fn detect_helix_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("helix") && !query_lower.contains("hx ") {
        return None;
    }

    find_seed_recipe(query, "helix")
}

/// v0.0.264: Detect emacs config intent using seed recipes
pub fn detect_emacs_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("emacs") {
        return None;
    }

    find_seed_recipe(query, "emacs")
}

/// Detect config intent from query and entities.
/// v0.0.264: Uses seed recipes as bootstrap, prefers learned recipes.
pub fn detect_config_intent(query: &str, entities: &[String]) -> Option<ConfigIntent> {
    // Check specific editors in order of specificity
    if let Some(intent) = detect_neovim_config_intent(query) {
        return Some(intent);
    }
    if let Some(intent) = detect_vim_config_intent(query) {
        return Some(intent);
    }
    if let Some(intent) = detect_nano_config_intent(query) {
        return Some(intent);
    }
    if let Some(intent) = detect_helix_config_intent(query) {
        return Some(intent);
    }
    if let Some(intent) = detect_emacs_config_intent(query) {
        return Some(intent);
    }

    // Check entities for editor mentions
    for entity in entities {
        let e = entity.to_lowercase();
        if e == "neovim" || e == "nvim" {
            return detect_neovim_config_intent(&format!("neovim {}", query));
        }
        if e == "vim" {
            return detect_vim_config_intent(&format!("vim {}", query));
        }
        if e == "nano" {
            return detect_nano_config_intent(&format!("nano {}", query));
        }
        if e == "helix" || e == "hx" {
            return detect_helix_config_intent(&format!("helix {}", query));
        }
        if e == "emacs" {
            return detect_emacs_config_intent(&format!("emacs {}", query));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_vim_uses_seed_recipe() {
        let intent = detect_vim_config_intent("enable syntax highlighting in vim").unwrap();
        assert_eq!(intent.target.app_id, "vim");
        // Seed recipes have lower confidence than learned recipes
        assert!(intent.confidence < 0.8);
    }

    #[test]
    fn test_detect_neovim_uses_seed_recipe() {
        let intent = detect_neovim_config_intent("nvim syntax highlighting").unwrap();
        assert_eq!(intent.target.app_id, "neovim");
        assert!(intent.confidence < 0.8);
    }

    #[test]
    fn test_neovim_not_detected_as_vim() {
        assert!(detect_vim_config_intent("neovim syntax highlighting").is_none());
    }
}
