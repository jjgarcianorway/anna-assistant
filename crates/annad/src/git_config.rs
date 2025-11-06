//! Git configuration intelligence
//!
//! Analyzes git installation and configuration, suggests improvements

use anna_common::{Advice, Priority, RiskLevel};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct GitConfig {
    #[allow(dead_code)]
    pub git_installed: bool,
    pub has_user_name: bool,
    pub has_user_email: bool,
    pub has_default_branch: bool,
    pub has_credential_helper: bool,
    pub has_diff_tool: bool,
    pub has_merge_tool: bool,
    pub has_useful_aliases: bool,
    pub has_push_default: bool,
    pub has_pull_rebase: bool,
    #[allow(dead_code)]
    pub config_path: Option<PathBuf>,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            git_installed: false,
            has_user_name: false,
            has_user_email: false,
            has_default_branch: false,
            has_credential_helper: false,
            has_diff_tool: false,
            has_merge_tool: false,
            has_useful_aliases: false,
            has_push_default: false,
            has_pull_rebase: false,
            config_path: None,
        }
    }
}

/// Detect and analyze git configuration
pub fn analyze_git() -> Option<GitConfig> {
    // Check if git is installed
    let git_installed = Command::new("which")
        .arg("git")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !git_installed {
        info!("Git not installed, skipping git config analysis");
        return None;
    }

    info!("Git detected, analyzing configuration");

    let home = std::env::var("HOME").ok()?;
    let config_path = PathBuf::from(format!("{}/.gitconfig", home));

    let mut config = GitConfig {
        git_installed: true,
        config_path: if config_path.exists() {
            Some(config_path.clone())
        } else {
            None
        },
        ..Default::default()
    };

    // Check git config values
    config.has_user_name = check_git_config("user.name");
    config.has_user_email = check_git_config("user.email");
    config.has_default_branch = check_git_config("init.defaultBranch");
    config.has_credential_helper = check_git_config("credential.helper");
    config.has_push_default = check_git_config("push.default");
    config.has_pull_rebase = check_git_config("pull.rebase");

    // Check for diff/merge tools
    config.has_diff_tool = check_git_config("diff.tool");
    config.has_merge_tool = check_git_config("merge.tool");

    // Check for useful aliases
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            config.has_useful_aliases = content.contains("[alias]")
                && (content.contains("st =") || content.contains("co =") || content.contains("br ="));
        }
    }

    Some(config)
}

/// Check if a git config key is set
fn check_git_config(key: &str) -> bool {
    Command::new("git")
        .args(&["config", "--global", "--get", key])
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false)
}

