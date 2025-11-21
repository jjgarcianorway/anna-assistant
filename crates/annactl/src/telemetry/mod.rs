//! Telemetry Module - Fetch and manage system telemetry
//!
//! Beta.200: Telemetry-first architecture
//!
//! This module is responsible for:
//! - Fetching real-time telemetry from annad daemon
//! - Caching telemetry data for performance
//! - Providing clean interfaces for telemetry access
//! - Zero hallucinations - all answers based on real data

pub mod fetcher;

// Re-export key types for convenience
pub use fetcher::TelemetryFetcher;
