//! Daemon Health Model - 6.20.0 / 6.22.0
//!
//! Tracks daemon startup health, initialization state, and crash loop detection.
//! Enables Safe Mode when critical failures are detected.
//!
//! 6.22.0: Extended with AnnaMode for comprehensive safe mode support

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};
use tracing::{info, warn};

const STATE_FILE: &str = "/var/lib/anna/daemon_health.json";
const CRASH_WINDOW_SECS: u64 = 120; // 2 minutes
const CRASH_THRESHOLD: usize = 5; // 5 crashes in window triggers Safe Mode

/// Daemon health state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DaemonHealthState {
    /// All systems operational
    Healthy,
    /// Non-critical issues detected, degraded functionality
    Degraded,
    /// Recoverable error, attempting auto-recovery
    BrokenRecoverable,
    /// Critical failure, minimal functionality only
    SafeMode,
}

impl DaemonHealthState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "HEALTHY",
            Self::Degraded => "DEGRADED",
            Self::BrokenRecoverable => "BROKEN (recoverable)",
            Self::SafeMode => "SAFE MODE",
        }
    }
}

/// Initialization state for background tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InitializationState {
    /// Background initialization in progress
    Initializing {
        started_at: SystemTime,
        progress: String,
    },
    /// Initialization complete and successful
    Ready {
        completed_at: SystemTime,
    },
    /// Initialization failed, will retry
    Failed {
        error: String,
        retry_at: SystemTime,
    },
}

impl Default for InitializationState {
    fn default() -> Self {
        Self::Initializing {
            started_at: SystemTime::now(),
            progress: "Starting up".to_string(),
        }
    }
}

/// Anna's operating mode - 6.22.0
/// Distinct from DaemonHealthState - this is about overall operational mode
#[derive(Debug, Clone, PartialEq)]
pub enum AnnaMode {
    /// Normal operation - all features available
    Normal,
    /// Safe mode - limited functionality due to critical self-health issue
    Safe {
        /// Why safe mode was entered
        reason: String,
        /// When safe mode was entered (not serialized)
        since: Instant,
    },
}

// Manual Serialize/Deserialize to handle Instant
impl Serialize for AnnaMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AnnaMode::Normal => {
                serializer.serialize_unit_variant("AnnaMode", 0, "Normal")
            }
            AnnaMode::Safe { reason, .. } => {
                use serde::ser::SerializeStructVariant;
                let mut state = serializer.serialize_struct_variant("AnnaMode", 1, "Safe", 1)?;
                state.serialize_field("reason", reason)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for AnnaMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Reason }

        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        enum AnnaMode_Internal {
            Normal,
            Safe { reason: String },
        }

        match AnnaMode_Internal::deserialize(deserializer)? {
            AnnaMode_Internal::Normal => Ok(AnnaMode::Normal),
            AnnaMode_Internal::Safe { reason } => Ok(AnnaMode::Safe {
                reason,
                since: Instant::now(),
            }),
        }
    }
}

impl Default for AnnaMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl AnnaMode {
    pub fn is_safe(&self) -> bool {
        matches!(self, Self::Safe { .. })
    }

    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Safe { .. } => "SAFE",
        }
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Normal => None,
            Self::Safe { reason, .. } => Some(reason),
        }
    }
}

impl InitializationState {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    pub fn is_initializing(&self) -> bool {
        matches!(self, Self::Initializing { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    /// Get human-readable status string
    pub fn status_string(&self) -> String {
        match self {
            Self::Initializing { progress, .. } => format!("Initializing: {}", progress),
            Self::Ready { .. } => "Ready".to_string(),
            Self::Failed { error, .. } => format!("Failed: {}", error),
        }
    }

    /// Get age since start/completion
    pub fn age(&self) -> Option<Duration> {
        match self {
            Self::Initializing { started_at, .. } | Self::Ready { completed_at: started_at } => {
                started_at.elapsed().ok()
            }
            Self::Failed { .. } => None,
        }
    }
}

/// Crash record for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrashRecord {
    timestamp: SystemTime,
    reason: String,
}

/// Persistent daemon health state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonHealth {
    /// Current health state
    pub health_state: DaemonHealthState,

    /// Last startup time
    pub last_start_time: SystemTime,

    /// Last exit reason (if abnormal)
    pub last_exit_reason: Option<String>,

    /// Recent crash history
    crash_history: Vec<CrashRecord>,

