//! Network routes: connectivity, DNS, gateway, ports (v0.0.172).

use anna_shared::probe_spine::{EvidenceKind, RouteCapability};
use anna_shared::rpc::{QueryIntent, SpecialistDomain};

use super::{DeterministicRoute, QueryClass};

/// Build route for network queries
pub fn build_network_route(class: QueryClass) -> Option<DeterministicRoute> {
    match class {
        QueryClass::NetworkConnectivity => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["ping_check".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::ListeningPorts => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["listening_ports".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::DnsServers => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["dns_servers".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::DefaultGateway => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["default_gateway".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::NetworkNamespaces => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["network_namespaces".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::IpRoutes => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["ip_routes".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::ArpTable => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["arp_table".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::WirelessNetworks => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["wireless_networks".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::HostsFile => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["hosts_file".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::NetworkBonding => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["network_bonding".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        QueryClass::NetworkStats => Some(DeterministicRoute {
            class,
            domain: SpecialistDomain::Network,
            intent: QueryIntent::Question,
            probes: vec!["network_stats".to_string()],
            capability: RouteCapability {
                can_answer_deterministically: true,
                evidence_required: true,
                required_evidence: vec![EvidenceKind::Network],
                spine_probes: vec![],
            },
        }),

        _ => None,
    }
}
