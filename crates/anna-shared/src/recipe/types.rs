//! Recipe type enums (v0.0.234).

use serde::{Deserialize, Serialize};

/// Kind of recipe action (v0.0.27, v0.0.100 added package/service)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Default)]
pub enum RecipeKind {
    /// Read-only query (default, no system changes)
    #[default]
    Query,
    /// Append a line to config file if not present
    ConfigEditLineAppend,
    /// Ensure a specific line exists in config file
    ConfigEnsureLine,
    /// Clarification template (v0.0.31) - learned pattern of what to ask
    ClarificationTemplate,
    /// v0.0.100: Install a package
    PackageInstall,
    /// v0.0.100: Manage a systemd service
    ServiceManage,
    /// v0.0.100: Shell config edit (.bashrc, .zshrc)
    ShellConfig,
    /// v0.0.100: Git config edit (.gitconfig)
    GitConfig,
    /// v0.0.104: SSH config edit (~/.ssh/config)
    SshConfig,
    /// v0.0.233: Systemd unit file recipes
    SystemdUnit,
    /// v0.0.234: Cron job recipes
    CronJob,
    /// Unknown/future kinds
    #[serde(other)]
    Unknown,
}

/// Action specification for config edit recipes (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
#[derive(Default)]
pub enum RecipeAction {
    /// Ensure a line exists in the config file
    EnsureLine { line: String },
    /// Append a line to the end of the config file
    AppendLine { line: String },
    /// No action (read-only query)
    #[default]
    None,
}
