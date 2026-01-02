//! Security department intent mappings.

use super::intent_mapping::IntentMapping;
use super::intent_schema::{CanonicalIntent, Department};
use std::collections::HashMap;

pub(super) fn register_security_mappings(mappings: &mut HashMap<CanonicalIntent, IntentMapping>) {
    mappings.insert(
        CanonicalIntent::SecurityFirewall,
        IntentMapping {
            intent: CanonicalIntent::SecurityFirewall,
            department: Department::Security,
            required_probes: vec!["firewall_status"],
            optional_probes: vec!["iptables_l", "nft_list"],
            can_answer_from_probes: true,
            description: "Firewall status",
        },
    );

    mappings.insert(
        CanonicalIntent::PermissionCheck,
        IntentMapping {
            intent: CanonicalIntent::PermissionCheck,
            department: Department::Security,
            required_probes: vec![], // Needs path
            optional_probes: vec![],
            can_answer_from_probes: false,
            description: "Permission check",
        },
    );

    mappings.insert(
        CanonicalIntent::VulnCheck,
        IntentMapping {
            intent: CanonicalIntent::VulnCheck,
            department: Department::Security,
            required_probes: vec!["arch_audit"],
            optional_probes: vec![],
            can_answer_from_probes: false, // Needs synthesis
            description: "Vulnerability check",
        },
    );
}
