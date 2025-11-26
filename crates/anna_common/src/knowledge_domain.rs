//! Knowledge Domain - Types and categorization for Anna's knowledge
//!
//! v6.55.1: Knowledge Backup, Export, Introspection and Pruning
//!
//! This module defines how Anna's knowledge is categorized and structured:
//! - KnowledgeDomain: What type of information (Telemetry, Paths, User Profile, etc.)
//! - KnowledgeCategory: High-level groupings
//! - Knowledge statistics and introspection types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Knowledge domain - specific types of information Anna stores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeDomain {
    /// System telemetry (CPU, memory, disk usage over time)
    Telemetry,

    /// Dynamic paths (learned file locations, project directories)
    DynamicPaths,

    /// File system watches (monitored directories, config files)
    Watches,

    /// Undo/rollback history (action episodes, change journal)
    UndoHistory,

    /// User profile (preferences, personality traits)
    UserProfile,

    /// Machine identity (hardware fingerprint, hostnames)
    MachineIdentity,

    /// Service health history (systemd units, restarts)
    ServiceHealth,

    /// Network telemetry (latency, DNS, packet loss)
    NetworkTelemetry,

    /// Boot sessions (boot times, shutdown times)
    BootSessions,

    /// Log signatures (error patterns, warnings)
    LogSignatures,

    /// LLM usage statistics (model calls, latency)
    LlmUsage,

    /// Baselines and deltas (performance baselines)
    Baselines,

    /// Usage patterns (active hours, heavy load times)
    UsagePatterns,

    /// Issue tracking (known issues, repair attempts)
    IssueTracking,

    /// Learning patterns (detected behaviors)
    LearningPatterns,

    /// Timeline events (upgrades, config changes)
    TimelineEvents,
}

impl KnowledgeDomain {
    /// Get all knowledge domains
    pub fn all() -> Vec<KnowledgeDomain> {
        vec![
            KnowledgeDomain::Telemetry,
            KnowledgeDomain::DynamicPaths,
            KnowledgeDomain::Watches,
            KnowledgeDomain::UndoHistory,
            KnowledgeDomain::UserProfile,
            KnowledgeDomain::MachineIdentity,
            KnowledgeDomain::ServiceHealth,
            KnowledgeDomain::NetworkTelemetry,
            KnowledgeDomain::BootSessions,
            KnowledgeDomain::LogSignatures,
            KnowledgeDomain::LlmUsage,
            KnowledgeDomain::Baselines,
            KnowledgeDomain::UsagePatterns,
            KnowledgeDomain::IssueTracking,
            KnowledgeDomain::LearningPatterns,
            KnowledgeDomain::TimelineEvents,
        ]
    }

