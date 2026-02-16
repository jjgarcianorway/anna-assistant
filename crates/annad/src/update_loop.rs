//! Update check loop for automatic updates.

use anna_shared::config::AnnaConfig;
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

        // Check GitHub for latest version (respects update_channel config)
        let config = AnnaConfig::load().unwrap_or_default();
        match check_latest_version(&config.update_channel).await {
            Ok((latest_version, published_at)) => {
                handle_successful_check(
                    &state, &latest_version, &published_at, &config,
                    check_start, check_interval,
                ).await;
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
    published_at: &Option<String>,
    config: &AnnaConfig,
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
        state.update.next_check_at = Some(now + chrono::Duration::seconds(check_interval as i64));
        state.update.latest_version = Some(latest_version.to_string());
        state.update.latest_checked_at = Some(now);
        state.update.update_available = should_update;
        state.update.check_state = UpdateCheckState::Success;
    }

    if should_update {
        info!("New version available: {} -> {}", VERSION, latest_version);
        if release_is_installable(published_at, config) {
            try_auto_update(state, latest_version).await;
        } else {
            info!("Update {} deferred (delay/stagger not elapsed)", latest_version);
        }
    } else {
        info!("Already on latest version: {}", VERSION);
    }
}

/// Returns true if enough time has passed since `published_at` to install the update.
///
/// Delay = `config.update_delay_minutes` + per-node stagger offset.
/// Per-node offset = first 8 hex chars of node_id (u32) mod `update_stagger_minutes`.
/// If `published_at` is None (legacy releases without timestamp), treat as installable.
fn release_is_installable(published_at: &Option<String>, config: &AnnaConfig) -> bool {
    let Some(ts) = published_at else {
        return true; // No timestamp → legacy release, install immediately
    };

    let Ok(pub_time) = chrono::DateTime::parse_from_rfc3339(ts) else {
        warn!("Failed to parse release published_at: {}", ts);
        return true;
    };

    let age_minutes = (chrono::Utc::now() - pub_time.with_timezone(&chrono::Utc)).num_minutes();
    let node_offset = compute_node_stagger(config.update_stagger_minutes);
    let required = config.update_delay_minutes as i64 + node_offset as i64;

    age_minutes >= required
}

/// Compute per-node deterministic stagger offset in minutes.
///
/// Reads the first 8 hex characters of node_id and maps them to 0..stagger_minutes.
/// If stagger_minutes is 0 or node_id is unavailable, returns 0.
fn compute_node_stagger(stagger_minutes: u32) -> u32 {
    if stagger_minutes == 0 {
        return 0;
    }
    let node_id_path = std::path::Path::new("/var/lib/anna/node_id");
    let Ok(content) = std::fs::read_to_string(node_id_path) else {
        return 0;
    };
    let hex8 = content.trim().get(..8).unwrap_or("");
    let Ok(val) = u32::from_str_radix(hex8, 16) else {
        return 0;
    };
    val % stagger_minutes
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

    // Inform user that an update is pending
    {
        let mut s = state.write().await;
        s.init_status = format!(
            "Update available (v{}) — finishing active sessions, then restarting...",
            latest_version
        );
    }

    // Wait for active connections to finish before updating
    info!("Waiting for active connections to drain before update...");
    state.wait_for_connections_to_drain(30).await;

    {
        let mut s = state.write().await;
        s.init_status = format!("Downloading and applying update v{}...", latest_version);
    }

    info!("Performing update to {}...", latest_version);
    match perform_update(latest_version).await {
        Ok(()) => {
            info!("Update to {} complete, daemon will restart", latest_version);
            {
                let mut s = state.write().await;
                s.init_status = format!(
                    "Updated to v{} — restarting in a moment...",
                    latest_version
                );
            }
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

            let mut s = state.write().await;
            s.init_status = "Ready".to_string();
            s.last_error = Some(format!("Auto-update failed: {}", e));
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

    // Update state
    let mut state = state.write().await;
    let now = Utc::now();
    state.update.last_check_at = Some(now);
    state.update.next_check_at = Some(now + chrono::Duration::seconds(check_interval as i64));
    state.update.check_state = UpdateCheckState::Failed;
}
