//! Intelligent Recommender - Context-aware suggestions based on user behavior
//!
//! Analyzes user patterns and suggests tools/improvements that match their workflow

use anna_common::{Advice, Priority, RiskLevel, SystemFacts};
use std::process::Command;

/// Generate intelligent recommendations based on detected behavior
pub fn generate_intelligent_advice(facts: &SystemFacts) -> Vec<Advice> {
    let mut advice = Vec::new();

    // Development tools recommendations
    advice.extend(recommend_dev_tools(facts));

    // CLI tool improvements (beautification)
    advice.extend(recommend_cli_improvements(facts));

    // Media player recommendations
    advice.extend(recommend_media_tools(facts));

    // Shell enhancements
    advice.extend(recommend_shell_enhancements(facts));

    advice
}

/// Recommend development tools based on detected languages and usage
fn recommend_dev_tools(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Python development
    if facts.common_file_types.iter().any(|t| t == "python") ||
       facts.dev_tools_detected.iter().any(|t| t == "python3") {

        // LSP server for editor integration
        if !package_installed("python-lsp-server") {
            result.push(Advice {
                id: "python-lsp".to_string(),
                title: "Install Python LSP server".to_string(),
                reason: "Python development detected - LSP provides autocomplete, linting, and go-to-definition".to_string(),
                action: "Install python-lsp-server for better editor integration".to_string(),
                command: Some("pacman -S --noconfirm python-lsp-server".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python".to_string()],
            });
        }

        // Black formatter
        if !package_installed("python-black") {
            result.push(Advice {
                id: "python-black".to_string(),
                title: "Install Black code formatter".to_string(),
                reason: "Python development detected - Black auto-formats code to PEP 8 standard".to_string(),
                action: "Install python-black for consistent code formatting".to_string(),
                command: Some("pacman -S --noconfirm python-black".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python".to_string()],
            });
        }

        // IPython for better REPL
        if !package_installed("ipython") {
            result.push(Advice {
                id: "ipython".to_string(),
                title: "Install IPython for enhanced Python REPL".to_string(),
                reason: "Python development detected - IPython provides better interactive shell".to_string(),
                action: "Install ipython for improved Python REPL experience".to_string(),
                command: Some("pacman -S --noconfirm ipython".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python".to_string()],
            });
        }
    }

    // Rust development
    if facts.common_file_types.iter().any(|t| t == "rust") ||
       facts.dev_tools_detected.iter().any(|t| t == "cargo") {

        if !package_installed("rust-analyzer") {
            result.push(Advice {
                id: "rust-analyzer".to_string(),
                title: "Install rust-analyzer LSP".to_string(),
                reason: "Rust development detected - rust-analyzer provides IDE-like features".to_string(),
                action: "Install rust-analyzer for autocomplete and inline documentation".to_string(),
                command: Some("pacman -S --noconfirm rust-analyzer".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rust".to_string()],
            });
        }

        // sccache for faster builds
        if !package_installed("sccache") {
            result.push(Advice {
                id: "sccache".to_string(),
                title: "Install sccache for faster Rust builds".to_string(),
                reason: "Rust development detected - sccache caches compiled crates".to_string(),
                action: "Install sccache to speed up compilation times".to_string(),
                command: Some("pacman -S --noconfirm sccache".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rust".to_string()],
            });
        }
    }

    // JavaScript/TypeScript development
    if facts.common_file_types.iter().any(|t| t == "javascript") ||
       facts.dev_tools_detected.iter().any(|t| t == "node" || t == "npm") {

        if !package_installed("typescript-language-server") {
            result.push(Advice {
                id: "typescript-lsp".to_string(),
                title: "Install TypeScript language server".to_string(),
                reason: "JavaScript/Node.js development detected - TSServer provides IDE features".to_string(),
                action: "Install typescript-language-server for better editor support".to_string(),
                command: Some("npm install -g typescript-language-server typescript".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Node.js".to_string()],
            });
        }
    }

    // Go development
    if facts.common_file_types.iter().any(|t| t == "go") ||
       facts.dev_tools_detected.iter().any(|t| t == "go") {

        if !package_installed("gopls") {
            result.push(Advice {
                id: "gopls".to_string(),
                title: "Install gopls language server".to_string(),
                reason: "Go development detected - gopls is the official Go language server".to_string(),
                action: "Install gopls for Go editor integration".to_string(),
                command: Some("go install golang.org/x/tools/gopls@latest".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Go".to_string()],
            });
        }
    }

    // Git user enhancements
    if facts.dev_tools_detected.iter().any(|t| t == "git") {

        // Delta for better diffs
        if !package_installed("git-delta") {
            result.push(Advice {
                id: "git-delta".to_string(),
                title: "Install delta for beautiful git diffs".to_string(),
                reason: "Git usage detected - delta provides syntax-highlighted diffs with line numbers".to_string(),
                action: "Install git-delta for improved git diff experience".to_string(),
                command: Some("pacman -S --noconfirm git-delta".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git".to_string()],
            });
        }

        // lazygit TUI
        if !package_installed("lazygit") {
            result.push(Advice {
                id: "lazygit".to_string(),
                title: "Install lazygit for terminal Git UI".to_string(),
                reason: "Git usage detected - lazygit provides intuitive terminal interface".to_string(),
                action: "Install lazygit for easier git operations".to_string(),
                command: Some("pacman -S --noconfirm lazygit".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git".to_string()],
            });
        }
    }

    // Docker user enhancements
    if facts.dev_tools_detected.iter().any(|t| t == "docker") {

        if !package_installed("docker-compose") {
            result.push(Advice {
                id: "docker-compose".to_string(),
                title: "Install docker-compose".to_string(),
                reason: "Docker usage detected - docker-compose manages multi-container applications".to_string(),
                action: "Install docker-compose for container orchestration".to_string(),
                command: Some("pacman -S --noconfirm docker-compose".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker".to_string()],
            });
        }

        if !package_installed("lazydocker") {
            result.push(Advice {
                id: "lazydocker".to_string(),
                title: "Install lazydocker for terminal Docker UI".to_string(),
                reason: "Docker usage detected - lazydocker provides TUI for managing containers".to_string(),
                action: "Install lazydocker for easier container management".to_string(),
                command: Some("pacman -S --noconfirm lazydocker".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker".to_string()],
            });
        }
    }

    // Vim/Neovim users
    if facts.dev_tools_detected.iter().any(|t| t == "vim") {

        if !facts.dev_tools_detected.iter().any(|t| t == "nvim") {
            result.push(Advice {
                id: "neovim".to_string(),
                title: "Upgrade to Neovim".to_string(),
                reason: "Vim usage detected - Neovim has better LSP support and modern features".to_string(),
                action: "Install neovim as a modern Vim alternative".to_string(),
                command: Some("pacman -S --noconfirm neovim".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Neovim".to_string()],
            });
        }
    }

    result
}

/// Recommend CLI tool improvements based on command usage
fn recommend_cli_improvements(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for frequently used commands
    let cmd_names: Vec<String> = facts.frequently_used_commands.iter()
        .map(|c| c.command.clone())
        .collect();

    // ls → eza (modern ls with colors and git integration)
    if cmd_names.iter().any(|c| c == "ls") && !command_exists("eza") {
        result.push(Advice {
            id: "eza".to_string(),
            title: "Install eza as modern 'ls' replacement".to_string(),
            reason: "You use 'ls' frequently - eza adds colors, icons, and git status".to_string(),
            action: "Install eza for a better file listing experience".to_string(),
            command: Some("pacman -S --noconfirm eza".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Cosmetic,
            category: "beautification".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
        });
    }

    // cat → bat (syntax highlighting)
    if cmd_names.iter().any(|c| c == "cat") && !command_exists("bat") {
        result.push(Advice {
            id: "bat".to_string(),
            title: "Install bat as modern 'cat' replacement".to_string(),
            reason: "You use 'cat' frequently - bat adds syntax highlighting and line numbers".to_string(),
            action: "Install bat for prettier file viewing".to_string(),
            command: Some("pacman -S --noconfirm bat".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Cosmetic,
            category: "beautification".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
        });
    }

    // grep → ripgrep (faster, smarter)
    if cmd_names.iter().any(|c| c == "grep") && !command_exists("rg") {
        result.push(Advice {
            id: "ripgrep".to_string(),
            title: "Install ripgrep as modern 'grep' replacement".to_string(),
            reason: "You use 'grep' frequently - ripgrep is faster and respects .gitignore".to_string(),
            action: "Install ripgrep for blazing-fast searches".to_string(),
            command: Some("pacman -S --noconfirm ripgrep".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "performance".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
        });
    }

    // find → fd (modern, intuitive)
    if cmd_names.iter().any(|c| c == "find") && !command_exists("fd") {
        result.push(Advice {
            id: "fd".to_string(),
            title: "Install fd as modern 'find' replacement".to_string(),
            reason: "You use 'find' frequently - fd has simpler syntax and is faster".to_string(),
            action: "Install fd for easier file searching".to_string(),
            command: Some("pacman -S --noconfirm fd".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "usability".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
        });
    }

    // du → dust (visual disk usage)
    if cmd_names.iter().any(|c| c == "du") && !command_exists("dust") {
        result.push(Advice {
            id: "dust".to_string(),
            title: "Install dust as visual 'du' replacement".to_string(),
            reason: "You use 'du' frequently - dust shows intuitive tree-based disk usage".to_string(),
            action: "Install dust for better disk usage visualization".to_string(),
            command: Some("pacman -S --noconfirm dust".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Cosmetic,
            category: "beautification".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
        });
    }

    // top/htop → btop (beautiful system monitor)
    if (cmd_names.iter().any(|c| c == "top" || c == "htop")) && !command_exists("btop") {
        result.push(Advice {
            id: "btop".to_string(),
            title: "Install btop as beautiful system monitor".to_string(),
            reason: "You monitor system resources - btop provides gorgeous real-time visualization".to_string(),
            action: "Install btop for eye-candy system monitoring".to_string(),
            command: Some("pacman -S --noconfirm btop".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Cosmetic,
            category: "beautification".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/System_monitor".to_string()],
        });
    }

    // General shell enhancements
    if !command_exists("fzf") {
        result.push(Advice {
            id: "fzf".to_string(),
            title: "Install fzf for fuzzy finding".to_string(),
            reason: "FZF provides fuzzy finding for command history, files, and more".to_string(),
            action: "Install fzf for supercharged shell navigation".to_string(),
            command: Some("pacman -S --noconfirm fzf".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "usability".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Command-line_shell#Utilities".to_string()],
        });
    }

    if !command_exists("zoxide") {
        result.push(Advice {
            id: "zoxide".to_string(),
            title: "Install zoxide for smart directory jumping".to_string(),
            reason: "Zoxide learns your habits and lets you jump to directories with 'z'".to_string(),
            action: "Install zoxide for faster directory navigation".to_string(),
            command: Some("pacman -S --noconfirm zoxide".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "usability".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Command-line_shell#Utilities".to_string()],
        });
    }

    result
}

/// Recommend media tools based on detected files
fn recommend_media_tools(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Video files without player
    if facts.media_usage.has_video_files && !facts.media_usage.video_player_installed {
        result.push(Advice {
            id: "mpv".to_string(),
            title: "Install mpv video player".to_string(),
            reason: "Video files detected but no video player installed".to_string(),
            action: "Install mpv for lightweight, powerful video playback".to_string(),
            command: Some("pacman -S --noconfirm mpv".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "media".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Mpv".to_string()],
        });
    }

    // Audio files without player
    if facts.media_usage.has_audio_files && !facts.media_usage.audio_player_installed {
        result.push(Advice {
            id: "cmus".to_string(),
            title: "Install cmus terminal music player".to_string(),
            reason: "Audio files detected but no music player installed".to_string(),
            action: "Install cmus for lightweight terminal-based music playback".to_string(),
            command: Some("pacman -S --noconfirm cmus".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "media".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Music_player".to_string()],
        });
    }

    // Images without viewer
    if facts.media_usage.has_images && !facts.media_usage.image_viewer_installed {
        result.push(Advice {
            id: "feh".to_string(),
            title: "Install feh image viewer".to_string(),
            reason: "Image files detected but no image viewer installed".to_string(),
            action: "Install feh for fast, lightweight image viewing".to_string(),
            command: Some("pacman -S --noconfirm feh".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "media".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Feh".to_string()],
        });
    }

    result
}

/// Recommend shell enhancements
fn recommend_shell_enhancements(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Starship prompt (works with any shell)
    if !command_exists("starship") {
        result.push(Advice {
            id: "starship".to_string(),
            title: "Install Starship prompt".to_string(),
            reason: "Starship provides a beautiful, fast, customizable shell prompt".to_string(),
            action: "Install starship for a modern shell prompt experience".to_string(),
            command: Some("pacman -S --noconfirm starship".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Cosmetic,
            category: "beautification".to_string(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Command-line_shell#Prompts".to_string()],
        });
    }

    // Zsh enhancements (if using zsh)
    if facts.shell == "zsh" {
        if !package_installed("zsh-autosuggestions") {
            result.push(Advice {
                id: "zsh-autosuggestions".to_string(),
                title: "Install zsh-autosuggestions".to_string(),
                reason: "You use zsh - autosuggestions shows command history suggestions".to_string(),
                action: "Install zsh-autosuggestions for fish-like suggestions".to_string(),
                command: Some("pacman -S --noconfirm zsh-autosuggestions".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "usability".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Autosuggestions".to_string()],
            });
        }

        if !package_installed("zsh-syntax-highlighting") {
            result.push(Advice {
                id: "zsh-syntax-highlighting".to_string(),
                title: "Install zsh-syntax-highlighting".to_string(),
                reason: "You use zsh - syntax highlighting shows valid/invalid commands".to_string(),
                action: "Install zsh-syntax-highlighting for colored command validation".to_string(),
                command: Some("pacman -S --noconfirm zsh-syntax-highlighting".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "usability".to_string(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Syntax_highlighting".to_string()],
            });
        }
    }

    result
}

fn package_installed(name: &str) -> bool {
    Command::new("pacman")
        .args(&["-Q", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
