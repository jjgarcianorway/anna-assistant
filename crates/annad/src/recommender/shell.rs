//! Shell recommendations

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_shell_enhancements(_facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Get user's shell
    let shell = std::env::var("SHELL").unwrap_or_default();

    if shell.contains("zsh") {
        // Check for oh-my-zsh
        let omz_path = std::path::Path::new("/root/.oh-my-zsh");
        let has_omz = omz_path.exists();

        if !has_omz {
            // Check for starship
            let has_starship = Command::new("pacman")
                .args(&["-Q", "starship"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_starship {
                result.push(Advice {
                    id: "shell-prompt".to_string(),
                    title: "Make your terminal beautiful with Starship".to_string(),
                    reason: "You're using zsh but your prompt is probably pretty basic. Starship is a blazing-fast, customizable prompt that shows git status, programming language versions, and looks gorgeous. It's like giving your terminal a makeover!".to_string(),
                    action: "Install Starship prompt for a beautiful terminal".to_string(),
                    command: Some("pacman -S --noconfirm starship && echo 'eval \"$(starship init zsh)\"' >> ~/.zshrc".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "Desktop Customization".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Starship".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }

        // Check for zsh-autosuggestions
        let has_autosuggestions = Command::new("pacman")
            .args(&["-Q", "zsh-autosuggestions"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_autosuggestions {
            result.push(Advice {
                id: "zsh-autosuggestions".to_string(),
                title: "Get smart command suggestions in zsh".to_string(),
                reason: "As you type commands, zsh-autosuggestions will suggest completions based on your command history. It's like having autocomplete for your terminal - super helpful and saves tons of typing!".to_string(),
                action: "Install zsh-autosuggestions".to_string(),
                command: Some("pacman -S --noconfirm zsh-autosuggestions && echo 'source /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh' >> ~/.zshrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Desktop Customization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Autosuggestions".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for zsh-syntax-highlighting
        let has_highlighting = Command::new("pacman")
            .args(&["-Q", "zsh-syntax-highlighting"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_highlighting {
            result.push(Advice {
                id: "zsh-syntax-highlighting".to_string(),
                title: "Add syntax highlighting to zsh".to_string(),
                reason: "This plugin colors your commands as you type them - valid commands are green, invalid ones are red. It helps catch typos before you hit enter and makes the terminal much easier to read.".to_string(),
                action: "Install zsh-syntax-highlighting".to_string(),
                command: Some("pacman -S --noconfirm zsh-syntax-highlighting && echo 'source /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh' >> ~/.zshrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Desktop Customization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Syntax_highlighting".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    } else if shell.contains("bash") {
        // Check for starship on bash
        let has_starship = Command::new("pacman")
            .args(&["-Q", "starship"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_starship {
            result.push(Advice {
                id: "bash-starship".to_string(),
                title: "Upgrade your bash prompt with Starship".to_string(),
                reason: "Your bash prompt is probably showing just basic info. Starship makes it beautiful and informative, showing git status, programming languages, and more. Works great with bash!".to_string(),
                action: "Install Starship prompt".to_string(),
                command: Some("pacman -S --noconfirm starship && echo 'eval \"$(starship init bash)\"' >> ~/.bashrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Desktop Customization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Starship".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Suggest upgrading to zsh
        result.push(Advice {
            id: "upgrade-to-zsh".to_string(),
            title: "Consider upgrading from bash to zsh".to_string(),
            reason: "You're using bash, which is great! But zsh offers powerful features like better tab completion, spelling correction, and tons of plugins. Many developers make the switch and never look back. It's not required, but if you want a more powerful shell, zsh is worth trying.".to_string(),
            action: "Install zsh and try it out".to_string(),
            command: Some("pacman -S --noconfirm zsh && chsh -s /bin/zsh".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Desktop Customization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for oh-my-posh (PowerShell-style prompt)
    let has_oh_my_posh = Command::new("pacman")
        .args(&["-Q", "oh-my-posh"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_oh_my_posh {
        result.push(Advice {
            id: "oh-my-posh-prompt".to_string(),
            title: "Try oh-my-posh for a PowerShell-style prompt".to_string(),
            reason: "Oh-my-posh brings beautiful, customizable prompts with themes from the PowerShell world. It has tons of pre-made themes and works across bash, zsh, and fish. Great alternative to Starship if you want something different!".to_string(),
            action: "Install oh-my-posh prompt engine".to_string(),
            command: Some("yay -S --noconfirm oh-my-posh".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Shell & Terminal".to_string(),
            alternatives: vec![
                Alternative {
                    name: "Starship".to_string(),
                    description: "Rust-based, minimal prompt (recommended)".to_string(),
                    install_command: "pacman -S starship".to_string(),
                },
                Alternative {
                    name: "Powerlevel10k".to_string(),
                    description: "Fast zsh theme with wizard setup".to_string(),
                    install_command: "yay -S --noconfirm zsh-theme-powerlevel10k-git".to_string(),
                },
            ],
            wiki_refs: vec!["https://wiki.archlinux.org/title/Zsh#Prompts".to_string()],
            depends_on: Vec::new(),
            related_to: vec!["shell-prompt".to_string(), "bash-starship".to_string()],
            bundle: Some("shell-beautification".to_string()),
            satisfies: Vec::new(),
            popularity: 35,
            requires: Vec::new(),
        });
    }

    result
}

pub(crate) fn check_essential_cli_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // bat - better cat with syntax highlighting
    if !Command::new("which")
        .arg("bat")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-bat".to_string(),
            title: "Install bat - a better 'cat' with syntax highlighting".to_string(),
            reason: "bat is like cat but with beautiful syntax highlighting, line numbers, and git integration. Perfect for viewing code files in the terminal!".to_string(),
            action: "Install bat".to_string(),
            command: Some("sudo pacman -S --noconfirm bat".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Development Tools".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
            depends_on: Vec::new(),
            related_to: vec!["install-exa".to_string()],
            bundle: Some("cli-essentials".to_string()),
            satisfies: Vec::new(),
            popularity: 85,
            requires: Vec::new(),
        });
    }

    // eza - modern ls replacement
    if !Command::new("which")
        .arg("eza")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-eza".to_string(),
            title: "Install eza - a modern replacement for 'ls'".to_string(),
            reason: "eza is a modern ls replacement with colors, icons, git integration, and tree view. Much more user-friendly than plain ls!".to_string(),
            action: "Install eza".to_string(),
            command: Some("sudo pacman -S --noconfirm eza".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Development Tools".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities#Alternatives".to_string()],
            depends_on: Vec::new(),
            related_to: vec!["install-bat".to_string()],
            bundle: Some("cli-essentials".to_string()),
            satisfies: Vec::new(),
            popularity: 80,
            requires: Vec::new(),
        });
    }

    // fzf - fuzzy finder
    if !Command::new("which")
        .arg("fzf")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-fzf".to_string(),
            title: "Install fzf - fuzzy command-line finder".to_string(),
            reason: "fzf is a game-changer for terminal productivity! Fuzzy search your command history (Ctrl+R), files, git commits, and more. Once you try it, you can't live without it!".to_string(),
            action: "Install fzf".to_string(),
            command: Some("sudo pacman -S --noconfirm fzf".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Development Tools".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Command-line_shell#Command_history".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: Some("cli-essentials".to_string()),
            satisfies: Vec::new(),
            popularity: 90,
            requires: Vec::new(),
        });
    }

    // tldr - simplified man pages
    if !Command::new("which")
        .arg("tldr")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-tldr".to_string(),
            title: "Install tldr - simplified and practical man pages".to_string(),
            reason: "tldr provides concise, practical examples for commands instead of verbose man pages. Want to know how to use tar? Just run 'tldr tar' for real-world examples!".to_string(),
            action: "Install tldr".to_string(),
            command: Some("sudo pacman -S --noconfirm tldr".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Development Tools".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: Some("cli-essentials".to_string()),
            satisfies: Vec::new(),
            popularity: 75,
            requires: Vec::new(),
        });
    }

    // ncdu - disk usage analyzer
    if !Command::new("which")
        .arg("ncdu")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-ncdu".to_string(),
            title: "Install ncdu - interactive disk usage analyzer".to_string(),
            reason: "ncdu is a fast disk usage analyzer with an ncurses interface. Much better than 'du' for finding what's eating your disk space!".to_string(),
            action: "Install ncdu".to_string(),
            command: Some("sudo pacman -S --noconfirm ncdu".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Utilities".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Core_utilities".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: Some("cli-essentials".to_string()),
            satisfies: Vec::new(),
            popularity: 70,
            requires: Vec::new(),
        });
    }

    result
}

pub(crate) fn check_cli_tools(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check command history for common tools
    let commands: Vec<String> = facts
        .frequently_used_commands
        .iter()
        .map(|c| c.command.clone())
        .collect();

    // ls → eza
    if commands.iter().any(|c| c.starts_with("ls ") || c == "ls") {
        let has_eza = Command::new("pacman")
            .args(&["-Q", "eza"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_eza {
            result.push(Advice {
                id: "cli-eza".to_string(),
                title: "Replace 'ls' with 'eza' for beautiful file listings".to_string(),
                reason: "You use 'ls' a lot. Eza is a modern replacement with colors, icons, git integration, and tree views built-in. It's faster and much prettier than plain ls. Once you try it, you won't go back!".to_string(),
                action: "Install eza as a better 'ls'".to_string(),
                command: Some("pacman -S --noconfirm eza && echo \"alias ls='eza'\" >> ~/.zshrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Desktop Customization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://github.com/eza-community/eza".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // cat → bat
    if commands.iter().any(|c| c.starts_with("cat ") || c == "cat") {
        let has_bat = Command::new("pacman")
            .args(&["-Q", "bat"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_bat {
            result.push(Advice {
                id: "cli-bat".to_string(),
                title: "Replace 'cat' with 'bat' for syntax-highlighted viewing".to_string(),
                reason: "You frequently use 'cat' to view files. Bat is like cat but with syntax highlighting, line numbers, and git integration. It makes reading code and config files so much easier!".to_string(),
                action: "Install bat as a better 'cat'".to_string(),
                command: Some("pacman -S --noconfirm bat && echo \"alias cat='bat'\" >> ~/.zshrc".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Desktop Customization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://github.com/sharkdp/bat".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // grep → ripgrep
    if commands.iter().any(|c| c.contains("grep")) {
        let has_ripgrep = Command::new("pacman")
            .args(&["-Q", "ripgrep"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_ripgrep {
            result.push(Advice {
                id: "cli-ripgrep".to_string(),
                title: "Use 'ripgrep' for lightning-fast code searching".to_string(),
                reason: "You use grep a lot. Ripgrep (command: 'rg') is 10x-100x faster than grep, automatically skips .gitignore files, and has better defaults. It's a game-changer for searching code!".to_string(),
                action: "Install ripgrep for faster searching".to_string(),
                command: Some("pacman -S --noconfirm ripgrep".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Performance & Optimization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://github.com/BurntSushi/ripgrep".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // find → fd
    if commands.iter().any(|c| c.starts_with("find ")) {
        let has_fd = Command::new("pacman")
            .args(&["-Q", "fd"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_fd {
            result.push(Advice {
                id: "cli-fd".to_string(),
                title: "Replace 'find' with 'fd' for easier file searching".to_string(),
                reason: "You use 'find' command. Fd is a simpler, faster alternative with intuitive syntax. Instead of 'find . -name \"*.txt\"' you just type 'fd txt'. It's also much faster and respects .gitignore by default.".to_string(),
                action: "Install fd as a better 'find'".to_string(),
                command: Some("pacman -S --noconfirm fd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Performance & Optimization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://github.com/sharkdp/fd".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for fzf (fuzzy finder)
    let has_fzf = Command::new("pacman")
        .args(&["-Q", "fzf"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_fzf {
        result.push(Advice {
            id: "cli-fzf".to_string(),
            title: "Install 'fzf' for fuzzy finding everything".to_string(),
            reason: "Fzf is a game-changer - it adds fuzzy finding to your terminal. Search command history with Ctrl+R, find files instantly, and integrate with other tools. It's one of those tools you wonder how you lived without!".to_string(),
            action: "Install fzf for fuzzy finding".to_string(),
            command: Some("pacman -S --noconfirm fzf".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Desktop Customization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://github.com/junegunn/fzf".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_archive_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for common archive formats support
    let has_unzip = Command::new("pacman")
        .args(&["-Q", "unzip"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_unrar = Command::new("pacman")
        .args(&["-Q", "unrar"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_p7zip = Command::new("pacman")
        .args(&["-Q", "p7zip"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_unzip {
        result.push(Advice {
            id: "archive-unzip".to_string(),
            title: "Install unzip for ZIP archive support".to_string(),
            reason: "You don't have unzip installed! ZIP is one of the most common archive formats - downloaded files, GitHub repos, Windows files all use it. Without unzip, you can't extract .zip files. It's a tiny package that you'll definitely need.".to_string(),
            action: "Install unzip".to_string(),
            command: Some("pacman -S --noconfirm unzip".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Archiving_and_compression".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    if !has_unrar {
        result.push(Advice {
            id: "archive-unrar".to_string(),
            title: "Install unrar for RAR archive support".to_string(),
            reason: "No RAR support detected! RAR files are super common, especially for downloads, game files, and Windows software. Without unrar, .rar files will just sit there looking useless. Small install, huge convenience!".to_string(),
            action: "Install unrar".to_string(),
            command: Some("pacman -S --noconfirm unrar".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Archiving_and_compression".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    if !has_p7zip {
        result.push(Advice {
            id: "archive-p7zip".to_string(),
            title: "Install p7zip for 7z archive support".to_string(),
            reason: "7z archives offer better compression than ZIP but you can't extract them! p7zip handles .7z files which are increasingly popular for software distribution and large file compression. It also provides better ZIP handling than the basic unzip command.".to_string(),
            action: "Install p7zip".to_string(),
            command: Some("pacman -S --noconfirm p7zip".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Archiving_and_compression".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_shell_productivity() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for bash/zsh completion
    let shell = std::env::var("SHELL").unwrap_or_default();

    if shell.contains("bash") {
        let has_completion = Command::new("pacman")
            .args(&["-Q", "bash-completion"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_completion {
            result.push(Advice {
                id: "shell-bash-completion".to_string(),
                title: "Install bash-completion for better tab completion".to_string(),
                reason: "You're missing bash-completion! This adds intelligent tab-completion for commands, options, and file paths. Press tab and it completes git commands, package names, SSH hosts, and hundreds of other things. Makes the terminal SO much faster!".to_string(),
                action: "Install bash-completion".to_string(),
                command: Some("pacman -S --noconfirm bash-completion".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Utilities".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Bash#Tab_completion".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for fzf (fuzzy finder)
    let has_fzf = Command::new("pacman")
        .args(&["-Q", "fzf"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_fzf {
        result.push(Advice {
            id: "shell-fzf".to_string(),
            title: "Install fzf for fuzzy finding".to_string(),
            reason: "fzf is a GAME CHANGER for terminal productivity! It adds fuzzy search to command history (Ctrl+R), file finding, directory jumping, and more. Instead of scrolling through history or typing paths, just type a few letters and fzf finds what you want. Every power user has this!".to_string(),
            action: "Install fzf fuzzy finder".to_string(),
            command: Some("pacman -S --noconfirm fzf".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fzf".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for tmux/screen
    let has_tmux = Command::new("pacman")
        .args(&["-Q", "tmux"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_screen = Command::new("pacman")
        .args(&["-Q", "screen"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_tmux && !has_screen {
        result.push(Advice {
            id: "shell-tmux".to_string(),
            title: "Install tmux for terminal multiplexing".to_string(),
            reason: "tmux is essential for terminal work! It lets you split your terminal into panes, create multiple windows, and most importantly - detach and reattach sessions. Start a long process over SSH, disconnect, come back later and it's still running! Also great for organizing workflows with split panes.".to_string(),
            action: "Install tmux".to_string(),
            command: Some("pacman -S --noconfirm tmux".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Tmux".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_shell_alternatives() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check current shell
    let current_shell = std::env::var("SHELL").unwrap_or_default();

    if current_shell.contains("bash") {
        // Suggest fish for beginners
        let has_fish = Command::new("pacman")
            .args(&["-Q", "fish"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_fish {
            result.push(Advice {
                id: "shell-fish".to_string(),
                title: "Try Fish shell for modern shell experience".to_string(),
                reason: "Fish (Friendly Interactive SHell) is amazing! Autosuggestions as you type, syntax highlighting, excellent completions out-of-box, web-based configuration. No setup needed - it just works. Try it with 'fish' command, change default with 'chsh -s /usr/bin/fish'!".to_string(),
                action: "Install Fish shell".to_string(),
                command: Some("pacman -S --noconfirm fish".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Terminal & CLI Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fish".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_compression_advanced() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for zstd (modern compression)
    let has_zstd = Command::new("pacman")
        .args(&["-Q", "zstd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_zstd {
        result.push(Advice {
            id: "compression-zstd".to_string(),
            title: "Install zstd for fast modern compression".to_string(),
            reason: "Zstandard (zstd) is the modern compression algorithm! Faster than gzip with better compression ratios. Used by Facebook, Linux kernel, package managers. Great for backups, archives, or compressing data. Command: 'zstd file' to compress, 'unzstd file.zst' to decompress!".to_string(),
            action: "Install zstd".to_string(),
            command: Some("pacman -S --noconfirm zstd".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Zstd".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_dotfile_managers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user has many dotfiles
    let has_many_dotfiles = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-maxdepth",
            "1",
            "-name",
            ".*",
            "-type",
            "f",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count() > 10)
        .unwrap_or(false);

    if has_many_dotfiles {
        // Check for GNU Stow
        let has_stow = Command::new("pacman")
            .args(&["-Q", "stow"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_stow {
            result.push(Advice {
                id: "dotfiles-stow".to_string(),
                title: "Install GNU Stow for dotfile management".to_string(),
                reason: "You have lots of dotfiles! GNU Stow makes managing them easy. Keep configs in git repo, use symlinks to deploy. Switch between different configs, share across machines, version control everything. Simple: 'stow vim' creates symlinks from ~/dotfiles/vim/ to ~/. Game changer!".to_string(),
                action: "Install GNU Stow".to_string(),
                command: Some("pacman -S --noconfirm stow".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Utilities".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Dotfiles#Version_control".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_documentation_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for tldr (simplified man pages)
    let has_tldr = Command::new("which")
        .arg("tldr")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_tldr {
        result.push(Advice {
            id: "docs-tldr".to_string(),
            title: "Install tldr for quick command examples".to_string(),
            reason: "tldr gives you practical command examples! Simpler than man pages, shows common use cases. 'tldr tar' shows how to actually use tar. Community-driven, works offline, way faster than googling. Every command should have a tldr page!".to_string(),
            action: "Install tldr".to_string(),
            command: Some("pacman -S --noconfirm tldr".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Tldr-pages".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

