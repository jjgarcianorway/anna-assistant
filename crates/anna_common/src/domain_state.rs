//! Domain State v7.39.0 - Incremental Refresh Architecture
//!
//! Manages knowledge domains with diff-based refresh:
//! - hw.static, hw.dynamic (hardware)
//! - sw.packages, sw.commands, sw.services (software)
//! - net.interfaces (networking)
//! - peripherals.usb, peripherals.thunderbolt, peripherals.bluetooth
//! - storage.devices, storage.filesystems
//! - docs.local (man-db, arch-wiki-docs)
//!
//! Each domain tracks:
//! - last_refresh_at, refresh_duration_ms, result, counts
//! - fingerprint for change detection
//! - next_suggested_refresh_at

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::atomic_write;
use crate::daemon_state::INTERNAL_DIR;

/// Schema version for domain state - bump on breaking changes
pub const DOMAIN_STATE_SCHEMA_VERSION: u32 = 1;

/// Directory for domain state files
pub const DOMAIN_STATE_DIR: &str = "/var/lib/anna/internal/domain_state";

/// Directory for on-demand requests
pub const REQUESTS_DIR: &str = "/var/lib/anna/internal/requests";

/// Directory for on-demand responses
pub const RESPONSES_DIR: &str = "/var/lib/anna/internal/responses";

/// All supported domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    // Hardware
    HwStatic,      // CPU model, RAM size, motherboard, GPU model (rarely changes)
    HwDynamic,     // Temps, throttling, link state (changes frequently)

    // Software
    SwPackages,    // Installed packages
    SwCommands,    // Commands on PATH
    SwServices,    // Systemd services
    SwConfigCoverage, // Config file coverage

    // Network
    NetInterfaces, // Network interfaces

    // Peripherals
    PeripheralsUsb,         // USB devices
    PeripheralsThunderbolt, // Thunderbolt devices
    PeripheralsBluetooth,   // Bluetooth devices

    // Storage
    StorageDevices,     // Block devices
    StorageFilesystems, // Filesystems

    // Documentation
    DocsLocal, // man-db, arch-wiki-docs presence
}

impl Domain {
    /// Get all domains
    pub fn all() -> &'static [Domain] {
        &[
            Domain::HwStatic,
            Domain::HwDynamic,
            Domain::SwPackages,
            Domain::SwCommands,
            Domain::SwServices,
            Domain::SwConfigCoverage,
            Domain::NetInterfaces,
            Domain::PeripheralsUsb,
            Domain::PeripheralsThunderbolt,
            Domain::PeripheralsBluetooth,
            Domain::StorageDevices,
            Domain::StorageFilesystems,
            Domain::DocsLocal,
        ]
    }

    /// Get domain name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Domain::HwStatic => "hw.static",
            Domain::HwDynamic => "hw.dynamic",
            Domain::SwPackages => "sw.packages",
            Domain::SwCommands => "sw.commands",
            Domain::SwServices => "sw.services",
            Domain::SwConfigCoverage => "sw.config_coverage",
            Domain::NetInterfaces => "net.interfaces",
            Domain::PeripheralsUsb => "peripherals.usb",
            Domain::PeripheralsThunderbolt => "peripherals.thunderbolt",
            Domain::PeripheralsBluetooth => "peripherals.bluetooth",
            Domain::StorageDevices => "storage.devices",
            Domain::StorageFilesystems => "storage.filesystems",
            Domain::DocsLocal => "docs.local",
        }
    }

    /// Parse domain from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "hw.static" => Some(Domain::HwStatic),
            "hw.dynamic" => Some(Domain::HwDynamic),
            "sw.packages" => Some(Domain::SwPackages),
            "sw.commands" => Some(Domain::SwCommands),
            "sw.services" => Some(Domain::SwServices),
            "sw.config_coverage" => Some(Domain::SwConfigCoverage),
            "net.interfaces" => Some(Domain::NetInterfaces),
            "peripherals.usb" => Some(Domain::PeripheralsUsb),
            "peripherals.thunderbolt" => Some(Domain::PeripheralsThunderbolt),
            "peripherals.bluetooth" => Some(Domain::PeripheralsBluetooth),
            "storage.devices" => Some(Domain::StorageDevices),
            "storage.filesystems" => Some(Domain::StorageFilesystems),
            "docs.local" => Some(Domain::DocsLocal),
            _ => None,
        }
    }

    /// Get default refresh interval in seconds
    pub fn default_refresh_interval_secs(&self) -> u64 {
        match self {
            // Static hardware - only on boot or hotplug
            Domain::HwStatic => 86400, // 24h (but triggered by boot_id change)

            // Dynamic hardware - moderate frequency
            Domain::HwDynamic => 60, // 1 minute

            // Packages - incremental via pacman.log
            Domain::SwPackages => 3600, // 1h full reconciliation

            // Commands - only when PATH changes
            Domain::SwCommands => 3600, // 1h

            // Services - moderate frequency
            Domain::SwServices => 300, // 5 minutes

            // Config coverage - infrequent
            Domain::SwConfigCoverage => 1800, // 30 minutes

            // Network - event-driven + periodic
            Domain::NetInterfaces => 120, // 2 minutes

            // Peripherals - event-driven + periodic
            Domain::PeripheralsUsb => 120,
            Domain::PeripheralsThunderbolt => 300,
            Domain::PeripheralsBluetooth => 120,

            // Storage - moderate frequency
            Domain::StorageDevices => 300, // 5 minutes
            Domain::StorageFilesystems => 60, // 1 minute (for usage)

            // Docs - infrequent
            Domain::DocsLocal => 3600, // 1h
        }
    }

    /// Is this domain required for `annactl status`?
    pub fn required_for_status(&self) -> bool {
        matches!(
            self,
            Domain::HwStatic
                | Domain::HwDynamic
                | Domain::SwPackages
                | Domain::SwServices
                | Domain::StorageFilesystems
        )
    }

    /// Is this domain required for `annactl hw`?
    pub fn required_for_hw(&self) -> bool {
        matches!(
            self,
            Domain::HwStatic
                | Domain::HwDynamic
                | Domain::NetInterfaces
                | Domain::PeripheralsUsb
                | Domain::PeripheralsThunderbolt
                | Domain::PeripheralsBluetooth
                | Domain::StorageDevices
                | Domain::StorageFilesystems
        )
    }

    /// Is this domain required for `annactl sw`?
    pub fn required_for_sw(&self) -> bool {
        matches!(
            self,
            Domain::SwPackages
                | Domain::SwCommands
                | Domain::SwServices
                | Domain::SwConfigCoverage
                | Domain::DocsLocal
        )
    }
}

