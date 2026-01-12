//! Shell configuration recipes (bash, zsh, fish).
//! v0.0.998: Initial implementation

use crate::changes::append_to_file;
use regex::Regex;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::info;

use super::RecipeResult;

/// Pending shell recipes awaiting confirmation
static PENDING_SHELL: RwLock<Option<HashMap<String, PendingShellChange>>> = RwLock::new(None);

#[derive(Clone)]
struct PendingShellChange {
    content: String,
    file: String,
    description: String,
}

/// Detect the user's shell
fn detect_shell() -> (&'static str, String) {
    // Check SHELL env var
    let shell = std::env::var("SHELL").unwrap_or_default();

    if shell.contains("zsh") {
        return ("zsh", shellrc_path("zsh"));
    } else if shell.contains("fish") {
        return ("fish", shellrc_path("fish"));
    }

    ("bash", shellrc_path("bash"))
}

fn shellrc_path(shell: &str) -> String {
    let home = dirs::home_dir().unwrap_or_default();

    match shell {
        "zsh" => home.join(".zshrc").to_string_lossy().to_string(),
        "fish" => dirs::config_dir()
            .unwrap_or(home.join(".config"))
            .join("fish/config.fish")
            .to_string_lossy()
            .to_string(),
        _ => home.join(".bashrc").to_string_lossy().to_string(),
    }
}

/// Try to match a shell-related recipe
pub fn try_recipe(q: &str) -> Option<RecipeResult> {
    // Add alias
    if q.contains("alias") && (q.contains("add") || q.contains("create") || q.contains("set")) {
        if let Some((name, value)) = extract_alias(q) {
            return Some(offer_add_alias(&name, &value));
        }
        return Some(ask_for_alias());
    }

    // Common alias: ll
    if q.contains("ll") && q.contains("alias") {
        return Some(offer_ll_alias());
    }

    // Add to PATH
    if q.contains("path") && (q.contains("add") || q.contains("append")) {
        if let Some(path) = extract_path(q) {
            return Some(offer_add_to_path(&path));
        }
        return Some(ask_for_path());
    }

    // Environment variable
    if q.contains("export") || (q.contains("environment") && q.contains("variable")) {
        if let Some((name, value)) = extract_env_var(q) {
            return Some(offer_export_var(&name, &value));
        }
    }

    None
}

fn extract_alias(text: &str) -> Option<(String, String)> {
    // Try to match: alias name='value' or alias name="value"
    let re = Regex::new(r#"alias\s+(\w+)\s*=\s*['"]([^'"]+)['"]"#).ok()?;
    if let Some(caps) = re.captures(text) {
        return Some((caps[1].to_string(), caps[2].to_string()));
    }

    // Try: ll='ls -la' format
    let re2 = Regex::new(r#"(\w+)\s*=\s*['"]([^'"]+)['"]"#).ok()?;
    if let Some(caps) = re2.captures(text) {
        return Some((caps[1].to_string(), caps[2].to_string()));
    }

    None
}

fn extract_path(text: &str) -> Option<String> {
    // Look for paths like /usr/local/bin or ~/bin
    let re = Regex::new(r"((?:/[\w.-]+)+|~/[\w./]+)").ok()?;
    re.find(text).map(|m| m.as_str().to_string())
}

fn extract_env_var(text: &str) -> Option<(String, String)> {
    // Try: export NAME=value or NAME=value
    let re = Regex::new(r#"(?:export\s+)?(\w+)\s*=\s*['"]?([^'"]+)['"]?"#).ok()?;
    if let Some(caps) = re.captures(text) {
        return Some((caps[1].to_string(), caps[2].to_string()));
    }
    None
}

fn ask_for_alias() -> RecipeResult {
    RecipeResult {
        success: true,
        message: "Tell me the alias you want to add, like: 'add alias ll=\"ls -la\"'".to_string(),
        needs_confirmation: false,
        confirmation_prompt: None,
    }
}

fn ask_for_path() -> RecipeResult {
    RecipeResult {
        success: true,
        message: "What directory should I add to your PATH? Tell me like: 'add /usr/local/bin to path'".to_string(),
        needs_confirmation: false,
        confirmation_prompt: None,
    }
}

fn offer_add_alias(name: &str, value: &str) -> RecipeResult {
    let (shell, rc_file) = detect_shell();
    let content = format!("alias {}='{}'", name, value);

    store_pending("shell-alias", PendingShellChange {
        content: content.clone(),
        file: rc_file.clone(),
        description: format!("Add alias {}='{}'", name, value),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I'll add this alias to your {}:\n  {}\n\nReload your shell or run 'source {}' to use it.",
            rc_file, content, rc_file
        ),
        needs_confirmation: true,
        confirmation_prompt: Some("Add this alias?".to_string()),
    }
}

fn offer_ll_alias() -> RecipeResult {
    offer_add_alias("ll", "ls -la")
}

fn offer_add_to_path(path: &str) -> RecipeResult {
    let (shell, rc_file) = detect_shell();

    let content = if shell == "fish" {
        format!("set -gx PATH {} $PATH", path)
    } else {
        format!("export PATH=\"{}:$PATH\"", path)
    };

    store_pending("shell-path", PendingShellChange {
        content: content.clone(),
        file: rc_file.clone(),
        description: format!("Add {} to PATH", path),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I'll add {} to your PATH in {}:\n  {}\n\nReload your shell to apply.",
            path, rc_file, content
        ),
        needs_confirmation: true,
        confirmation_prompt: Some("Add to PATH?".to_string()),
    }
}

fn offer_export_var(name: &str, value: &str) -> RecipeResult {
    let (shell, rc_file) = detect_shell();

    let content = if shell == "fish" {
        format!("set -gx {} {}", name, value)
    } else {
        format!("export {}=\"{}\"", name, value)
    };

    store_pending("shell-export", PendingShellChange {
        content: content.clone(),
        file: rc_file.clone(),
        description: format!("Set {}={}", name, value),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I'll add this to {}:\n  {}\n\nReload your shell to apply.",
            rc_file, content
        ),
        needs_confirmation: true,
        confirmation_prompt: Some("Add this environment variable?".to_string()),
    }
}

fn store_pending(id: &str, change: PendingShellChange) {
    if let Ok(mut guard) = PENDING_SHELL.write() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(id.to_string(), change);
    }
}

fn take_pending(id: &str) -> Option<PendingShellChange> {
    if let Ok(mut guard) = PENDING_SHELL.write() {
        if let Some(map) = guard.as_mut() {
            return map.remove(id);
        }
    }
    None
}

/// Execute a confirmed shell recipe
pub fn execute_confirmed(recipe_id: &str) -> RecipeResult {
    let pending = match take_pending(recipe_id) {
        Some(p) => p,
        None => {
            return RecipeResult {
                success: false,
                message: "Recipe expired or not found. Please try again.".to_string(),
                needs_confirmation: false,
                confirmation_prompt: None,
            };
        }
    };

    match append_to_file(&pending.file, &pending.content, recipe_id, &pending.description, "shell") {
        Ok(_) => {
            info!("Applied shell recipe: {}", recipe_id);
            RecipeResult {
                success: true,
                message: format!(
                    "Done! Added to {}:\n  {}\n\nRun 'source {}' or open a new terminal to apply.",
                    pending.file, pending.content, pending.file
                ),
                needs_confirmation: false,
                confirmation_prompt: None,
            }
        }
        Err(e) => RecipeResult {
            success: false,
            message: format!("Failed to apply changes: {}", e),
            needs_confirmation: false,
            confirmation_prompt: None,
        },
    }
}
