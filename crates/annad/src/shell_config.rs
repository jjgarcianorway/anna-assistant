//! Shell Configuration Intelligence
//!
//! Detects, analyzes, and improves shell configurations for bash, zsh, and fish
//! Everyone uses a shell - this helps 100% of users!

use anna_common::{Advice, Priority, RiskLevel};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

/// Shell type detected
#[derive(Debug, Clone, PartialEq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Unknown,
}

/// Shell configuration analysis
#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub shell_type: ShellType,
    #[allow(dead_code)]
    pub config_path: Option<PathBuf>,
    pub has_starship: bool,
    pub has_modern_ls: bool,  // eza or lsd
    pub has_modern_cat: bool, // bat
    pub has_modern_find: bool, // fd
    pub has_fzf: bool,
    pub has_zoxide: bool, // z/autojump alternative
    pub has_ripgrep: bool,
    pub has_useful_aliases: bool,
    pub has_git_aliases: bool,
    pub has_syntax_highlighting: bool, // for zsh/fish
    pub has_autosuggestions: bool,      // for zsh/fish
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            shell_type: ShellType::Unknown,
            config_path: None,
            has_starship: false,
            has_modern_ls: false,
            has_modern_cat: false,
            has_modern_find: false,
            has_fzf: false,
            has_zoxide: false,
            has_ripgrep: false,
            has_useful_aliases: false,
            has_git_aliases: false,
            has_syntax_highlighting: false,
            has_autosuggestions: false,
        }
    }
}

/// Detect and analyze shell configuration
pub fn analyze_shell() -> Option<ShellConfig> {
    info!("Analyzing shell configuration");

    let shell_type = detect_shell_type();
    if shell_type == ShellType::Unknown {
        debug!("Could not detect shell type");
        return None;
    }

    let config_path = find_shell_config(&shell_type);

    let mut config = ShellConfig {
        shell_type: shell_type.clone(),
        config_path: config_path.clone(),
        ..Default::default()
    };

    // Detect installed tools
    detect_shell_tools(&mut config);

    // Analyze config file if it exists
    if let Some(ref path) = config_path {
        if let Ok(content) = fs::read_to_string(path) {
            analyze_shell_config_content(&content, &mut config);
        }
    }

    Some(config)
}

/// Detect which shell the user is using
fn detect_shell_type() -> ShellType {
    // Try SHELL environment variable first
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("zsh") {
            return ShellType::Zsh;
        } else if shell.contains("bash") {
            return ShellType::Bash;
        } else if shell.contains("fish") {
            return ShellType::Fish;
        }
    }

    // Fallback: check which shells exist
    if is_command_available("zsh") {
        return ShellType::Zsh;
    } else if is_command_available("bash") {
        return ShellType::Bash;
    } else if is_command_available("fish") {
        return ShellType::Fish;
    }

    ShellType::Unknown
}

/// Find shell configuration file
fn find_shell_config(shell_type: &ShellType) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    let config_file = match shell_type {
        ShellType::Bash => ".bashrc",
        ShellType::Zsh => ".zshrc",
        ShellType::Fish => ".config/fish/config.fish",
        ShellType::Unknown => return None,
    };

    let path = PathBuf::from(&home).join(config_file);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Detect installed shell enhancement tools
fn detect_shell_tools(config: &mut ShellConfig) {
    config.has_starship = is_package_installed("starship");
    config.has_modern_ls = is_package_installed("eza") || is_package_installed("lsd");
    config.has_modern_cat = is_package_installed("bat");
    config.has_modern_find = is_package_installed("fd");
    config.has_fzf = is_package_installed("fzf");
    config.has_zoxide = is_package_installed("zoxide");
    config.has_ripgrep = is_package_installed("ripgrep");

    // Shell-specific plugins
    match config.shell_type {
        ShellType::Zsh => {
            config.has_syntax_highlighting = is_package_installed("zsh-syntax-highlighting");
            config.has_autosuggestions = is_package_installed("zsh-autosuggestions");
        }
        ShellType::Fish => {
            // Fish has these built-in
            config.has_syntax_highlighting = true;
            config.has_autosuggestions = true;
        }
        _ => {}
    }
}

/// Analyze shell config file content
fn analyze_shell_config_content(content: &str, config: &mut ShellConfig) {
    let content_lower = content.to_lowercase();

    // Check for useful aliases
    config.has_useful_aliases = content_lower.contains("alias ls=")
        || content_lower.contains("alias ll=")
        || content_lower.contains("alias la=");

    // Check for git aliases
    config.has_git_aliases = content_lower.contains("alias g=")
        || content_lower.contains("alias gs=")
        || content_lower.contains("alias gst=");

    // Check for starship initialization
    if content.contains("starship init") {
        config.has_starship = true;
    }

    // Check for zoxide initialization
    if content.contains("zoxide init") {
        config.has_zoxide = true;
    }
}

