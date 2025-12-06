//! Git configuration recipes for .gitconfig.
//!
//! v0.0.100: Learned from specialists, reusable by translator.
//!
//! These recipes configure git via `git config` commands or direct
//! .gitconfig edits.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Git configuration scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitScope {
    /// User-level (~/.gitconfig)
    Global,
    /// Repository-level (.git/config)
    Local,
    /// System-level (/etc/gitconfig)
    System,
}

impl GitScope {
    pub fn flag(&self) -> &'static str {
        match self {
            GitScope::Global => "--global",
            GitScope::Local => "--local",
            GitScope::System => "--system",
        }
    }

    pub fn config_path(&self) -> Option<PathBuf> {
        match self {
            GitScope::Global => {
                let home = std::env::var("HOME").ok()?;
                Some(PathBuf::from(home).join(".gitconfig"))
            }
            GitScope::Local => None, // Depends on repo location
            GitScope::System => Some(PathBuf::from("/etc/gitconfig")),
        }
    }
}

/// Git configuration features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitFeature {
    /// User identity (name, email)
    UserIdentity,
    /// Default branch name
    DefaultBranch,
    /// Editor for commits
    Editor,
    /// Merge tool
    MergeTool,
    /// Diff tool
    DiffTool,
    /// Colored output
    Colors,
    /// Aliases
    Aliases,
    /// Push defaults
    PushDefaults,
    /// Pull defaults
    PullDefaults,
    /// Credential helper
    CredentialHelper,
    /// GPG signing
    GpgSigning,
}

impl GitFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            GitFeature::UserIdentity => "user identity",
            GitFeature::DefaultBranch => "default branch",
            GitFeature::Editor => "commit editor",
            GitFeature::MergeTool => "merge tool",
            GitFeature::DiffTool => "diff tool",
            GitFeature::Colors => "colored output",
            GitFeature::Aliases => "aliases",
            GitFeature::PushDefaults => "push defaults",
            GitFeature::PullDefaults => "pull defaults",
            GitFeature::CredentialHelper => "credential helper",
            GitFeature::GpgSigning => "GPG signing",
        }
    }

    /// Keywords that indicate this feature
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            GitFeature::UserIdentity => &["name", "email", "user", "identity"],
            GitFeature::DefaultBranch => &["default", "branch", "main", "master", "init"],
            GitFeature::Editor => &["editor", "vim", "nano", "commit"],
            GitFeature::MergeTool => &["merge", "tool", "conflict"],
            GitFeature::DiffTool => &["diff", "tool"],
            GitFeature::Colors => &["color", "highlight"],
            GitFeature::Aliases => &["alias"],
            GitFeature::PushDefaults => &["push", "upstream"],
            GitFeature::PullDefaults => &["pull", "rebase", "merge"],
            GitFeature::CredentialHelper => &["credential", "password", "cache", "store"],
            GitFeature::GpgSigning => &["gpg", "sign", "key"],
        }
    }
}

/// A git configuration recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRecipe {
    pub feature: GitFeature,
    pub scope: GitScope,
    pub description: String,
    /// Commands to run (git config ...)
    pub commands: Vec<String>,
    /// Parameters that need user input
    pub parameters: Vec<GitParameter>,
    pub rollback_hint: Option<String>,
}

/// A parameter that needs user input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitParameter {
    pub name: String,
    pub prompt: String,
    pub default: Option<String>,
}

impl GitRecipe {
    pub fn new(feature: GitFeature, scope: GitScope, desc: &str, commands: Vec<&str>) -> Self {
        Self {
            feature,
            scope,
            description: desc.to_string(),
            commands: commands.into_iter().map(|s| s.to_string()).collect(),
            parameters: vec![],
            rollback_hint: None,
        }
    }

    pub fn with_param(mut self, name: &str, prompt: &str, default: Option<&str>) -> Self {
        self.parameters.push(GitParameter {
            name: name.to_string(),
            prompt: prompt.to_string(),
            default: default.map(|s| s.to_string()),
        });
        self
    }

    pub fn with_rollback(mut self, hint: &str) -> Self {
        self.rollback_hint = Some(hint.to_string());
        self
    }

    /// Check if recipe needs parameters
    pub fn needs_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }

    /// Apply parameters to commands
    pub fn apply_params(&self, values: &[(String, String)]) -> Vec<String> {
        self.commands.iter().map(|cmd| {
            let mut result = cmd.clone();
            for (name, value) in values {
                result = result.replace(&format!("{{{}}}", name), value);
            }
            result
        }).collect()
    }
}

