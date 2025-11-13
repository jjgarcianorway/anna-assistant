//! Repair subsystem for corrective actions
//!
//! Phase 0.7: System Guardian - active correction of failed probes
//! Citation: [archwiki:System_maintenance]

mod actions;

pub use actions::{
    bluetooth_service_repair, core_dump_cleanup_repair, disk_space_repair,
    firmware_microcode_repair, journal_cleanup_repair, missing_firmware_repair,
    orphaned_packages_repair, pacman_db_repair, services_failed_repair, tlp_config_repair,
};

use anna_common::ipc::RepairAction;
use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::health::ProbeResult;

/// Repair a specific probe
///
/// # Arguments
/// * `probe` - Probe name (e.g., "disk-space", "pacman-db") or "all" for all failed probes
/// * `dry_run` - If true, simulates repair without executing
/// * `health_results` - Current health probe results (needed for "all" mode)
pub async fn repair_probe(
    probe: &str,
    dry_run: bool,
    health_results: Option<&[ProbeResult]>,
) -> Result<Vec<RepairAction>> {
    info!(
        "Repair requested: probe={}, dry_run={}",
        probe, dry_run
    );

    if probe == "all" {
        // Repair all failed probes
        let results = health_results.context("Health results required for 'all' mode")?;
        repair_all_failed(results, dry_run).await
    } else {
        // Repair single probe
        let action = repair_single_probe(probe, dry_run).await?;
        Ok(vec![action])
    }
}

/// Repair all failed probes
async fn repair_all_failed(
    health_results: &[ProbeResult],
    dry_run: bool,
) -> Result<Vec<RepairAction>> {
    let mut repairs = Vec::new();

    for result in health_results {
        if matches!(result.status, crate::health::ProbeStatus::Fail) {
            info!("Repairing failed probe: {}", result.probe);
            match repair_single_probe(&result.probe, dry_run).await {
                Ok(action) => repairs.push(action),
                Err(e) => {
                    warn!("Failed to repair probe {}: {}", result.probe, e);
                    // Create failed repair action
                    repairs.push(RepairAction {
                        probe: result.probe.clone(),
                        action: format!("repair_{}", result.probe),
                        command: None,
                        exit_code: None,
                        success: false,
                        details: format!("Repair failed: {}", e),
                        citation: result.citation.clone(),
                    });
                }
            }
        }
    }

    Ok(repairs)
}

/// Repair a single probe by name
async fn repair_single_probe(probe: &str, dry_run: bool) -> Result<RepairAction> {
    match probe {
        "disk-space" => disk_space_repair(dry_run).await,
        "pacman-db" => pacman_db_repair(dry_run).await,
        "services-failed" => services_failed_repair(dry_run).await,
        "firmware-microcode" => firmware_microcode_repair(dry_run).await,
        "tlp-config" => tlp_config_repair(dry_run).await,
        "missing-firmware" => missing_firmware_repair(dry_run).await,
        "bluetooth-service" => bluetooth_service_repair(dry_run).await,
        "journal-cleanup" => journal_cleanup_repair(dry_run).await,
        "orphaned-packages" => orphaned_packages_repair(dry_run).await,
        "core-dump-cleanup" => core_dump_cleanup_repair(dry_run).await,
        _ => Err(anyhow::anyhow!("Unknown probe: {}", probe)),
    }
}
