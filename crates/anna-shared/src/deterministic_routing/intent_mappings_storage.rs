//! Storage department intent mappings.

use super::intent_mapping::IntentMapping;
use super::intent_schema::{CanonicalIntent, Department};
use std::collections::HashMap;

pub(super) fn register_storage_mappings(mappings: &mut HashMap<CanonicalIntent, IntentMapping>) {
    mappings.insert(
        CanonicalIntent::DiskUsage,
        IntentMapping {
            intent: CanonicalIntent::DiskUsage,
            department: Department::Storage,
            required_probes: vec!["df_h"],
            optional_probes: vec!["lsblk", "du_top_dirs"],
            can_answer_from_probes: true, // Disk % is a direct fact
            description: "Disk usage and free space",
        },
    );

    mappings.insert(
        CanonicalIntent::MountHealth,
        IntentMapping {
            intent: CanonicalIntent::MountHealth,
            department: Department::Storage,
            required_probes: vec!["mount", "findmnt"],
            optional_probes: vec!["fstab_check"],
            can_answer_from_probes: true,
            description: "Mount point health",
        },
    );

    mappings.insert(
        CanonicalIntent::SmartStatus,
        IntentMapping {
            intent: CanonicalIntent::SmartStatus,
            department: Department::Storage,
            required_probes: vec!["smartctl_health"],
            optional_probes: vec!["smartctl_attributes"],
            can_answer_from_probes: true,
            description: "SMART disk health",
        },
    );

    mappings.insert(
        CanonicalIntent::BtrfsHealth,
        IntentMapping {
            intent: CanonicalIntent::BtrfsHealth,
            department: Department::Storage,
            required_probes: vec!["btrfs_fi_show", "btrfs_device_stats"],
            optional_probes: vec!["btrfs_scrub_status"],
            can_answer_from_probes: true,
            description: "Btrfs filesystem health",
        },
    );
}
