//! Kernel routes: modules, dmesg, journal, sysctl (v0.0.172).

use anna_shared::probe_spine::{EvidenceKind, RouteCapability};
use anna_shared::rpc::{QueryIntent, SpecialistDomain};

use super::{DeterministicRoute, QueryClass};

/// Build route for kernel/system-level queries
pub fn build_kernel_route(class: QueryClass) -> Option<DeterministicRoute> {
    match class {
        QueryClass::KernelModules => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["kernel_modules".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::ModuleParams => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["module_params".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::KernelCmdline => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["kernel_cmdline".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::SysctlSettings => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["sysctl_settings".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::DmesgErrors => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["dmesg_errors".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Journal],
                spine_probes: vec![],
            },
        }),

        QueryClass::SystemdJournal => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["systemd_journal".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Journal],
                spine_probes: vec![],
            },
        }),

        QueryClass::CoredumpList => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["coredump_list".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::BootLoader => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["boot_loader".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::XorgLog => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["xorg_log".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Journal],
                spine_probes: vec![],
            },
        }),

        QueryClass::LoginctlSessions => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["loginctl_sessions".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::NtpStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["ntp_status".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        _ => None,
    }
}
