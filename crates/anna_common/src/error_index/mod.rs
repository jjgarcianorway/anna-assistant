//! Error Index v5.2.3 - Universal Error Inspection
//!
//! v5.2.3: Universal error model for ALL objects:
//! - Services, executables, packages, units, configs, sockets, daemons, timers
//! - Every error grouped by object with dynamic classification
//! - Categories: startup, runtime, config, dependency, intrusion, filesystem, network, permission
//! - No hardcoded categories - inferred from log text and patterns
//! - No generic messages - every line traceable to real events
//!
//! Every object in Anna's knowledge inventory must include:
//! - Errors, Warnings, Failures, Misconfigurations
//! - Permission issues, Runtime crashes, Dependency failures
//! - Missing files or directories, Unexpected exits
//! - Intrusion attempts, System-level faults
//!
//! All captured and shown. No filtering. No guessing. No generic messages.

mod category;
mod error_type;
mod index;
mod log_entry;
mod object_errors;
mod scan_state;
mod severity;
mod summary;

// Re-export all public types
pub use category::LogCategory;
pub use error_type::ErrorType;
pub use index::{ErrorIndex, ERROR_INDEX_PATH};
pub use log_entry::LogEntry;
pub use object_errors::{ObjectErrors, MAX_ERRORS_PER_OBJECT, MAX_LOGS_PER_OBJECT};
pub use scan_state::{LogScanState, LOG_SCAN_STATE_PATH};
pub use severity::LogSeverity;
pub use summary::{GroupedErrorSummary, ObjectErrorEntry, UniversalErrorSummary};
