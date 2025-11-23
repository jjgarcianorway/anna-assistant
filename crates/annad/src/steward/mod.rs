//! Steward subsystem for lifecycle management
//!
//! Phase 0.9: System Steward - monitor, heal, and evolve Arch systems post-installation
//! Citation: [archwiki:System_maintenance]

mod audit;
mod health;
pub mod logging;
mod types;
mod update;

pub use types::{AuditReport, HealthReport, HealthStatus, LogIssue, ServiceStatus, UpdateReport, ProactiveIssueSummary};

use anyhow::Result;
use tracing::info;

/// Perform system health check
///
/// Monitors system services, packages, and logs to detect issues.
/// Returns detailed health report with actionable recommendations.
///
/// # Returns
/// * `HealthReport` - Comprehensive system health status
pub async fn check_system_health() -> Result<HealthReport> {
    info!("Performing system health check");
    health::check_health().await
}

/// Perform system update
///
/// Orchestrates pacman updates with signature verification and rollback snapshots.
/// Includes rust package rebuilds and service restarts.
///
/// # Arguments
/// * `dry_run` - If true, simulates update without execution
///
/// # Returns
/// * `UpdateReport` - Detailed update results with package changes
pub async fn perform_system_update(dry_run: bool) -> Result<UpdateReport> {
    info!("Performing system update: dry_run={}", dry_run);
    update::perform_update(dry_run).await
}

/// Perform system audit
///
/// Generates compliance and integrity report including:
/// - Package signature verification
/// - Checksum validation
/// - Service configuration checks
/// - Security baseline compliance
///
/// # Returns
/// * `AuditReport` - Comprehensive audit results
pub async fn perform_system_audit() -> Result<AuditReport> {
    info!("Performing system audit");
    audit::perform_audit().await
}