/// Get built-in git recipes
pub fn builtin_recipes() -> Vec<GitRecipe> {
    vec![
        // User identity (needs parameters)
        GitRecipe::new(
            GitFeature::UserIdentity,
            GitScope::Global,
            "Set git user name and email",
            vec![
                "git config --global user.name \"{name}\"",
                "git config --global user.email \"{email}\"",
            ],
        )
        .with_param("name", "Your full name", None)
        .with_param("email", "Your email address", None)
        .with_rollback("git config --global --unset user.name && git config --global --unset user.email"),

        // Default branch
        GitRecipe::new(
            GitFeature::DefaultBranch,
            GitScope::Global,
            "Set default branch to main",
            vec!["git config --global init.defaultBranch main"],
        )
        .with_rollback("git config --global init.defaultBranch master"),

        // Editor - vim
        GitRecipe::new(
            GitFeature::Editor,
            GitScope::Global,
            "Set vim as git editor",
            vec!["git config --global core.editor vim"],
        )
        .with_rollback("git config --global --unset core.editor"),

        // Editor - nano
        GitRecipe::new(
            GitFeature::Editor,
            GitScope::Global,
            "Set nano as git editor",
            vec!["git config --global core.editor nano"],
        )
        .with_rollback("git config --global --unset core.editor"),

        // Colors
        GitRecipe::new(
            GitFeature::Colors,
            GitScope::Global,
            "Enable colored git output",
            vec![
                "git config --global color.ui auto",
                "git config --global color.branch auto",
                "git config --global color.diff auto",
                "git config --global color.status auto",
            ],
        )
        .with_rollback("git config --global color.ui false"),

        // Common aliases
        GitRecipe::new(
            GitFeature::Aliases,
            GitScope::Global,
            "Add common git aliases",
            vec![
                "git config --global alias.st status",
                "git config --global alias.co checkout",
                "git config --global alias.br branch",
                "git config --global alias.ci commit",
                "git config --global alias.lg \"log --oneline --graph --decorate\"",
                "git config --global alias.last \"log -1 HEAD\"",
                "git config --global alias.unstage \"reset HEAD --\"",
            ],
        )
        .with_rollback("git config --global --remove-section alias"),

        // Push defaults
        GitRecipe::new(
            GitFeature::PushDefaults,
            GitScope::Global,
            "Set push to current branch by default",
            vec![
                "git config --global push.default current",
                "git config --global push.autoSetupRemote true",
            ],
        )
        .with_rollback("git config --global push.default simple"),

        // Pull defaults - rebase
        GitRecipe::new(
            GitFeature::PullDefaults,
            GitScope::Global,
            "Set pull to rebase by default",
            vec!["git config --global pull.rebase true"],
        )
        .with_rollback("git config --global pull.rebase false"),

        // Credential helper - cache
        GitRecipe::new(
            GitFeature::CredentialHelper,
            GitScope::Global,
            "Cache git credentials for 1 hour",
            vec!["git config --global credential.helper 'cache --timeout=3600'"],
        )
        .with_rollback("git config --global --unset credential.helper"),

        // Credential helper - store (less secure)
        GitRecipe::new(
            GitFeature::CredentialHelper,
            GitScope::Global,
            "Store git credentials in plaintext (less secure)",
            vec!["git config --global credential.helper store"],
        )
        .with_rollback("git config --global --unset credential.helper"),

        // Merge tool - vimdiff
        GitRecipe::new(
            GitFeature::MergeTool,
            GitScope::Global,
            "Set vimdiff as merge tool",
            vec![
                "git config --global merge.tool vimdiff",
                "git config --global mergetool.vimdiff.prompt false",
            ],
        )
        .with_rollback("git config --global --unset merge.tool"),

        // Diff tool - vimdiff
        GitRecipe::new(
            GitFeature::DiffTool,
            GitScope::Global,
            "Set vimdiff as diff tool",
            vec![
                "git config --global diff.tool vimdiff",
                "git config --global difftool.vimdiff.prompt false",
            ],
        )
        .with_rollback("git config --global --unset diff.tool"),
    ]
}

/// Find a recipe for a feature
pub fn find_recipe(feature: GitFeature) -> Vec<GitRecipe> {
    builtin_recipes()
        .into_iter()
        .filter(|r| r.feature == feature)
        .collect()
}

/// Find recipes matching keywords
pub fn find_recipes_by_keywords(keywords: &[&str]) -> Vec<GitRecipe> {
    builtin_recipes()
        .into_iter()
        .filter(|r| {
            let feature_keywords = r.feature.keywords();
            keywords.iter().any(|kw| {
                let kw_lower = kw.to_lowercase();
                feature_keywords.iter().any(|fk| fk.contains(&kw_lower) || kw_lower.contains(fk))
            })
        })
        .collect()
}

/// Detect feature from query
pub fn detect_feature(query: &str) -> Option<GitFeature> {
    let q = query.to_lowercase();

    if q.contains("name") || q.contains("email") || q.contains("identity") {
        return Some(GitFeature::UserIdentity);
    }
    if q.contains("default") && q.contains("branch") {
        return Some(GitFeature::DefaultBranch);
    }
    if q.contains("editor") {
        return Some(GitFeature::Editor);
    }
    if q.contains("merge") && q.contains("tool") {
        return Some(GitFeature::MergeTool);
    }
    if q.contains("diff") && q.contains("tool") {
        return Some(GitFeature::DiffTool);
    }
    if q.contains("color") {
        return Some(GitFeature::Colors);
    }
    if q.contains("alias") {
        return Some(GitFeature::Aliases);
    }
    if q.contains("push") {
        return Some(GitFeature::PushDefaults);
    }
    if q.contains("pull") || q.contains("rebase") {
        return Some(GitFeature::PullDefaults);
    }
    if q.contains("credential") || q.contains("password") || q.contains("cache") {
        return Some(GitFeature::CredentialHelper);
    }
    if q.contains("gpg") || q.contains("sign") {
        return Some(GitFeature::GpgSigning);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_and_detect() {
        let recipes = find_recipe(GitFeature::UserIdentity);
        assert!(!recipes.is_empty());
        assert!(recipes[0].needs_parameters());
        assert_eq!(detect_feature("set git name"), Some(GitFeature::UserIdentity));
        assert_eq!(detect_feature("change default branch"), Some(GitFeature::DefaultBranch));
        assert!(builtin_recipes().len() >= 10);
    }

    #[test]
    fn test_apply_params() {
        let recipe = find_recipe(GitFeature::UserIdentity).into_iter().next().unwrap();
        let commands = recipe.apply_params(&[
            ("name".to_string(), "John".to_string()),
            ("email".to_string(), "j@x.com".to_string()),
        ]);
        assert!(commands[0].contains("John"));
    }
}
