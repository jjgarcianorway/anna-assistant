//! Background task spawning and management.
//! Spawns update checks, health checks, snapshots, and telemetry collection.

use std::sync::Arc;

use anna_shared::system_telemetry::TelemetryStore;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::health::health_check_loop;
use crate::snapshot_loop::snapshot_loop;
use crate::state::SharedState;
use crate::telemetry_collector;
use crate::update_loop::update_check_loop;

/// Spawn all background tasks for the daemon.
pub(super) fn spawn_background_tasks(state: SharedState) {
    // v0.0.291: Enhanced background loop lifecycle logging
    // Start update check loop
    let state_clone = state.clone();
    tokio::spawn(async move {
        info!("Background loop started: update_check");
        update_check_loop(state_clone).await;
        // This should never be reached unless loop panics/returns
        error!("Background loop terminated unexpectedly: update_check");
    });

    // Start health check loop
    let state_clone = state.clone();
    tokio::spawn(async move {
        info!("Background loop started: health_check");
        health_check_loop(state_clone).await;
        error!("Background loop terminated unexpectedly: health_check");
    });

    // v0.0.266: Start snapshot collection loop
    tokio::spawn(async move {
        info!("Background loop started: snapshot_collector");
        snapshot_loop().await;
        error!("Background loop terminated unexpectedly: snapshot_collector");
    });

    // v0.0.281: Start telemetry collector
    let telemetry_store = Arc::new(RwLock::new(TelemetryStore::load()));
    telemetry_collector::start_collector(telemetry_store);
    info!("Telemetry collector started");
}