/// Generate git configuration recommendations
pub fn generate_git_recommendations(config: &GitConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    // Critical: User identity
    if !config.has_user_name || !config.has_user_email {
        recommendations.push(Advice::new(
            "git-user-identity".to_string(),
            "Configure Git user identity".to_string(),
            "Git requires your name and email for commits. This information appears in every commit you make and helps identify your contributions.".to_string(),
            "Set up git user.name and user.email".to_string(),
            Some(r#"git config --global user.name "Your Name"
git config --global user.email "your.email@example.com""#.to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Git#Configuration".to_string()],
            "development".to_string(),
        ));
    }

    // Default branch name
    if !config.has_default_branch {
        recommendations.push(Advice::new(
            "git-default-branch".to_string(),
            "Set default branch name to 'main'".to_string(),
            "Modern git projects use 'main' instead of 'master' as the default branch. This sets it globally for all new repositories.".to_string(),
            "Configure git init.defaultBranch".to_string(),
            Some("git config --global init.defaultBranch main".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Git#Configuration".to_string()],
            "development".to_string(),
        ));
    }

    // Credential caching
    if !config.has_credential_helper {
        recommendations.push(Advice::new(
            "git-credential-helper".to_string(),
            "Enable git credential caching".to_string(),
            "Credential helper caches your passwords/tokens so you don't have to enter them repeatedly. Cache stores credentials in memory for 15 minutes.".to_string(),
            "Set up git credential caching".to_string(),
            Some("git config --global credential.helper cache".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Git#Credential_management".to_string()],
            "development".to_string(),
        ));
    }

    // Push default behavior
    if !config.has_push_default {
        recommendations.push(Advice::new(
            "git-push-default".to_string(),
            "Set safe git push behavior".to_string(),
            "Setting 'push.default = simple' ensures you only push the current branch, preventing accidental pushes of all branches.".to_string(),
            "Configure git push.default".to_string(),
            Some("git config --global push.default simple".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Git#Configuration".to_string()],
            "development".to_string(),
        ));
    }

    // Pull rebase by default
    if !config.has_pull_rebase {
        recommendations.push(Advice::new(
            "git-pull-rebase".to_string(),
            "Use rebase when pulling changes".to_string(),
            "Setting 'pull.rebase = true' keeps your git history cleaner by rebasing your local commits on top of the remote branch instead of creating merge commits.".to_string(),
            "Configure git pull.rebase".to_string(),
            Some("git config --global pull.rebase true".to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Git#Tips_and_tricks".to_string()],
            "development".to_string(),
        ));
    }

    // Useful aliases
    if !config.has_useful_aliases {
        recommendations.push(Advice::new(
            "git-aliases".to_string(),
            "Add useful git aliases".to_string(),
            "Git aliases make common commands shorter and faster. Examples: 'git st' for status, 'git co' for checkout, 'git br' for branch, 'git lg' for a pretty log.".to_string(),
            "Configure git aliases".to_string(),
            Some(r#"git config --global alias.st status
git config --global alias.co checkout
git config --global alias.br branch
git config --global alias.ci commit
git config --global alias.unstage 'reset HEAD --'
git config --global alias.last 'log -1 HEAD'
git config --global alias.lg "log --color --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit""#.to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Git#Tips_and_tricks".to_string()],
            "development".to_string(),
        ));
    }

    // Diff tool
    if !config.has_diff_tool {
        recommendations.push(Advice::new(
            "git-diff-tool".to_string(),
            "Configure a visual diff tool".to_string(),
            "Visual diff tools make it easier to review changes. 'vimdiff' is lightweight and comes with vim, or use 'meld' for a GUI.".to_string(),
            "Set up git difftool".to_string(),
            Some(r#"# Option 1: vimdiff (lightweight, terminal-based)
git config --global diff.tool vimdiff
git config --global difftool.prompt false

# Option 2: meld (GUI, requires installation)
# sudo pacman -S --noconfirm meld
# git config --global diff.tool meld"#.to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Git#Visual_diff_and_merge_tools".to_string()],
            "development".to_string(),
        ));
    }

    // Merge tool
    if !config.has_merge_tool {
        recommendations.push(Advice::new(
            "git-merge-tool".to_string(),
            "Configure a visual merge tool".to_string(),
            "Visual merge tools make resolving conflicts easier. 'vimdiff' is lightweight, or use 'meld' for a GUI experience.".to_string(),
            "Set up git mergetool".to_string(),
            Some(r#"# Option 1: vimdiff (lightweight, terminal-based)
git config --global merge.tool vimdiff
git config --global mergetool.prompt false

# Option 2: meld (GUI, requires installation)
# sudo pacman -S --noconfirm meld
# git config --global merge.tool meld"#.to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Git#Visual_diff_and_merge_tools".to_string()],
            "development".to_string(),
        ));
    }

    // Additional useful configs
    recommendations.push(Advice::new(
        "git-extra-config".to_string(),
        "Enable useful git features".to_string(),
        "Additional helpful git configurations: colored output, automatic pruning of deleted remote branches, and better diff algorithm.".to_string(),
        "Apply recommended git settings".to_string(),
        Some(r#"# Enable colored output
git config --global color.ui auto

# Automatically prune deleted remote branches
git config --global fetch.prune true

# Use better diff algorithm
git config --global diff.algorithm histogram

# Show more context in diffs
git config --global diff.context 5"#.to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Git#Tips_and_tricks".to_string()],
        "development".to_string(),
    ));

    recommendations
}
