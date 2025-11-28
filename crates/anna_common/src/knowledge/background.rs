//! Background Telemetry v0.20.0
//!
//! Runs probes periodically in the background to keep facts warm.
//! Monitors system for changes and triggers learning events.

use super::warmup::{FactCache, WarmupConfig, WarmupResult};
use super::{Fact, KnowledgeStore};
use chrono::{Duration, Utc};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Background telemetry state
#[derive(Debug)]
pub struct BackgroundTelemetry {
    /// Probes scheduled for next run
    pending_probes: VecDeque<String>,
    /// Last run timestamps by probe ID
    last_run: std::collections::HashMap<String, chrono::DateTime<Utc>>,
    /// Whether background tasks are running
    running: Arc<AtomicBool>,
    /// Configuration
    config: BackgroundConfig,
    /// Statistics
    stats: TelemetryStats,
}

/// Configuration for background telemetry
#[derive(Debug, Clone)]
pub struct BackgroundConfig {
    /// How often to check for stale probes (seconds)
    pub check_interval_secs: u64,
    /// Maximum probes per background cycle
    pub max_probes_per_cycle: usize,
    /// Probe refresh intervals by category (seconds)
    pub refresh_intervals: RefreshIntervals,
}

/// Refresh intervals for different probe categories
#[derive(Debug, Clone)]
pub struct RefreshIntervals {
    /// Hardware probes (CPU, memory) - rarely change
    pub hardware: u64,
    /// Storage probes - moderate change frequency
    pub storage: u64,
    /// Network probes - can change frequently
    pub network: u64,
    /// Software probes - OS, packages
    pub software: u64,
}

impl Default for RefreshIntervals {
    fn default() -> Self {
        Self {
            hardware: 3600,  // 1 hour
            storage: 300,    // 5 minutes
            network: 120,    // 2 minutes
            software: 1800,  // 30 minutes
        }
    }
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60,
            max_probes_per_cycle: 2,
            refresh_intervals: RefreshIntervals::default(),
        }
    }
}

/// Statistics for background telemetry
#[derive(Debug, Clone, Default)]
pub struct TelemetryStats {
    /// Total probes run in background
    pub probes_run: u64,
    /// Facts updated
    pub facts_updated: u64,
    /// Facts created
    pub facts_created: u64,
    /// Cycles completed
    pub cycles_completed: u64,
    /// Errors encountered
    pub errors: u64,
}

