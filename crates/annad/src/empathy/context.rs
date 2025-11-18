//! Contextual analysis engine for empathy kernel
//!
//! Phase 1.2: Detects environmental and human stress signals
//! Citation: [archwiki:System_maintenance]

use super::types::{ContextMetrics, EmpathyConfig};
use anyhow::Result;
use sysinfo::System;
use tracing::{debug, warn};

/// Context analyzer - detects system and human stress signals
pub struct ContextAnalyzer {
    config: EmpathyConfig,
    system: System,
}

impl ContextAnalyzer {
    /// Create new context analyzer
    pub fn new(config: EmpathyConfig) -> Self {
        Self {
            config,
            system: System::new_all(),
        }
    }

    /// Analyze current context and return metrics
    pub async fn analyze(&mut self) -> Result<ContextMetrics> {
        debug!("Analyzing system context for empathy kernel");

        // Refresh system information
        self.system.refresh_all();

        // Calculate metrics
        let error_rate = self.calculate_error_rate().await;
        let cpu_load = self.calculate_cpu_load();
        let memory_pressure = self.calculate_memory_pressure();
        let user_activity = self.detect_user_activity().await;
        let time_since_user_interaction = self.time_since_last_interaction().await;
        let fatigue_indicators = self.detect_fatigue_indicators().await;

        let metrics = ContextMetrics {
            error_rate,
            cpu_load,
            memory_pressure,
            user_activity,
            time_since_user_interaction,
            fatigue_indicators,
        };

        debug!(
            "Context analysis: CPU={:.1}%, Mem={:.1}%, Errors={:.2}/h, UserActivity={:.1}%",
            cpu_load * 100.0,
            memory_pressure * 100.0,
            error_rate,
            user_activity * 100.0
        );

        Ok(metrics)
    }

    /// Calculate recent error rate from systemd journal
    async fn calculate_error_rate(&self) -> f64 {
        // TODO: Parse systemd journal for errors in last hour
        // For now, return a baseline estimate
        match self.parse_journal_errors().await {
            Ok(count) => count as f64,
            Err(e) => {
                warn!("Failed to parse journal errors: {}", e);
                0.0
            }
        }
    }

    /// Parse systemd journal for error count
    async fn parse_journal_errors(&self) -> Result<u32> {
        // Use journalctl to count errors in last hour
        let output = tokio::process::Command::new("journalctl")
            .args(&["--priority=err", "--since=1 hour ago", "--no-pager", "-q"])
            .output()
            .await?;

        if output.status.success() {
            let lines = String::from_utf8_lossy(&output.stdout);
            Ok(lines.lines().count() as u32)
        } else {
            Ok(0)
        }
    }

    /// Calculate CPU load average
    fn calculate_cpu_load(&self) -> f64 {
        let cpus = self.system.cpus();
        if cpus.is_empty() {
            return 0.0;
        }

        let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
        let avg_usage = total_usage / cpus.len() as f32;

        (avg_usage / 100.0) as f64
    }

    /// Calculate memory pressure
    fn calculate_memory_pressure(&self) -> f64 {
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();

        if total_memory == 0 {
            return 0.0;
        }

        used_memory as f64 / total_memory as f64
    }

    /// Detect user activity level
    async fn detect_user_activity(&self) -> f64 {
        // Check various indicators of user activity:
        // 1. Active X/Wayland sessions
        // 2. Recent shell history
        // 3. Running user processes

        let mut activity_score: f64 = 0.0;

        // Check for active display session
        if self.has_active_display_session().await {
            activity_score += 0.4;
        }

        // Check for recent user commands
        if self.has_recent_user_commands().await {
            activity_score += 0.3;
        }

        // Check for user processes
        if self.has_active_user_processes() {
            activity_score += 0.3;
        }

        activity_score.min(1.0)
    }

