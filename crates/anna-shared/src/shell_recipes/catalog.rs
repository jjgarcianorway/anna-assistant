//! Shell recipe catalog (v0.0.231).

use super::types::{Shell, ShellFeature, ShellRecipe};

/// Get built-in shell recipes
pub fn builtin_recipes() -> Vec<ShellRecipe> {
    vec![
        // Bash - colored prompt
        ShellRecipe::new(
            Shell::Bash,
            ShellFeature::ColoredPrompt,
            "Add colored prompt to bash",
            vec![
                "# Colored prompt",
                r#"PS1='\[\e[32m\]\u@\h\[\e[0m\]:\[\e[34m\]\w\[\e[0m\]$ '"#,
            ],
        ).with_rollback("Remove the PS1= line from .bashrc"),

        // Bash - git prompt
        ShellRecipe::new(
            Shell::Bash,
            ShellFeature::GitPrompt,
            "Show git branch in bash prompt",
            vec![
                "# Git branch in prompt",
                r#"parse_git_branch() { git branch 2>/dev/null | grep '^*' | cut -d' ' -f2 | sed 's/.*/ (&)/'; }"#,
                r#"PS1='\[\e[32m\]\u@\h\[\e[0m\]:\[\e[34m\]\w\[\e[33m\]$(parse_git_branch)\[\e[0m\]$ '"#,
            ],
        ).with_rollback("Remove parse_git_branch function and PS1 line"),

        // Bash - colored ls
        ShellRecipe::new(
            Shell::Bash,
            ShellFeature::ColoredLs,
            "Enable colored ls output",
            vec![
                "# Colored ls",
                "alias ls='ls --color=auto'",
                "alias ll='ls -la --color=auto'",
            ],
        ).with_rollback("Remove the ls aliases from .bashrc"),

        // Bash - history settings
        ShellRecipe::new(
            Shell::Bash,
            ShellFeature::HistorySettings,
            "Improve bash history",
            vec![
                "# History settings",
                "HISTSIZE=10000",
                "HISTFILESIZE=20000",
                "HISTCONTROL=ignoreboth:erasedups",
                "shopt -s histappend",
            ],
        ).with_rollback("Remove HIST* lines from .bashrc"),

        // Zsh - syntax highlighting
        ShellRecipe::new(
            Shell::Zsh,
            ShellFeature::SyntaxHighlighting,
            "Enable zsh syntax highlighting",
            vec![
                "# Syntax highlighting (requires zsh-syntax-highlighting package)",
                "source /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh 2>/dev/null",
            ],
        ).with_rollback("Remove the source line for syntax highlighting"),

        // Zsh - auto-suggestions
        ShellRecipe::new(
            Shell::Zsh,
            ShellFeature::AutoSuggestions,
            "Enable zsh auto-suggestions",
            vec![
                "# Auto-suggestions (requires zsh-autosuggestions package)",
                "source /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh 2>/dev/null",
            ],
        ).with_rollback("Remove the source line for auto-suggestions"),

        // Zsh - colored prompt
        ShellRecipe::new(
            Shell::Zsh,
            ShellFeature::ColoredPrompt,
            "Add colored prompt to zsh",
            vec![
                "# Colored prompt",
                "autoload -U colors && colors",
                r#"PROMPT='%{$fg[green]%}%n@%m%{$reset_color%}:%{$fg[blue]%}%~%{$reset_color%}$ '"#,
            ],
        ).with_rollback("Remove PROMPT= line from .zshrc"),

        // Zsh - git prompt
        ShellRecipe::new(
            Shell::Zsh,
            ShellFeature::GitPrompt,
            "Show git branch in zsh prompt",
            vec![
                "# Git branch in prompt",
                "autoload -Uz vcs_info",
                "precmd() { vcs_info }",
                "zstyle ':vcs_info:git:*' formats ' (%b)'",
                "setopt PROMPT_SUBST",
                r#"PROMPT='%{$fg[green]%}%n@%m%{$reset_color%}:%{$fg[blue]%}%~%{$fg[yellow]%}${vcs_info_msg_0_}%{$reset_color%}$ '"#,
            ],
        ).with_rollback("Remove vcs_info and PROMPT lines"),

        // Fish - syntax highlighting (built-in, but can configure)
        ShellRecipe::new(
            Shell::Fish,
            ShellFeature::SyntaxHighlighting,
            "Configure fish syntax highlighting colors",
            vec![
                "# Syntax highlighting colors",
                "set fish_color_command green",
                "set fish_color_param cyan",
                "set fish_color_error red",
            ],
        ).with_rollback("Remove set fish_color_* lines"),

        // Common aliases (all shells)
        ShellRecipe::new(
            Shell::Bash,
            ShellFeature::Aliases,
            "Common useful aliases",
            vec![
                "# Common aliases",
                "alias ..='cd ..'",
                "alias ...='cd ../..'",
                "alias grep='grep --color=auto'",
                "alias df='df -h'",
                "alias du='du -h'",
            ],
        ).with_rollback("Remove the alias lines from .bashrc"),
    ]
}
