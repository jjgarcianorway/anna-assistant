//! Timeline - Temporal data structures and snapshot diffs
//!
//! Phase 1.5: Temporal reasoning for predictive ethics
//! Citation: [archwiki:System_maintenance]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// Snapshot ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// System state metrics
    pub metrics: SystemMetrics,
    /// Active processes count
    pub processes_count: usize,
    /// Memory usage (MB)
    pub memory_usage_mb: u64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Network activity (bytes/sec)
    pub network_bytes_per_sec: u64,
    /// Service states
    pub service_states: HashMap<String, ServiceState>,
    /// Pending actions
    pub pending_actions: Vec<String>,
}

/// System health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Health score (0.0-1.0)
    pub health_score: f64,
    /// Empathy index (0.0-1.0)
    pub empathy_index: f64,
    /// Strain index (0.0-1.0)
    pub strain_index: f64,
    /// Network coherence (0.0-1.0)
    pub network_coherence: f64,
    /// Trust score average
    pub avg_trust_score: f64,
}

/// Service state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceState {
    Running,
    Stopped,
    Failed,
    Degraded,
}

/// Timeline - sequence of snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// Timeline ID
    pub id: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time (if bounded)
    pub end_time: Option<DateTime<Utc>>,
    /// Snapshots in chronological order
    pub snapshots: Vec<SystemSnapshot>,
    /// Maximum snapshots to retain
    pub max_snapshots: usize,
}

impl Timeline {
    /// Create new timeline
    pub fn new(id: String, max_snapshots: usize) -> Self {
        Self {
            id,
            start_time: Utc::now(),
            end_time: None,
            snapshots: Vec::new(),
            max_snapshots,
        }
    }

    /// Add snapshot to timeline
    pub fn add_snapshot(&mut self, snapshot: SystemSnapshot) {
        self.snapshots.push(snapshot);

        // Trim old snapshots if exceeds max
        if self.snapshots.len() > self.max_snapshots {
            let excess = self.snapshots.len() - self.max_snapshots;
            self.snapshots.drain(0..excess);
        }
    }

    /// Get latest snapshot
    pub fn latest(&self) -> Option<&SystemSnapshot> {
        self.snapshots.last()
    }

    /// Get snapshot at index
    pub fn get(&self, index: usize) -> Option<&SystemSnapshot> {
        self.snapshots.get(index)
    }

    /// Calculate diff between two snapshots
    pub fn diff(&self, from_idx: usize, to_idx: usize) -> Option<SnapshotDiff> {
        let from = self.snapshots.get(from_idx)?;
        let to = self.snapshots.get(to_idx)?;

        Some(SnapshotDiff::calculate(from, to))
    }

    /// Get trend over window
    pub fn trend(&self, window_size: usize) -> Option<Trend> {
        if self.snapshots.len() < 2 {
            return None;
        }

        let start_idx = if self.snapshots.len() > window_size {
            self.snapshots.len() - window_size
        } else {
            0
        };

        let window = &self.snapshots[start_idx..];

        let health_trend = calculate_metric_trend(window.iter().map(|s| s.metrics.health_score));
        let empathy_trend = calculate_metric_trend(window.iter().map(|s| s.metrics.empathy_index));
        let strain_trend = calculate_metric_trend(window.iter().map(|s| s.metrics.strain_index));
        let coherence_trend =
            calculate_metric_trend(window.iter().map(|s| s.metrics.network_coherence));

        Some(Trend {
            health: trend_direction(health_trend),
            empathy: trend_direction(empathy_trend),
            strain: trend_direction(strain_trend),
            coherence: trend_direction(coherence_trend),
            health_velocity: health_trend,
            empathy_velocity: empathy_trend,
            strain_velocity: strain_trend,
            coherence_velocity: coherence_trend,
        })
    }
}

/// Difference between two snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    /// Time delta (seconds)
    pub time_delta_secs: i64,
    /// Health score change
    pub health_delta: f64,
    /// Empathy index change
    pub empathy_delta: f64,
    /// Strain index change
    pub strain_delta: f64,
    /// Coherence change
    pub coherence_delta: f64,
    /// Process count change
    pub processes_delta: i64,
    /// Memory change (MB)
    pub memory_delta_mb: i64,
    /// CPU usage change
    pub cpu_delta: f64,
    /// Service state changes
    pub service_changes: Vec<ServiceChange>,
    /// New pending actions
    pub new_actions: Vec<String>,
}

