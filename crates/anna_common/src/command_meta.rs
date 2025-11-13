// Command Classification Metadata
// Phase 3.1: Contextual Autonomy and Adaptive Simplicity
//
// Every command in Anna has metadata that determines:
// - Who should use it (user-safe, advanced, internal)
// - What risks it poses (none, low, medium, high, critical)
// - When it's available (system requirements, daemon status)
// - How to display it (adaptive help based on context)

use serde::{Deserialize, Serialize};

/// Command category determines visibility and safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandCategory {
    /// User-Safe: Everyday commands safe for all users
    /// Examples: help, status, health, metrics
    UserSafe,

    /// Advanced: System administration commands requiring knowledge
    /// Examples: update, install, doctor, repair
    Advanced,

    /// Internal: Developer/diagnostic commands for experts
    /// Examples: sentinel, conscience, consensus
    Internal,
}

impl std::fmt::Display for CommandCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandCategory::UserSafe => write!(f, "user-safe"),
            CommandCategory::Advanced => write!(f, "advanced"),
            CommandCategory::Internal => write!(f, "internal"),
        }
    }
}

/// Risk level of command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// No risk: Read-only operations
    None,

    /// Low: Minimal impact (display info, check status)
    Low,

    /// Medium: Can modify system (restart services, clear cache)
    Medium,

    /// High: Potentially destructive (filesystem ops, major changes)
    High,

    /// Critical: Extremely dangerous (partitioning, bootloader)
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::None => write!(f, "none"),
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Medium => write!(f, "medium"),
            RiskLevel::High => write!(f, "high"),
            RiskLevel::Critical => write!(f, "critical"),
        }
    }
}

/// Command metadata
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    /// Command name (e.g., "help", "status", "update")
    pub name: &'static str,

    /// Category determines visibility
    pub category: CommandCategory,

    /// Risk level determines warnings
    pub risk_level: RiskLevel,

    /// Requires root/sudo access
    pub requires_root: bool,

    /// Requires daemon to be running
    pub requires_daemon: bool,

    /// Minimum system states where command is available
    pub available_states: &'static [&'static str],

    /// Short description (1 line)
    pub description_short: &'static str,

    /// Long description (multi-line, markdown supported)
    pub description_long: &'static str,

    /// Usage examples
    pub examples: &'static [&'static str],

    /// Related commands
    pub see_also: &'static [&'static str],
}

impl CommandMetadata {
    /// Check if command should be visible in current context
    pub fn is_visible(&self, context: &DisplayContext) -> bool {
        // Filter by category based on user level
        let category_visible = match context.user_level {
            UserLevel::Beginner => self.category == CommandCategory::UserSafe,
            UserLevel::Intermediate => self.category != CommandCategory::Internal,
            UserLevel::Expert => true,
        };

        // Filter by daemon availability
        let daemon_visible = if self.requires_daemon {
            context.daemon_available
        } else {
            true
        };

        // Filter by system state
        let state_visible = if self.available_states.is_empty() {
            true
        } else {
            self.available_states.contains(&context.system_state.as_str())
        };

        category_visible && daemon_visible && state_visible
    }

    /// Check if command should be highlighted in current context
    pub fn is_highlighted(&self, context: &DisplayContext) -> bool {
        // Highlight commands based on system state
        match context.system_state.as_str() {
            "degraded" => {
                matches!(self.name, "doctor" | "repair" | "health" | "rollback")
            }
            "iso_live" => {
                matches!(self.name, "install" | "rescue")
            }
            _ => false,
        }
    }

    /// Get display priority (lower = shown first)
    pub fn display_priority(&self, context: &DisplayContext) -> u32 {
        let base_priority = match self.category {
            CommandCategory::UserSafe => 0,
            CommandCategory::Advanced => 100,
            CommandCategory::Internal => 200,
        };

        // Boost priority if highlighted
        if self.is_highlighted(context) {
            base_priority / 2
        } else {
            base_priority
        }
    }
}

/// Display context for adaptive help
#[derive(Debug, Clone)]
pub struct DisplayContext {
    /// User experience level
    pub user_level: UserLevel,

    /// Is daemon available?
    pub daemon_available: bool,

    /// Current system state
    pub system_state: String,

    /// Is system resource-constrained?
    pub is_constrained: bool,

    /// Current monitoring mode
    pub monitoring_mode: Option<String>,
}

impl Default for DisplayContext {
    fn default() -> Self {
        Self {
            user_level: UserLevel::Intermediate,
            daemon_available: false,
            system_state: "unknown".to_string(),
            is_constrained: false,
            monitoring_mode: None,
        }
    }
}

