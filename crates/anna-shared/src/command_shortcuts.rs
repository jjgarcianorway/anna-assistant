//! Command Shortcuts System (v0.0.483).
//!
//! Provides short aliases for common operations.
//! Users can say brief commands and Anna expands them.
//!
//! Examples:
//! - "du" -> "show disk usage"
//! - "mem" -> "show memory usage"
//! - "top5" -> "show top 5 processes by CPU"

use std::collections::HashMap;

/// A command shortcut
#[derive(Debug, Clone)]
pub struct CommandShortcut {
    /// The short form (e.g., "du")
    pub short: &'static str,
    /// The expanded form (e.g., "show disk usage")
    pub expanded: &'static str,
    /// Description of what it does
    pub description: &'static str,
    /// Category for organization
    pub category: ShortcutCategory,
}

/// Shortcut categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutCategory {
    /// System info shortcuts
    System,
    /// Disk/storage shortcuts
    Storage,
    /// Network shortcuts
    Network,
    /// Process shortcuts
    Process,
    /// Service shortcuts
    Service,
    /// Package shortcuts
    Package,
    /// Docker shortcuts
    Docker,
    /// Git shortcuts
    Git,
}

impl ShortcutCategory {
    /// Display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Storage => "Storage",
            Self::Network => "Network",
            Self::Process => "Process",
            Self::Service => "Service",
            Self::Package => "Package",
            Self::Docker => "Docker",
            Self::Git => "Git",
        }
    }

    /// All categories
    pub fn all() -> &'static [Self] {
        &[
            Self::System,
            Self::Storage,
            Self::Network,
            Self::Process,
            Self::Service,
            Self::Package,
            Self::Docker,
            Self::Git,
        ]
    }
}

