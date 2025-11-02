//! Threshold-Based Trigger System for Anna v0.14.0 "Orion III" Phase 2.3
//!
//! Automatic action triggers based on predictive and anomaly metrics

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::anomaly::{AnomalyDetector, Severity};
use crate::forecast::ForecastEngine;
use crate::profiled::Profiler;

/// Trigger threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerThreshold {
    pub id: String,
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub condition: TriggerCondition,
    pub action_id: String,           // Action to trigger
    pub cooldown_hours: u64,         // Minimum hours between triggers
    pub last_triggered: Option<u64>, // Last trigger timestamp
    pub enabled: bool,
}

/// Metric type to monitor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    ForecastDeviation,      // Forecast deviation > 2σ
    AnomalyCritical,        // Persistent critical anomaly
    PerformanceDrift,       // Performance drift > threshold
    DiskSpaceLow,           // Disk space < threshold
    MemoryPressure,         // Memory usage > threshold
}

/// Trigger condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub threshold: f32,              // Numeric threshold
    pub operator: String,            // ">", "<", ">=", "<=", "=="
    pub persistence_cycles: u32,     // How many cycles must condition persist
}

/// Trigger event (logged when trigger fires)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvent {
    pub trigger_id: String,
    pub timestamp: u64,
    pub metric_value: f32,
    pub threshold: f32,
    pub confidence: f32,             // Confidence in trigger decision (0.0-1.0)
    pub action_id: String,
    pub reason: String,
    pub executed: bool,              // Was action actually executed?
}

impl TriggerThreshold {
    /// Check if trigger is in cooldown
    pub fn is_in_cooldown(&self) -> bool {
        if let Some(last) = self.last_triggered {
            let cooldown_secs = self.cooldown_hours * 3600;
            let now = current_timestamp();
            now < last + cooldown_secs
        } else {
            false
        }
    }

    /// Check if condition is met
    pub fn check_condition(&self, value: f32) -> bool {
        match self.condition.operator.as_str() {
            ">" => value > self.condition.threshold,
            "<" => value < self.condition.threshold,
            ">=" => value >= self.condition.threshold,
            "<=" => value <= self.condition.threshold,
            "==" => (value - self.condition.threshold).abs() < 0.01,
            _ => false,
        }
    }

    /// Update last triggered timestamp
    pub fn mark_triggered(&mut self) {
        self.last_triggered = Some(current_timestamp());
    }
}

/// Trigger manager
pub struct TriggerManager {
    thresholds: Vec<TriggerThreshold>,
    events_path: PathBuf,
    config_path: PathBuf,
    anomaly_detector: AnomalyDetector,
    forecast_engine: ForecastEngine,
    profiler: Option<Profiler>,
}

