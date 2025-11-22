//! Debug utilities for developer observability
//!
//! Beta.240: Environment variable controlled debug output
//! These features are UNSTABLE and for developer use only.
//!
//! Supported environment variables:
//! - ANNA_DEBUG_RPC_STATS=1: Print RPC statistics at process exit
//! - ANNA_DEBUG_DIAG_PHRASES=1: Log diagnostic phrase matches

use crate::rpc_client::ConnectionStats;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to check if RPC stats debug is enabled
static RPC_STATS_ENABLED: AtomicBool = AtomicBool::new(false);

/// Global flag to check if diagnostic phrase debug is enabled
static DIAG_PHRASES_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initialize debug flags from environment variables
/// Beta.240: Called once at process start
pub fn init_debug_flags() {
    // Check RPC stats debug flag
    if std::env::var("ANNA_DEBUG_RPC_STATS").is_ok() {
        RPC_STATS_ENABLED.store(true, Ordering::Relaxed);
    }

    // Check diagnostic phrase debug flag
    if std::env::var("ANNA_DEBUG_DIAG_PHRASES").is_ok() {
        DIAG_PHRASES_ENABLED.store(true, Ordering::Relaxed);
    }
}

/// Check if RPC stats debug is enabled
pub fn is_rpc_stats_enabled() -> bool {
    RPC_STATS_ENABLED.load(Ordering::Relaxed)
}

/// Check if diagnostic phrase debug is enabled
pub fn is_diag_phrases_enabled() -> bool {
    DIAG_PHRASES_ENABLED.load(Ordering::Relaxed)
}

/// Print RPC statistics in debug format
/// Beta.240: Developer-only output, unstable format
///
/// Example output:
/// ```
/// [DEBUG][RPC] Connection Statistics:
/// [DEBUG][RPC]   Connections created: 1
/// [DEBUG][RPC]   Connections reused: 5
/// [DEBUG][RPC]   Reconnections attempted: 0
/// [DEBUG][RPC]   Successful reconnects: 0
/// [DEBUG][RPC]   Connect time total: 45ms
/// [DEBUG][RPC]
/// [DEBUG][RPC] RPC Call Statistics:
/// [DEBUG][RPC]   Successful calls: 6
/// [DEBUG][RPC]   Failed calls (total): 0
/// [DEBUG][RPC]     - Daemon unavailable: 0
/// [DEBUG][RPC]     - Permission denied: 0
/// [DEBUG][RPC]     - Timeout: 0
/// [DEBUG][RPC]     - Connection failed: 0
/// [DEBUG][RPC]     - Internal error: 0
/// ```
pub fn print_rpc_stats(stats: &ConnectionStats) {
    if !is_rpc_stats_enabled() {
        return;
    }

    eprintln!("[DEBUG][RPC] ═══════════════════════════════════════");
    eprintln!("[DEBUG][RPC] Connection Statistics:");
    eprintln!("[DEBUG][RPC]   Connections created: {}", stats.connections_created);
    eprintln!("[DEBUG][RPC]   Connections reused: {}", stats.connections_reused);
    eprintln!("[DEBUG][RPC]   Reconnections attempted: {}", stats.reconnections);
    eprintln!("[DEBUG][RPC]   Successful reconnects: {}", stats.successful_reconnects);
    eprintln!("[DEBUG][RPC]   Connect time total: {}μs ({:.2}ms)",
        stats.connect_time_us,
        stats.connect_time_us as f64 / 1000.0);
    eprintln!("[DEBUG][RPC]");

    let total_failed = stats.failed_daemon_unavailable
        + stats.failed_permission_denied
        + stats.failed_timeout
        + stats.failed_connection
        + stats.failed_internal;

    eprintln!("[DEBUG][RPC] RPC Call Statistics:");
    eprintln!("[DEBUG][RPC]   Successful calls: {}", stats.successful_calls);
    eprintln!("[DEBUG][RPC]   Failed calls (total): {}", total_failed);

    if total_failed > 0 {
        eprintln!("[DEBUG][RPC]     - Daemon unavailable: {}", stats.failed_daemon_unavailable);
        eprintln!("[DEBUG][RPC]     - Permission denied: {}", stats.failed_permission_denied);
        eprintln!("[DEBUG][RPC]     - Timeout: {}", stats.failed_timeout);
        eprintln!("[DEBUG][RPC]     - Connection failed: {}", stats.failed_connection);
        eprintln!("[DEBUG][RPC]     - Internal error: {}", stats.failed_internal);
    }

    eprintln!("[DEBUG][RPC] ═══════════════════════════════════════");
}

/// Log a diagnostic phrase match
/// Beta.240: Developer-only logging, unstable format
///
/// Example output:
/// ```
/// [DEBUG][DIAG] matched="is my system ok" pattern="exact_match"
/// ```
pub fn log_diagnostic_phrase_match(matched_phrase: &str, pattern_type: &str) {
    if !is_diag_phrases_enabled() {
        return;
    }

    eprintln!(
        "[DEBUG][DIAG] matched=\"{}\" pattern=\"{}\"",
        matched_phrase, pattern_type
    );
}
