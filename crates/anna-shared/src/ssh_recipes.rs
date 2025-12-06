//! SSH key and configuration recipes.
//!
//! v0.0.104: Recipes for SSH key generation, copying, and config management.
//!
//! These are common tasks that Anna can help with using deterministic recipes.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// SSH key types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SshKeyType {
    Ed25519,  // Modern, recommended
    Rsa4096,  // Widely compatible
    Ecdsa,    // NIST curves
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

/// Get the SSH directory path
pub fn ssh_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".ssh")
}

/// Get the SSH config file path
pub fn ssh_config_path() -> PathBuf {
    ssh_dir().join("config")
}

/// Get built-in SSH recipes
pub fn builtin_recipes() -> Vec<SshRecipe> {
    vec![
        // Generate ed25519 key (recommended)
        SshRecipe {
            feature: SshFeature::GenerateKey,
            description: "Generate a new ed25519 SSH key".to_string(),
            steps: vec![
                SshStep::command(
                    "Generate the key pair",
                    "ssh-keygen -t ed25519 -C \"your_email@example.com\"",
                ).with_note("You'll be prompted for a passphrase (recommended for security)"),
                SshStep::command(
                    "Start the SSH agent",
                    "eval \"$(ssh-agent -s)\"",
                ),
                SshStep::command(
                    "Add the key to the agent",
                    "ssh-add ~/.ssh/id_ed25519",
                ),
            ],
            answer_template: r#"To generate a new SSH key:

1. **Generate the key:**
   ```
   ssh-keygen -t ed25519 -C "your_email@example.com"
   ```
   Press Enter for default location, then enter a passphrase.

2. **Start the SSH agent:**
   ```
   eval "$(ssh-agent -s)"
   ```

3. **Add your key to the agent:**
   ```
   ssh-add ~/.ssh/id_ed25519
   ```

4. **Copy your public key:**
   ```
   cat ~/.ssh/id_ed25519.pub
   ```
   Add this to your server's `~/.ssh/authorized_keys` or GitHub/GitLab settings."#.to_string(),
        },

        // Copy key to server
        SshRecipe {
            feature: SshFeature::CopyKey,
            description: "Copy SSH public key to a remote server".to_string(),
            steps: vec![
                SshStep::command(
                    "Copy the key using ssh-copy-id",
                    "ssh-copy-id user@hostname",
                ).with_note("Replace user@hostname with your server details"),
            ],
            answer_template: r#"To copy your SSH key to a server:

**Method 1: Using ssh-copy-id (recommended)**
```
ssh-copy-id user@hostname
```

**Method 2: Manual copy**
```
cat ~/.ssh/id_ed25519.pub | ssh user@hostname "mkdir -p ~/.ssh && chmod 700 ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys"
```

After copying, you should be able to login without a password:
```
ssh user@hostname
```"#.to_string(),
        },

        // Add host alias
        SshRecipe {
            feature: SshFeature::HostAlias,
            description: "Add SSH host alias for easier connections".to_string(),
            steps: vec![
                SshStep::config(
                    "Add to ~/.ssh/config",
                    vec![
                        "Host myserver",
                        "    HostName example.com",
                        "    User myuser",
                        "    IdentityFile ~/.ssh/id_ed25519",
                    ],
                ),
            ],
            answer_template: r#"To create an SSH host alias, add to `~/.ssh/config`:

```
Host myserver
    HostName example.com
    User myuser
    IdentityFile ~/.ssh/id_ed25519
    Port 22
```

Then connect with just:
```
ssh myserver
```

Common options:
- `HostName` - actual server address
- `User` - login username
- `Port` - SSH port (default: 22)
- `IdentityFile` - path to private key
- `ForwardAgent yes` - forward SSH agent (use with caution)"#.to_string(),
        },

        // SSH agent configuration
        SshRecipe {
            feature: SshFeature::SshAgent,
            description: "Configure SSH agent to auto-start".to_string(),
            steps: vec![
                SshStep::config(
                    "Add to shell config (.bashrc/.zshrc)",
                    vec![
                        "# Start SSH agent",
                        "if [ -z \"$SSH_AUTH_SOCK\" ]; then",
                        "    eval \"$(ssh-agent -s)\" > /dev/null",
                        "fi",
                    ],
                ),
            ],
            answer_template: r#"To auto-start SSH agent, add to your `.bashrc` or `.zshrc`:

```bash
# Start SSH agent
if [ -z "$SSH_AUTH_SOCK" ]; then
    eval "$(ssh-agent -s)" > /dev/null
fi
```

To automatically add keys, also add:
```bash
ssh-add ~/.ssh/id_ed25519 2>/dev/null
```

For systemd-based systems, you can also use:
```bash
systemctl --user enable ssh-agent
systemctl --user start ssh-agent
```

And add to your shell config:
```bash
export SSH_AUTH_SOCK="$XDG_RUNTIME_DIR/ssh-agent.socket"
```"#.to_string(),
        },

        // GitHub SSH setup
        SshRecipe {
            feature: SshFeature::GitHubSsh,
            description: "Setup SSH authentication for GitHub".to_string(),
            steps: vec![
                SshStep::command(
                    "Generate key for GitHub",
                    "ssh-keygen -t ed25519 -C \"your_github_email@example.com\" -f ~/.ssh/id_github",
                ),
                SshStep::config(
                    "Add to ~/.ssh/config",
                    vec![
                        "Host github.com",
                        "    HostName github.com",
                        "    User git",
                        "    IdentityFile ~/.ssh/id_github",
                    ],
                ),
                SshStep::command(
                    "Copy the public key",
                    "cat ~/.ssh/id_github.pub",
                ).with_note("Add this to GitHub: Settings > SSH and GPG keys > New SSH key"),
            ],
            answer_template: r#"To setup SSH for GitHub:

1. **Generate a key:**
   ```
   ssh-keygen -t ed25519 -C "your_github_email@example.com"
   ```

2. **Add to ~/.ssh/config:**
   ```
   Host github.com
       HostName github.com
       User git
       IdentityFile ~/.ssh/id_ed25519
   ```

3. **Copy your public key:**
   ```
   cat ~/.ssh/id_ed25519.pub
   ```

4. **Add to GitHub:**
   - Go to github.com → Settings → SSH and GPG keys → New SSH key
   - Paste your public key

5. **Test the connection:**
   ```
   ssh -T git@github.com
   ```
   You should see: "Hi username! You've successfully authenticated...""#.to_string(),
        },

        // Harden SSH client config
        SshRecipe {
            feature: SshFeature::HardenConfig,
            description: "Harden SSH client configuration".to_string(),
            steps: vec![
                SshStep::config(
                    "Add to ~/.ssh/config",
                    vec![
                        "Host *",
                        "    # Use strong ciphers",
                        "    Ciphers aes256-gcm@openssh.com,chacha20-poly1305@openssh.com",
                        "    # Prefer ed25519 keys",
                        "    IdentitiesOnly yes",
                        "    # Hash known hosts",
                        "    HashKnownHosts yes",
                        "    # Strict host key checking",
                        "    StrictHostKeyChecking ask",
                    ],
                ),
            ],
            answer_template: r#"To harden your SSH client, add to `~/.ssh/config`:

```
Host *
    # Use strong key exchange and ciphers
    KexAlgorithms curve25519-sha256@libssh.org,diffie-hellman-group-exchange-sha256
    Ciphers chacha20-poly1305@openssh.com,aes256-gcm@openssh.com

    # Security options
    IdentitiesOnly yes          # Only use specified keys
    HashKnownHosts yes          # Hash hostnames in known_hosts
    StrictHostKeyChecking ask   # Confirm new host keys

    # Connection options
    ServerAliveInterval 60      # Keep connections alive
    ServerAliveCountMax 3       # Max keepalive attempts
```

Also ensure proper permissions:
```
chmod 700 ~/.ssh
chmod 600 ~/.ssh/config
chmod 600 ~/.ssh/id_*
chmod 644 ~/.ssh/id_*.pub
```"#.to_string(),
        },
    ]
}

