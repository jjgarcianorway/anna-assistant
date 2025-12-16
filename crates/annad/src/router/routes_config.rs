//! Configuration routes: editor, shell, git, service management (v0.0.172).

use anna_shared::probe_spine::{EvidenceKind, ProbeId, RouteCapability};
use anna_shared::rpc::{QueryIntent, SpecialistDomain};

use super::{DeterministicRoute, QueryClass};

/// Build route for configuration/action queries
pub fn build_config_route(class: QueryClass) -> Option<DeterministicRoute> {
    match class {
        QueryClass::ConfigureEditor => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Request,
            probes: vec![
                "command_v_vim".to_string(),
                "command_v_nvim".to_string(),
                "command_v_nano".to_string(),
                "command_v_emacs".to_string(),
                "command_v_micro".to_string(),
                "command_v_helix".to_string(),
                "command_v_hx".to_string(),
                "command_v_code".to_string(),
                "command_v_kate".to_string(),
                "command_v_gedit".to_string(),
            ],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::ToolExists],
                spine_probes: vec![],
            },
        }),

        QueryClass::ConfigureShell => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Request,
            probes: vec![],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: false,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::ConfigureGit => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Request,
            probes: vec![],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: false,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::ManageService => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Request,
            probes: vec![],
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

/// Build route for diagnostic/LLM-required queries
pub fn build_diagnostic_route(class: QueryClass) -> Option<DeterministicRoute> {
    match class {
        QueryClass::SystemSlow => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Investigate,
            probes: vec![
                "top_cpu".to_string(),
                "top_memory".to_string(),
                "disk_usage".to_string(),
            ],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Processes, EvidenceKind::Disk],
                spine_probes: vec![ProbeId::TopCpu, ProbeId::TopMemory, ProbeId::Df],
            },
        }),

        QueryClass::SystemHealthSummary => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec![
                "disk_usage".to_string(),
                "free".to_string(),
                "failed_units".to_string(),
                "top_cpu".to_string(),
            ],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![
                    EvidenceKind::Disk,
                    EvidenceKind::Memory,
                    EvidenceKind::Services,
                    EvidenceKind::Processes,
                ],
                spine_probes: vec![ProbeId::Df, ProbeId::Free, ProbeId::FailedUnits],
            },
        }),

        QueryClass::BootTimeStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["boot_time".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: false,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::BootTime],
                spine_probes: vec![ProbeId::SystemdAnalyze],
            },
        }),

        // v0.0.799: Boot blame for "why is my boot slow?"
        QueryClass::BootBlame => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Boot,
            intent: QueryIntent::Question,
            probes: vec!["boot_time".to_string(), "boot_blame".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::BootTime],
                spine_probes: vec![ProbeId::SystemdAnalyze],
            },
        }),

        QueryClass::SwapInfo => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["free".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Memory],
                spine_probes: vec![ProbeId::Free],
            },
        }),

        _ => None,
    }
}