impl TriggerManager {
    /// Create new trigger manager
    pub fn new() -> Result<Self> {
        let state_dir = Self::get_state_dir()?;
        fs::create_dir_all(&state_dir)?;

        let events_path = state_dir.join("trigger_events.jsonl");
        let config_path = state_dir.join("triggers.json");

        let thresholds = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| Self::default_thresholds())
        } else {
            Self::default_thresholds()
        };

        let anomaly_detector = AnomalyDetector::new()?;
        let forecast_engine = ForecastEngine::new()?;
        let profiler = Profiler::new().ok();

        Ok(Self {
            thresholds,
            events_path,
            config_path,
            anomaly_detector,
            forecast_engine,
            profiler,
        })
    }

    /// Get state directory
    fn get_state_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".local/state/anna"))
    }

    /// Default trigger thresholds
    fn default_thresholds() -> Vec<TriggerThreshold> {
        vec![
            TriggerThreshold {
                id: "forecast_deviation_critical".to_string(),
                name: "Critical Forecast Deviation".to_string(),
                description: "Triggers when forecast deviates >2σ from baseline".to_string(),
                metric_type: MetricType::ForecastDeviation,
                condition: TriggerCondition {
                    threshold: 2.0,
                    operator: ">".to_string(),
                    persistence_cycles: 2,
                },
                action_id: "check_failed_services".to_string(),
                cooldown_hours: 12,
                last_triggered: None,
                enabled: true,
            },
            TriggerThreshold {
                id: "performance_drift_high".to_string(),
                name: "High Performance Drift".to_string(),
                description: "Triggers when Anna's performance degrades >15%".to_string(),
                metric_type: MetricType::PerformanceDrift,
                condition: TriggerCondition {
                    threshold: 15.0,
                    operator: ">".to_string(),
                    persistence_cycles: 3,
                },
                action_id: "cleanup_user_cache".to_string(),
                cooldown_hours: 24,
                last_triggered: None,
                enabled: true,
            },
            TriggerThreshold {
                id: "disk_space_critical".to_string(),
                name: "Critical Disk Space".to_string(),
                description: "Triggers when disk space <10%".to_string(),
                metric_type: MetricType::DiskSpaceLow,
                condition: TriggerCondition {
                    threshold: 10.0,
                    operator: "<".to_string(),
                    persistence_cycles: 1,
                },
                action_id: "cleanup_pacman_cache".to_string(),
                cooldown_hours: 6,
                last_triggered: None,
                enabled: true,
            },
        ]
    }

    /// Check all triggers and return which should fire
    pub fn check_triggers(&mut self) -> Result<Vec<TriggerEvent>> {
        let mut events = Vec::new();

        // Check forecast deviation
        if let Ok(Some(forecast)) = self.forecast_engine.forecast_7d() {
            let deviation_sigma = (forecast.deviation / 1.0).abs(); // Simplified σ calculation
            events.extend(self.check_metric_triggers(
                MetricType::ForecastDeviation,
                deviation_sigma as f32,
                format!("Forecast deviation: {:.2}σ", deviation_sigma),
            )?);
        }

        // Check anomalies
        if let Ok(anomalies) = self.anomaly_detector.detect_anomalies() {
            let critical_count = anomalies.iter().filter(|a| a.severity == Severity::Critical).count();
            if critical_count > 0 {
                events.extend(self.check_metric_triggers(
                    MetricType::AnomalyCritical,
                    critical_count as f32,
                    format!("{} critical anomalies detected", critical_count),
                )?);
            }
        }

        // Check performance drift
        if let Some(profiler) = &self.profiler {
            if let Ok(summary) = profiler.get_summary() {
                let baseline = &summary.baseline;
                let current = summary.current_avg_rpc_ms;

                if baseline.avg_rpc_latency_ms > 0.0 {
                    let drift_pct = ((current - baseline.avg_rpc_latency_ms) / baseline.avg_rpc_latency_ms) * 100.0;
                    events.extend(self.check_metric_triggers(
                        MetricType::PerformanceDrift,
                        drift_pct,
                        format!("Performance drift: {:.1}%", drift_pct),
                    )?);
                }
            }
        }

        Ok(events)
    }

    /// Check triggers for specific metric type
    fn check_metric_triggers(
        &mut self,
        metric_type: MetricType,
        value: f32,
        reason: String,
    ) -> Result<Vec<TriggerEvent>> {
        let mut events = Vec::new();

        for threshold in self.thresholds.iter_mut() {
            if !threshold.enabled || threshold.metric_type != metric_type {
                continue;
            }

            if threshold.is_in_cooldown() {
                continue;
            }

            if threshold.check_condition(value) {
                // Calculate confidence based on how far past threshold we are
                let excess = if threshold.condition.operator.contains('>') {
                    value - threshold.condition.threshold
                } else {
                    threshold.condition.threshold - value
                };
                let confidence = (0.5 + (excess / threshold.condition.threshold) * 0.5).min(1.0).max(0.0);

                let event = TriggerEvent {
                    trigger_id: threshold.id.clone(),
                    timestamp: current_timestamp(),
                    metric_value: value,
                    threshold: threshold.condition.threshold,
                    confidence,
                    action_id: threshold.action_id.clone(),
                    reason: reason.clone(),
                    executed: false, // Will be set by executor
                };

                threshold.mark_triggered();
                events.push(event);
            }
        }

        // Save updated thresholds
        self.save_config()?;

        Ok(events)
    }

    /// Log trigger event
    pub fn log_event(&self, event: &TriggerEvent) -> Result<()> {
        let json = serde_json::to_string(event)?;
        let mut content = String::new();

        if self.events_path.exists() {
            content = fs::read_to_string(&self.events_path)?;
        }

        content.push_str(&json);
        content.push('\n');

        fs::write(&self.events_path, content)?;

        Ok(())
    }

    /// Load trigger events
    pub fn load_events(&self) -> Result<Vec<TriggerEvent>> {
        if !self.events_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.events_path)?;
        let mut events = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<TriggerEvent>(line) {
                Ok(event) => events.push(event),
                Err(e) => {
                    eprintln!("Warning: Failed to parse trigger event: {}", e);
                    continue;
                }
            }
        }

        Ok(events)
    }

    /// Get trigger summary
    pub fn get_summary(&self) -> Result<TriggerSummary> {
        let events = self.load_events()?;
        let total_triggers = events.len();
        let executed_count = events.iter().filter(|e| e.executed).count();
        let cooldown_count = self.thresholds.iter().filter(|t| t.is_in_cooldown()).count();

        Ok(TriggerSummary {
            total_thresholds: self.thresholds.len(),
            enabled_thresholds: self.thresholds.iter().filter(|t| t.enabled).count(),
            cooldown_count,
            total_triggers,
            executed_count,
            recent_events: events.into_iter().rev().take(10).collect(),
        })
    }

    /// Save config
    fn save_config(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.thresholds)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// Get all thresholds
    pub fn get_thresholds(&self) -> &[TriggerThreshold] {
        &self.thresholds
    }

    /// Simulate triggers without executing
    pub fn simulate(&mut self) -> Result<Vec<TriggerEvent>> {
        let mut events = self.check_triggers()?;

        // Mark as not executed (simulation)
        for event in &mut events {
            event.executed = false;
        }

        Ok(events)
    }
}