    /// Current initialization state
    #[serde(skip)]
    pub init_state: InitializationState,
}

impl Default for DaemonHealth {
    fn default() -> Self {
        Self {
            health_state: DaemonHealthState::Healthy,
            last_start_time: SystemTime::now(),
            last_exit_reason: None,
            crash_history: Vec::new(),
            init_state: InitializationState::Initializing {
                started_at: SystemTime::now(),
                progress: "Starting up".to_string(),
            },
        }
    }
}

impl DaemonHealth {
    /// Load daemon health from disk, or create default
    pub fn load_or_init(path: &Path) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(contents) => match serde_json::from_str::<DaemonHealth>(&contents) {
                    Ok(mut health) => {
                        info!("Loaded daemon health state: {:?}", health.health_state);

                        // Initialize runtime state
                        health.init_state = InitializationState::Initializing {
                            started_at: SystemTime::now(),
                            progress: "Starting up".to_string(),
                        };

                        return health;
                    }
                    Err(e) => {
                        warn!("Failed to parse daemon health: {}. Using defaults.", e);
                    }
                },
                Err(e) => {
                    warn!("Failed to read daemon health: {}. Using defaults.", e);
                }
            }
        }

        Self::default()
    }

    /// Save daemon health to disk
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize daemon health")?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        // Atomic write
        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, json)
            .with_context(|| format!("Failed to write {}", temp_path.display()))?;
        fs::rename(&temp_path, path)
            .with_context(|| format!("Failed to rename to {}", path.display()))?;

        Ok(())
    }

    /// Record daemon startup
    pub fn record_startup(&mut self) {
        self.last_start_time = SystemTime::now();

        // Clean old crash records outside the window
        let cutoff = SystemTime::now() - Duration::from_secs(CRASH_WINDOW_SECS);
        self.crash_history.retain(|c| c.timestamp > cutoff);
    }

    /// Record a crash or fatal error
    pub fn record_crash(&mut self, reason: String) {
        self.crash_history.push(CrashRecord {
            timestamp: SystemTime::now(),
            reason: reason.clone(),
        });

        self.last_exit_reason = Some(reason);
    }

    /// Check if we should enter Safe Mode due to crash loop
    pub fn should_enter_safe_mode(&self) -> bool {
        // Clean recent crashes within window
        let cutoff = SystemTime::now() - Duration::from_secs(CRASH_WINDOW_SECS);
        let recent_crashes: Vec<_> = self.crash_history.iter()
            .filter(|c| c.timestamp > cutoff)
            .collect();

        if recent_crashes.len() >= CRASH_THRESHOLD {
            warn!(
                "Crash loop detected: {} crashes in {} seconds. Entering Safe Mode.",
                recent_crashes.len(),
                CRASH_WINDOW_SECS
            );
            true
        } else {
            false
        }
    }

    /// Enter Safe Mode
    pub fn enter_safe_mode(&mut self, reason: String) {
        warn!("ANNAD SAFE MODE: entering SafeMode. Reason: {}", reason);
        self.health_state = DaemonHealthState::SafeMode;
        self.last_exit_reason = Some(reason);
    }

    /// Exit Safe Mode (after successful recovery)
    pub fn exit_safe_mode(&mut self) {
        if self.health_state == DaemonHealthState::SafeMode {
            info!("ANNAD SAFE MODE: leaving SafeMode, normal operation restored.");
            self.health_state = DaemonHealthState::Healthy;
            self.crash_history.clear();
            self.last_exit_reason = None;
        }
    }

    /// Set initialization state
    pub fn set_init_state(&mut self, state: InitializationState) {
        self.init_state = state;
    }

    /// Mark as degraded
    pub fn set_degraded(&mut self, reason: String) {
        self.health_state = DaemonHealthState::Degraded;
        self.last_exit_reason = Some(reason);
    }
}

/// Get default health state path
pub fn default_health_path() -> PathBuf {
    PathBuf::from(STATE_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_loop_detection() {
        let mut health = DaemonHealth::default();

        // Record 3 crashes - should not trigger Safe Mode
        for i in 0..3 {
            health.record_crash(format!("test crash {}", i));
        }
        assert!(!health.should_enter_safe_mode());

        // Record 2 more - should trigger Safe Mode
        for i in 3..5 {
            health.record_crash(format!("test crash {}", i));
        }
        assert!(health.should_enter_safe_mode());
    }

    #[test]
    fn test_crash_window_cleanup() {
        let mut health = DaemonHealth::default();

        // Add old crash outside window
        let old_crash = CrashRecord {
            timestamp: SystemTime::now() - Duration::from_secs(CRASH_WINDOW_SECS + 10),
            reason: "old crash".to_string(),
        };
        health.crash_history.push(old_crash);

        // Record startup - should clean old crashes
        health.record_startup();

        // Old crash should be gone
        assert_eq!(health.crash_history.len(), 0);
    }

    #[test]
    fn test_initialization_states() {
        let init = InitializationState::Initializing {
            started_at: SystemTime::now(),
            progress: "test".to_string(),
        };
        assert!(init.is_initializing());
        assert!(!init.is_ready());

        let ready = InitializationState::Ready {
            completed_at: SystemTime::now(),
        };
        assert!(ready.is_ready());
        assert!(!init.is_failed());
    }

    #[test]
    fn test_safe_mode_transition() {
        let mut health = DaemonHealth::default();
        assert_eq!(health.health_state, DaemonHealthState::Healthy);

        health.enter_safe_mode("test reason".to_string());
        assert_eq!(health.health_state, DaemonHealthState::SafeMode);
        assert_eq!(health.last_exit_reason, Some("test reason".to_string()));

        health.exit_safe_mode();
        assert_eq!(health.health_state, DaemonHealthState::Healthy);
        assert_eq!(health.last_exit_reason, None);
    }
}
