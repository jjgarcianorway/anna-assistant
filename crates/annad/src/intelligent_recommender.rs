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

    // Missing configs for installed packages
    advice.extend(check_missing_configs());

    // Hardware support (gamepad, bluetooth, wifi, etc.)
    advice.extend(recommend_hardware_support(facts));

    // Desktop environment and display server enhancements
    advice.extend(recommend_desktop_enhancements(facts));

    // Font and rendering improvements
    advice.extend(recommend_fonts(facts));

    // Multimedia tools (video players, downloaders, etc.)
    advice.extend(recommend_multimedia_tools(facts));

    advice
}

/// Recommend development tools based on detected languages and usage
fn recommend_dev_tools(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Python development - ONLY if actual .py files found AND recent python usage in commands
    let has_python_files = facts.common_file_types.iter().any(|t| t == "python");
    let uses_python = facts.frequently_used_commands.iter()
        .any(|cmd| cmd.command.contains("python") || cmd.command.contains("pip"));

    if has_python_files && uses_python {
        // LSP server for editor integration
        if !package_installed("python-lsp-server") {
            result.push(Advice {
                id: "python-lsp".to_string(),
                title: "Install Python LSP server".to_string(),
                reason: "Active Python development detected (.py files + python command usage) - LSP provides autocomplete, linting, and go-to-definition".to_string(),
                action: "Install python-lsp-server for better editor integration".to_string(),
                command: Some("pacman -S --noconfirm python-lsp-server".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python".to_string()],
                            depends_on: Vec::new(),
                bundle: Some("Python Development Stack".to_string()),
            satisfies: Vec::new(),
                related_to: vec!["python-black".to_string(), "ipython".to_string()],
                popularity: 50,
            });
        }

        // Black formatter
        if !package_installed("python-black") {
            result.push(Advice {
                id: "python-black".to_string(),
                title: "Install Black code formatter".to_string(),
                reason: "Active Python development detected - Black auto-formats code to PEP 8 standard".to_string(),
                action: "Install python-black for consistent code formatting".to_string(),
                command: Some("pacman -S --noconfirm python-black".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python".to_string()],
                            depends_on: Vec::new(),
                bundle: Some("Python Development Stack".to_string()),
            satisfies: Vec::new(),
                related_to: vec!["python-lsp".to_string(), "ipython".to_string()],
                popularity: 50,
            });
        }

        // IPython for better REPL
        if !package_installed("ipython") {
            result.push(Advice {
                id: "ipython".to_string(),
                title: "Install IPython for enhanced Python REPL".to_string(),
                reason: "Active Python development detected - IPython provides better interactive shell".to_string(),
                action: "Install ipython for improved Python REPL experience".to_string(),
                command: Some("pacman -S --noconfirm ipython".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python".to_string()],
                            depends_on: Vec::new(),
                bundle: Some("Python Development Stack".to_string()),
            satisfies: Vec::new(),
                related_to: vec!["python-lsp".to_string(), "python-black".to_string()],
                popularity: 50,
            });
        }
    }

    // Rust development - check for actual .rs files AND cargo usage
    let has_rust_files = facts.common_file_types.iter().any(|t| t == "rust");
    let uses_cargo = facts.frequently_used_commands.iter()
        .any(|cmd| cmd.command.contains("cargo"));

    if has_rust_files && uses_cargo {
        if !package_installed("rust-analyzer") {
            result.push(Advice {
                id: "rust-analyzer".to_string(),
                title: "Install rust-analyzer LSP".to_string(),
                reason: "Active Rust development detected (.rs files + cargo usage) - rust-analyzer provides IDE-like features".to_string(),
                action: "Install rust-analyzer for autocomplete and inline documentation".to_string(),
                command: Some("pacman -S --noconfirm rust-analyzer".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rust".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("Rust Development Stack".to_string()),
            satisfies: Vec::new(),
                popularity: 50,
            });
        }

        // sccache for faster builds
        if !package_installed("sccache") {
            result.push(Advice {
                id: "sccache".to_string(),
                title: "Install sccache for faster Rust builds".to_string(),
                reason: "Active Rust development detected - sccache caches compiled crates, significantly speeds up rebuilds".to_string(),
                action: "Install sccache to speed up compilation times".to_string(),
                command: Some("pacman -S --noconfirm sccache".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rust".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Node.js".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
        }
    }

    // Go development - ONLY trigger if actual .go files found
    if facts.common_file_types.iter().any(|t| t == "go") {
        if !package_installed("gopls") {
            result.push(Advice {
                id: "gopls".to_string(),
                title: "Install gopls language server".to_string(),
                reason: "Go development detected (.go files found) - gopls is the official Go language server".to_string(),
                action: "Install gopls for Go editor integration".to_string(),
                command: Some("go install golang.org/x/tools/gopls@latest".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Go".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker".to_string()],
                            depends_on: vec!["docker-install".to_string()],
                related_to: vec!["lazydocker-install".to_string()],
                bundle: Some("Container Development Stack".to_string()),
            satisfies: Vec::new(),
                popularity: 50,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Neovim".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/System_monitor".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Command-line_shell#Utilities".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Command-line_shell#Utilities".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Mpv".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Music_player".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Feh".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Command-line_shell#Prompts".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Autosuggestions".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Syntax_highlighting".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
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

/// Check for installed packages missing their configuration files
fn check_missing_configs() -> Vec<Advice> {
    let mut result = Vec::new();

    // bat without config
    if package_installed("bat") {
        let config_path = std::path::Path::new("/root/.config/bat/config");
        if !config_path.exists() {
            result.push(Advice {
                id: "bat-config".to_string(),
                title: "Configure bat (syntax highlighter)".to_string(),
                reason: "bat is installed but not configured - add theme and options for better output".to_string(),
                action: "Create bat config with theme and paging options".to_string(),
                command: Some("mkdir -p ~/.config/bat && bat --generate-config-file".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "beautification".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec![],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
        }
    }

    // starship without config
    if package_installed("starship") {
        let config_path = std::path::Path::new("/root/.config/starship.toml");
        if !config_path.exists() {
            result.push(Advice {
                id: "starship-config".to_string(),
                title: "Configure Starship prompt".to_string(),
                reason: "Starship is installed but not configured - customize your prompt appearance".to_string(),
                action: "Create starship config and add to shell init".to_string(),
                command: Some("starship preset nerd-font-symbols > ~/.config/starship.toml".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "beautification".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec![],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
        }
    }

    // git without user config
    if package_installed("git") {
        let output = Command::new("git")
            .args(&["config", "--global", "user.name"])
            .output();

        let has_username = output
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false);

        if !has_username {
            result.push(Advice {
                id: "git-config".to_string(),
                title: "Set your Git identity for commits".to_string(),
                reason: "Git needs to know who you are when you make commits. Your name and email show up in the commit history so people know who authored the changes. This is separate from authentication (SSH keys or gh CLI) - it's just for commit attribution. If you try to commit without this, git will complain!".to_string(),
                action: "Configure your name and email for git commits".to_string(),
                command: Some("git config --global user.name 'Your Name' && git config --global user.email 'your.email@example.com'".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "development".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git#Configuration".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
        }
    }

    // zoxide without shell integration
    if package_installed("zoxide") {
        // Check if zoxide is sourced in shell rc
        let bashrc = std::fs::read_to_string("/root/.bashrc").unwrap_or_default();
        let zshrc = std::fs::read_to_string("/root/.zshrc").unwrap_or_default();

        if !bashrc.contains("zoxide") && !zshrc.contains("zoxide") {
            result.push(Advice {
                id: "zoxide-integration".to_string(),
                title: "Enable zoxide shell integration".to_string(),
                reason: "zoxide is installed but not integrated into your shell - won't work without hook".to_string(),
                action: "Add zoxide initialization to shell rc file".to_string(),
                command: Some("echo 'eval \"$(zoxide init bash)\"' >> ~/.bashrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "beautification".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec![],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
        }
    }

    result
}

/// Recommend hardware support packages
fn recommend_hardware_support(_facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Gamepad support
    if !package_installed("xboxdrv") && !package_installed("xpadneo-dkms") {
        // Check if USB game controllers are present
        let has_gamepad = Command::new("lsusb")
            .output()
            .map(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                output.contains("Xbox") || output.contains("PlayStation") ||
                output.contains("Nintendo") || output.contains("Gamepad") ||
                output.contains("Controller")
            })
            .unwrap_or(false);

        if has_gamepad {
            result.push(Advice {
                id: "gamepad-drivers".to_string(),
                title: "Install gamepad drivers for controller support".to_string(),
                reason: "Game controller detected via USB. xpadneo provides kernel drivers for Xbox controllers, and jstest tools let you test/calibrate any gamepad.".to_string(),
                action: "Install gamepad drivers and testing utilities".to_string(),
                command: Some("pacman -S --noconfirm jstest-gtk linuxconsole".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "gaming".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Gamepad".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
        }
    }

    // Bluetooth support
    if !package_installed("bluez") && !package_installed("bluez-utils") {
        result.push(Advice {
            id: "bluetooth-stack".to_string(),
            title: "Install Bluetooth support".to_string(),
            reason: "No Bluetooth software stack detected. bluez provides the core Bluetooth protocol stack and bluez-utils gives you bluetoothctl for pairing devices.".to_string(),
            action: "Install Bluetooth stack and utilities".to_string(),
            command: Some("pacman -S --noconfirm bluez bluez-utils && systemctl enable bluetooth".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "hardware".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Bluetooth".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // WiFi firmware - check for common WiFi chipsets
    let wifi_firmware_needed = Command::new("lspci")
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout).to_lowercase();
            // Intel WiFi
            if output.contains("wireless") && output.contains("intel") {
                !package_installed("linux-firmware")
            } else if output.contains("qualcomm") || output.contains("atheros") {
                !package_installed("linux-firmware")
            } else if output.contains("broadcom") {
                !package_installed("broadcom-wl") && !package_installed("linux-firmware")
            } else {
                false
            }
        })
        .unwrap_or(false);

    if wifi_firmware_needed {
        result.push(Advice {
            id: "wifi-firmware".to_string(),
            title: "Install WiFi firmware".to_string(),
            reason: "WiFi hardware detected but firmware may be missing. linux-firmware contains drivers for Intel, Qualcomm, and Atheros chipsets.".to_string(),
            action: "Install WiFi firmware package".to_string(),
            command: Some("pacman -S --noconfirm linux-firmware".to_string()),
            risk: RiskLevel::Medium,
            priority: Priority::Recommended,
            category: "hardware".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Network_configuration/Wireless".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // USB automount with udisks2
    if !package_installed("udisks2") {
        result.push(Advice {
            id: "usb-automount".to_string(),
            title: "Enable USB drive automount".to_string(),
            reason: "udisks2 provides automatic mounting of USB drives and external media. Without it, you'll need to manually mount drives with the mount command.".to_string(),
            action: "Install udisks2 for automatic USB mounting".to_string(),
            command: Some("pacman -S --noconfirm udisks2".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "hardware".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Udisks".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // NetworkManager for easier WiFi management
    if !package_installed("networkmanager") && !package_installed("iwd") {
        result.push(Advice {
            id: "network-manager".to_string(),
            title: "Install NetworkManager for WiFi management".to_string(),
            reason: "No network management daemon detected. NetworkManager provides easy WiFi configuration via nmcli or GUI applets.".to_string(),
            action: "Install NetworkManager and enable it".to_string(),
            command: Some("pacman -S --noconfirm networkmanager && systemctl enable NetworkManager".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "networking".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Power management for laptops
    let is_laptop = std::path::Path::new("/sys/class/power_supply/BAT0").exists() ||
                    std::path::Path::new("/sys/class/power_supply/BAT1").exists();

    if is_laptop && !package_installed("tlp") {
        result.push(Advice {
            id: "laptop-power-management".to_string(),
            title: "Install TLP for laptop power management".to_string(),
            reason: "Laptop battery detected. TLP automatically optimizes power settings to extend battery life without manual configuration.".to_string(),
            action: "Install TLP for better battery life".to_string(),
            command: Some("pacman -S --noconfirm tlp && systemctl enable tlp".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "power".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/TLP".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    result
}

/// Recommend desktop environment and display server enhancements
fn recommend_desktop_enhancements(_facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Detect display server
    let using_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
    let using_x11 = std::env::var("DISPLAY").is_ok() && !using_wayland;

    // XWayland for running X11 apps on Wayland
    if using_wayland && !package_installed("xorg-xwayland") {
        result.push(Advice {
            id: "xwayland".to_string(),
            title: "Install XWayland for X11 app compatibility".to_string(),
            reason: "You're using Wayland, but some apps only work on X11. XWayland lets you run X11 apps seamlessly on Wayland.".to_string(),
            action: "Install XWayland compatibility layer".to_string(),
            command: Some("pacman -S --noconfirm xorg-xwayland".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wayland#XWayland".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Compositor for X11 (picom for window effects)
    if using_x11 && !package_installed("picom") {
        result.push(Advice {
            id: "x11-compositor".to_string(),
            title: "Install Picom compositor for window effects".to_string(),
            reason: "You're on X11. Picom adds transparency, shadows, smooth window transitions, and fixes screen tearing.".to_string(),
            action: "Install Picom compositor".to_string(),
            command: Some("pacman -S --noconfirm picom".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Picom".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Terminal recommendations based on DE
    let desktop_env = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_default()
        .to_lowercase();

    // Modern terminal emulators
    if !package_installed("alacritty") && !package_installed("kitty") && !package_installed("wezterm") {
        result.push(Advice {
            id: "modern-terminal".to_string(),
            title: "Install a modern GPU-accelerated terminal".to_string(),
            reason: "Modern terminals like Alacritty, Kitty, or WezTerm offer GPU acceleration, true color, ligatures, and better performance than xterm.".to_string(),
            action: "Install Alacritty terminal emulator".to_string(),
            command: Some("pacman -S --noconfirm alacritty".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Alacritty".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Status bars for tiling WMs
    if desktop_env.contains("i3") && !package_installed("i3status") && !package_installed("i3blocks") {
        result.push(Advice {
            id: "i3-status-bar".to_string(),
            title: "Install i3status or i3blocks for i3 status bar".to_string(),
            reason: "You're using i3 window manager. i3blocks provides a customizable status bar with system info.".to_string(),
            action: "Install i3blocks for status bar".to_string(),
            command: Some("pacman -S --noconfirm i3blocks".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/I3#i3blocks".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    if (desktop_env.contains("sway") || desktop_env.contains("hyprland")) &&
       !package_installed("waybar") && !package_installed("yambar") {
        result.push(Advice {
            id: "wayland-status-bar".to_string(),
            title: "Install Waybar for Wayland status bar".to_string(),
            reason: "You're using a Wayland compositor. Waybar provides a highly customizable status bar with system monitoring.".to_string(),
            action: "Install Waybar".to_string(),
            command: Some("pacman -S --noconfirm waybar".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Waybar".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Application launcher
    if !package_installed("rofi") && !package_installed("wofi") && !package_installed("dmenu") {
        let launcher = if using_wayland { "wofi" } else { "rofi" };
        result.push(Advice {
            id: "app-launcher".to_string(),
            title: format!("Install {} application launcher", launcher),
            reason: format!("{} provides a fast, keyboard-driven app launcher. Essential for tiling window managers.", launcher),
            action: format!("Install {} launcher", launcher),
            command: Some(format!("pacman -S --noconfirm {}", launcher)),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec![format!("https://wiki.archlinux.org/title/{}", launcher.to_uppercase())],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Notification daemon
    if !package_installed("dunst") && !package_installed("mako") {
        let notif_daemon = if using_wayland { "mako" } else { "dunst" };
        result.push(Advice {
            id: "notification-daemon".to_string(),
            title: format!("Install {} notification daemon", notif_daemon),
            reason: "No notification daemon detected. You won't see application notifications without one.".to_string(),
            action: format!("Install {} for desktop notifications", notif_daemon),
            command: Some(format!("pacman -S --noconfirm {}", notif_daemon)),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Desktop_notifications".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    result
}

/// Recommend font packages
fn recommend_fonts(_facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Nerd Fonts for terminal icons
    if !package_installed("ttf-nerd-fonts-symbols") && !package_installed("ttf-jetbrains-mono-nerd") {
        result.push(Advice {
            id: "nerd-fonts".to_string(),
            title: "Install Nerd Fonts for terminal icons".to_string(),
            reason: "Nerd Fonts add icons and glyphs to terminals. Required for modern prompts like Starship, and status bars that show icons.".to_string(),
            action: "Install Nerd Fonts".to_string(),
            command: Some("pacman -S --noconfirm ttf-jetbrains-mono-nerd".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "beautification".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fonts#Patched_packages".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Emoji fonts
    if !package_installed("noto-fonts-emoji") {
        result.push(Advice {
            id: "emoji-fonts".to_string(),
            title: "Install emoji font support".to_string(),
            reason: "Without emoji fonts, emoji characters show as empty boxes. Noto Emoji provides comprehensive emoji rendering.".to_string(),
            action: "Install Noto Emoji fonts".to_string(),
            command: Some("pacman -S --noconfirm noto-fonts-emoji".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "beautification".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fonts#Emoji_and_symbols".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Asian fonts for international text
    if !package_installed("noto-fonts-cjk") {
        result.push(Advice {
            id: "cjk-fonts".to_string(),
            title: "Install CJK (Chinese, Japanese, Korean) fonts".to_string(),
            reason: "Websites and apps with Asian text will show empty boxes without CJK fonts. Noto CJK provides comprehensive coverage.".to_string(),
            action: "Install Noto CJK fonts".to_string(),
            command: Some("pacman -S --noconfirm noto-fonts-cjk".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "beautification".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fonts#Chinese,_Japanese,_Korean,_Vietnamese".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Font rendering improvements
    if !package_installed("freetype2") {
        result.push(Advice {
            id: "font-rendering".to_string(),
            title: "Ensure font rendering library is installed".to_string(),
            reason: "FreeType provides font rendering. Should be installed, but checking to ensure proper font display.".to_string(),
            action: "Install FreeType font rendering".to_string(),
            command: Some("pacman -S --noconfirm freetype2".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Font_configuration".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    result
}

/// Recommend multimedia tools
fn recommend_multimedia_tools(_facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // yt-dlp for downloading videos
    if !package_installed("yt-dlp") {
        result.push(Advice {
            id: "yt-dlp".to_string(),
            title: "Install yt-dlp for downloading videos".to_string(),
            reason: "yt-dlp downloads videos from YouTube, Twitch, and 1000+ sites. It's the actively maintained fork of youtube-dl.".to_string(),
            action: "Install yt-dlp video downloader".to_string(),
            command: Some("pacman -S --noconfirm yt-dlp".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Youtube-dl".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // FFmpeg for video processing
    if !package_installed("ffmpeg") {
        result.push(Advice {
            id: "ffmpeg".to_string(),
            title: "Install FFmpeg for video/audio processing".to_string(),
            reason: "FFmpeg is essential for converting, editing, and streaming media. Many apps depend on it for video playback and encoding.".to_string(),
            action: "Install FFmpeg".to_string(),
            command: Some("pacman -S --noconfirm ffmpeg".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/FFmpeg".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // VLC as alternative video player (more codec support than mpv)
    if !package_installed("vlc") && !package_installed("mpv") {
        result.push(Advice {
            id: "vlc".to_string(),
            title: "Install VLC media player".to_string(),
            reason: "VLC plays virtually any video or audio format without additional codecs. It's the Swiss Army knife of media players.".to_string(),
            action: "Install VLC media player".to_string(),
            command: Some("pacman -S --noconfirm vlc".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/VLC_media_player".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Image manipulation
    if !package_installed("imagemagick") {
        result.push(Advice {
            id: "imagemagick".to_string(),
            title: "Install ImageMagick for image editing".to_string(),
            reason: "ImageMagick provides command-line image editing: resize, convert, composite, annotate. Essential for batch operations.".to_string(),
            action: "Install ImageMagick".to_string(),
            command: Some("pacman -S --noconfirm imagemagick".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/ImageMagick".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Screenshot tools
    let using_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
    if using_wayland && !package_installed("grim") && !package_installed("slurp") {
        result.push(Advice {
            id: "wayland-screenshot".to_string(),
            title: "Install grim + slurp for Wayland screenshots".to_string(),
            reason: "You're on Wayland. grim captures screenshots and slurp lets you select screen regions. Essential for taking screenshots.".to_string(),
            action: "Install Wayland screenshot tools".to_string(),
            command: Some("pacman -S --noconfirm grim slurp".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Screen_capture#Wayland".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    } else if !using_wayland && !package_installed("scrot") && !package_installed("maim") {
        result.push(Advice {
            id: "x11-screenshot".to_string(),
            title: "Install scrot or maim for X11 screenshots".to_string(),
            reason: "You're on X11. scrot or maim provide screenshot capabilities. Bind to a key for quick captures.".to_string(),
            action: "Install screenshot tool".to_string(),
            command: Some("pacman -S --noconfirm maim".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "desktop".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Screen_capture#X11".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    // Audio/video codecs
    if !package_installed("gstreamer") {
        result.push(Advice {
            id: "gstreamer-plugins".to_string(),
            title: "Install GStreamer multimedia framework".to_string(),
            reason: "GStreamer provides audio/video codec support for many applications. Required for media playback in GTK apps.".to_string(),
            action: "Install GStreamer and common plugins".to_string(),
            command: Some("pacman -S --noconfirm gstreamer gst-plugins-good gst-plugins-bad gst-plugins-ugly".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "multimedia".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GStreamer".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            });
    }

    result
}