/// Trigger summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSummary {
    pub total_thresholds: usize,
    pub enabled_thresholds: usize,
    pub cooldown_count: usize,
    pub total_triggers: usize,
    pub executed_count: usize,
    pub recent_events: Vec<TriggerEvent>,
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_threshold_cooldown() {
        let mut threshold = TriggerThreshold {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            metric_type: MetricType::PerformanceDrift,
            condition: TriggerCondition {
                threshold: 10.0,
                operator: ">".to_string(),
                persistence_cycles: 1,
            },
            action_id: "test_action".to_string(),
            cooldown_hours: 1,
            last_triggered: None,
            enabled: true,
        };

        assert!(!threshold.is_in_cooldown());

        threshold.mark_triggered();
        assert!(threshold.is_in_cooldown());
    }

    #[test]
    fn test_check_condition_operators() {
        let mut threshold = TriggerThreshold {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            metric_type: MetricType::PerformanceDrift,
            condition: TriggerCondition {
                threshold: 10.0,
                operator: ">".to_string(),
                persistence_cycles: 1,
            },
            action_id: "test_action".to_string(),
            cooldown_hours: 1,
            last_triggered: None,
            enabled: true,
        };

        assert!(threshold.check_condition(15.0));
        assert!(!threshold.check_condition(5.0));

        threshold.condition.operator = "<".to_string();
        assert!(threshold.check_condition(5.0));
        assert!(!threshold.check_condition(15.0));

        threshold.condition.operator = ">=".to_string();
        assert!(threshold.check_condition(10.0));
        assert!(threshold.check_condition(15.0));
        assert!(!threshold.check_condition(9.0));
    }

    #[test]
    fn test_metric_type_equality() {
        assert_eq!(MetricType::PerformanceDrift, MetricType::PerformanceDrift);
        assert_ne!(MetricType::PerformanceDrift, MetricType::AnomalyCritical);
    }

    #[test]
    fn test_trigger_event_serialization() {
        let event = TriggerEvent {
            trigger_id: "test_trigger".to_string(),
            timestamp: 1699000000,
            metric_value: 25.5,
            threshold: 15.0,
            confidence: 0.8,
            action_id: "test_action".to_string(),
            reason: "Test trigger".to_string(),
            executed: true,
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: TriggerEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.trigger_id, "test_trigger");
        assert_eq!(parsed.confidence, 0.8);
        assert!(parsed.executed);
    }

    #[test]
    fn test_default_thresholds_creation() {
        let thresholds = TriggerManager::default_thresholds();

        assert!(!thresholds.is_empty());

        // All should be enabled by default
        assert!(thresholds.iter().all(|t| t.enabled));

        // All should have valid cooldowns
        assert!(thresholds.iter().all(|t| t.cooldown_hours > 0));
    }

    #[test]
    fn test_trigger_condition_persistence() {
        let condition = TriggerCondition {
            threshold: 10.0,
            operator: ">".to_string(),
            persistence_cycles: 3,
        };

        assert_eq!(condition.persistence_cycles, 3);
    }

    #[test]
    fn test_trigger_threshold_mark_triggered() {
        let mut threshold = TriggerThreshold {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            metric_type: MetricType::DiskSpaceLow,
            condition: TriggerCondition {
                threshold: 10.0,
                operator: "<".to_string(),
                persistence_cycles: 1,
            },
            action_id: "cleanup".to_string(),
            cooldown_hours: 6,
            last_triggered: None,
            enabled: true,
        };

        assert!(threshold.last_triggered.is_none());
        threshold.mark_triggered();
        assert!(threshold.last_triggered.is_some());
    }

    #[test]
    fn test_state_dir_path() {
        if let Ok(dir) = TriggerManager::get_state_dir() {
            assert!(dir.to_string_lossy().contains(".local/state/anna"));
        }
    }
}