    /// Get display name for the domain
    pub fn display_name(&self) -> &'static str {
        match self {
            KnowledgeDomain::Telemetry => "System Telemetry",
            KnowledgeDomain::DynamicPaths => "Dynamic Paths",
            KnowledgeDomain::Watches => "File Watches",
            KnowledgeDomain::UndoHistory => "Undo History",
            KnowledgeDomain::UserProfile => "User Profile",
            KnowledgeDomain::MachineIdentity => "Machine Identity",
            KnowledgeDomain::ServiceHealth => "Service Health",
            KnowledgeDomain::NetworkTelemetry => "Network Telemetry",
            KnowledgeDomain::BootSessions => "Boot Sessions",
            KnowledgeDomain::LogSignatures => "Log Signatures",
            KnowledgeDomain::LlmUsage => "LLM Usage",
            KnowledgeDomain::Baselines => "Baselines",
            KnowledgeDomain::UsagePatterns => "Usage Patterns",
            KnowledgeDomain::IssueTracking => "Issue Tracking",
            KnowledgeDomain::LearningPatterns => "Learning Patterns",
            KnowledgeDomain::TimelineEvents => "Timeline Events",
        }
    }

    /// Get emoji for the domain
    pub fn emoji(&self) -> &'static str {
        match self {
            KnowledgeDomain::Telemetry => "📊",
            KnowledgeDomain::DynamicPaths => "📁",
            KnowledgeDomain::Watches => "👁️",
            KnowledgeDomain::UndoHistory => "↩️",
            KnowledgeDomain::UserProfile => "👤",
            KnowledgeDomain::MachineIdentity => "🖥️",
            KnowledgeDomain::ServiceHealth => "⚙️",
            KnowledgeDomain::NetworkTelemetry => "🌐",
            KnowledgeDomain::BootSessions => "🔄",
            KnowledgeDomain::LogSignatures => "📋",
            KnowledgeDomain::LlmUsage => "🧠",
            KnowledgeDomain::Baselines => "📈",
            KnowledgeDomain::UsagePatterns => "📅",
            KnowledgeDomain::IssueTracking => "🐛",
            KnowledgeDomain::LearningPatterns => "🎓",
            KnowledgeDomain::TimelineEvents => "⏰",
        }
    }

    /// Get database table(s) for this domain
    pub fn tables(&self) -> Vec<&'static str> {
        match self {
            KnowledgeDomain::Telemetry => vec![
                "cpu_windows",
                "cpu_top_processes",
                "mem_windows",
                "oom_events",
                "fs_capacity_daily",
                "fs_growth",
                "fs_io_windows",
            ],
            KnowledgeDomain::DynamicPaths => vec!["file_index"],
            KnowledgeDomain::Watches => vec!["file_changes"],
            KnowledgeDomain::UndoHistory => vec!["action_history", "repair_history"],
            KnowledgeDomain::UserProfile => vec!["user_preferences", "personality"],
            KnowledgeDomain::MachineIdentity => vec!["system_state_log"],
            KnowledgeDomain::ServiceHealth => vec!["service_health", "service_restarts"],
            KnowledgeDomain::NetworkTelemetry => vec!["net_windows", "net_events"],
            KnowledgeDomain::BootSessions => vec!["boot_sessions", "boot_unit_slowlog"],
            KnowledgeDomain::LogSignatures => vec!["log_window_counts", "log_signatures"],
            KnowledgeDomain::LlmUsage => vec!["llm_usage_windows", "llm_model_changes"],
            KnowledgeDomain::Baselines => vec!["baselines", "baseline_deltas"],
            KnowledgeDomain::UsagePatterns => vec!["usage_patterns", "app_usage"],
            KnowledgeDomain::IssueTracking => vec!["issue_tracking", "issue_decisions"],
            KnowledgeDomain::LearningPatterns => vec!["learning_patterns"],
            KnowledgeDomain::TimelineEvents => vec!["timeline_events"],
        }
    }

    /// Get category for this domain
    pub fn category(&self) -> KnowledgeCategory {
        match self {
            KnowledgeDomain::Telemetry
            | KnowledgeDomain::NetworkTelemetry
            | KnowledgeDomain::BootSessions
            | KnowledgeDomain::ServiceHealth => KnowledgeCategory::SystemMetrics,

            KnowledgeDomain::DynamicPaths
            | KnowledgeDomain::Watches
            | KnowledgeDomain::LogSignatures => KnowledgeCategory::FileSystem,

            KnowledgeDomain::UndoHistory
            | KnowledgeDomain::IssueTracking => KnowledgeCategory::ActionHistory,

            KnowledgeDomain::UserProfile
            | KnowledgeDomain::UsagePatterns
            | KnowledgeDomain::LearningPatterns => KnowledgeCategory::UserBehavior,

            KnowledgeDomain::MachineIdentity => KnowledgeCategory::Identity,

            KnowledgeDomain::LlmUsage
            | KnowledgeDomain::Baselines
            | KnowledgeDomain::TimelineEvents => KnowledgeCategory::AnnaInternal,
        }
    }

    /// Check if this domain contains sensitive data
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            KnowledgeDomain::UserProfile
                | KnowledgeDomain::DynamicPaths
                | KnowledgeDomain::UsagePatterns
        )
    }

    /// Check if this domain supports time-based pruning
    pub fn supports_time_pruning(&self) -> bool {
        !matches!(
            self,
            KnowledgeDomain::UserProfile
                | KnowledgeDomain::MachineIdentity
                | KnowledgeDomain::LearningPatterns
        )
    }
}