/// Get all built-in shortcuts
pub fn builtin_shortcuts() -> Vec<CommandShortcut> {
    vec![
        // System shortcuts
        CommandShortcut {
            short: "mem",
            expanded: "show memory usage",
            description: "Display current memory usage",
            category: ShortcutCategory::System,
        },
        CommandShortcut {
            short: "cpu",
            expanded: "show CPU usage",
            description: "Display current CPU usage",
            category: ShortcutCategory::System,
        },
        CommandShortcut {
            short: "uptime",
            expanded: "show system uptime",
            description: "How long the system has been running",
            category: ShortcutCategory::System,
        },
        CommandShortcut {
            short: "health",
            expanded: "system health check",
            description: "Overall system health status",
            category: ShortcutCategory::System,
        },
        // Storage shortcuts
        CommandShortcut {
            short: "du",
            expanded: "show disk usage",
            description: "Display disk space usage",
            category: ShortcutCategory::Storage,
        },
        CommandShortcut {
            short: "df",
            expanded: "show disk free space",
            description: "Display free disk space",
            category: ShortcutCategory::Storage,
        },
        CommandShortcut {
            short: "mounts",
            expanded: "list mounted drives",
            description: "Show all mounted filesystems",
            category: ShortcutCategory::Storage,
        },
        CommandShortcut {
            short: "big",
            expanded: "find large files",
            description: "Find files using the most space",
            category: ShortcutCategory::Storage,
        },
        // Network shortcuts
        CommandShortcut {
            short: "ip",
            expanded: "show my IP address",
            description: "Display network IP addresses",
            category: ShortcutCategory::Network,
        },
        CommandShortcut {
            short: "ports",
            expanded: "show listening ports",
            description: "List ports that are listening",
            category: ShortcutCategory::Network,
        },
        CommandShortcut {
            short: "netstat",
            expanded: "show network connections",
            description: "Display active network connections",
            category: ShortcutCategory::Network,
        },
        CommandShortcut {
            short: "ping",
            expanded: "test network connectivity",
            description: "Check if network is working",
            category: ShortcutCategory::Network,
        },
        // Process shortcuts
        CommandShortcut {
            short: "top5",
            expanded: "show top 5 processes by CPU",
            description: "Processes using most CPU",
            category: ShortcutCategory::Process,
        },
        CommandShortcut {
            short: "ps",
            expanded: "list running processes",
            description: "Show all running processes",
            category: ShortcutCategory::Process,
        },
        CommandShortcut {
            short: "hogs",
            expanded: "show resource hogs",
            description: "Processes using most resources",
            category: ShortcutCategory::Process,
        },
        // Service shortcuts
        CommandShortcut {
            short: "services",
            expanded: "list running services",
            description: "Show active systemd services",
            category: ShortcutCategory::Service,
        },
        CommandShortcut {
            short: "failed",
            expanded: "show failed services",
            description: "List services that failed to start",
            category: ShortcutCategory::Service,
        },
        CommandShortcut {
            short: "logs",
            expanded: "show system logs",
            description: "Display recent system logs",
            category: ShortcutCategory::Service,
        },
        // Package shortcuts
        CommandShortcut {
            short: "updates",
            expanded: "check for updates",
            description: "See available package updates",
            category: ShortcutCategory::Package,
        },
        CommandShortcut {
            short: "upgrade",
            expanded: "update my system",
            description: "Install all available updates",
            category: ShortcutCategory::Package,
        },
        CommandShortcut {
            short: "orphans",
            expanded: "find orphan packages",
            description: "Packages no longer needed",
            category: ShortcutCategory::Package,
        },
        // Docker shortcuts
        CommandShortcut {
            short: "dps",
            expanded: "list docker containers",
            description: "Show running Docker containers",
            category: ShortcutCategory::Docker,
        },
        CommandShortcut {
            short: "dimages",
            expanded: "list docker images",
            description: "Show Docker images",
            category: ShortcutCategory::Docker,
        },
        CommandShortcut {
            short: "dclean",
            expanded: "clean up docker",
            description: "Remove unused Docker resources",
            category: ShortcutCategory::Docker,
        },
        // Git shortcuts
        CommandShortcut {
            short: "gs",
            expanded: "git status",
            description: "Show git repository status",
            category: ShortcutCategory::Git,
        },
        CommandShortcut {
            short: "gl",
            expanded: "git log",
            description: "Show recent git commits",
            category: ShortcutCategory::Git,
        },
        CommandShortcut {
            short: "gd",
            expanded: "git diff",
            description: "Show uncommitted changes",
            category: ShortcutCategory::Git,
        },
    ]
}

/// Build a lookup map for quick expansion
pub fn build_shortcut_map() -> HashMap<&'static str, &'static str> {
    builtin_shortcuts()
        .iter()
        .map(|s| (s.short, s.expanded))
        .collect()
}

/// Expand a shortcut if it matches
pub fn expand_shortcut(input: &str) -> Option<String> {
    let trimmed = input.trim().to_lowercase();

    for shortcut in builtin_shortcuts() {
        if trimmed == shortcut.short {
            return Some(shortcut.expanded.to_string());
        }
    }

    None
}

/// Check if input is a shortcut
pub fn is_shortcut(input: &str) -> bool {
    let trimmed = input.trim().to_lowercase();
    builtin_shortcuts().iter().any(|s| s.short == trimmed)
}

/// Get shortcuts for a specific category
pub fn shortcuts_by_category(category: ShortcutCategory) -> Vec<CommandShortcut> {
    builtin_shortcuts()
        .into_iter()
        .filter(|s| s.category == category)
        .collect()
}

/// Format all shortcuts for display
pub fn format_shortcuts() -> String {
    let mut output = String::new();

    output.push_str("Command Shortcuts\n");
    output.push_str("══════════════════════════════════════\n\n");

    for category in ShortcutCategory::all() {
        let shortcuts = shortcuts_by_category(*category);
        if shortcuts.is_empty() {
            continue;
        }

        output.push_str(&format!("▸ {}\n", category.name()));
        for shortcut in shortcuts {
            output.push_str(&format!(
                "  {:8} → {}\n",
                shortcut.short, shortcut.expanded
            ));
        }
        output.push('\n');
    }

    output.push_str("──────────────────────────────────────\n");
    output.push_str("Type any shortcut to expand it.\n");

    output
}

