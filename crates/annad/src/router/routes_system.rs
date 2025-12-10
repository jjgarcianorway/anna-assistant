//! System routes: uptime, users, processes, shell, locale, desktop (v0.0.309).

use anna_shared::probe_spine::{EvidenceKind, RouteCapability};
use anna_shared::rpc::{QueryIntent, SpecialistDomain};

use super::{DeterministicRoute, QueryClass};

/// Build route for system info queries
pub fn build_system_route(class: QueryClass) -> Option<DeterministicRoute> {
    match class {
        QueryClass::SystemUptime => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["uptime".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::LoggedInUsers => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["who".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::BatteryStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["battery".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::SystemLoad => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["load_average".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::LastBoot => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["last_boot".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::Hostname => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["hostname".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::OsInfo => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["os_release".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::CurrentUser => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["current_user".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::SystemArchitecture => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["arch".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::EnvironmentVars => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["env_vars".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::ProcessTree => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["pstree".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Processes],
                spine_probes: vec![],
            },
        }),

        QueryClass::OpenFiles => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["open_files".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::SystemLocale => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["locale".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::TimezoneInfo => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["timedatectl".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::AvailableShells => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["available_shells".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::UserGroups => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["user_groups".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::VirtualizationInfo => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["virtualization_info".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::EnvironmentVariables => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["environment_variables".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::InstalledDesktops => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["installed_desktops".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Packages],
                spine_probes: vec![],
            },
        }),

        // v0.0.309: Desktop wallpaper - checks facts first, asks user if unknown
        QueryClass::DesktopWallpaper => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::System,
            intent: QueryIntent::Question,
            probes: vec!["desktop_wallpaper".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: false, // We check facts first, not probes
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        _ => None,
    }
}
