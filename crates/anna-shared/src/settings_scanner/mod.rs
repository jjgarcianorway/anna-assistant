// v0.0.684: Settings Scanner (Phase 260)
// Scan settings for patterns and anomalies
//
// This module is organized into:
// - types: Core types (ScanType, ScanSeverity, ScannerConfig, ScanFinding, ScanResult, ScannerStats)
// - scanner: Main SettingsScanner implementation
// - registry: ScannerRegistry for managing multiple scanners
// - utils: Utility functions for queries and fun facts
// - tests: Comprehensive test suite

mod types;
mod scanner;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{
    ScanType,
    ScanSeverity,
    ScannerConfig,
    ScanFinding,
    ScanResult,
    ScannerStats,
};

pub use scanner::SettingsScanner;
pub use registry::{ScannerRegistry, format_scanner_registry};
pub use utils::{is_scanner_query, scanner_fun_fact};
