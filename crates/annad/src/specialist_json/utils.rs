//! Utility functions for specialist handling

use anna_shared::rpc::SpecialistDomain;

/// Convert domain enum to string
/// v0.0.405: Expanded for all domains
pub fn domain_to_string(domain: SpecialistDomain) -> &'static str {
    match domain {
        SpecialistDomain::System => "system",
        SpecialistDomain::Boot => "boot",
        SpecialistDomain::Services => "services",
        SpecialistDomain::Network => "network",
        SpecialistDomain::Storage => "storage",
        SpecialistDomain::Packages => "packages",
        SpecialistDomain::Audio => "audio",
        SpecialistDomain::Display => "display",
        SpecialistDomain::Desktop => "desktop",
        SpecialistDomain::Security => "security",
    }
}
