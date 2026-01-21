//! Built-in recipes for common tasks.
//! NOTE: Anna does not modify user home directories (invariant 2).
//! Recipes that would modify ~/.* files have been removed.

use super::types::*;

/// Add built-in recipes to a recipe book
pub fn add_builtin_recipes(book: &mut RecipeBook) {
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

    // Recipe: Disable sleep/suspend
    book.recipes.push(Recipe {
        id: "disable-sleep".to_string(),
        name: "Disable Sleep and Suspend".to_string(),
        keywords: vec![
            "sleep".to_string(),
            "suspend".to_string(),
            "hibernate".to_string(),
            "disable".to_string(),
            "never".to_string(),
            "prevent".to_string(),
        ],
        patterns: vec![
            "disable sleep".to_string(),
            "prevent sleep".to_string(),
            "never sleep".to_string(),
            "stop sleeping".to_string(),
            "disable suspend".to_string(),
            "no suspend".to_string(),
            "disable hibernate".to_string(),
            "computer never sleep".to_string(),
        ],
        context: RecipeContext::default(),
        commands: vec![RecipeCommand {
            command: "systemctl mask sleep.target suspend.target hibernate.target hybrid-sleep.target".to_string(),
            description: "Mask all sleep/suspend systemd targets to prevent sleep".to_string(),
            modifies_system: true,
            backup_file: None,
            needs_root: true,
        }],
        verification: Some(VerificationStep {
            command: "systemctl status sleep.target".to_string(),
            expected_contains: Some("masked".to_string()),
            expected_not_contains: None,
        }),
        source: RecipeSource::BuiltIn,
        success_count: 0,
        last_used: None,
        enabled: true,
    });

    // Recipe: GDM HiDPI scaling
    book.recipes.push(Recipe {
        id: "gdm-hidpi-scaling".to_string(),
        name: "Scale GDM Login Screen (HiDPI)".to_string(),
        keywords: vec![
            "gdm".to_string(),
            "login".to_string(),
            "scale".to_string(),
            "scaling".to_string(),
            "hidpi".to_string(),
            "tiny".to_string(),
            "small".to_string(),
            "screen".to_string(),
        ],
        patterns: vec![
            "gdm scale".to_string(),
            "gdm scaling".to_string(),
            "login screen scale".to_string(),
            "gdm tiny".to_string(),
            "gdm small".to_string(),
            "gdm hidpi".to_string(),
            "scale gdm".to_string(),
            "gdm 2x".to_string(),
            "login screen too small".to_string(),
        ],
        context: RecipeContext::default(),
        commands: vec![
            RecipeCommand {
                command: "sudo -u gdm dbus-launch gsettings set org.gnome.desktop.interface scaling-factor 2".to_string(),
                description: "Set GDM interface scaling to 2x".to_string(),
                modifies_system: true,
                backup_file: None,
                needs_root: true,
            },
            RecipeCommand {
                command: "sudo -u gdm dbus-launch gsettings set org.gnome.desktop.interface text-scaling-factor 1.5".to_string(),
                description: "Set GDM text scaling to 1.5x".to_string(),
                modifies_system: true,
                backup_file: None,
                needs_root: true,
            },
        ],
        verification: Some(VerificationStep {
            command: "sudo -u gdm dbus-launch gsettings get org.gnome.desktop.interface scaling-factor".to_string(),
            expected_contains: Some("2".to_string()),
            expected_not_contains: None,
        }),
        source: RecipeSource::BuiltIn,
        success_count: 0,
        last_used: None,
        enabled: true,
    });
}
