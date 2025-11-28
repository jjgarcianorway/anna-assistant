//! Stats & Telemetry Engine - v0.24.0
//!
//! Three-tier telemetry: System, User, Anna-internal.
//! All metrics are probe-derived or calculated from observed data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Stats Categories
// ============================================================================

/// Stats category for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StatsCategory {
    /// System-level stats (hardware, OS)
    System,
    /// User-level stats (session, activity)
    User,
    /// Anna internal stats (performance, usage)
    AnnaInternal,
}

// ============================================================================
// System Stats
// ============================================================================

/// System-level statistics (probe-derived)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemStats {
    /// CPU statistics
    pub cpu: CpuStats,
    /// Memory statistics
    pub memory: MemoryStats,
    /// Disk statistics
    pub disk: DiskStats,
    /// Network statistics
    pub network: NetworkStats,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Load averages (1, 5, 15 min)
    pub load_average: (f32, f32, f32),
    /// Number of running processes
    pub process_count: u32,
    /// Kernel version
    pub kernel_version: Option<String>,
    /// When these stats were captured
    pub captured_at: i64,
}

/// CPU statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuStats {
    /// Overall CPU usage percentage (0-100)
    pub usage_percent: f32,
    /// Per-core usage percentages
    pub per_core_usage: Vec<f32>,
    /// CPU frequency in MHz
    pub frequency_mhz: Option<u32>,
    /// Number of physical cores
    pub physical_cores: u32,
    /// Number of logical cores
    pub logical_cores: u32,
    /// CPU temperature in Celsius (if available)
    pub temperature_celsius: Option<f32>,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    /// Total RAM in bytes
    pub total_bytes: u64,
    /// Used RAM in bytes
    pub used_bytes: u64,
    /// Available RAM in bytes
    pub available_bytes: u64,
    /// Cached memory in bytes
    pub cached_bytes: u64,
    /// Swap total in bytes
    pub swap_total_bytes: u64,
    /// Swap used in bytes
    pub swap_used_bytes: u64,
    /// Memory pressure indicator (0.0-1.0)
    pub pressure: f32,
}

impl MemoryStats {
    /// Calculate usage percentage
    pub fn usage_percent(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.used_bytes as f32 / self.total_bytes as f32) * 100.0
    }
}

/// Disk statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiskStats {
    /// Per-mount-point stats
    pub mounts: Vec<MountStats>,
    /// Total read bytes since boot
    pub total_read_bytes: u64,
    /// Total write bytes since boot
    pub total_write_bytes: u64,
    /// Current I/O utilization (0.0-1.0)
    pub io_utilization: f32,
}

/// Per-mount statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountStats {
    /// Mount point path
    pub mount_point: String,
    /// Filesystem type
    pub fs_type: String,
    /// Total space in bytes
    pub total_bytes: u64,
    /// Used space in bytes
    pub used_bytes: u64,
    /// Available space in bytes
    pub available_bytes: u64,
    /// Device path
    pub device: String,
}

impl MountStats {
    /// Calculate usage percentage
    pub fn usage_percent(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.used_bytes as f32 / self.total_bytes as f32) * 100.0
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkStats {
    /// Per-interface stats
    pub interfaces: Vec<InterfaceStats>,
    /// Total bytes received
    pub total_rx_bytes: u64,
    /// Total bytes transmitted
    pub total_tx_bytes: u64,
    /// Active connections count
    pub active_connections: u32,
}

/// Per-interface statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceStats {
    /// Interface name
    pub name: String,
    /// Is interface up?
    pub is_up: bool,
    /// Bytes received
    pub rx_bytes: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Packets received
    pub rx_packets: u64,
    /// Packets transmitted
    pub tx_packets: u64,
    /// IPv4 addresses
    pub ipv4_addrs: Vec<String>,
    /// IPv6 addresses
    pub ipv6_addrs: Vec<String>,
}

// ============================================================================
// User Stats
// ============================================================================

