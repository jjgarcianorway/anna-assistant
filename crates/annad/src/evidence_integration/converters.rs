//! Domain and intent conversion utilities (v0.0.410).

use anna_shared::evidence_engine::{EvidenceDomain, EvidenceIntent};
use anna_shared::rpc::SpecialistDomain;

/// Convert SpecialistDomain to EvidenceDomain
pub fn specialist_to_evidence_domain(domain: SpecialistDomain) -> EvidenceDomain {
    match domain {
        SpecialistDomain::System => EvidenceDomain::System,
        SpecialistDomain::Boot => EvidenceDomain::Boot,
        SpecialistDomain::Services => EvidenceDomain::Services,
        SpecialistDomain::Network => EvidenceDomain::Network,
        SpecialistDomain::Storage => EvidenceDomain::Storage,
        SpecialistDomain::Packages => EvidenceDomain::Packages,
        SpecialistDomain::Audio => EvidenceDomain::Audio,
        SpecialistDomain::Display => EvidenceDomain::Display,
        SpecialistDomain::Desktop => EvidenceDomain::Desktop,
        SpecialistDomain::Security => EvidenceDomain::Security,
    }
}

/// Convert query intent string to EvidenceIntent
pub fn query_intent_to_evidence_intent(intent: &str) -> EvidenceIntent {
    match intent.to_lowercase().as_str() {
        "question" | "querymetric" => EvidenceIntent::Diagnose,
        "investigate" | "diagnose" => EvidenceIntent::Diagnose,
        "request" | "configure" => EvidenceIntent::Configure,
        "list" | "checkstatus" => EvidenceIntent::Inspect,
        "explain" => EvidenceIntent::Explain,
        _ => EvidenceIntent::Diagnose,
    }
}
