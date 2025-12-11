//! Package routes: updates, install, kernels (v0.0.172).

use anna_shared::probe_spine::{EvidenceKind, ProbeId, RouteCapability};
use anna_shared::rpc::{QueryIntent, SpecialistDomain};

use super::{DeterministicRoute, QueryClass};

/// Build route for package queries
pub fn build_packages_route(class: QueryClass) -> Option<DeterministicRoute> {
    match class {
        QueryClass::PackageUpdates => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["package_updates".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Packages],
                spine_probes: vec![],
            },
        }),

        QueryClass::PackageCount => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec!["pacman_count".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Packages],
                spine_probes: vec![ProbeId::PacmanCount],
            },
        }),

        // v0.0.388: Use installed_packages probe for actual package listing
        QueryClass::InstalledPackagesOverview => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec!["installed_packages".to_string(), "package_count".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Packages],
                spine_probes: vec![],
            },
        }),

        QueryClass::InstalledToolCheck => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["command_v".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::ToolExists],
                spine_probes: vec![],
            },
        }),

        QueryClass::AppAlternatives => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Question,
            probes: vec![],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: false,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::InstalledKernels => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["installed_kernels".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Packages],
                spine_probes: vec![],
            },
        }),

        QueryClass::InstallPackage => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Packages,
            intent: QueryIntent::Request,
            probes: vec![],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: false,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        // v0.0.311: System update action
        QueryClass::SystemUpdate => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Request,
            probes: vec!["package_updates".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: false,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        _ => None,
    }
}
