//! Capability categories and their metadata.

/// Categories of capabilities Anna can offer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityCategory {
    /// System information and monitoring
    SystemInfo,
    /// Package management
    Packages,
    /// Service management
    Services,
    /// File and config editing
    Configuration,
    /// Network troubleshooting
    Network,
    /// Storage management
    Storage,
    /// Hardware information
    Hardware,
    /// Learning and recipes
    Learning,
    /// Statistics and progress
    Stats,
}

impl CapabilityCategory {
    /// Get all categories
    pub fn all() -> &'static [Self] {
        &[
            Self::SystemInfo,
            Self::Packages,
            Self::Services,
            Self::Configuration,
            Self::Network,
            Self::Storage,
            Self::Hardware,
            Self::Learning,
            Self::Stats,
        ]
    }

    /// Display name for category
    pub fn name(&self) -> &'static str {
        match self {
            Self::SystemInfo => "System Information",
            Self::Packages => "Package Management",
            Self::Services => "Service Management",
            Self::Configuration => "Configuration",
            Self::Network => "Network",
            Self::Storage => "Storage",
            Self::Hardware => "Hardware",
            Self::Learning => "Learning",
            Self::Stats => "Statistics",
        }
    }

    /// Example queries for this category
    pub fn examples(&self) -> &'static [&'static str] {
        match self {
            Self::SystemInfo => &[
                "How much RAM do I have?",
                "What's my CPU?",
                "Check disk space",
                "System health",
            ],
            Self::Packages => &[
                "Install htop",
                "Update my system",
                "What packages are installed?",
                "Remove unused packages",
            ],
            Self::Services => &[
                "Restart docker",
                "Check if nginx is running",
                "Enable ssh on boot",
                "View systemd logs",
            ],
            Self::Configuration => &[
                "Edit my bashrc",
                "Configure git username",
                "Enable syntax highlighting in vim",
                "Change my shell to fish",
            ],
            Self::Network => &[
                "Check my IP address",
                "Test connectivity to google.com",
                "What ports are listening?",
                "Diagnose slow network",
            ],
            Self::Storage => &[
                "List mounted drives",
                "Check disk usage",
                "Find large files",
                "Mount a USB drive",
            ],
            Self::Hardware => &[
                "What GPU do I have?",
                "Check CPU temperature",
                "List USB devices",
                "Audio device info",
            ],
            Self::Learning => &[
                "Enable learning mode",
                "What commands have you learned?",
                "Explain this command",
                "Show me recipes",
            ],
            Self::Stats => &[
                "Show my stats",
                "What's my XP?",
                "Fun statistics",
                "Session summary",
            ],
        }
    }

    /// Brief description of what Anna can do in this category
    pub fn description(&self) -> &'static str {
        match self {
            Self::SystemInfo => {
                "Check system resources, uptime, memory, CPU, and overall health"
            }
            Self::Packages => "Install, update, remove packages across package managers",
            Self::Services => {
                "Start, stop, restart, enable services; view logs and status"
            }
            Self::Configuration => {
                "Edit config files, set up git, configure shell and editors"
            }
            Self::Network => {
                "Diagnose network issues, check connectivity, ports, and IPs"
            }
            Self::Storage => {
                "Check disk space, mount drives, find large files, manage partitions"
            }
            Self::Hardware => {
                "Identify hardware, check temperatures, list devices"
            }
            Self::Learning => {
                "Learn from interactions, explain commands, build recipes"
            }
            Self::Stats => {
                "View XP, level, streaks, statistics, and usage history"
            }
        }
    }
}
