//! Follow-up hint generation logic.

use crate::rpc::SpecialistDomain;
use super::types::FollowupHint;
use super::domain_hints::*;

/// Generate follow-up hints based on the query and domain
pub fn generate_followup_hints(
    query: &str,
    domain: SpecialistDomain,
    _answer: &str,
) -> Vec<FollowupHint> {
    let query_lower = query.to_lowercase();
    let mut hints = Vec::new();

    // Domain-specific follow-up suggestions
    match domain {
        SpecialistDomain::Storage => {
            hints.extend(storage_followups(&query_lower));
        }
        SpecialistDomain::System => {
            hints.extend(system_followups(&query_lower));
        }
        SpecialistDomain::Network => {
            hints.extend(network_followups(&query_lower));
        }
        SpecialistDomain::Security => {
            hints.extend(security_followups(&query_lower));
        }
        SpecialistDomain::Packages => {
            hints.extend(package_followups(&query_lower));
        }
        // v0.0.405: New domains - basic hints for now
        SpecialistDomain::Boot => {
            hints.extend(boot_followups(&query_lower));
        }
        SpecialistDomain::Services => {
            hints.extend(services_followups(&query_lower));
        }
        SpecialistDomain::Audio => {
            hints.extend(audio_followups(&query_lower));
        }
        SpecialistDomain::Display => {
            hints.extend(display_followups(&query_lower));
        }
        SpecialistDomain::Desktop => {
            hints.extend(desktop_followups(&query_lower));
        }
    }

    // Sort by relevance and take top 2
    hints.sort_by(|a, b| b.relevance.cmp(&a.relevance));
    hints.truncate(2);
    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_followups() {
        let hints = generate_followup_hints(
            "how much disk space do I have",
            SpecialistDomain::Storage,
            "",
        );
        assert!(!hints.is_empty());
        assert!(hints.iter().any(|h| h.command.is_some()));
    }

    #[test]
    fn test_system_followups() {
        let hints = generate_followup_hints(
            "what processes are using memory",
            SpecialistDomain::System,
            "",
        );
        assert!(!hints.is_empty());
    }
}
