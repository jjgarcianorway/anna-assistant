// v0.0.693: Settings Ledger Module (Phase 269)
// Modular implementation of the settings ledger system

mod types;
mod entry;
mod ledger;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to maintain the same API
pub use types::{
    LedgerEntryType,
    LedgerStatus,
    LedgerConfig,
    LedgerStats,
};

pub use entry::{
    LedgerEntry,
    LedgerPage,
};

pub use ledger::SettingsLedger;
pub use registry::LedgerRegistry;
pub use helpers::{
    format_ledger_registry,
    is_ledger_query,
    ledger_fun_fact,
};
