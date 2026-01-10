//! Built-in recipes for common tasks.

use super::types::*;

/// Add built-in recipes to a recipe book
pub fn add_builtin_recipes(book: &mut RecipeBook) {
    // Recipe: Enable vim syntax highlighting
    book.recipes.push(Recipe {
        id: "vim-syntax-highlighting".to_string(),
        name: "Enable Vim Syntax Highlighting".to_string(),
        keywords: vec![
            "vim".to_string(),
            "syntax".to_string(),
            "highlighting".to_string(),
            "color".to_string(),
        ],
        patterns: vec![
            "enable vim syntax highlighting".to_string(),
            "turn on vim syntax".to_string(),
            "vim colors".to_string(),
            "syntax highlighting vim".to_string(),
        ],
        context: RecipeContext {
            editor: Some("vim".to_string()),
            ..Default::default()
        },
        commands: vec![RecipeCommand {
            command:
                "grep -q 'syntax on' ~/.vimrc 2>/dev/null || echo 'syntax on' >> ~/.vimrc"
                    .to_string(),
            description: "Enable syntax highlighting in .vimrc".to_string(),
            modifies_system: true,
            backup_file: Some("~/.vimrc".to_string()),
            needs_root: false,
        }],
        verification: Some(VerificationStep {
            command: "grep 'syntax on' ~/.vimrc".to_string(),
            expected_contains: Some("syntax on".to_string()),
            expected_not_contains: None,
        }),
        source: RecipeSource::BuiltIn,
        success_count: 0,
        last_used: None,
        enabled: true,
    });

    // Recipe: Enable nvim syntax highlighting
    book.recipes.push(Recipe {
        id: "nvim-syntax-highlighting".to_string(),
        name: "Enable Neovim Syntax Highlighting".to_string(),
        keywords: vec![
            "nvim".to_string(),
            "neovim".to_string(),
            "syntax".to_string(),
            "highlighting".to_string(),
        ],
        patterns: vec![
            "enable neovim syntax highlighting".to_string(),
            "nvim syntax".to_string(),
            "neovim colors".to_string(),
        ],
        context: RecipeContext {
            editor: Some("nvim".to_string()),
            ..Default::default()
        },
        commands: vec![RecipeCommand {
            command: "mkdir -p ~/.config/nvim && grep -q 'syntax on' ~/.config/nvim/init.vim 2>/dev/null || echo 'syntax on' >> ~/.config/nvim/init.vim".to_string(),
            description: "Enable syntax highlighting in nvim config".to_string(),
            modifies_system: true,
            backup_file: Some("~/.config/nvim/init.vim".to_string()),
            needs_root: false,
        }],
        verification: Some(VerificationStep {
            command: "grep 'syntax on' ~/.config/nvim/init.vim".to_string(),
            expected_contains: Some("syntax on".to_string()),
            expected_not_contains: None,
        }),
        source: RecipeSource::BuiltIn,
        success_count: 0,
        last_used: None,
        enabled: true,
    });

    // Recipe: Check disk usage
    book.recipes.push(Recipe {
        id: "disk-usage".to_string(),
        name: "Check Disk Usage".to_string(),
        keywords: vec![
            "disk".to_string(),
            "space".to_string(),
            "usage".to_string(),
            "storage".to_string(),
            "full".to_string(),
        ],
        patterns: vec![
            "disk usage".to_string(),
            "disk space".to_string(),
            "how much disk".to_string(),
            "storage space".to_string(),
            "disk full".to_string(),
        ],
        context: RecipeContext::default(),
        commands: vec![RecipeCommand {
            command: "df -h".to_string(),
            description: "Show disk usage in human-readable format".to_string(),
            modifies_system: false,
            backup_file: None,
            needs_root: false,
        }],
        verification: None,
        source: RecipeSource::BuiltIn,
        success_count: 0,
        last_used: None,
        enabled: true,
    });

    // Recipe: Check memory usage
    book.recipes.push(Recipe {
        id: "memory-usage".to_string(),
        name: "Check Memory Usage".to_string(),
        keywords: vec![
            "memory".to_string(),
            "ram".to_string(),
            "usage".to_string(),
            "free".to_string(),
        ],
        patterns: vec![
            "memory usage".to_string(),
            "ram usage".to_string(),
            "how much memory".to_string(),
            "how much ram".to_string(),
            "free memory".to_string(),
        ],
        context: RecipeContext::default(),
        commands: vec![RecipeCommand {
            command: "free -h".to_string(),
            description: "Show memory usage in human-readable format".to_string(),
            modifies_system: false,
            backup_file: None,
            needs_root: false,
        }],
        verification: None,
        source: RecipeSource::BuiltIn,
        success_count: 0,
        last_used: None,
        enabled: true,
    });

    // Recipe: Check CPU usage
    book.recipes.push(Recipe {
        id: "cpu-usage".to_string(),
        name: "Check CPU Usage".to_string(),
        keywords: vec![
            "cpu".to_string(),
            "processor".to_string(),
            "usage".to_string(),
            "load".to_string(),
        ],
        patterns: vec![
            "cpu usage".to_string(),
            "what is using cpu".to_string(),
            "high cpu".to_string(),
            "cpu load".to_string(),
        ],
        context: RecipeContext::default(),
        commands: vec![RecipeCommand {
            command: "ps aux --sort=-%cpu | head -10".to_string(),
            description: "Show top CPU-consuming processes".to_string(),
            modifies_system: false,
            backup_file: None,
            needs_root: false,
        }],
        verification: None,
        source: RecipeSource::BuiltIn,
        success_count: 0,
        last_used: None,
        enabled: true,
    });

    // Recipe: Check kernel version
    book.recipes.push(Recipe {
        id: "kernel-version".to_string(),
        name: "Check Kernel Version".to_string(),
        keywords: vec![
            "kernel".to_string(),
            "version".to_string(),
            "linux".to_string(),
        ],
        patterns: vec![
            "kernel version".to_string(),
            "what kernel".to_string(),
            "linux version".to_string(),
        ],
        context: RecipeContext::default(),
        commands: vec![RecipeCommand {
            command: "uname -r".to_string(),
            description: "Show kernel version".to_string(),
            modifies_system: false,
            backup_file: None,
            needs_root: false,
        }],
        verification: None,
        source: RecipeSource::BuiltIn,
        success_count: 0,
        last_used: None,
        enabled: true,
    });

    // Recipe: Update system (pacman)
    book.recipes.push(Recipe {
        id: "update-system-pacman".to_string(),
        name: "Update System with Pacman".to_string(),
        keywords: vec![
            "update".to_string(),
            "upgrade".to_string(),
            "pacman".to_string(),
            "system".to_string(),
        ],
        patterns: vec![
            "update system".to_string(),
            "upgrade system".to_string(),
            "update packages".to_string(),
            "pacman update".to_string(),
        ],
        context: RecipeContext {
            os: Some("Arch Linux".to_string()),
            ..Default::default()
        },
        commands: vec![RecipeCommand {
            command: "pacman -Syu --noconfirm".to_string(),
            description: "Update all packages".to_string(),
            modifies_system: true,
            backup_file: None,
            needs_root: true,
        }],
        verification: None,
        source: RecipeSource::BuiltIn,
        success_count: 0,
        last_used: None,
        enabled: true,
    });

    // Recipe: List failing services
    book.recipes.push(Recipe {
        id: "failing-services".to_string(),
        name: "Check Failing Services".to_string(),
        keywords: vec![
            "service".to_string(),
            "systemd".to_string(),
            "failing".to_string(),
            "failed".to_string(),
        ],
        patterns: vec![
            "failing services".to_string(),
            "failed services".to_string(),
            "what services".to_string(),
            "systemd failed".to_string(),
        ],
        context: RecipeContext::default(),
        commands: vec![RecipeCommand {
            command: "systemctl --failed".to_string(),
            description: "List failed systemd services".to_string(),
            modifies_system: false,
            backup_file: None,
            needs_root: false,
        }],
        verification: None,
        source: RecipeSource::BuiltIn,
        success_count: 0,
        last_used: None,
        enabled: true,
    });
}