/// High-level knowledge categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeCategory {
    /// System metrics and telemetry
    SystemMetrics,

    /// File system tracking
    FileSystem,

    /// Action and repair history
    ActionHistory,

    /// User behavior and preferences
    UserBehavior,

    /// Machine and user identity
    Identity,

    /// Anna's internal tracking
    AnnaInternal,
}

impl KnowledgeCategory {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            KnowledgeCategory::SystemMetrics => "System Metrics",
            KnowledgeCategory::FileSystem => "File System",
            KnowledgeCategory::ActionHistory => "Action History",
            KnowledgeCategory::UserBehavior => "User Behavior",
            KnowledgeCategory::Identity => "Identity",
            KnowledgeCategory::AnnaInternal => "Anna Internal",
        }
    }

    /// Get emoji for category
    pub fn emoji(&self) -> &'static str {
        match self {
            KnowledgeCategory::SystemMetrics => "📊",
            KnowledgeCategory::FileSystem => "📂",
            KnowledgeCategory::ActionHistory => "📜",
            KnowledgeCategory::UserBehavior => "👤",
            KnowledgeCategory::Identity => "🪪",
            KnowledgeCategory::AnnaInternal => "🤖",
        }
    }

    /// Get all domains in this category
    pub fn domains(&self) -> Vec<KnowledgeDomain> {
        KnowledgeDomain::all()
            .into_iter()
            .filter(|d| d.category() == *self)
            .collect()
    }
}

/// Statistics for a single knowledge domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    /// Domain type
    pub domain: KnowledgeDomain,

    /// Total record count
    pub record_count: u64,

    /// Size in bytes (approximate)
    pub size_bytes: u64,

    /// Oldest record timestamp
    pub oldest_record: Option<DateTime<Utc>>,

    /// Newest record timestamp
    pub newest_record: Option<DateTime<Utc>>,

    /// Days of history
    pub days_of_history: u32,
}

impl DomainStats {
    /// Create empty stats for a domain
    pub fn empty(domain: KnowledgeDomain) -> Self {
        Self {
            domain,
            record_count: 0,
            size_bytes: 0,
            oldest_record: None,
            newest_record: None,
            days_of_history: 0,
        }
    }

    /// Format size for display
    pub fn formatted_size(&self) -> String {
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else if self.size_bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!(
                "{:.1} GB",
                self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            )
        }
    }
}

/// Overall knowledge summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSummary {
    /// Stats per domain
    pub domains: HashMap<KnowledgeDomain, DomainStats>,

    /// Total size in bytes
    pub total_size_bytes: u64,

    /// Total record count
    pub total_records: u64,

    /// Snapshot timestamp
    pub snapshot_at: DateTime<Utc>,

    /// Database location (system or user)
    pub db_location: String,
}

impl KnowledgeSummary {
    /// Create empty summary
    pub fn empty(db_location: String) -> Self {
        Self {
            domains: HashMap::new(),
            total_size_bytes: 0,
            total_records: 0,
            snapshot_at: Utc::now(),
            db_location,
        }
    }

    /// Get formatted total size
    pub fn formatted_total_size(&self) -> String {
        if self.total_size_bytes < 1024 {
            format!("{} B", self.total_size_bytes)
        } else if self.total_size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.total_size_bytes as f64 / 1024.0)
        } else if self.total_size_bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.total_size_bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!(
                "{:.1} GB",
                self.total_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            )
        }
    }

    /// Get stats grouped by category
    pub fn by_category(&self) -> HashMap<KnowledgeCategory, Vec<&DomainStats>> {
        let mut result: HashMap<KnowledgeCategory, Vec<&DomainStats>> = HashMap::new();

        for stats in self.domains.values() {
            let category = stats.domain.category();
            result.entry(category).or_default().push(stats);
        }

        result
    }
}