    /// Check for active display session
    async fn has_active_display_session(&self) -> bool {
        // Check DISPLAY or WAYLAND_DISPLAY environment
        std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    /// Check for recent user commands in shell history
    async fn has_recent_user_commands(&self) -> bool {
        // Check if .bash_history was modified in last 10 minutes
        if let Ok(home) = std::env::var("HOME") {
            let hist_path = std::path::Path::new(&home).join(".bash_history");
            if let Ok(metadata) = tokio::fs::metadata(&hist_path).await {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        return elapsed.as_secs() < 600; // 10 minutes
                    }
                }
            }
        }
        false
    }

    /// Check for active user processes
    fn has_active_user_processes(&self) -> bool {
        // Count user processes (not system/root)
        let user_proc_count = self
            .system
            .processes()
            .values()
            .filter(|p| {
                // Check if process belongs to non-root user
                p.user_id()
                    .map(|uid| uid.to_string() != "0")
                    .unwrap_or(false)
            })
            .count();

        user_proc_count > 5 // More than minimal baseline
    }

    /// Time since last user interaction
    async fn time_since_last_interaction(&self) -> u64 {
        // Check various interaction timestamps
        let mut timestamps = Vec::new();

        // Check .bash_history modification time
        if let Ok(home) = std::env::var("HOME") {
            let hist_path = std::path::Path::new(&home).join(".bash_history");
            if let Ok(metadata) = tokio::fs::metadata(&hist_path).await {
                if let Ok(modified) = metadata.modified() {
                    timestamps.push(modified);
                }
            }
        }

        // Find most recent timestamp
        if let Some(latest) = timestamps.iter().max() {
            if let Ok(elapsed) = latest.elapsed() {
                return elapsed.as_secs();
            }
        }

        // Default: very long time ago
        86400 // 24 hours
    }

    /// Detect fatigue indicators from system state
    async fn detect_fatigue_indicators(&self) -> Vec<String> {
        let mut indicators = Vec::new();

        // High error rate suggests system or user fatigue
        let error_rate = self.calculate_error_rate().await;
        if error_rate > 10.0 {
            indicators.push(format!("High error rate: {:.1} errors/hour", error_rate));
        }

        // High memory pressure suggests system strain
        let memory_pressure = self.calculate_memory_pressure();
        if memory_pressure > 0.8 {
            indicators.push(format!(
                "High memory pressure: {:.0}%",
                memory_pressure * 100.0
            ));
        }

        // High CPU load suggests system strain
        let cpu_load = self.calculate_cpu_load();
        if cpu_load > 0.7 {
            indicators.push(format!("High CPU load: {:.0}%", cpu_load * 100.0));
        }

        // Long idle time might indicate user absence or fatigue
        let idle_time = self.time_since_last_interaction().await;
        if idle_time > 3600 {
            // More than 1 hour
            indicators.push(format!("Extended idle time: {} minutes", idle_time / 60));
        }

        indicators
    }

    /// Calculate overall strain index from context metrics
    pub fn calculate_strain_index(
        &self,
        metrics: &ContextMetrics,
        weights: &super::types::ContextWeights,
    ) -> f64 {
        let mut strain = 0.0;

        // Weighted contribution from each metric
        strain += (metrics.error_rate / 20.0).min(1.0) * weights.error_rate_weight;
        strain += metrics.cpu_load * weights.cpu_load_weight;
        strain += metrics.memory_pressure * weights.memory_pressure_weight;
        strain += (1.0 - metrics.user_activity) * weights.user_activity_weight; // Inverted
        strain += (metrics.fatigue_indicators.len() as f64 / 5.0).min(1.0) * weights.fatigue_weight;

        // Normalize
        let total_weight = weights.error_rate_weight
            + weights.cpu_load_weight
            + weights.memory_pressure_weight
            + weights.user_activity_weight
            + weights.fatigue_weight;

        if total_weight > 0.0 {
            strain / total_weight
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_load_calculation() {
        let config = EmpathyConfig::default();
        let analyzer = ContextAnalyzer::new(config);

        let load = analyzer.calculate_cpu_load();
        assert!(load >= 0.0 && load <= 1.0);
    }

    #[test]
    fn test_memory_pressure_calculation() {
        let config = EmpathyConfig::default();
        let analyzer = ContextAnalyzer::new(config);

        let pressure = analyzer.calculate_memory_pressure();
        assert!(pressure >= 0.0 && pressure <= 1.0);
    }

    #[test]
    fn test_strain_index_calculation() {
        let config = EmpathyConfig::default();
        let analyzer = ContextAnalyzer::new(config.clone());

        let metrics = ContextMetrics {
            error_rate: 5.0,
            cpu_load: 0.5,
            memory_pressure: 0.6,
            user_activity: 0.7,
            time_since_user_interaction: 300,
            fatigue_indicators: vec!["test".to_string()],
        };

        let strain = analyzer.calculate_strain_index(&metrics, &config.context_weights);
        assert!(strain >= 0.0 && strain <= 1.0);
    }
}