impl BackgroundTelemetry {
    /// Create new background telemetry manager
    pub fn new() -> Self {
        Self {
            pending_probes: VecDeque::new(),
            last_run: std::collections::HashMap::new(),
            running: Arc::new(AtomicBool::new(false)),
            config: BackgroundConfig::default(),
            stats: TelemetryStats::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: BackgroundConfig) -> Self {
        Self {
            pending_probes: VecDeque::new(),
            last_run: std::collections::HashMap::new(),
            running: Arc::new(AtomicBool::new(false)),
            config,
            stats: TelemetryStats::default(),
        }
    }

    /// Get refresh interval for a probe
    pub fn get_refresh_interval(&self, probe_id: &str) -> u64 {
        // Categorize probe by prefix
        if probe_id.starts_with("cpu.") || probe_id.starts_with("mem.") || probe_id.starts_with("gpu.") {
            self.config.refresh_intervals.hardware
        } else if probe_id.starts_with("disk.") || probe_id.starts_with("fs.") {
            self.config.refresh_intervals.storage
        } else if probe_id.starts_with("net.") || probe_id.starts_with("dns.") {
            self.config.refresh_intervals.network
        } else {
            self.config.refresh_intervals.software
        }
    }

    /// Check if a probe needs refresh
    pub fn needs_refresh(&self, probe_id: &str) -> bool {
        let interval = self.get_refresh_interval(probe_id);
        match self.last_run.get(probe_id) {
            Some(last) => {
                let elapsed = Utc::now().signed_duration_since(*last).num_seconds() as u64;
                elapsed >= interval
            }
            None => true,
        }
    }

    /// Get next probes to run (up to max_probes_per_cycle)
    pub fn get_next_probes(&mut self, available: &[String]) -> Vec<String> {
        let mut result = Vec::new();

        // First check pending probes
        while result.len() < self.config.max_probes_per_cycle {
            if let Some(probe) = self.pending_probes.pop_front() {
                if available.contains(&probe) {
                    result.push(probe);
                }
            } else {
                break;
            }
        }

        // Then check for probes needing refresh
        for probe_id in available {
            if result.len() >= self.config.max_probes_per_cycle {
                break;
            }
            if !result.contains(probe_id) && self.needs_refresh(probe_id) {
                result.push(probe_id.clone());
            }
        }

        result
    }

    /// Record that a probe was run
    pub fn record_probe_run(&mut self, probe_id: &str, success: bool) {
        self.last_run.insert(probe_id.to_string(), Utc::now());
        self.stats.probes_run += 1;
        if !success {
            self.stats.errors += 1;
        }
    }

    /// Record facts updated/created
    pub fn record_facts(&mut self, created: usize, updated: usize) {
        self.stats.facts_created += created as u64;
        self.stats.facts_updated += updated as u64;
    }

    /// Complete a telemetry cycle
    pub fn complete_cycle(&mut self) {
        self.stats.cycles_completed += 1;
    }

    /// Schedule a probe for immediate run
    pub fn schedule_probe(&mut self, probe_id: String) {
        if !self.pending_probes.contains(&probe_id) {
            self.pending_probes.push_back(probe_id);
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> &TelemetryStats {
        &self.stats
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Set running state
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::SeqCst);
    }

    /// Get running flag (for async tasks)
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    /// Get config
    pub fn config(&self) -> &BackgroundConfig {
        &self.config
    }
}

impl Default for BackgroundTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

/// Event for background telemetry triggers
#[derive(Debug, Clone)]
pub enum BackgroundEvent {
    /// A probe was run and returned new data
    ProbeCompleted {
        probe_id: String,
        facts_created: usize,
        facts_updated: usize,
    },
    /// A fact was invalidated (needs refresh)
    FactInvalidated {
        fact_key: String,
        reason: String,
    },
    /// User asked about a topic (triggers learning)
    TopicQueried {
        topic: String,
        relevant_probes: Vec<String>,
    },
    /// System state changed (e.g., network reconnected)
    SystemEvent {
        event_type: String,
        affected_probes: Vec<String>,
    },
}

/// Background event handler
pub struct BackgroundEventHandler {
    /// Pending events
    events: VecDeque<BackgroundEvent>,
    /// Maximum queue size
    max_queue: usize,
}

impl BackgroundEventHandler {
    pub fn new(max_queue: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_queue),
            max_queue,
        }
    }

    /// Push a background event
    pub fn push(&mut self, event: BackgroundEvent) {
        if self.events.len() >= self.max_queue {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Pop next event
    pub fn pop(&mut self) -> Option<BackgroundEvent> {
        self.events.pop_front()
    }

    /// Get pending count
    pub fn pending_count(&self) -> usize {
        self.events.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for BackgroundEventHandler {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_config_default() {
        let config = BackgroundConfig::default();
        assert_eq!(config.check_interval_secs, 60);
        assert_eq!(config.max_probes_per_cycle, 2);
    }

    #[test]
    fn test_refresh_intervals() {
        let telem = BackgroundTelemetry::new();
        assert_eq!(telem.get_refresh_interval("cpu.info"), 3600);
        assert_eq!(telem.get_refresh_interval("net.info"), 120);
        assert_eq!(telem.get_refresh_interval("disk.info"), 300);
    }

    #[test]
    fn test_needs_refresh() {
        let telem = BackgroundTelemetry::new();
        // New probe should need refresh
        assert!(telem.needs_refresh("cpu.info"));
    }

    #[test]
    fn test_record_probe_run() {
        let mut telem = BackgroundTelemetry::new();
        telem.record_probe_run("cpu.info", true);
        assert_eq!(telem.stats.probes_run, 1);
        assert!(!telem.needs_refresh("cpu.info"));
    }

    #[test]
    fn test_schedule_probe() {
        let mut telem = BackgroundTelemetry::new();
        telem.schedule_probe("cpu.info".to_string());
        telem.schedule_probe("cpu.info".to_string()); // Duplicate
        assert_eq!(telem.pending_probes.len(), 1);
    }

    #[test]
    fn test_background_event_handler() {
        let mut handler = BackgroundEventHandler::new(5);
        handler.push(BackgroundEvent::ProbeCompleted {
            probe_id: "cpu.info".to_string(),
            facts_created: 2,
            facts_updated: 0,
        });
        assert_eq!(handler.pending_count(), 1);
        assert!(!handler.is_empty());

        let event = handler.pop();
        assert!(event.is_some());
        assert!(handler.is_empty());
    }
}