/// User-level statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserStats {
    /// Session duration in seconds
    pub session_duration_seconds: u64,
    /// Number of commands/questions asked
    pub questions_asked: u64,
    /// Number of successful answers
    pub answers_received: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,
    /// User preferences learned
    pub preferences_learned: u32,
    /// Facts stored for this user
    pub facts_stored: u32,
    /// Last activity timestamp
    pub last_activity_at: i64,
    /// Session start timestamp
    pub session_started_at: i64,
}

impl UserStats {
    /// Calculate answer success rate
    pub fn success_rate(&self) -> f32 {
        if self.questions_asked == 0 {
            return 0.0;
        }
        (self.answers_received as f32 / self.questions_asked as f32) * 100.0
    }
}

// ============================================================================
// Anna Internal Stats
// ============================================================================

/// Anna's internal performance statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnnaStats {
    /// LLM statistics
    pub llm: LlmStats,
    /// Probe execution statistics
    pub probes: ProbeStats,
    /// Cache statistics
    pub cache: CacheStats,
    /// Knowledge store statistics
    pub knowledge: KnowledgeStats,
    /// Answer pipeline statistics
    pub pipeline: PipelineStats,
    /// Daemon statistics
    pub daemon: DaemonStats,
}

/// LLM statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmStats {
    /// Total LLM calls made
    pub total_calls: u64,
    /// Junior (LLM-A) calls
    pub junior_calls: u64,
    /// Senior (LLM-B) calls
    pub senior_calls: u64,
    /// Total tokens generated
    pub total_tokens: u64,
    /// Average response time in ms
    pub avg_response_ms: u64,
    /// 95th percentile response time in ms
    pub p95_response_ms: u64,
    /// Number of timeouts
    pub timeouts: u64,
    /// Number of errors
    pub errors: u64,
    /// Current model name
    pub current_model: Option<String>,
}

/// Probe execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProbeStats {
    /// Total probes executed
    pub total_executed: u64,
    /// Probes that succeeded
    pub succeeded: u64,
    /// Probes that failed
    pub failed: u64,
    /// Probes that timed out
    pub timed_out: u64,
    /// Average execution time in ms
    pub avg_execution_ms: u64,
    /// Most frequently used probes
    pub top_probes: Vec<(String, u64)>,
}

impl ProbeStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        if self.total_executed == 0 {
            return 0.0;
        }
        (self.succeeded as f32 / self.total_executed as f32) * 100.0
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    /// Total cache lookups
    pub total_lookups: u64,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Current cache size in entries
    pub current_size: u32,
    /// Maximum cache size
    pub max_size: u32,
    /// Cache evictions
    pub evictions: u64,
    /// TTL expirations
    pub ttl_expirations: u64,
}

impl CacheStats {
    /// Calculate hit rate percentage
    pub fn hit_rate(&self) -> f32 {
        if self.total_lookups == 0 {
            return 0.0;
        }
        (self.hits as f32 / self.total_lookups as f32) * 100.0
    }
}

/// Knowledge store statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeStats {
    /// Total facts stored
    pub total_facts: u64,
    /// System-scope facts
    pub system_facts: u64,
    /// User-scope facts
    pub user_facts: u64,
    /// Facts with high confidence (>0.9)
    pub high_confidence_facts: u64,
    /// Facts that expired
    pub expired_facts: u64,
    /// Fact updates (overwrites)
    pub fact_updates: u64,
    /// Database size in bytes
    pub db_size_bytes: u64,
}

/// Pipeline statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PipelineStats {
    /// Total questions processed
    pub questions_processed: u64,
    /// Fast-path answers (cache/direct)
    pub fast_path_answers: u64,
    /// Questions requiring probes
    pub probed_answers: u64,
    /// Questions requiring LLM
    pub llm_answers: u64,
    /// Questions that couldn't be answered
    pub no_answer: u64,
    /// Average question complexity (1-10)
    pub avg_complexity: f32,
    /// Question type distribution
    pub type_distribution: HashMap<String, u64>,
}

