//! Health probe runner and orchestration
//!
//! Phase 0.5: Run health probes and aggregate results
//! Citation: [archwiki:System_maintenance]

use super::{
    DiskSpaceProbe, HealthProbe, PacmanDbProbe, ProbeResult, ProbeStatus, SystemdUnitsProbe,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};
use uuid::Uuid;

/// Health summary with all probe results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub run_id: String,
    pub timestamp: String,
    pub probes: Vec<ProbeResult>,
    pub overall_status: ProbeStatus,
}

/// Run all registered health probes
pub async fn run_all_probes() -> Result<HealthSummary> {
    let run_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    info!("Starting health check run: {}", run_id);

    // Create probe instances
    let probes: Vec<Box<dyn HealthProbe>> = vec![
        Box::new(DiskSpaceProbe {
            warn_threshold: 80,
            fail_threshold: 95,
        }),
        Box::new(SystemdUnitsProbe),
        Box::new(PacmanDbProbe),
    ];

    let mut results = Vec::new();
    let mut has_fail = false;
    let mut has_warn = false;

    // Run each probe
    for probe in probes {
        match probe.run() {
            Ok(result) => {
                match result.status {
                    ProbeStatus::Fail => has_fail = true,
                    ProbeStatus::Warn => has_warn = true,
                    ProbeStatus::Ok => {}
                }
                results.push(result);
            }
            Err(e) => {
                error!("Probe {} failed: {}", probe.name(), e);
                // Create a failure result
                results.push(ProbeResult {
                    probe: probe.name().to_string(),
                    status: ProbeStatus::Fail,
                    details: serde_json::json!({
                        "error": e.to_string(),
                    }),
                    citation: probe.citation().to_string(),
                    duration_ms: 0,
                });
                has_fail = true;
            }
        }
    }

    // Determine overall status
    let overall_status = if has_fail {
        ProbeStatus::Fail
    } else if has_warn {
        ProbeStatus::Warn
    } else {
        ProbeStatus::Ok
    };

    let summary = HealthSummary {
        run_id: run_id.clone(),
        timestamp,
        probes: results.clone(),
        overall_status: overall_status.clone(),
    };

    // Write to health log
    if let Err(e) = write_health_log(&summary).await {
        error!("Failed to write health log: {}", e);
    }

    // Create alerts for failed probes
    for result in &results {
        if result.status == ProbeStatus::Fail {
            if let Err(e) = create_alert(&run_id, result).await {
                error!("Failed to create alert for {}: {}", result.probe, e);
            }
        }
    }

    info!("Health check complete: {:?}", overall_status);

    Ok(summary)
}

/// Run a single probe by name
pub async fn run_probe(probe_name: &str) -> Result<ProbeResult> {
    let probe: Box<dyn HealthProbe> = match probe_name {
        "disk-space" => Box::new(DiskSpaceProbe {
            warn_threshold: 80,
            fail_threshold: 95,
        }),
        "systemd-units" => Box::new(SystemdUnitsProbe),
        "pacman-db" => Box::new(PacmanDbProbe),
        _ => anyhow::bail!("Unknown probe: {}", probe_name),
    };

    probe.run()
}

/// Get health summary from last run
pub async fn get_health_summary() -> Result<Option<HealthSummary>> {
    // Read last line from health log
    let log_path = get_health_log_path();

    if !log_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&log_path)?;
    let lines: Vec<&str> = content.lines().collect();

    if let Some(last_line) = lines.last() {
        let summary: HealthSummary = serde_json::from_str(last_line)?;
        Ok(Some(summary))
    } else {
        Ok(None)
    }
}

/// Write health summary to JSONL log
async fn write_health_log(summary: &HealthSummary) -> Result<()> {
    let log_path = get_health_log_path();

    // Ensure directory exists
    if let Some(parent) = log_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Append to log file
    let json = serde_json::to_string(summary)?;
    let content = format!("{}\n", json);

    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await?;

    file.write_all(content.as_bytes()).await?;

    Ok(())
}

/// Create alert file for failed probe
async fn create_alert(run_id: &str, result: &ProbeResult) -> Result<()> {
    let alert_dir = get_alerts_dir();
    tokio::fs::create_dir_all(&alert_dir).await?;

    let alert_path = alert_dir.join(format!("{}.json", result.probe));

    let alert = serde_json::json!({
        "run_id": run_id,
        "probe": result.probe,
        "status": result.status,
        "details": result.details,
        "citation": result.citation,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let content = serde_json::to_string_pretty(&alert)?;
    tokio::fs::write(&alert_path, content).await?;

    Ok(())
}

/// Get health log path
fn get_health_log_path() -> PathBuf {
    PathBuf::from("/var/log/anna/health.jsonl")
}

/// Get alerts directory path
fn get_alerts_dir() -> PathBuf {
    PathBuf::from("/var/lib/anna/alerts")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_single_probe() {
        let result = run_probe("disk-space").await;
        // May fail in CI without df command
        if result.is_ok() {
            let r = result.unwrap();
            assert_eq!(r.probe, "disk-space");
        }
    }

    #[test]
    fn test_health_summary_serialization() {
        let summary = HealthSummary {
            run_id: "test-123".to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            probes: vec![],
            overall_status: ProbeStatus::Ok,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("test-123"));
    }
}
