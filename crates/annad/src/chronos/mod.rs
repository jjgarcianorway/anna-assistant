//! Chronos Loop - Temporal reasoning and predictive ethics
//!
//! Phase 1.5: Grant the Collective capacity for forward empathy
//! Citation: [archwiki:System_maintenance]

pub mod chronicle;
pub mod ethics_projection;
pub mod forecast;
pub mod timeline;

use anyhow::{Context, Result};
use chronicle::Chronicle;
use ethics_projection::EthicsProjector;
use forecast::{ForecastConfig, ForecastEngine};
use timeline::{SystemMetrics, SystemSnapshot, Timeline};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Chronos Loop daemon
pub struct ChronosLoop {
    /// Configuration
    config: ChronosConfig,
    /// Timeline of system states
    timeline: Arc<RwLock<Timeline>>,
    /// Chronicle (forecast archive)
    chronicle: Arc<RwLock<Chronicle>>,
    /// Ethics projector
    ethics_projector: Arc<EthicsProjector>,
}

/// Chronos configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronosConfig {
    /// Is chronos enabled?
    pub enabled: bool,
    /// Snapshot interval (minutes)
    pub snapshot_interval_minutes: u64,
    /// Forecast interval (hours)
    pub forecast_interval_hours: u64,
    /// Default forecast window (hours)
    pub default_forecast_window_hours: u64,
    /// Forecast engine config
    pub forecast_config: ForecastConfig,
    /// Timeline retention (number of snapshots)
    pub timeline_retention: usize,
}

impl Default for ChronosConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            snapshot_interval_minutes: 15,
            forecast_interval_hours: 6,
            default_forecast_window_hours: 24,
            forecast_config: ForecastConfig::default(),
            timeline_retention: 672, // 1 week at 15min intervals
        }
    }
}

impl ChronosLoop {
    /// Create new Chronos Loop
    pub async fn new() -> Result<Self> {
        Self::new_with_path("/var/lib/anna/chronos/archive.json").await
    }

    /// Create new Chronos Loop with custom chronicle path (for testing)
    async fn new_with_path(chronicle_path: &str) -> Result<Self> {
        let config = Self::load_or_create_config().await?;
        let timeline = Arc::new(RwLock::new(Timeline::new(
            "main".to_string(),
            config.timeline_retention,
        )));
        let chronicle = Arc::new(RwLock::new(
            Chronicle::new(chronicle_path.to_string()).await?,
        ));
        let ethics_projector = Arc::new(EthicsProjector::new());

        info!("Chronos Loop initialized");

        Ok(Self {
            config,
            timeline,
            chronicle,
            ethics_projector,
        })
    }

