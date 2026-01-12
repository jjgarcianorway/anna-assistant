//! Git configuration recipes.
//! v0.0.998: Initial implementation

use crate::changes::run_command;
use regex::Regex;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::info;

use super::RecipeResult;

/// Pending git recipes awaiting confirmation
static PENDING_GIT: RwLock<Option<HashMap<String, PendingGitChange>>> = RwLock::new(None);

#[derive(Clone)]
struct PendingGitChange {
    commands: Vec<String>,
    description: String,
}

/// Try to match a git-related recipe
pub fn try_recipe(q: &str) -> Option<RecipeResult> {
    // Set git email
    if q.contains("git") && q.contains("email") {
        if let Some(email) = extract_email(q) {
            return Some(offer_set_email(&email));
        }
        return Some(ask_for_email());
    }

    // Set git name
    if q.contains("git") && (q.contains("name") || q.contains("user")) && !q.contains("email") {
        if let Some(name) = extract_quoted_string(q) {
            return Some(offer_set_name(&name));
        }
        return Some(ask_for_name());
    }

    // Git aliases
    if q.contains("git") && q.contains("alias") {
        return Some(offer_git_aliases());
    }

    // Default branch
    if q.contains("git") && q.contains("default") && q.contains("branch") {
        return Some(offer_default_branch());
    }

    None
}

fn extract_email(text: &str) -> Option<String> {
    let email_re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").ok()?;
    email_re.find(text).map(|m| m.as_str().to_string())
}

fn extract_quoted_string(text: &str) -> Option<String> {
    // Try double quotes first
    if let Some(start) = text.find('"') {
        if let Some(end) = text[start + 1..].find('"') {
            return Some(text[start + 1..start + 1 + end].to_string());
        }
    }
    // Try single quotes
    if let Some(start) = text.find('\'') {
        if let Some(end) = text[start + 1..].find('\'') {
            return Some(text[start + 1..start + 1 + end].to_string());
        }
    }
    None
}

fn ask_for_email() -> RecipeResult {
    RecipeResult {
        success: true,
        message: "What email address should I set for git? Just tell me like: 'set git email to user@example.com'".to_string(),
        needs_confirmation: false,
        confirmation_prompt: None,
    }
}

fn ask_for_name() -> RecipeResult {
    RecipeResult {
        success: true,
        message: "What name should I set for git? Tell me like: 'set git name to \"John Doe\"'".to_string(),
        needs_confirmation: false,
        confirmation_prompt: None,
    }
}

fn offer_set_email(email: &str) -> RecipeResult {
    let cmd = format!("git config --global user.email \"{}\"", email);

    store_pending("git-email", PendingGitChange {
        commands: vec![cmd.clone()],
        description: format!("Set git email to {}", email),
    });

    RecipeResult {
        success: true,
        message: format!("I'll set your git email globally:\n  {}\n\nThis affects all your repositories.", cmd),
        needs_confirmation: true,
        confirmation_prompt: Some("Apply this change?".to_string()),
    }
}

fn offer_set_name(name: &str) -> RecipeResult {
    let cmd = format!("git config --global user.name \"{}\"", name);

    store_pending("git-name", PendingGitChange {
        commands: vec![cmd.clone()],
        description: format!("Set git name to {}", name),
    });

    RecipeResult {
        success: true,
        message: format!("I'll set your git name globally:\n  {}\n\nThis affects all your repositories.", cmd),
        needs_confirmation: true,
        confirmation_prompt: Some("Apply this change?".to_string()),
    }
}

fn offer_git_aliases() -> RecipeResult {
    let commands = vec![
        "git config --global alias.co checkout".to_string(),
        "git config --global alias.br branch".to_string(),
        "git config --global alias.ci commit".to_string(),
        "git config --global alias.st status".to_string(),
        "git config --global alias.lg \"log --oneline --graph --decorate\"".to_string(),
    ];

    store_pending("git-aliases", PendingGitChange {
        commands: commands.clone(),
        description: "Add common git aliases".to_string(),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I can add these handy git aliases:\n  co = checkout\n  br = branch\n  ci = commit\n  st = status\n  lg = log --oneline --graph\n\nThis makes git commands shorter to type."
        ),
        needs_confirmation: true,
        confirmation_prompt: Some("Add these aliases?".to_string()),
    }
}

fn offer_default_branch() -> RecipeResult {
    let cmd = "git config --global init.defaultBranch main".to_string();

    store_pending("git-default-branch", PendingGitChange {
        commands: vec![cmd.clone()],
        description: "Set default branch to main".to_string(),
    });

    RecipeResult {
        success: true,
        message: "I'll set the default branch name to 'main' for new repositories:\n  git config --global init.defaultBranch main".to_string(),
        needs_confirmation: true,
        confirmation_prompt: Some("Apply this change?".to_string()),
    }
}

fn store_pending(id: &str, change: PendingGitChange) {
    if let Ok(mut guard) = PENDING_GIT.write() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(id.to_string(), change);
    }
}

fn take_pending(id: &str) -> Option<PendingGitChange> {
    if let Ok(mut guard) = PENDING_GIT.write() {
        if let Some(map) = guard.as_mut() {
            return map.remove(id);
        }
    }
    None
}

/// Execute a confirmed git recipe
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

    let mut outputs = Vec::new();
    let mut success = true;

    for cmd in &pending.commands {
        match run_command(cmd) {
            Ok(output) => {
                outputs.push(format!("$ {}", cmd));
                if !output.is_empty() {
                    outputs.push(output);
                }
            }
            Err(e) => {
                success = false;
                outputs.push(format!("$ {} (failed: {})", cmd, e));
            }
        }
    }

    info!("Executed git recipe: {} (success={})", recipe_id, success);

    if success {
        RecipeResult {
            success: true,
            message: format!("Done! {}\n\n{}", pending.description, outputs.join("\n")),
            needs_confirmation: false,
            confirmation_prompt: None,
        }
    } else {
        RecipeResult {
            success: false,
            message: format!("Some commands failed:\n{}", outputs.join("\n")),
            needs_confirmation: false,
            confirmation_prompt: None,
        }
    }
}
