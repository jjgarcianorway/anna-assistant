//! Disk and storage-related probe rules.

use crate::deterministic_probes::types::ProbeRule;

pub fn storage_rules() -> Vec<ProbeRule> {
    vec![
        ProbeRule {
            intent_id: "disk.usage",
            keywords: &["disk", "usage"],
            negative_keywords: &[],
            probes: &["disk_usage", "findmnt"],
            description: "Disk usage",
        },
        ProbeRule {
            intent_id: "disk.space",
            keywords: &["disk", "space"],
            negative_keywords: &[],
            probes: &["disk_usage", "findmnt"],
            description: "Disk space",
        },
        ProbeRule {
            intent_id: "disk.free",
            keywords: &["free", "disk"],
            negative_keywords: &[],
            probes: &["disk_usage"],
            description: "Free disk space",
        },
        ProbeRule {
            intent_id: "storage.largest",
            keywords: &["largest", "folder"],
            negative_keywords: &[],
            probes: &["largest_dirs", "largest_home"],
            description: "Largest folders",
        },
        ProbeRule {
            intent_id: "storage.biggest",
            keywords: &["biggest"],
            negative_keywords: &[],
            probes: &["largest_dirs", "largest_home"],
            description: "Biggest folders",
        },
    ]
}