/// Pruning criteria for knowledge cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningCriteria {
    /// Domains to prune (None = all pruneable domains)
    pub domains: Option<Vec<KnowledgeDomain>>,

    /// Records older than this many days will be removed
    pub older_than_days: u32,

    /// Dry run mode - report what would be deleted without deleting
    pub dry_run: bool,
}

impl Default for PruningCriteria {
    fn default() -> Self {
        Self {
            domains: None,
            older_than_days: 90,
            dry_run: true,
        }
    }
}

/// Result of a pruning operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningResult {
    /// Records deleted per domain
    pub deleted_per_domain: HashMap<KnowledgeDomain, u64>,

    /// Total records deleted
    pub total_deleted: u64,

    /// Space reclaimed in bytes
    pub space_reclaimed_bytes: u64,

    /// Was this a dry run?
    pub was_dry_run: bool,

    /// Errors encountered
    pub errors: Vec<String>,
}

impl PruningResult {
    /// Create empty result
    pub fn empty(dry_run: bool) -> Self {
        Self {
            deleted_per_domain: HashMap::new(),
            total_deleted: 0,
            space_reclaimed_bytes: 0,
            was_dry_run: dry_run,
            errors: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_domains() {
        let domains = KnowledgeDomain::all();
        assert_eq!(domains.len(), 16);
    }

    #[test]
    fn test_domain_tables() {
        let telemetry = KnowledgeDomain::Telemetry;
        let tables = telemetry.tables();
        assert!(!tables.is_empty());
        assert!(tables.contains(&"cpu_windows"));
    }

    #[test]
    fn test_domain_category() {
        assert_eq!(
            KnowledgeDomain::Telemetry.category(),
            KnowledgeCategory::SystemMetrics
        );
        assert_eq!(
            KnowledgeDomain::UserProfile.category(),
            KnowledgeCategory::UserBehavior
        );
    }

    #[test]
    fn test_sensitive_domains() {
        assert!(KnowledgeDomain::UserProfile.is_sensitive());
        assert!(!KnowledgeDomain::Telemetry.is_sensitive());
    }

    #[test]
    fn test_supports_time_pruning() {
        assert!(KnowledgeDomain::Telemetry.supports_time_pruning());
        assert!(!KnowledgeDomain::UserProfile.supports_time_pruning());
    }

    #[test]
    fn test_domain_stats_format() {
        let stats = DomainStats {
            domain: KnowledgeDomain::Telemetry,
            record_count: 1000,
            size_bytes: 1024 * 1024,
            oldest_record: None,
            newest_record: None,
            days_of_history: 30,
        };
        assert_eq!(stats.formatted_size(), "1.0 MB");
    }

    #[test]
    fn test_category_domains() {
        let metrics = KnowledgeCategory::SystemMetrics;
        let domains = metrics.domains();
        assert!(domains.contains(&KnowledgeDomain::Telemetry));
        assert!(!domains.contains(&KnowledgeDomain::UserProfile));
    }

    #[test]
    fn test_summary_by_category() {
        let mut summary = KnowledgeSummary::empty("test".to_string());
        summary.domains.insert(
            KnowledgeDomain::Telemetry,
            DomainStats::empty(KnowledgeDomain::Telemetry),
        );
        summary.domains.insert(
            KnowledgeDomain::UserProfile,
            DomainStats::empty(KnowledgeDomain::UserProfile),
        );

        let by_cat = summary.by_category();
        assert!(by_cat.contains_key(&KnowledgeCategory::SystemMetrics));
        assert!(by_cat.contains_key(&KnowledgeCategory::UserBehavior));
    }
}
