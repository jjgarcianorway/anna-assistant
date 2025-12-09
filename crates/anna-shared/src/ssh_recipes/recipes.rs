//! Built-in SSH recipes (v0.0.196).

use super::types::{SshFeature, SshRecipe, SshStep};

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