/// Result of a domain refresh operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefreshResult {
    Ok,
    Failed,
    Skipped, // No change detected, refresh not needed
    Timeout,
}

/// State of a single domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRefreshState {
    /// Domain identifier
    pub domain: Domain,
    /// Schema version
    pub schema_version: u32,
    /// Last refresh timestamp
    pub last_refresh_at: Option<DateTime<Utc>>,
    /// Duration of last refresh in milliseconds
    pub refresh_duration_ms: u64,
    /// Result of last refresh
    pub result: RefreshResult,
    /// Fingerprint for change detection
    pub fingerprint: String,
    /// Count of entities in this domain
    pub entity_count: usize,
    /// Counts from last refresh
    pub added: usize,
    pub changed: usize,
    pub removed: usize,
    /// Next suggested refresh time
    pub next_suggested_refresh_at: Option<DateTime<Utc>>,
    /// Boot ID when last refreshed (for hw.static)
    pub boot_id: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl DomainRefreshState {
    /// Create a new domain state
    pub fn new(domain: Domain) -> Self {
        Self {
            domain,
            schema_version: DOMAIN_STATE_SCHEMA_VERSION,
            last_refresh_at: None,
            refresh_duration_ms: 0,
            result: RefreshResult::Skipped,
            fingerprint: String::new(),
            entity_count: 0,
            added: 0,
            changed: 0,
            removed: 0,
            next_suggested_refresh_at: None,
            boot_id: None,
            error_message: None,
        }
    }

    /// Get file path for this domain state
    pub fn file_path(&self) -> PathBuf {
        PathBuf::from(DOMAIN_STATE_DIR).join(format!("{}.json", self.domain.as_str().replace('.', "_")))
    }

    /// Load domain state from disk
    pub fn load(domain: Domain) -> Self {
        let state = Self::new(domain);
        let path = state.file_path();

        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(loaded) = serde_json::from_str::<Self>(&content) {
                // Check schema version - migrate or rebuild if needed
                if loaded.schema_version == DOMAIN_STATE_SCHEMA_VERSION {
                    return loaded;
                }
                // Schema mismatch - return fresh state (will trigger full refresh)
            }
        }

        state
    }

    /// Save domain state to disk
    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(DOMAIN_STATE_DIR)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&self.file_path().to_string_lossy(), &content)
    }

    /// Check if refresh is needed based on time
    pub fn needs_refresh(&self) -> bool {
        let Some(last) = self.last_refresh_at else {
            return true; // Never refreshed
        };

        let interval = self.domain.default_refresh_interval_secs();
        let elapsed = (Utc::now() - last).num_seconds().max(0) as u64;
        elapsed >= interval
    }

    /// Check if data is stale (beyond 2x refresh interval)
    pub fn is_stale(&self) -> bool {
        let Some(last) = self.last_refresh_at else {
            return true;
        };

        let interval = self.domain.default_refresh_interval_secs();
        let elapsed = (Utc::now() - last).num_seconds().max(0) as u64;
        elapsed >= interval * 2
    }

    /// Record a successful refresh
    pub fn record_refresh(
        &mut self,
        duration_ms: u64,
        fingerprint: String,
        entity_count: usize,
        added: usize,
        changed: usize,
        removed: usize,
    ) {
        self.last_refresh_at = Some(Utc::now());
        self.refresh_duration_ms = duration_ms;
        self.result = RefreshResult::Ok;
        self.fingerprint = fingerprint;
        self.entity_count = entity_count;
        self.added = added;
        self.changed = changed;
        self.removed = removed;
        self.error_message = None;

        // Schedule next refresh
        let interval_secs = self.domain.default_refresh_interval_secs();
        self.next_suggested_refresh_at = Some(
            Utc::now() + chrono::Duration::seconds(interval_secs as i64)
        );
    }

    /// Record a skipped refresh (no changes)
    pub fn record_skip(&mut self) {
        self.result = RefreshResult::Skipped;

        // Still schedule next check
        let interval_secs = self.domain.default_refresh_interval_secs();
        self.next_suggested_refresh_at = Some(
            Utc::now() + chrono::Duration::seconds(interval_secs as i64)
        );
    }

    /// Record a failed refresh
    pub fn record_failure(&mut self, error: &str) {
        self.result = RefreshResult::Failed;
        self.error_message = Some(error.to_string());

        // Retry sooner on failure
        self.next_suggested_refresh_at = Some(
            Utc::now() + chrono::Duration::seconds(60)
        );
    }

    /// Format age for display
    pub fn format_age(&self) -> String {
        let Some(last) = self.last_refresh_at else {
            return "never".to_string();
        };

        let secs = (Utc::now() - last).num_seconds().max(0) as u64;
        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    }
}

