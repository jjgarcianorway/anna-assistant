//! Security routes: firewall, logins, SSH, SELinux (v0.0.172).

use anna_shared::probe_spine::{EvidenceKind, RouteCapability};
use anna_shared::rpc::{QueryIntent, SpecialistDomain};

use super::{DeterministicRoute, QueryClass};

/// Build route for security queries
pub fn build_security_route(class: QueryClass) -> Option<DeterministicRoute> {
    match class {
        QueryClass::FirewallStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Security,
            intent: QueryIntent::Question,
            probes: vec!["firewall_status".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::SshConnections => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Security,
            intent: QueryIntent::Question,
            probes: vec!["ssh_connections".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::LastLogins => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Security,
            intent: QueryIntent::Question,
            probes: vec!["last_logins".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::FailedLogins => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Security,
            intent: QueryIntent::Question,
            probes: vec!["failed_logins".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::SudoersInfo => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Security,
            intent: QueryIntent::Question,
            probes: vec!["sudoers_info".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::SelinuxStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Security,
            intent: QueryIntent::Question,
            probes: vec!["selinux_status".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::AppArmorStatus => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Security,
            intent: QueryIntent::Question,
            probes: vec!["apparmor_status".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::IptablesRules => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Security,
            intent: QueryIntent::Question,
            probes: vec!["iptables_rules".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![],
                spine_probes: vec![],
            },
        }),

        QueryClass::SshKeyManagement => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Security,
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
