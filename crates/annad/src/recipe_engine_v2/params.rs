//! Parameter extraction module.
//!
//! Handles extracting parameters from queries and checking parameter requirements.

use anna_shared::recipe_engine::Recipe as LearnedRecipe;

/// Extract params that need to be filled from query
pub fn extract_missing_params(query: &str, recipe: &LearnedRecipe) -> Vec<String> {
    let query_lower = query.to_lowercase();
    let mut missing = vec![];

    for param in &recipe.parameters {
        if param.required && param.default.is_none() {
            // Try to extract from query
            let extracted = extract_param_value(&query_lower, &param.name, &param.extraction_hint);
            if extracted.is_none() {
                missing.push(param.name.clone());
            }
        }
    }

    missing
}

/// Check if recipe has defaults for all required params
pub fn has_default_params(recipe: &LearnedRecipe) -> bool {
    recipe
        .parameters
        .iter()
        .filter(|p| p.required)
        .all(|p| p.default.is_some())
}

/// Try to extract a parameter value from query
pub fn extract_param_value(query: &str, param_name: &str, hint: &str) -> Option<String> {
    let words: Vec<&str> = query.split_whitespace().collect();

    // Common extraction patterns
    match param_name {
        "service_name" | "service" => {
            // Look for word before "service" or known service names
            let services = [
                "nginx",
                "sshd",
                "httpd",
                "docker",
                "mysql",
                "postgresql",
                "redis",
            ];
            for word in &words {
                let w = word.trim_matches(|c: char| !c.is_alphanumeric());
                if services.contains(&w) {
                    return Some(w.to_string());
                }
            }
            // Word before "service"
            if let Some(pos) = words.iter().position(|&w| w == "service") {
                if pos > 0 {
                    return Some(words[pos - 1].to_string());
                }
            }
        }
        "package_name" | "package" => {
            // Word after "install", "remove", "is X installed"
            if let Some(pos) = words
                .iter()
                .position(|&w| w == "installed" || w == "install")
            {
                if words.len() > pos + 1 {
                    return Some(words[pos + 1].to_string());
                }
                if pos > 0 && words[pos] == "installed" {
                    return Some(words[pos - 1].to_string());
                }
            }
        }
        "mount_path" | "path" => {
            // Look for path-like strings
            for word in &words {
                if word.starts_with('/') || word.starts_with("~/") {
                    return Some(word.to_string());
                }
            }
        }
        _ => {}
    }

    None
}