/// On-demand refresh request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshRequest {
    /// Unique request ID
    pub id: String,
    /// Unix UID of requester
    pub requested_by: u32,
    /// Command that triggered this (status, sw, hw)
    pub command: String,
    /// Target (e.g., "sw:steam", "hw:cpu", "hw:wlp2s0")
    pub target: Option<String>,
    /// Required domains for this request
    pub required_domains: Vec<Domain>,
    /// Deadline in milliseconds from creation
    pub deadline_ms: u64,
    /// When request was created
    pub created_at: DateTime<Utc>,
}

impl RefreshRequest {
    /// Create a new request
    pub fn new(command: &str, target: Option<&str>, required_domains: Vec<Domain>, deadline_ms: u64) -> Self {
        // Get UID from /proc/self/status or fallback to PID
        let requested_by = std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("Uid:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|u| u.parse::<u32>().ok())
            })
            .unwrap_or(std::process::id());

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            requested_by,
            command: command.to_string(),
            target: target.map(|s| s.to_string()),
            required_domains,
            deadline_ms,
            created_at: Utc::now(),
        }
    }

    /// Get file path for this request
    pub fn file_path(&self) -> PathBuf {
        PathBuf::from(REQUESTS_DIR).join(format!("{}.json", self.id))
    }

    /// Save request to disk
    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(REQUESTS_DIR)?;
        let content = serde_json::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&self.file_path().to_string_lossy(), &content)
    }

    /// Delete request file
    pub fn delete(&self) {
        let _ = std::fs::remove_file(self.file_path());
    }

    /// Check if request has timed out
    pub fn is_expired(&self) -> bool {
        let elapsed_ms = (Utc::now() - self.created_at).num_milliseconds().max(0) as u64;
        elapsed_ms > self.deadline_ms
    }
}

/// On-demand refresh response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResponse {
    /// Request ID this responds to
    pub request_id: String,
    /// Whether data was served from cache
    pub cache_hit: bool,
    /// Whether a refresh was performed
    pub refresh_performed: bool,
    /// Domains that were refreshed
    pub refreshed_domains: Vec<Domain>,
    /// Domains that are still stale/missing
    pub stale_domains: Vec<Domain>,
    /// Time to process in milliseconds
    pub process_time_ms: u64,
    /// When response was created
    pub created_at: DateTime<Utc>,
    /// Error message if failed
    pub error: Option<String>,
}