    /// Start Chronos Loop daemon
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Chronos Loop disabled in configuration");
            return Ok(());
        }

        info!("Starting Chronos Loop daemon");

        // Spawn snapshot task
        self.spawn_snapshot_task().await;

        // Spawn forecast task
        self.spawn_forecast_task().await;

        // Spawn persistence task
        self.spawn_persistence_task().await;

        Ok(())
    }

    /// Spawn snapshot collection task
    async fn spawn_snapshot_task(&self) {
        let timeline = Arc::clone(&self.timeline);
        let interval_minutes = self.config.snapshot_interval_minutes;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_minutes * 60));

            loop {
                ticker.tick().await;

                match collect_system_snapshot().await {
                    Ok(snapshot) => {
                        let mut tl = timeline.write().await;
                        tl.add_snapshot(snapshot);
                        debug!("System snapshot collected");
                    }
                    Err(e) => {
                        error!("Failed to collect system snapshot: {}", e);
                    }
                }
            }
        });

        info!(
            "Snapshot task started (interval: {} minutes)",
            interval_minutes
        );
    }

    /// Spawn forecast generation task
    async fn spawn_forecast_task(&self) {
        let timeline = Arc::clone(&self.timeline);
        let chronicle = Arc::clone(&self.chronicle);
        let ethics_projector = Arc::clone(&self.ethics_projector);
        let forecast_config = self.config.forecast_config.clone();
        let forecast_window = self.config.default_forecast_window_hours;
        let interval_hours = self.config.forecast_interval_hours;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_hours * 3600));

            loop {
                ticker.tick().await;

                info!("Generating periodic forecast");

                let tl = timeline.read().await;
                let engine = ForecastEngine::new((*tl).clone(), forecast_config.clone());

                let forecast = engine.forecast(forecast_window);
                let projection = ethics_projector.project(&forecast);

                // Archive forecast
                let mut chr = chronicle.write().await;
                if let Err(e) = chr.archive(&forecast, &projection, &tl).await {
                    error!("Failed to archive forecast: {}", e);
                } else {
                    info!(
                        "Forecast {} archived with confidence {:.1}%",
                        forecast.forecast_id,
                        forecast.confidence * 100.0
                    );
                }

                drop(chr);
                drop(tl);

                // Log warnings
                if !forecast.divergence_warnings.is_empty() {
                    warn!(
                        "Forecast divergence warnings: {:?}",
                        forecast.divergence_warnings
                    );
                }

                if projection.moral_cost > 0.5 {
                    warn!("High moral cost projected: {:.2}", projection.moral_cost);
                }
            }
        });

        info!("Forecast task started (interval: {} hours)", interval_hours);
    }

    /// Spawn persistence task
    async fn spawn_persistence_task(&self) {
        let timeline = Arc::clone(&self.timeline);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(3600)); // Every hour

            loop {
                ticker.tick().await;

                let tl = timeline.read().await;
                if let Err(e) = save_timeline(&tl).await {
                    error!("Failed to save timeline: {}", e);
                } else {
                    debug!("Timeline persisted");
                }
            }
        });

        info!("Persistence task started (interval: 1 hour)");
    }

    /// Generate forecast on-demand
    pub async fn generate_forecast(&self, window_hours: u64) -> Result<ForecastWithProjection> {
        info!("Generating on-demand forecast for {} hours", window_hours);

        let timeline = self.timeline.read().await;
        let engine = ForecastEngine::new((*timeline).clone(), self.config.forecast_config.clone());

        let forecast = engine.forecast(window_hours);
        let projection = self.ethics_projector.project(&forecast);

        // Archive it
        let mut chronicle = self.chronicle.write().await;
        let hash = chronicle
            .archive(&forecast, &projection, &timeline)
            .await
            .context("Failed to archive forecast")?;

        info!("Forecast generated and archived with hash {}", hash);

        Ok(ForecastWithProjection {
            forecast,
            projection,
            archive_hash: hash,
        })
    }

    /// Get audit summary for CLI
    pub async fn get_audit_summary(&self) -> ChronosAuditSummary {
        let chronicle = self.chronicle.read().await;
        let recent = chronicle.recent_forecasts(5);

        let recent_forecasts: Vec<ForecastSummary> = recent
            .iter()
            .map(|af| ForecastSummary {
                forecast_id: af.forecast_id.clone(),
                generated_at: af.archived_at,
                horizon_hours: af.forecast.horizon_hours,
                confidence: af.forecast.confidence,
                warnings_count: af.forecast.divergence_warnings.len(),
                moral_cost: af.projection.moral_cost,
            })
            .collect();

        ChronosAuditSummary {
            total_archived: chronicle.total_forecasts(),
            recent_forecasts,
        }
    }

    /// Synchronize forecast parameters with peers
    pub async fn align_parameters(&self) -> Result<()> {
        info!("Aligning forecast parameters across network");

        // Placeholder: In full implementation, would:
        // 1. Query peers for their forecast configs
        // 2. Calculate median/consensus config
        // 3. Update local config if diverged

        Ok(())
    }

    /// Load or create configuration
    async fn load_or_create_config() -> Result<ChronosConfig> {
        let config_path = Path::new("/etc/anna/chronos.yml");

        if config_path.exists() {
            let yaml = fs::read_to_string(config_path).await?;
            let config: ChronosConfig = serde_yaml::from_str(&yaml)?;
            info!("Loaded chronos configuration from {:?}", config_path);
            Ok(config)
        } else {
            let config = ChronosConfig::default();
            warn!("Using default chronos configuration");
            Ok(config)
        }
    }
}

/// Forecast with ethics projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastWithProjection {
    pub forecast: forecast::ForecastResult,
    pub projection: ethics_projection::EthicsProjection,
    pub archive_hash: String,
}

/// Summary for CLI audit command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronosAuditSummary {
    pub total_archived: usize,
    pub recent_forecasts: Vec<ForecastSummary>,
}

/// Forecast summary for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastSummary {
    pub forecast_id: String,
    pub generated_at: chrono::DateTime<Utc>,
    pub horizon_hours: u64,
    pub confidence: f64,
    pub warnings_count: usize,
    pub moral_cost: f64,
}

/// Collect current system snapshot
async fn collect_system_snapshot() -> Result<SystemSnapshot> {
    // Placeholder: In full implementation, would collect real metrics
    // from system probes, empathy kernel, collective mind, etc.

    Ok(SystemSnapshot {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        metrics: SystemMetrics {
            health_score: 0.8,
            empathy_index: 0.75,
            strain_index: 0.25,
            network_coherence: 0.8,
            avg_trust_score: 0.82,
        },
        processes_count: 150,
        memory_usage_mb: 2048,
        cpu_usage_percent: 25.0,
        network_bytes_per_sec: 5000,
        service_states: HashMap::new(),
        pending_actions: vec![],
    })
}

/// Save timeline to disk
async fn save_timeline(timeline: &Timeline) -> Result<()> {
    let path = Path::new("/var/lib/anna/chronos/timeline.log");
    fs::create_dir_all(path.parent().unwrap()).await?;

    let json = serde_json::to_string_pretty(timeline)?;
    fs::write(path, json).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chronos_creation() {
        // Use temp file for testing to avoid permission issues
        let temp_dir = tempfile::tempdir().unwrap();
        let chronicle_path = temp_dir.path().join("test_archive.json");
        let result = ChronosLoop::new_with_path(chronicle_path.to_str().unwrap()).await;
        assert!(result.is_ok());
    }
}
