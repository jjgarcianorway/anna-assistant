//! Chronicle - Long-term memory persistence and audit trails
//!
//! Phase 1.5: Archive forecasts and ethics projections for verification
//! Citation: [archwiki:System_maintenance]

use super::ethics_projection::EthicsProjection;
use super::forecast::ForecastResult;
use super::timeline::Timeline;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};

/// Chronicle - historical archive of forecasts and projections
pub struct Chronicle {
    /// Archive path
    archive_path: String,
    /// Archived forecasts
    forecasts: Vec<ArchivedForecast>,
    /// Maximum archive size
    max_archives: usize,
}

impl Chronicle {
    /// Create new chronicle
    pub async fn new(archive_path: String) -> Result<Self> {
        // Ensure directory exists
        let path = Path::new(&archive_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Load existing archive if present
        let forecasts = Self::load_archives(&archive_path).await.unwrap_or_default();

        info!(
            "Chronicle initialized with {} archived forecasts",
            forecasts.len()
        );

        Ok(Self {
            archive_path,
            forecasts,
            max_archives: 1000,
        })
    }

    /// Archive a forecast and its ethics projection
    pub async fn archive(
        &mut self,
        forecast: &ForecastResult,
        projection: &EthicsProjection,
        timeline: &Timeline,
    ) -> Result<String> {
        info!("Archiving forecast {}", forecast.forecast_id);

        // Calculate hash for integrity
        let hash = self.calculate_hash(forecast, projection);

        // Create archive entry
        let archived = ArchivedForecast {
            forecast_id: forecast.forecast_id.clone(),
            projection_id: projection.projection_id.clone(),
            archived_at: Utc::now(),
            forecast: forecast.clone(),
            projection: projection.clone(),
            timeline_snapshot: self.snapshot_timeline(timeline),
            hash: hash.clone(),
            verified: false,
        };

        // Add to collection
        self.forecasts.push(archived);

        // Trim old archives if needed
        if self.forecasts.len() > self.max_archives {
            let excess = self.forecasts.len() - self.max_archives;
            self.forecasts.drain(0..excess);
        }

        // Persist to disk
        self.save_archives().await?;

        Ok(hash)
    }

    /// Retrieve archived forecast
    pub fn get_forecast(&self, forecast_id: &str) -> Option<&ArchivedForecast> {
        self.forecasts
            .iter()
            .find(|f| f.forecast_id == forecast_id)
    }

    /// Get recent forecasts
    pub fn recent_forecasts(&self, count: usize) -> Vec<&ArchivedForecast> {
        self.forecasts.iter().rev().take(count).collect()
    }

    /// Get total forecast count
    pub fn total_forecasts(&self) -> usize {
        self.forecasts.len()
    }

    /// Verify forecast integrity
    pub fn verify_forecast(&self, forecast_id: &str) -> Result<bool> {
        let archived = self
            .get_forecast(forecast_id)
            .context("Forecast not found")?;

        let current_hash = self.calculate_hash(&archived.forecast, &archived.projection);

        Ok(current_hash == archived.hash)
    }

    /// Calculate hash for forecast + projection
    fn calculate_hash(&self, forecast: &ForecastResult, projection: &EthicsProjection) -> String {
        // Simple deterministic hash based on key fields
        // In production, would use cryptographic hash
        format!(
            "hash_{}_{}_{}",
            forecast.forecast_id,
            projection.projection_id,
            forecast.generated_at.timestamp()
        )
    }

    /// Create timeline snapshot for archive
    fn snapshot_timeline(&self, timeline: &Timeline) -> TimelineSnapshot {
        let recent_snapshots = timeline
            .snapshots
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect();

        TimelineSnapshot {
            snapshot_count: timeline.snapshots.len(),
            recent_snapshots,
        }
    }

    /// Load archives from disk
    async fn load_archives(path: &str) -> Result<Vec<ArchivedForecast>> {
        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Ok(Vec::new());
        }

        let json = fs::read_to_string(path).await?;
        let archives: Vec<ArchivedForecast> = serde_json::from_str(&json)?;

        debug!("Loaded {} archives from disk", archives.len());
        Ok(archives)
    }

    /// Save archives to disk
    async fn save_archives(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.forecasts)?;
        fs::write(&self.archive_path, json).await?;

