//! Domain-specific validation thresholds.
//!
//! v0.0.376: Different domains have different validation requirements:
//! - Security: 90 (high stakes, must be accurate)
//! - System: 80 (standard reliability)
//! - Network: 75 (often partial visibility)
//! - Storage: 80 (standard reliability)
//! - Packages: 75 (version info can vary)

use anna_shared::rpc::SpecialistDomain;

use super::types::BASE_ACCEPTABLE_SCORE;

/// v0.0.376: Get domain-specific validation threshold
/// v0.0.405: Expanded for all domains
pub fn domain_threshold(domain: Option<SpecialistDomain>) -> u8 {
    match domain {
        Some(SpecialistDomain::Security) => 90, // Security: high stakes
        Some(SpecialistDomain::System) => 80,   // System: standard
        Some(SpecialistDomain::Storage) => 80,  // Storage: standard
        Some(SpecialistDomain::Network) => 75,  // Network: often partial
        Some(SpecialistDomain::Packages) => 75, // Packages: versions vary
        Some(SpecialistDomain::Boot) => 80,     // Boot: standard
        Some(SpecialistDomain::Services) => 80, // Services: standard
        Some(SpecialistDomain::Audio) => 75,    // Audio: hardware varies
        Some(SpecialistDomain::Display) => 75,  // Display: hardware varies
        Some(SpecialistDomain::Desktop) => 75,  // Desktop: DE-specific
        None => BASE_ACCEPTABLE_SCORE,          // Fallback to base
    }
}
