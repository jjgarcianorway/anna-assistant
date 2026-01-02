//! Command shortcuts types and categories.

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