        debug!("Saved {} archives to disk", self.forecasts.len());
        Ok(())
    }

    /// Audit trail - compare forecast accuracy to actual outcomes
    pub fn audit_accuracy(&self, forecast_id: &str, actual: &Timeline) -> Result<AuditReport> {
        let archived = self
            .get_forecast(forecast_id)
            .context("Forecast not found")?;

        info!("Auditing forecast accuracy: {}", forecast_id);

        // Get consensus scenario
        let consensus = archived
            .forecast
            .consensus_scenario
            .as_ref()
            .context("No consensus scenario")?;

        // Compare final projected state with actual
        let projected_final = consensus
            .snapshots
            .last()
            .context("No projected snapshots")?;

        let actual_final = actual.latest().context("No actual snapshots")?;

        // Calculate accuracy metrics
        let health_error =
            (projected_final.metrics.health_score - actual_final.metrics.health_score).abs();
        let empathy_error =
            (projected_final.metrics.empathy_index - actual_final.metrics.empathy_index).abs();
        let strain_error =
            (projected_final.metrics.strain_index - actual_final.metrics.strain_index).abs();
        let coherence_error =
            (projected_final.metrics.network_coherence - actual_final.metrics.network_coherence)
                .abs();

        let avg_error = (health_error + empathy_error + strain_error + coherence_error) / 4.0;
        let accuracy_score = (1.0 - avg_error).clamp(0.0, 1.0);

        // Check if warnings were accurate
        let warnings_validated = self.validate_warnings(archived, actual_final);

        Ok(AuditReport {
            audit_id: uuid::Uuid::new_v4().to_string(),
            forecast_id: forecast_id.to_string(),
            audited_at: Utc::now(),
            accuracy_score,
            health_error,
            empathy_error,
            strain_error,
            coherence_error,
            warnings_validated,
            recommendations: self.generate_audit_recommendations(accuracy_score),
        })
    }

    /// Validate if divergence warnings were accurate
    fn validate_warnings(
        &self,
        archived: &ArchivedForecast,
        actual: &super::timeline::SystemSnapshot,
    ) -> Vec<WarningValidation> {
        let mut validations = Vec::new();

        for warning in &archived.forecast.divergence_warnings {
            let was_accurate = if warning.contains("Health score") {
                actual.metrics.health_score < 0.5
            } else if warning.contains("Strain index") {
                actual.metrics.strain_index > 0.7
            } else if warning.contains("Network coherence") {
                actual.metrics.network_coherence < 0.6
            } else if warning.contains("Empathy index") {
                actual.metrics.empathy_index < 0.5
            } else {
                false
            };

            validations.push(WarningValidation {
                warning: warning.clone(),
                was_accurate,
            });
        }

        validations
    }

    /// Generate recommendations for improving forecast accuracy
    fn generate_audit_recommendations(&self, accuracy_score: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if accuracy_score < 0.6 {
            recommendations.push(
                "Low forecast accuracy - consider increasing Monte Carlo iterations".to_string(),
            );
            recommendations.push("Review forecast parameters and trend calculations".to_string());
        } else if accuracy_score < 0.8 {
            recommendations
                .push("Moderate forecast accuracy - minor parameter tuning recommended".to_string());
        } else {
            recommendations.push("High forecast accuracy - maintain current parameters".to_string());
        }

        recommendations
    }
}

/// Archived forecast with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedForecast {
    /// Forecast ID
    pub forecast_id: String,
    /// Ethics projection ID
    pub projection_id: String,
    /// When archived
    pub archived_at: DateTime<Utc>,
    /// The forecast
    pub forecast: ForecastResult,
    /// The ethics projection
    pub projection: EthicsProjection,
    /// Timeline snapshot at time of forecast
    pub timeline_snapshot: TimelineSnapshot,
    /// Integrity hash
    pub hash: String,
    /// Has been verified?
    pub verified: bool,
}

/// Snapshot of timeline for archival
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSnapshot {
    /// Total snapshot count
    pub snapshot_count: usize,
    /// Recent snapshots (last 10)
    pub recent_snapshots: Vec<super::timeline::SystemSnapshot>,
}

/// Audit report comparing forecast to reality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// Audit ID
    pub audit_id: String,
    /// Forecast being audited
    pub forecast_id: String,
    /// When audit was performed
    pub audited_at: DateTime<Utc>,
    /// Overall accuracy (0.0-1.0)
    pub accuracy_score: f64,
    /// Health forecast error
    pub health_error: f64,
    /// Empathy forecast error
    pub empathy_error: f64,
    /// Strain forecast error
    pub strain_error: f64,
    /// Coherence forecast error
    pub coherence_error: f64,
    /// Warning validation results
    pub warnings_validated: Vec<WarningValidation>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Validation of a divergence warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningValidation {
    /// The warning that was issued
    pub warning: String,
    /// Was it accurate?
    pub was_accurate: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chronicle_creation() {
        let chronicle = Chronicle::new("/tmp/test_chronicle.json".to_string())
            .await
            .unwrap();
        assert_eq!(chronicle.forecasts.len(), 0);
    }
}
