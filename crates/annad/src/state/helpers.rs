//! Helper functions for state initialization and loading.

use std::sync::Arc;

use anna_shared::ledger::Ledger;
use anna_shared::truth_ledger::TruthLedger;
use tokio::sync::RwLock;
use tracing::info;

use super::types::{DaemonStateInner, SharedState, TRUTH_LEDGER_PATH};

pub async fn load_initial_state() -> SharedState {
    let shared_state = Arc::new(RwLock::new(DaemonStateInner::new()));
    {
        let mut state_write = shared_state.write().await;
        // Load existing ledger if available
        if let Ok(ledger) = Ledger::load() {
            state_write.ledger = ledger;
            info!("Loaded existing ledger");
        }
        // Load existing truth ledger if available
        if let Ok(truth_ledger) = TruthLedger::load(TRUTH_LEDGER_PATH) {
            state_write.truth_ledger = truth_ledger;
            info!("Loaded existing truth ledger");
        }
    }
    shared_state
}