/// User experience level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserLevel {
    /// Beginner: Show only user-safe commands
    Beginner,

    /// Intermediate: Show user-safe + common advanced
    Intermediate,

    /// Expert: Show all commands
    Expert,
}

impl std::fmt::Display for UserLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserLevel::Beginner => write!(f, "beginner"),
            UserLevel::Intermediate => write!(f, "intermediate"),
            UserLevel::Expert => write!(f, "expert"),
        }
    }
}

/// Registry of all command metadata
pub struct CommandRegistry {
    commands: Vec<CommandMetadata>,
}

impl CommandRegistry {
    /// Create registry with all commands
    pub fn new() -> Self {
        Self {
            commands: create_command_metadata(),
        }
    }

    /// Get metadata for a command by name
    pub fn get(&self, name: &str) -> Option<&CommandMetadata> {
        self.commands.iter().find(|cmd| cmd.name == name)
    }

    /// Get all commands
    pub fn all(&self) -> &[CommandMetadata] {
        &self.commands
    }

    /// Get visible commands for context
    pub fn visible(&self, context: &DisplayContext) -> Vec<&CommandMetadata> {
        let mut visible: Vec<_> = self
            .commands
            .iter()
            .filter(|cmd| cmd.is_visible(context))
            .collect();

        // Sort by priority (highlighted first, then by category)
        visible.sort_by_key(|cmd| cmd.display_priority(context));

        visible
    }

