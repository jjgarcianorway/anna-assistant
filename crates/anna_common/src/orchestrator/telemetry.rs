//! Telemetry adapter for planner
//!
//! Provides stable, concise schema for system state

/// Minimal telemetry summary for DNS scenario
#[derive(Debug, Clone)]
pub struct TelemetrySummary {
    pub dns_suspected_broken: bool,
    pub network_reachable: bool,
}

impl TelemetrySummary {
    /// Create summary indicating DNS issue with working network
    pub fn dns_issue() -> Self {
        Self {
            dns_suspected_broken: true,
            network_reachable: true,
        }
    }

    /// Create summary indicating healthy system
    pub fn healthy() -> Self {
        Self {
            dns_suspected_broken: false,
            network_reachable: true,
        }
    }
}