/// Match a query to an SSH recipe
pub fn match_query(query: &str) -> Option<&'static SshRecipe> {
    let query_lower = query.to_lowercase();

    // Must mention SSH
    if !query_lower.contains("ssh") && !query_lower.contains("key") {
        return None;
    }

    // Lazy static for builtin recipes
    static RECIPES: std::sync::OnceLock<Vec<SshRecipe>> = std::sync::OnceLock::new();
    let recipes = RECIPES.get_or_init(builtin_recipes);

    // Score each feature by keyword matches
    let mut best_match: Option<(usize, &SshRecipe)> = None;

    for recipe in recipes {
        let score = recipe.feature.keywords().iter()
            .filter(|kw| query_lower.contains(*kw))
            .count();

        if score > 0 {
            match best_match {
                None => best_match = Some((score, recipe)),
                Some((best_score, _)) if score > best_score => best_match = Some((score, recipe)),
                _ => {}
            }
        }
    }

    best_match.map(|(_, recipe)| recipe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keygen_command() {
        let cmd = SshKeyType::Ed25519.keygen_command("test@example.com");
        assert!(cmd.contains("ed25519"));
        assert!(cmd.contains("test@example.com"));
    }

    #[test]
    fn test_match_generate_key() {
        let recipe = match_query("how do I generate an ssh key");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SshFeature::GenerateKey);
    }

    #[test]
    fn test_match_copy_key() {
        let recipe = match_query("ssh copy key to server");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SshFeature::CopyKey);
    }

    #[test]
    fn test_match_github() {
        let recipe = match_query("setup ssh for github");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, SshFeature::GitHubSsh);
    }

    #[test]
    fn test_no_match_unrelated() {
        let recipe = match_query("what is the weather");
        assert!(recipe.is_none());
    }
}
