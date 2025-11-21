//! Intelligence Module - Systems Intelligence and Diagnostics
//!
//! Beta.217a: Sysadmin Brain Foundation
//!
//! Provides deterministic rule-based intelligence for system diagnostics
//! without LLM inference. Pure logic-based insights from telemetry data.

pub mod sysadmin_brain;

pub use sysadmin_brain::{analyze_system_health, DiagnosticInsight, DiagnosticSeverity};
