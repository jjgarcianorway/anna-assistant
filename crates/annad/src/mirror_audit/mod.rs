//! Mirror Audit: temporal self-reflection and adaptive learning
//!
//! Phase 1.6: Closes the loop between prediction and reality
//! Citation: [archwiki:System_maintenance]

pub mod adjust;
pub mod align;
pub mod bias;
pub mod types;

use adjust::generate_adjustment_plan;
use align::{calculate_temporal_integrity, compute_errors};
use bias::scan_for_biases;
use types::{AuditEntry, MirrorAuditState, SystemMetrics, TemporalIntegrityScore};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

/// Mirror Audit system orchestrator
pub struct MirrorAudit {
    /// Configuration
    config: MirrorAuditConfig,
    /// Current state
    state: MirrorAuditState,
    /// State file path
    state_path: String,
    /// Append-only audit log path
    audit_log_path: String,
}

/// Configuration for mirror audit system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorAuditConfig {
    /// Audit schedule (hours)
    pub schedule_hours: u64,
    /// Minimum confidence for bias reporting
    pub min_confidence: f64,
    /// Write JSONL audit log?
    pub write_jsonl: bool,
    /// Enable bias scanning?
    pub enable_bias_scan: bool,
    /// Advisory only mode (never auto-apply)?
    pub advisory_only: bool,
}

impl Default for MirrorAuditConfig {
    fn default() -> Self {
        Self {
            schedule_hours: 24,
            min_confidence: 0.6,
            write_jsonl: true,
            enable_bias_scan: true,
            advisory_only: true,
        }
    }
}

impl MirrorAudit {
    /// Create new mirror audit system
    pub async fn new(state_path: String, audit_log_path: String) -> Result<Self> {
        // Load or create config
        let config = Self::load_or_create_config().await?;

        // Load or create state
        let state = Self::load_state(&state_path).await.unwrap_or_default();

        // Ensure directories exist
        if let Some(parent) = Path::new(&state_path).parent() {
            fs::create_dir_all(parent).await?;
        }
        if let Some(parent) = Path::new(&audit_log_path).parent() {
            fs::create_dir_all(parent).await?;
        }

        info!(
            "Mirror Audit initialized (total audits: {})",
            state.total_audits
        );

        Ok(Self {
            config,
            state,
            state_path,
            audit_log_path,
        })
    }

