//! SSH recipe types (v0.0.196).

use serde::{Deserialize, Serialize};

/// SSH key types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SshKeyType {
    Ed25519, // Modern, recommended
    Rsa4096, // Widely compatible
    Ecdsa,   // NIST curves
}

impl SshKeyType {
    pub fn display_name(&self) -> &'static str {
        match self {
            SshKeyType::Ed25519 => "ed25519",
            SshKeyType::Rsa4096 => "rsa (4096-bit)",
            SshKeyType::Ecdsa => "ecdsa",
        }
    }

    pub fn algorithm_name(&self) -> &'static str {
        match self {
            SshKeyType::Ed25519 => "ed25519",
            SshKeyType::Rsa4096 => "rsa",
            SshKeyType::Ecdsa => "ecdsa",
        }
    }

    /// Default key filename (without path)
    pub fn default_filename(&self) -> &'static str {
        match self {
            SshKeyType::Ed25519 => "id_ed25519",
            SshKeyType::Rsa4096 => "id_rsa",
            SshKeyType::Ecdsa => "id_ecdsa",
        }
    }

    /// Command to generate this key type
    pub fn keygen_command(&self, comment: &str) -> String {
        match self {
            SshKeyType::Ed25519 => format!("ssh-keygen -t ed25519 -C \"{}\"", comment),
            SshKeyType::Rsa4096 => format!("ssh-keygen -t rsa -b 4096 -C \"{}\"", comment),
            SshKeyType::Ecdsa => format!("ssh-keygen -t ecdsa -b 521 -C \"{}\"", comment),
        }
    }
}

impl std::fmt::Display for SshKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// SSH configuration features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SshFeature {
    /// Generate a new SSH key
    GenerateKey,
    /// Copy public key to server
    CopyKey,
    /// Add host alias to ~/.ssh/config
    HostAlias,
    /// Configure SSH agent
    SshAgent,
    /// Harden SSH config (client-side)
    HardenConfig,
    /// GitHub SSH setup
    GitHubSsh,
    /// GitLab SSH setup
    GitLabSsh,
}

impl SshFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            SshFeature::GenerateKey => "generate SSH key",
            SshFeature::CopyKey => "copy SSH key to server",
            SshFeature::HostAlias => "add SSH host alias",
            SshFeature::SshAgent => "configure SSH agent",
            SshFeature::HardenConfig => "harden SSH config",
            SshFeature::GitHubSsh => "setup GitHub SSH",
            SshFeature::GitLabSsh => "setup GitLab SSH",
        }
    }

    /// Keywords that indicate this feature
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            SshFeature::GenerateKey => &["generate", "create", "new", "keygen", "ssh-keygen"],
            SshFeature::CopyKey => &["copy", "ssh-copy-id", "authorized_keys", "upload"],
            SshFeature::HostAlias => &["alias", "config", "host", "shortcut"],
            SshFeature::SshAgent => &["agent", "ssh-agent", "ssh-add"],
            SshFeature::HardenConfig => &["harden", "secure", "security"],
            SshFeature::GitHubSsh => &["github", "gh"],
            SshFeature::GitLabSsh => &["gitlab", "gl"],
        }
    }
}

/// An SSH recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshRecipe {
    pub feature: SshFeature,
    pub description: String,
    pub steps: Vec<SshStep>,
    pub answer_template: String,
}

/// A step in an SSH recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshStep {
    pub description: String,
    pub command: Option<String>,
    pub config_lines: Option<Vec<String>>,
    pub note: Option<String>,
}

impl SshStep {
    pub fn command(desc: &str, cmd: &str) -> Self {
        Self {
            description: desc.to_string(),
            command: Some(cmd.to_string()),
            config_lines: None,
            note: None,
        }
    }

    pub fn config(desc: &str, lines: Vec<&str>) -> Self {
        Self {
            description: desc.to_string(),
            command: None,
            config_lines: Some(lines.into_iter().map(|s| s.to_string()).collect()),
            note: None,
        }
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.note = Some(note.to_string());
        self
    }
}
