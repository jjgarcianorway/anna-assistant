//! Git recipe types (v0.0.224).

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
