//! Built-in command shortcuts definitions.

use super::types::{CommandShortcut, ShortcutCategory};

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

/// Get shortcuts for a specific category
pub fn shortcuts_by_category(category: ShortcutCategory) -> Vec<CommandShortcut> {
    builtin_shortcuts()
        .into_iter()
        .filter(|s| s.category == category)
        .collect()
}