impl SnapshotDiff {
    /// Calculate diff between snapshots
    pub fn calculate(from: &SystemSnapshot, to: &SystemSnapshot) -> Self {
        let time_delta_secs = (to.timestamp - from.timestamp).num_seconds();

        // Calculate service changes
        let mut service_changes = Vec::new();
        for (service, to_state) in &to.service_states {
            if let Some(from_state) = from.service_states.get(service) {
                if from_state != to_state {
                    service_changes.push(ServiceChange {
                        service: service.clone(),
                        from: from_state.clone(),
                        to: to_state.clone(),
                    });
                }
            } else {
                // New service
                service_changes.push(ServiceChange {
                    service: service.clone(),
                    from: ServiceState::Stopped,
                    to: to_state.clone(),
                });
            }
        }

        // Find new actions
        let new_actions: Vec<String> = to
            .pending_actions
            .iter()
            .filter(|a| !from.pending_actions.contains(a))
            .cloned()
            .collect();

        Self {
            time_delta_secs,
            health_delta: to.metrics.health_score - from.metrics.health_score,
            empathy_delta: to.metrics.empathy_index - from.metrics.empathy_index,
            strain_delta: to.metrics.strain_index - from.metrics.strain_index,
            coherence_delta: to.metrics.network_coherence - from.metrics.network_coherence,
            processes_delta: to.processes_count as i64 - from.processes_count as i64,
            memory_delta_mb: to.memory_usage_mb as i64 - from.memory_usage_mb as i64,
            cpu_delta: to.cpu_usage_percent - from.cpu_usage_percent,
            service_changes,
            new_actions,
        }
    }
}

/// Service state change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceChange {
    pub service: String,
    pub from: ServiceState,
    pub to: ServiceState,
}

/// Trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    /// Health trend direction
    pub health: TrendDirection,
    /// Empathy trend direction
    pub empathy: TrendDirection,
    /// Strain trend direction
    pub strain: TrendDirection,
    /// Coherence trend direction
    pub coherence: TrendDirection,
    /// Health velocity (change per snapshot)
    pub health_velocity: f64,
    /// Empathy velocity
    pub empathy_velocity: f64,
    /// Strain velocity
    pub strain_velocity: f64,
    /// Coherence velocity
    pub coherence_velocity: f64,
}

/// Trend direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

/// Calculate metric trend (average change per snapshot)
fn calculate_metric_trend<I>(values: I) -> f64
where
    I: Iterator<Item = f64>,
{
    let values: Vec<f64> = values.collect();
    if values.len() < 2 {
        return 0.0;
    }

    let mut sum_changes = 0.0;
    for i in 1..values.len() {
        sum_changes += values[i] - values[i - 1];
    }

    sum_changes / (values.len() - 1) as f64
}

/// Determine trend direction from velocity
fn trend_direction(velocity: f64) -> TrendDirection {
    if velocity > 0.05 {
        TrendDirection::Improving
    } else if velocity < -0.05 {
        TrendDirection::Degrading
    } else {
        TrendDirection::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_timeline_creation() {
        let timeline = Timeline::new("test".to_string(), 10);
        assert_eq!(timeline.id, "test");
        assert_eq!(timeline.snapshots.len(), 0);
    }

    #[test]
    fn test_snapshot_diff() {
        let from = SystemSnapshot {
            id: "1".to_string(),
            timestamp: Utc::now(),
            metrics: SystemMetrics {
                health_score: 0.8,
                empathy_index: 0.7,
                strain_index: 0.3,
                network_coherence: 0.75,
                avg_trust_score: 0.8,
            },
            processes_count: 100,
            memory_usage_mb: 1000,
            cpu_usage_percent: 20.0,
            network_bytes_per_sec: 1000,
            service_states: HashMap::new(),
            pending_actions: vec![],
        };

        let to = SystemSnapshot {
            id: "2".to_string(),
            timestamp: Utc::now(),
            metrics: SystemMetrics {
                health_score: 0.85,
                empathy_index: 0.75,
                strain_index: 0.25,
                network_coherence: 0.8,
                avg_trust_score: 0.82,
            },
            processes_count: 105,
            memory_usage_mb: 1100,
            cpu_usage_percent: 25.0,
            network_bytes_per_sec: 1200,
            service_states: HashMap::new(),
            pending_actions: vec![],
        };

        let diff = SnapshotDiff::calculate(&from, &to);

        assert_relative_eq!(diff.health_delta, 0.05, epsilon = 1e-10);
        assert_relative_eq!(diff.empathy_delta, 0.05, epsilon = 1e-10);
        assert_eq!(diff.processes_delta, 5);
    }

    #[test]
    fn test_trend_calculation() {
        let mut timeline = Timeline::new("test".to_string(), 10);

        // Add snapshots with improving health
        for i in 0..5 {
            timeline.add_snapshot(SystemSnapshot {
                id: i.to_string(),
                timestamp: Utc::now(),
                metrics: SystemMetrics {
                    health_score: 0.5 + (i as f64 * 0.1),
                    empathy_index: 0.7,
                    strain_index: 0.3,
                    network_coherence: 0.75,
                    avg_trust_score: 0.8,
                },
                processes_count: 100,
                memory_usage_mb: 1000,
                cpu_usage_percent: 20.0,
                network_bytes_per_sec: 1000,
                service_states: HashMap::new(),
                pending_actions: vec![],
            });
        }

        let trend = timeline.trend(5).unwrap();
        assert_eq!(trend.health, TrendDirection::Improving);
    }
}