impl RefreshResponse {
    /// Get file path for this response
    pub fn file_path(request_id: &str) -> PathBuf {
        PathBuf::from(RESPONSES_DIR).join(format!("{}.json", request_id))
    }

    /// Load response from disk
    pub fn load(request_id: &str) -> Option<Self> {
        let path = Self::file_path(request_id);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
    }

    /// Save response to disk
    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(RESPONSES_DIR)?;
        let content = serde_json::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&Self::file_path(&self.request_id).to_string_lossy(), &content)
    }

    /// Delete response file
    pub fn delete(&self) {
        let _ = std::fs::remove_file(Self::file_path(&self.request_id));
    }
}

/// Summary of all domain states (for status display)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainSummary {
    /// Timestamp of this summary
    pub generated_at: DateTime<Utc>,
    /// Total entity count across all domains
    pub total_entities: usize,
    /// Count of domains that are fresh
    pub fresh_domains: usize,
    /// Count of domains that are stale
    pub stale_domains: usize,
    /// Count of domains that have never been refreshed
    pub missing_domains: usize,
    /// Any domains currently being refreshed
    pub refreshing_domains: Vec<String>,
    /// Oldest domain age in seconds
    pub oldest_refresh_secs: u64,
    /// Combined refresh duration in ms (last cycle)
    pub last_cycle_duration_ms: u64,
}

impl DomainSummary {
    /// Build summary from all domain states (loads from disk)
    pub fn build() -> Self {
        let states: Vec<DomainRefreshState> = Domain::all()
            .iter()
            .map(|d| DomainRefreshState::load(*d))
            .collect();
        Self::from_states(&states)
    }

    /// Build summary from pre-loaded domain states
    pub fn from_states(states: &[DomainRefreshState]) -> Self {
        let mut summary = Self {
            generated_at: Utc::now(),
            ..Default::default()
        };

        let mut oldest_secs: u64 = 0;

        for state in states {
            summary.total_entities += state.entity_count;

            if state.last_refresh_at.is_none() {
                summary.missing_domains += 1;
            } else if state.is_stale() {
                summary.stale_domains += 1;
            } else {
                summary.fresh_domains += 1;
            }

            if let Some(last) = state.last_refresh_at {
                let age = (Utc::now() - last).num_seconds().max(0) as u64;
                if age > oldest_secs {
                    oldest_secs = age;
                }
            }

            summary.last_cycle_duration_ms += state.refresh_duration_ms;
        }

        summary.oldest_refresh_secs = oldest_secs;
        summary
    }

    /// File path for summary
    pub fn file_path() -> PathBuf {
        PathBuf::from(INTERNAL_DIR).join("domain_summary.json")
    }

    /// Load summary from disk
    pub fn load() -> Option<Self> {
        let path = Self::file_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
    }

    /// Save summary to disk
    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(INTERNAL_DIR)?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&Self::file_path().to_string_lossy(), &content)
    }
}

/// Clean up old request/response files (older than 5 minutes)
pub fn cleanup_old_requests() {
    let cutoff = Utc::now() - chrono::Duration::minutes(5);

    for dir in &[REQUESTS_DIR, RESPONSES_DIR] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let modified_utc: DateTime<Utc> = modified.into();
                        if modified_utc < cutoff {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_str_roundtrip() {
        for domain in Domain::all() {
            let s = domain.as_str();
            let parsed = Domain::from_str(s);
            assert_eq!(parsed, Some(*domain), "Failed roundtrip for {:?}", domain);
        }
    }

    #[test]
    fn test_domain_refresh_intervals() {
        // Static hardware should have long intervals
        assert!(Domain::HwStatic.default_refresh_interval_secs() >= 3600);

        // Dynamic hardware should be more frequent
        assert!(Domain::HwDynamic.default_refresh_interval_secs() <= 120);

        // Filesystems need frequent updates for usage
        assert!(Domain::StorageFilesystems.default_refresh_interval_secs() <= 120);
    }

    #[test]
    fn test_request_expiry() {
        let req = RefreshRequest::new("status", None, vec![Domain::HwStatic], 100);
        assert!(!req.is_expired()); // Just created

        // After deadline, should be expired
        std::thread::sleep(std::time::Duration::from_millis(150));
        assert!(req.is_expired());
    }
}
