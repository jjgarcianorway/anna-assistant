//! Telemetry adapter for planner
//!
//! Provides stable, concise schema for system state

/// Service status in telemetry
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub is_failed: bool,
}

/// Telemetry summary for planner
#[derive(Debug, Clone)]
pub struct TelemetrySummary {
    // Network
    pub dns_suspected_broken: bool,
    pub network_reachable: bool,

    // Services (6.2.1)
    pub failed_services: Vec<ServiceStatus>,
}

impl TelemetrySummary {
    /// Create summary indicating DNS issue with working network
    pub fn dns_issue() -> Self {
        Self {
            dns_suspected_broken: true,
            network_reachable: true,
            failed_services: Vec::new(),
        }
    }

    /// Create summary indicating healthy system
    pub fn healthy() -> Self {
        Self {
            dns_suspected_broken: false,
            network_reachable: true,
            failed_services: Vec::new(),
        }
    }

    /// Create summary with a failed service
    pub fn with_failed_service(service_name: &str) -> Self {
        Self {
            dns_suspected_broken: false,
            network_reachable: true,
            failed_services: vec![ServiceStatus {
                name: service_name.to_string(),
                is_failed: true,
            }],
        }
    }
}