/// Check if a command is available
fn is_command_available(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if a package is installed
fn is_package_installed(package: &str) -> bool {
    std::process::Command::new("pacman")
        .args(&["-Q", package])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Generate shell improvement recommendations
pub fn generate_shell_recommendations(config: &ShellConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    info!("Generating shell improvement recommendations for {:?}", config.shell_type);

    // Starship prompt (universal, works for all shells)
    if !config.has_starship {
        let init_command = match config.shell_type {
            ShellType::Bash => {
                r#"sudo pacman -S --noconfirm starship && \
echo 'eval "$(starship init bash)"' >> ~/.bashrc"#
            }
            ShellType::Zsh => {
                r#"sudo pacman -S --noconfirm starship && \
echo 'eval "$(starship init zsh)"' >> ~/.zshrc"#
            }
            ShellType::Fish => {
                r#"sudo pacman -S --noconfirm starship && \
echo 'starship init fish | source' >> ~/.config/fish/config.fish"#
            }
            ShellType::Unknown => return recommendations,
        };

        recommendations.push(
            Advice::new(
                "shell-starship-prompt".to_string(),
                "Install Starship prompt for your shell".to_string(),
                format!(
                    "Starship is a beautiful, fast, and customizable prompt that works across all shells. It shows git status, languages, and more. Perfect for {:?}.",
                    config.shell_type
                ),
                "Install and configure Starship prompt".to_string(),
                Some(init_command.to_string()),
                RiskLevel::Low,
                Priority::Recommended,
                vec!["https://wiki.archlinux.org/title/Starship".to_string()],
                "Utilities".to_string(),
            ),
        );
    }

    // Modern ls alternative (eza)
    if !config.has_modern_ls {
        let alias_command = match config.shell_type {
            ShellType::Bash => {
                r#"sudo pacman -S --noconfirm eza && \
echo "alias ls='eza --icons'" >> ~/.bashrc && \
echo "alias ll='eza -la --icons'" >> ~/.bashrc && \
echo "alias lt='eza --tree --icons'" >> ~/.bashrc"#
            }
            ShellType::Zsh => {
                r#"sudo pacman -S --noconfirm eza && \
echo "alias ls='eza --icons'" >> ~/.zshrc && \
echo "alias ll='eza -la --icons'" >> ~/.zshrc && \
echo "alias lt='eza --tree --icons'" >> ~/.zshrc"#
            }
            ShellType::Fish => {
                r#"sudo pacman -S --noconfirm eza && \
echo "alias ls='eza --icons'" >> ~/.config/fish/config.fish && \
echo "alias ll='eza -la --icons'" >> ~/.config/fish/config.fish && \
echo "alias lt='eza --tree --icons'" >> ~/.config/fish/config.fish"#
            }
            ShellType::Unknown => return recommendations,
        };

        recommendations.push(
            Advice::new(
                "shell-modern-ls".to_string(),
                "Install eza (modern ls replacement)".to_string(),
                "eza is a modern replacement for ls with colors, icons, git integration, and tree view. Makes file listing beautiful and informative.".to_string(),
                "Install eza and add aliases".to_string(),
                Some(alias_command.to_string()),
                RiskLevel::Low,
                Priority::Recommended,
                vec!["https://github.com/eza-community/eza".to_string()],
                "Utilities".to_string(),
            ),
        );
    }

    // bat (modern cat)
    if !config.has_modern_cat {
        let alias_command = match config.shell_type {
            ShellType::Bash => r#"sudo pacman -S --noconfirm bat && echo "alias cat='bat'" >> ~/.bashrc"#,
            ShellType::Zsh => r#"sudo pacman -S --noconfirm bat && echo "alias cat='bat'" >> ~/.zshrc"#,
            ShellType::Fish => r#"sudo pacman -S --noconfirm bat && echo "alias cat='bat'" >> ~/.config/fish/config.fish"#,
            ShellType::Unknown => return recommendations,
        };

        recommendations.push(
            Advice::new(
                "shell-modern-cat".to_string(),
                "Install bat (cat with syntax highlighting)".to_string(),
                "bat is cat with wings - syntax highlighting, git integration, line numbers, and more. Makes reading files in the terminal much better.".to_string(),
                "Install bat and alias cat".to_string(),
                Some(alias_command.to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec!["https://wiki.archlinux.org/title/Bat".to_string()],
                "Utilities".to_string(),
            ),
        );
    }

    // fzf (fuzzy finder)
    if !config.has_fzf {
        recommendations.push(
            Advice::new(
                "shell-fzf".to_string(),
                "Install fzf (fuzzy finder)".to_string(),
                "fzf is a command-line fuzzy finder. Press Ctrl+R for fuzzy history search, Ctrl+T for fuzzy file search. Essential productivity tool.".to_string(),
                "Install fzf".to_string(),
                Some("sudo pacman -S --noconfirm fzf".to_string()),
                RiskLevel::Low,
                Priority::Recommended,
                vec!["https://wiki.archlinux.org/title/Fzf".to_string()],
                "Utilities".to_string(),
            ),
        );
    }

    // zoxide (smart cd)
    if !config.has_zoxide {
        let init_command = match config.shell_type {
            ShellType::Bash => {
                r#"sudo pacman -S --noconfirm zoxide && \
echo 'eval "$(zoxide init bash)"' >> ~/.bashrc"#
            }
            ShellType::Zsh => {
                r#"sudo pacman -S --noconfirm zoxide && \
echo 'eval "$(zoxide init zsh)"' >> ~/.zshrc"#
            }
            ShellType::Fish => {
                r#"sudo pacman -S --noconfirm zoxide && \
echo 'zoxide init fish | source' >> ~/.config/fish/config.fish"#
            }
            ShellType::Unknown => return recommendations,
        };

        recommendations.push(
            Advice::new(
                "shell-zoxide".to_string(),
                "Install zoxide (smarter cd)".to_string(),
                "zoxide remembers your most used directories. Just type 'z proj' to jump to ~/projects. Way faster than cd.".to_string(),
                "Install and configure zoxide".to_string(),
                Some(init_command.to_string()),
                RiskLevel::Low,
                Priority::Recommended,
                vec!["https://wiki.archlinux.org/title/Zoxide".to_string()],
                "Utilities".to_string(),
            ),
        );
    }

    // ripgrep (better grep)
    if !config.has_ripgrep {
        recommendations.push(
            Advice::new(
                "shell-ripgrep".to_string(),
                "Install ripgrep (faster grep)".to_string(),
                "ripgrep (rg) is ridiculously fast text search. Respects .gitignore, has great defaults, and is 10-100x faster than grep.".to_string(),
                "Install ripgrep".to_string(),
                Some("sudo pacman -S --noconfirm ripgrep".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec!["https://wiki.archlinux.org/title/Ripgrep".to_string()],
                "Utilities".to_string(),
            ),
        );
    }

    // fd (better find)
    if !config.has_modern_find {
        recommendations.push(
            Advice::new(
                "shell-fd".to_string(),
                "Install fd (user-friendly find)".to_string(),
                "fd is a simple, fast, and user-friendly alternative to find. Much easier to use with better defaults.".to_string(),
                "Install fd".to_string(),
                Some("sudo pacman -S --noconfirm fd".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec!["https://wiki.archlinux.org/title/Fd".to_string()],
                "Utilities".to_string(),
            ),
        );
    }

    // Zsh-specific plugins
    if config.shell_type == ShellType::Zsh {
        if !config.has_syntax_highlighting {
            recommendations.push(
                Advice::new(
                    "shell-zsh-syntax-highlighting".to_string(),
                    "Install zsh syntax highlighting".to_string(),
                    "Syntax highlighting for zsh - see command validity as you type. Green = valid command, red = invalid.".to_string(),
                    "Install zsh-syntax-highlighting".to_string(),
                    Some(r#"sudo pacman -S --noconfirm zsh-syntax-highlighting && \
echo "source /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh" >> ~/.zshrc"#.to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://wiki.archlinux.org/title/Zsh".to_string()],
                    "Utilities".to_string(),
                ),
            );
        }

        if !config.has_autosuggestions {
            recommendations.push(
                Advice::new(
                    "shell-zsh-autosuggestions".to_string(),
                    "Install zsh autosuggestions".to_string(),
                    "Fish-like autosuggestions for zsh. Suggests commands based on history as you type. Press → to accept.".to_string(),
                    "Install zsh-autosuggestions".to_string(),
                    Some(r#"sudo pacman -S --noconfirm zsh-autosuggestions && \
echo "source /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh" >> ~/.zshrc"#.to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://wiki.archlinux.org/title/Zsh".to_string()],
                    "Utilities".to_string(),
                ),
            );
        }
    }

    // Useful git aliases if none exist
    if !config.has_git_aliases && is_package_installed("git") {
        let alias_command = match config.shell_type {
            ShellType::Bash | ShellType::Zsh => {
                let config_file = if config.shell_type == ShellType::Bash { "~/.bashrc" } else { "~/.zshrc" };
                format!(
                    r#"cat >> {} << 'EOF'

# Git aliases
alias g='git'
alias gs='git status'
alias ga='git add'
alias gc='git commit'
alias gp='git push'
alias gl='git pull'
alias gd='git diff'
alias gco='git checkout'
alias gb='git branch'
EOF"#,
                    config_file
                )
            }
            ShellType::Fish => {
                r#"cat >> ~/.config/fish/config.fish << 'EOF'

# Git aliases
alias g='git'
alias gs='git status'
alias ga='git add'
alias gc='git commit'
alias gp='git push'
alias gl='git pull'
alias gd='git diff'
alias gco='git checkout'
alias gb='git branch'
EOF"#.to_string()
            }
            ShellType::Unknown => return recommendations,
        };

        recommendations.push(
            Advice::new(
                "shell-git-aliases".to_string(),
                "Add useful Git aliases to your shell".to_string(),
                "Common git aliases like 'gs' for 'git status', 'ga' for 'git add', etc. Saves tons of typing for git users.".to_string(),
                "Add git aliases to shell config".to_string(),
                Some(alias_command),
                RiskLevel::Low,
                Priority::Optional,
                vec!["https://wiki.archlinux.org/title/Git".to_string()],
                "development".to_string(),
            ),
        );
    }

    recommendations
}
