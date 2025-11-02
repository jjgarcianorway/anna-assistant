// Anna v0.12.7 - Signal Handlers Module
// SIGHUP handler for configuration reload

use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{info, warn};

/// Global flag indicating a reload has been requested
#[derive(Clone)]
pub struct ReloadSignal {
    reload_requested: Arc<AtomicBool>,
}

impl ReloadSignal {
    /// Create a new reload signal tracker
    pub fn new() -> Self {
        Self {
            reload_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if reload has been requested
    pub fn is_reload_requested(&self) -> bool {
        self.reload_requested.load(Ordering::Relaxed)
    }

    /// Mark reload as requested
    pub fn request_reload(&self) {
        self.reload_requested.store(true, Ordering::Relaxed);
    }

    /// Clear the reload request flag
    pub fn clear_reload_request(&self) {
        self.reload_requested.store(false, Ordering::Relaxed);
    }

    /// Get a clone of the underlying Arc for sharing
    pub fn clone_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.reload_requested)
    }
}

impl Default for ReloadSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn SIGHUP signal handler task
///
/// This handler sets the reload_requested flag when SIGHUP is received.
/// The main loop or config manager should check this flag periodically
/// and perform the actual reload.
pub fn spawn_sighup_handler(reload_signal: Arc<AtomicBool>) -> Result<()> {
    tokio::spawn(async move {
        // Register signal handler for SIGHUP
        let mut sighup = match signal(SignalKind::hangup()) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to register SIGHUP handler: {}", e);
                return;
            }
        };

        info!("SIGHUP handler registered - listening for reload signals");

        loop {
            // Wait for SIGHUP
            sighup.recv().await;

            info!("SIGHUP received - reload requested");

            // Set reload flag
            reload_signal.store(true, Ordering::Relaxed);
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reload_signal() {
        let signal = ReloadSignal::new();

        // Initially false
        assert!(!signal.is_reload_requested());

        // Set to true
        signal.request_reload();
        assert!(signal.is_reload_requested());

        // Clear back to false
        signal.clear_reload_request();
        assert!(!signal.is_reload_requested());
    }

    #[test]
    fn test_reload_signal_sharing() {
        let signal = ReloadSignal::new();
        let flag = signal.clone_flag();

        // Modify via cloned Arc
        flag.store(true, Ordering::Relaxed);

        // Should be visible via signal
        assert!(signal.is_reload_requested());
    }
}
