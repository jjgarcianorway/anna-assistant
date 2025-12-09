//! Built-in git recipes catalog (v0.0.224).

use super::recipe::GitRecipe;
use super::types::{GitFeature, GitScope};

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
        .with_rollback(
            "git config --global --unset user.name && git config --global --unset user.email",
        ),
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
