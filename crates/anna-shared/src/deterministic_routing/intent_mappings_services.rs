//! Services department intent mappings (including package management).

use super::intent_mapping::IntentMapping;
use super::intent_schema::{CanonicalIntent, Department};
use std::collections::HashMap;

pub(super) fn register_services_mappings(mappings: &mut HashMap<CanonicalIntent, IntentMapping>) {
    mappings.insert(
        CanonicalIntent::SvcFailed,
        IntentMapping {
            intent: CanonicalIntent::SvcFailed,
            department: Department::Services,
            required_probes: vec!["systemctl_failed"],
            optional_probes: vec!["journalctl_failed_units"],
            can_answer_from_probes: true, // List of failed services is a fact
            description: "Failed systemd services",
        },
    );

    mappings.insert(
        CanonicalIntent::SvcHealth,
        IntentMapping {
            intent: CanonicalIntent::SvcHealth,
            department: Department::Services,
            required_probes: vec!["systemctl_status_all"],
            optional_probes: vec!["systemctl_list_units"],
            can_answer_from_probes: false, // Needs synthesis for "health"
            description: "Overall service health",
        },
    );

    mappings.insert(
        CanonicalIntent::SvcStatus,
        IntentMapping {
            intent: CanonicalIntent::SvcStatus,
            department: Department::Services,
            required_probes: vec!["systemctl_status"], // Needs service name
            optional_probes: vec!["journalctl_unit"],
            can_answer_from_probes: true,
            description: "Specific service status",
        },
    );

    mappings.insert(
        CanonicalIntent::LogsRecentErrors,
        IntentMapping {
            intent: CanonicalIntent::LogsRecentErrors,
            department: Department::Services,
            required_probes: vec!["journalctl_errors_20"],
            optional_probes: vec!["dmesg_errors"],
            can_answer_from_probes: true,
            description: "Recent error logs",
        },
    );

    mappings.insert(
        CanonicalIntent::TimerStatus,
        IntentMapping {
            intent: CanonicalIntent::TimerStatus,
            department: Department::Services,
            required_probes: vec!["systemctl_list_timers"],
            optional_probes: vec![],
            can_answer_from_probes: true,
            description: "Systemd timer status",
        },
    );

    // Package management (part of Services)
    mappings.insert(
        CanonicalIntent::PkgInventory,
        IntentMapping {
            intent: CanonicalIntent::PkgInventory,
            department: Department::Services,
            required_probes: vec!["pacman_q_count"],
            optional_probes: vec!["pacman_qe"],
            can_answer_from_probes: true,
            description: "Package inventory",
        },
    );

    mappings.insert(
        CanonicalIntent::PkgUpdates,
        IntentMapping {
            intent: CanonicalIntent::PkgUpdates,
            department: Department::Services,
            required_probes: vec!["checkupdates"],
            optional_probes: vec![],
            can_answer_from_probes: true,
            description: "Available package updates",
        },
    );

    mappings.insert(
        CanonicalIntent::PkgSearch,
        IntentMapping {
            intent: CanonicalIntent::PkgSearch,
            department: Department::Services,
            required_probes: vec![], // Needs package name
            optional_probes: vec![],
            can_answer_from_probes: false,
            description: "Package search",
        },
    );
}