impl PipelineStats {
    /// Calculate fast-path rate
    pub fn fast_path_rate(&self) -> f32 {
        if self.questions_processed == 0 {
            return 0.0;
        }
        (self.fast_path_answers as f32 / self.questions_processed as f32) * 100.0
    }
}

/// Daemon statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DaemonStats {
    /// Daemon uptime in seconds
    pub uptime_seconds: u64,
    /// Total requests handled
    pub requests_handled: u64,
    /// Current memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Thread count
    pub thread_count: u32,
    /// Restart count
    pub restart_count: u32,
    /// Last update check timestamp
    pub last_update_check: Option<i64>,
    /// Version string
    pub version: String,
}

// ============================================================================
// Aggregated Stats
// ============================================================================

/// Complete stats snapshot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsSnapshot {
    /// System statistics
    pub system: SystemStats,
    /// User statistics
    pub user: UserStats,
    /// Anna internal statistics
    pub anna: AnnaStats,
    /// When this snapshot was taken
    pub timestamp: i64,
    /// Snapshot duration (how long to collect)
    pub collection_duration_ms: u64,
}

/// Stats query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsQuery {
    /// Which categories to include
    pub categories: Vec<StatsCategory>,
    /// Time range (start, end) in Unix timestamps
    pub time_range: Option<(i64, i64)>,
    /// Aggregation granularity in seconds
    pub granularity_seconds: Option<u64>,
    /// Include historical data?
    pub include_history: bool,
}

impl Default for StatsQuery {
    fn default() -> Self {
        Self {
            categories: vec![
                StatsCategory::System,
                StatsCategory::User,
                StatsCategory::AnnaInternal,
            ],
            time_range: None,
            granularity_seconds: None,
            include_history: false,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_usage_percent() {
        let stats = MemoryStats {
            total_bytes: 16 * 1024 * 1024 * 1024, // 16GB
            used_bytes: 8 * 1024 * 1024 * 1024,   // 8GB
            ..Default::default()
        };
        assert!((stats.usage_percent() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_mount_usage_percent() {
        let stats = MountStats {
            mount_point: "/".to_string(),
            fs_type: "ext4".to_string(),
            total_bytes: 500 * 1024 * 1024 * 1024, // 500GB
            used_bytes: 250 * 1024 * 1024 * 1024,  // 250GB
            available_bytes: 250 * 1024 * 1024 * 1024,
            device: "/dev/sda1".to_string(),
        };
        assert!((stats.usage_percent() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_user_success_rate() {
        let stats = UserStats {
            questions_asked: 100,
            answers_received: 95,
            ..Default::default()
        };
        assert!((stats.success_rate() - 95.0).abs() < 0.01);
    }

    #[test]
    fn test_cache_hit_rate() {
        let stats = CacheStats {
            total_lookups: 1000,
            hits: 800,
            misses: 200,
            ..Default::default()
        };
        assert!((stats.hit_rate() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_probe_success_rate() {
        let stats = ProbeStats {
            total_executed: 50,
            succeeded: 48,
            failed: 1,
            timed_out: 1,
            ..Default::default()
        };
        assert!((stats.success_rate() - 96.0).abs() < 0.01);
    }

    #[test]
    fn test_pipeline_fast_path_rate() {
        let stats = PipelineStats {
            questions_processed: 100,
            fast_path_answers: 70,
            probed_answers: 20,
            llm_answers: 10,
            ..Default::default()
        };
        assert!((stats.fast_path_rate() - 70.0).abs() < 0.01);
    }

    #[test]
    fn test_stats_snapshot_serialize() {
        let snapshot = StatsSnapshot {
            system: SystemStats {
                uptime_seconds: 3600,
                ..Default::default()
            },
            user: UserStats {
                questions_asked: 10,
                ..Default::default()
            },
            anna: AnnaStats::default(),
            timestamp: 1700000000,
            collection_duration_ms: 100,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("3600"));

        let parsed: StatsSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.system.uptime_seconds, 3600);
    }

    #[test]
    fn test_stats_query_default() {
        let query = StatsQuery::default();
        assert_eq!(query.categories.len(), 3);
        assert!(!query.include_history);
    }
}