    /// Run audit for a specific forecast
    pub async fn audit_forecast(
        &mut self,
        forecast_id: &str,
        predicted: SystemMetrics,
        actual: SystemMetrics,
        forecast_generated_at: chrono::DateTime<Utc>,
        horizon_hours: u64,
    ) -> Result<AuditEntry> {
        info!("Auditing forecast {}", forecast_id);

        // Compute errors
        let errors = compute_errors(&predicted, &actual);

        // Calculate temporal integrity
        let integrity = calculate_temporal_integrity(&errors, &predicted, &actual)?;

        // Scan for biases in recent audits
        let bias_findings = if self.config.enable_bias_scan {
            let recent_audits = self.get_recent_audits(10);
            scan_for_biases(&recent_audits).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Generate adjustment plan if biases detected
        let adjustment_plan = if !bias_findings.is_empty() {
            let recent_scores: Vec<TemporalIntegrityScore> =
                self.state.recent_scores.iter().map(|&overall| {
                    TemporalIntegrityScore {
                        overall,
                        prediction_accuracy: overall,
                        ethical_alignment: overall,
                        coherence_stability: overall,
                        confidence: 0.8,
                    }
                }).collect();

            generate_adjustment_plan(&bias_findings, &recent_scores).unwrap_or(None)
        } else {
            None
        };

        // Create audit entry
        let audit_entry = AuditEntry {
            audit_id: uuid::Uuid::new_v4().to_string(),
            forecast_id: forecast_id.to_string(),
            audited_at: Utc::now(),
            forecast_generated_at,
            horizon_hours,
            predicted,
            actual,
            errors,
            temporal_integrity_score: integrity.overall,
            bias_findings: bias_findings.clone(),
            adjustment_plan: adjustment_plan.clone(),
        };

        // Update state
        self.state.total_audits += 1;
        self.state.last_audit_at = Some(Utc::now());
        self.state.recent_scores.push(integrity.overall);
        if self.state.recent_scores.len() > 10 {
            self.state.recent_scores.remove(0);
        }
        self.state.active_biases = bias_findings;
        if let Some(plan) = adjustment_plan {
            self.state.pending_adjustments.push(plan);
            // Keep only last 5 plans
            if self.state.pending_adjustments.len() > 5 {
                self.state.pending_adjustments.remove(0);
            }
        }

        // Persist state
        self.save_state().await?;

        // Write to audit log
        if self.config.write_jsonl {
            self.append_to_audit_log(&audit_entry).await?;
        }

        info!(
            "Audit complete: temporal integrity {:.1}%, {} biases detected",
            integrity.overall * 100.0,
            self.state.active_biases.len()
        );

        Ok(audit_entry)
    }

    /// Get recent audit entries (stub - would load from log in production)
    fn get_recent_audits(&self, _count: usize) -> Vec<AuditEntry> {
        // In production, would parse audit log file
        // For now, return empty to avoid blocking
        Vec::new()
    }

    /// Get current audit summary
    pub fn get_summary(&self) -> MirrorAuditSummary {
        let avg_score = if self.state.recent_scores.is_empty() {
            None
        } else {
            Some(
                self.state.recent_scores.iter().sum::<f64>() / self.state.recent_scores.len() as f64,
            )
        };

        MirrorAuditSummary {
            total_audits: self.state.total_audits,
            last_audit_at: self.state.last_audit_at,
            average_temporal_integrity: avg_score,
            active_biases: self.state.active_biases.clone(),
            pending_adjustments: self.state.pending_adjustments.clone(),
        }
    }

    /// Append audit entry to JSONL log
    async fn append_to_audit_log(&self, entry: &AuditEntry) -> Result<()> {
        let json = serde_json::to_string(entry)?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_log_path)
            .await?;

        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.sync_all().await?;

        debug!("Audit entry appended to {}", self.audit_log_path);
        Ok(())
    }

    /// Save state to disk
    async fn save_state(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.state)?;
        fs::write(&self.state_path, json).await?;
        debug!("Mirror audit state saved to {}", self.state_path);
        Ok(())
    }

    /// Load state from disk
    async fn load_state(path: &str) -> Result<MirrorAuditState> {
        let json = fs::read_to_string(path).await?;
        let state: MirrorAuditState = serde_json::from_str(&json)?;
        debug!("Loaded mirror audit state from {}", path);
        Ok(state)
    }

    /// Load or create configuration
    async fn load_or_create_config() -> Result<MirrorAuditConfig> {
        let config_path = Path::new("/etc/anna/mirror_audit.yml");

        if config_path.exists() {
            let yaml = fs::read_to_string(config_path).await?;
            let config: MirrorAuditConfig = serde_yaml::from_str(&yaml)?;
            info!("Loaded mirror audit config from {:?}", config_path);
            Ok(config)
        } else {
            let config = MirrorAuditConfig::default();
            warn!("Using default mirror audit configuration");
            Ok(config)
        }
    }
}

/// Summary for CLI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorAuditSummary {
    pub total_audits: usize,
    pub last_audit_at: Option<chrono::DateTime<Utc>>,
    pub average_temporal_integrity: Option<f64>,
    pub active_biases: Vec<types::BiasFinding>,
    pub pending_adjustments: Vec<types::AdjustmentPlan>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mirror_audit_creation() {
        let result = MirrorAudit::new(
            "/tmp/anna_test_mirror_audit_state.json".to_string(),
            "/tmp/anna_test_mirror_audit.jsonl".to_string(),
        )
        .await;

        assert!(result.is_ok());
    }
}