/// Format shortcuts for a specific category
pub fn format_category_shortcuts(category: ShortcutCategory) -> String {
    let mut output = String::new();

    output.push_str(&format!("{} Shortcuts\n", category.name()));
    output.push_str("══════════════════════════════════════\n\n");

    for shortcut in shortcuts_by_category(category) {
        output.push_str(&format!(
            "  {:8} → {}\n           {}\n\n",
            shortcut.short, shortcut.expanded, shortcut.description
        ));
    }

    output
}

/// Detect if query is asking about shortcuts
pub fn is_shortcuts_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "shortcut",
        "short command",
        "quick command",
        "alias",
        "abbreviation",
        "what shortcuts",
        "list shortcuts",
        "show shortcuts",
    ];

    for pattern in patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_shortcuts() {
        let shortcuts = builtin_shortcuts();
        assert!(shortcuts.len() >= 20);

        // Check each shortcut is valid
        for shortcut in &shortcuts {
            assert!(!shortcut.short.is_empty());
            assert!(!shortcut.expanded.is_empty());
            assert!(!shortcut.description.is_empty());
        }
    }

    #[test]
    fn test_expand_shortcut() {
        assert_eq!(
            expand_shortcut("mem"),
            Some("show memory usage".to_string())
        );
        assert_eq!(
            expand_shortcut("du"),
            Some("show disk usage".to_string())
        );
        assert_eq!(expand_shortcut("unknown"), None);
    }

    #[test]
    fn test_expand_shortcut_case_insensitive() {
        assert_eq!(
            expand_shortcut("MEM"),
            Some("show memory usage".to_string())
        );
        assert_eq!(
            expand_shortcut("Du"),
            Some("show disk usage".to_string())
        );
    }

    #[test]
    fn test_is_shortcut() {
        assert!(is_shortcut("mem"));
        assert!(is_shortcut("du"));
        assert!(is_shortcut("dps"));
        assert!(!is_shortcut("not a shortcut"));
    }

    #[test]
    fn test_shortcuts_by_category() {
        let system = shortcuts_by_category(ShortcutCategory::System);
        assert!(!system.is_empty());
        assert!(system.iter().all(|s| s.category == ShortcutCategory::System));

        let docker = shortcuts_by_category(ShortcutCategory::Docker);
        assert!(!docker.is_empty());
        assert!(docker.iter().all(|s| s.category == ShortcutCategory::Docker));
    }

    #[test]
    fn test_format_shortcuts() {
        let output = format_shortcuts();
        assert!(output.contains("Command Shortcuts"));
        assert!(output.contains("System"));
        assert!(output.contains("mem"));
        assert!(output.contains("show memory usage"));
    }

    #[test]
    fn test_format_category_shortcuts() {
        let output = format_category_shortcuts(ShortcutCategory::Docker);
        assert!(output.contains("Docker Shortcuts"));
        assert!(output.contains("dps"));
        assert!(output.contains("list docker containers"));
    }

    #[test]
    fn test_is_shortcuts_query() {
        assert!(is_shortcuts_query("show shortcuts"));
        assert!(is_shortcuts_query("what shortcuts are available?"));
        assert!(is_shortcuts_query("list all aliases"));

        assert!(!is_shortcuts_query("show disk usage"));
        assert!(!is_shortcuts_query("restart docker"));
    }

    #[test]
    fn test_build_shortcut_map() {
        let map = build_shortcut_map();
        assert_eq!(map.get("mem"), Some(&"show memory usage"));
        assert_eq!(map.get("du"), Some(&"show disk usage"));
        assert!(map.len() >= 20);
    }

    #[test]
    fn test_all_categories_have_shortcuts() {
        for category in ShortcutCategory::all() {
            let shortcuts = shortcuts_by_category(*category);
            assert!(
                !shortcuts.is_empty(),
                "{:?} should have shortcuts",
                category
            );
        }
    }
}
