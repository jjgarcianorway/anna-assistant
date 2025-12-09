//! Update check loop for automatic updates.
//!
//! Extracted from server.rs (v0.0.159) for modularization.
//! Periodically checks GitHub for new versions and performs auto-updates.

use anna_shared::status::UpdateCheckState;
use anna_shared::update_ledger::{
    load_update_ledger, save_update_ledger, UpdateCheckEntry, UpdateCheckResult,
};
use anna_shared::VERSION;
use chrono::Utc;
use std::time::Instant;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

use crate::state::SharedState;
use crate::update::{check_latest_version, is_newer_version, perform_update};

/// Background loop that periodically checks for updates
pub async fn update_check_loop(state: SharedState) {
    // Get check interval from state
    let check_interval = {
        let state = state.read().await;
        state.update.check_interval_secs
    };

    let mut interval = interval(Duration::from_secs(check_interval));

    // Set initial next_check time
    {
        let mut state = state.write().await;
        state.update.next_check_at =
            Some(Utc::now() + chrono::Duration::seconds(check_interval as i64));
    }

    loop {
        interval.tick().await;

        info!("Checking for updates...");
        let check_start = Instant::now();

        // Check GitHub for latest version
        match check_latest_version().await {
            Ok(latest_version) => {
                handle_successful_check(
                    &state,
                    &latest_version,
                    check_start,
                    check_interval,
                )
                .await;
            }
            Err(e) => {
                handle_failed_check(&state, &e.to_string(), check_start, check_interval).await;
            }
        }
    }
}

/// Handle a successful update check
async fn handle_successful_check(
    state: &SharedState,
    latest_version: &str,
    check_start: Instant,
    check_interval: u64,
) {
    let duration_ms = check_start.elapsed().as_millis() as u64;
    let should_update = is_newer_version(VERSION, latest_version);

    // Write to update ledger
    let ledger_result = if should_update {
        UpdateCheckResult::UpdateAvailable {
            version: latest_version.to_string(),
        }
    } else {
        UpdateCheckResult::UpToDate
    };
    let entry = UpdateCheckEntry::new(VERSION, ledger_result, duration_ms)
        .with_remote_tag(format!("v{}", latest_version));
    let mut ledger = load_update_ledger();
    ledger.push(entry);
    if let Err(e) = save_update_ledger(&ledger) {
        warn!("Failed to save update ledger: {}", e);
    }

    // Update state
    {
        let mut state = state.write().await;
        let now = Utc::now();
        state.update.last_check_at = Some(now);
        state.update.next_check_at =
            Some(now + chrono::Duration::seconds(check_interval as i64));
        state.update.latest_version = Some(latest_version.to_string());
        state.update.latest_checked_at = Some(now);
        state.update.update_available = should_update;
        state.update.check_state = UpdateCheckState::Success;
    }

    if should_update {
        info!("New version available: {} -> {}", VERSION, latest_version);
        try_auto_update(state, latest_version).await;
    } else {
        info!("Already on latest version: {}", VERSION);
    }
}

/// Attempt auto-update if enabled
async fn try_auto_update(state: &SharedState, latest_version: &str) {
    let auto_update_enabled = {
        let state = state.read().await;
        state.update.enabled
    };

    if !auto_update_enabled {
        info!("Auto-update disabled, skipping");
        return;
    }

    info!("Auto-update enabled, performing update...");
    match perform_update(latest_version).await {
        Ok(()) => {
            info!("Update initiated, daemon will restart");
            // Record successful install in ledger
            let entry = UpdateCheckEntry::new(
                VERSION,
                UpdateCheckResult::Installed {
                    version: latest_version.to_string(),
                },
                0,
            );
            let mut ledger = load_update_ledger();
            ledger.push(entry);
            let _ = save_update_ledger(&ledger);
        }
        Err(e) => {
            error!("Auto-update failed: {}", e);
            // Record failure in ledger
            let entry = UpdateCheckEntry::new(
                VERSION,
                UpdateCheckResult::Failed {
                    reason: e.to_string(),
                },
                0,
            );
            let mut ledger = load_update_ledger();
            ledger.push(entry);
            let _ = save_update_ledger(&ledger);

            let mut state = state.write().await;
            state.last_error = Some(format!("Auto-update failed: {}", e));
        }
    }
}

/// Handle a failed update check
async fn handle_failed_check(
    state: &SharedState,
    error_msg: &str,
    check_start: Instant,
    check_interval: u64,
) {
    let duration_ms = check_start.elapsed().as_millis() as u64;
    warn!("Failed to check for updates: {}", error_msg);

    // Record failure in ledger
    let entry = UpdateCheckEntry::new(
        VERSION,
        UpdateCheckResult::Failed {
            reason: error_msg.to_string(),
        },
        duration_ms,
    );
    let mut ledger = load_update_ledger();
    ledger.push(entry);
    let _ = save_update_ledger(&ledger);

    // v0.0.72: On failure, preserve last known version but mark as failed
    let mut state = state.write().await;
    let now = Utc::now();
    state.update.last_check_at = Some(now);
    state.update.next_check_at = Some(now + chrono::Duration::seconds(check_interval as i64));
    state.update.check_state = UpdateCheckState::Failed;
    // NOTE: We do NOT clear latest_version - preserve last known good value
}