    /// Get commands by category
    pub fn by_category(&self, category: CommandCategory) -> Vec<&CommandMetadata> {
        self.commands
            .iter()
            .filter(|cmd| cmd.category == category)
            .collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create metadata for all commands
fn create_command_metadata() -> Vec<CommandMetadata> {
    vec![
        // User-Safe Commands (🟢)
        CommandMetadata {
            name: "init",
            category: CommandCategory::Advanced,
            risk_level: RiskLevel::Low,
            requires_root: true,
            requires_daemon: false,
            available_states: &[],
            description_short: "Initialize Anna (first-run wizard)",
            description_long: "Create /etc/anna directory and generate default configuration files. Detects system constraints and provides guidance for first-time setup. Requires root permissions.",
            examples: &[
                "sudo annactl init",
            ],
            see_also: &["help", "status", "profile"],
        },
        CommandMetadata {
            name: "help",
            category: CommandCategory::UserSafe,
            risk_level: RiskLevel::None,
            requires_root: false,
            requires_daemon: false,
            available_states: &[],
            description_short: "Show available commands and help",
            description_long: "Display comprehensive help about Anna commands, adapted to your experience level and system state.",
            examples: &[
                "annactl help",
                "annactl help update",
                "annactl help --all",
            ],
            see_also: &[],
        },
        CommandMetadata {
            name: "status",
            category: CommandCategory::UserSafe,
            risk_level: RiskLevel::None,
            requires_root: false,
            requires_daemon: true,
            available_states: &[],
            description_short: "Show current system status",
            description_long: "Display Anna's view of system health, including state, probes, and recommendations.",
            examples: &[
                "annactl status",
                "annactl status --json",
            ],
            see_also: &["health", "metrics"],
        },
        CommandMetadata {
            name: "health",
            category: CommandCategory::UserSafe,
            risk_level: RiskLevel::None,
            requires_root: false,
            requires_daemon: true,
            available_states: &[],
            description_short: "Check system health probes",
            description_long: "Run health probes to detect system issues like failed services, low disk space, or outdated packages.",
            examples: &[
                "annactl health",
                "annactl health --json",
                "annactl health --probe systemd-failed-units",
            ],
            see_also: &["status", "doctor"],
        },
        CommandMetadata {
            name: "metrics",
            category: CommandCategory::UserSafe,
            risk_level: RiskLevel::None,
            requires_root: false,
            requires_daemon: true,
            available_states: &[],
            description_short: "Display system metrics",
            description_long: "Show current system metrics including memory, CPU, disk, and adaptive intelligence context.",
            examples: &[
                "annactl metrics",
                "annactl metrics --prometheus",
                "annactl metrics --json",
            ],
            see_also: &["profile", "status"],
        },
        CommandMetadata {
            name: "profile",
            category: CommandCategory::UserSafe,
            risk_level: RiskLevel::None,
            requires_root: false,
            requires_daemon: true,
            available_states: &[],
            description_short: "Show system profile and capabilities",
            description_long: "Display Anna's understanding of your system: resources, monitoring mode, constraints, and recommendations.",
            examples: &[
                "annactl profile",
                "annactl profile --json",
            ],
            see_also: &["metrics", "status"],
        },
        CommandMetadata {
            name: "ping",
            category: CommandCategory::UserSafe,
            risk_level: RiskLevel::None,
            requires_root: false,
            requires_daemon: true,
            available_states: &[],
            description_short: "Test daemon connection",
            description_long: "Verify that Anna daemon is responsive and healthy.",
            examples: &[
                "annactl ping",
            ],
            see_also: &["status"],
        },
        CommandMetadata {
            name: "learn",
            category: CommandCategory::UserSafe,
            risk_level: RiskLevel::None,
            requires_root: false,
            requires_daemon: false,
            available_states: &[],
            description_short: "Analyze patterns in action history",
            description_long: "Detect patterns in system behavior including maintenance windows, recurring failures, and usage habits. Uses local action history from the context database.",
            examples: &[
                "annactl learn",
                "annactl learn --json",
                "annactl learn --min-confidence high --days 60",
            ],
            see_also: &["predict", "status"],
        },
        CommandMetadata {
            name: "predict",
            category: CommandCategory::UserSafe,
            risk_level: RiskLevel::None,
            requires_root: false,
            requires_daemon: false,
            available_states: &[],
            description_short: "Show predictive intelligence",
            description_long: "Display predictions and recommendations based on detected patterns. By default shows only high and critical priority predictions.",
            examples: &[
                "annactl predict",
                "annactl predict --all",
                "annactl predict --json",
            ],
            see_also: &["learn", "status", "health"],
        },

        // Advanced Commands (🟡)
        CommandMetadata {
            name: "update",
            category: CommandCategory::Advanced,
            risk_level: RiskLevel::Medium,
            requires_root: true,
            requires_daemon: true,
            available_states: &["configured", "healthy", "degraded"],
            description_short: "Update system packages",
            description_long: "Update all system packages safely with automatic rollback support. Anna checks resources before proceeding.",
            examples: &[
                "sudo annactl update",
                "sudo annactl update --dry-run",
            ],
            see_also: &["doctor", "rollback"],
        },
        CommandMetadata {
            name: "install",
            category: CommandCategory::Advanced,
            risk_level: RiskLevel::High,
            requires_root: true,
            requires_daemon: true,
            available_states: &["iso_live"],
            description_short: "Install Arch Linux",
            description_long: "Interactive Arch Linux installation with automatic partitioning, package selection, and configuration.",
            examples: &[
                "sudo annactl install",
                "sudo annactl install --config install.toml",
            ],
            see_also: &["rescue"],
        },
        CommandMetadata {
            name: "doctor",
            category: CommandCategory::Advanced,
            risk_level: RiskLevel::Medium,
            requires_root: true,
            requires_daemon: true,
            available_states: &[],
            description_short: "Diagnose and fix system issues",
            description_long: "Run comprehensive diagnostics and propose fixes for detected issues. Requires confirmation before applying fixes.",
            examples: &[
                "sudo annactl doctor",
                "sudo annactl doctor --auto-fix",
            ],
            see_also: &["health", "repair", "rollback"],
        },
        CommandMetadata {
            name: "repair",
            category: CommandCategory::Advanced,
            risk_level: RiskLevel::Medium,
            requires_root: true,
            requires_daemon: true,
            available_states: &[],
            description_short: "Repair failed health probes",
            description_long: "Attempt to repair specific failed probes. Less aggressive than doctor, more targeted.",
            examples: &[
                "sudo annactl repair --probe systemd-failed-units",
                "sudo annactl repair --all",
            ],
            see_also: &["doctor", "health"],
        },

        // Internal Commands (🔴)
        CommandMetadata {
            name: "sentinel",
            category: CommandCategory::Internal,
            risk_level: RiskLevel::Low,
            requires_root: false,
            requires_daemon: true,
            available_states: &[],
            description_short: "Sentinel framework management",
            description_long: "Manage Anna's sentinel framework for automated monitoring and response.",
            examples: &[
                "annactl sentinel status",
                "annactl sentinel history",
            ],
            see_also: &[],
        },
        CommandMetadata {
            name: "conscience",
            category: CommandCategory::Internal,
            risk_level: RiskLevel::Low,
            requires_root: false,
            requires_daemon: true,
            available_states: &[],
            description_short: "Conscience governance diagnostics",
            description_long: "Inspect Anna's ethical decision-making and governance layer.",
            examples: &[
                "annactl conscience status",
                "annactl conscience history",
            ],
            see_also: &[],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry() {
        let registry = CommandRegistry::new();
        assert!(!registry.all().is_empty());
        assert!(registry.get("help").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_visibility_beginner() {
        let registry = CommandRegistry::new();
        let context = DisplayContext {
            user_level: UserLevel::Beginner,
            daemon_available: true,
            system_state: "healthy".to_string(),
            is_constrained: false,
            monitoring_mode: Some("light".to_string()),
        };

        let visible = registry.visible(&context);
        assert!(!visible.is_empty());

        // All visible commands should be user-safe
        for cmd in visible {
            assert_eq!(cmd.category, CommandCategory::UserSafe);
        }
    }

    #[test]
    fn test_visibility_intermediate() {
        let registry = CommandRegistry::new();
        let context = DisplayContext {
            user_level: UserLevel::Intermediate,
            daemon_available: true,
            system_state: "healthy".to_string(),
            is_constrained: false,
            monitoring_mode: Some("light".to_string()),
        };

        let visible = registry.visible(&context);
        let has_user_safe = visible.iter().any(|cmd| cmd.category == CommandCategory::UserSafe);
        let has_advanced = visible.iter().any(|cmd| cmd.category == CommandCategory::Advanced);
        let has_internal = visible.iter().any(|cmd| cmd.category == CommandCategory::Internal);

        assert!(has_user_safe);
        assert!(has_advanced);
        assert!(!has_internal); // Intermediate shouldn't see internal
    }

    #[test]
    fn test_visibility_expert() {
        let registry = CommandRegistry::new();
        let context = DisplayContext {
            user_level: UserLevel::Expert,
            daemon_available: true,
            system_state: "healthy".to_string(),
            is_constrained: false,
            monitoring_mode: Some("full".to_string()),
        };

        let visible = registry.visible(&context);
        let has_user_safe = visible.iter().any(|cmd| cmd.category == CommandCategory::UserSafe);
        let has_advanced = visible.iter().any(|cmd| cmd.category == CommandCategory::Advanced);
        let has_internal = visible.iter().any(|cmd| cmd.category == CommandCategory::Internal);

        assert!(has_user_safe);
        assert!(has_advanced);
        assert!(has_internal); // Expert sees everything
    }

    #[test]
    fn test_daemon_requirement_filtering() {
        let registry = CommandRegistry::new();
        let context = DisplayContext {
            user_level: UserLevel::Expert,
            daemon_available: false, // Daemon not available
            system_state: "unknown".to_string(),
            is_constrained: false,
            monitoring_mode: None,
        };

        let visible = registry.visible(&context);

        // Commands requiring daemon should not be visible
        for cmd in visible {
            assert!(!cmd.requires_daemon);
        }

        // Help should always be visible
        assert!(registry.get("help").unwrap().is_visible(&context));
    }

    #[test]
    fn test_highlighting() {
        let registry = CommandRegistry::new();
        let degraded_context = DisplayContext {
            user_level: UserLevel::Intermediate,
            daemon_available: true,
            system_state: "degraded".to_string(),
            is_constrained: false,
            monitoring_mode: Some("light".to_string()),
        };

        let doctor = registry.get("doctor").unwrap();
        assert!(doctor.is_highlighted(&degraded_context));

        let help = registry.get("help").unwrap();
        assert!(!help.is_highlighted(&degraded_context));
    }

    #[test]
    fn test_display_priority() {
        let registry = CommandRegistry::new();
        let degraded_context = DisplayContext {
            user_level: UserLevel::Intermediate,
            daemon_available: true,
            system_state: "degraded".to_string(),
            is_constrained: false,
            monitoring_mode: Some("light".to_string()),
        };

        let doctor = registry.get("doctor").unwrap();
        let repair = registry.get("repair").unwrap();
        let update = registry.get("update").unwrap();

        // Doctor should have higher priority (lower number) than non-highlighted advanced commands
        // when in degraded state (both are Advanced, but doctor is highlighted)
        assert!(doctor.display_priority(&degraded_context) < update.display_priority(&degraded_context));
        assert!(repair.display_priority(&degraded_context) < update.display_priority(&degraded_context));
    }

    #[test]
    fn test_category_filtering() {
        let registry = CommandRegistry::new();
        let user_safe = registry.by_category(CommandCategory::UserSafe);
        let advanced = registry.by_category(CommandCategory::Advanced);
        let internal = registry.by_category(CommandCategory::Internal);

        assert!(!user_safe.is_empty());
        assert!(!advanced.is_empty());
        assert!(!internal.is_empty());
    }
}
