//! System information and service-related probe rules.

use crate::deterministic_probes::types::ProbeRule;

pub fn system_rules() -> Vec<ProbeRule> {
    vec![
        // Boot queries
        ProbeRule {
            intent_id: "boot.slow",
            keywords: &["boot", "slow"],
            negative_keywords: &[],
            probes: &["boot_time", "boot_blame", "failed_services"],
            description: "Why is boot slow",
        },
        ProbeRule {
            intent_id: "boot.time",
            keywords: &["boot", "time"],
            negative_keywords: &[],
            probes: &["boot_time", "boot_blame"],
            description: "Boot time analysis",
        },
        ProbeRule {
            intent_id: "boot.analyze",
            keywords: &["boot"],
            negative_keywords: &["loader"],
            probes: &["boot_time", "boot_blame"],
            description: "Boot analysis",
        },
        // Service queries
        ProbeRule {
            intent_id: "services.failed",
            keywords: &["failed", "service"],
            negative_keywords: &[],
            probes: &["failed_services"],
            description: "Failed services",
        },
        ProbeRule {
            intent_id: "services.running",
            keywords: &["running", "service"],
            negative_keywords: &[],
            probes: &["running_services"],
            description: "Running services",
        },
        ProbeRule {
            intent_id: "services.status",
            keywords: &["service", "status"],
            negative_keywords: &[],
            probes: &["running_services", "failed_services"],
            description: "Service status",
        },
        // System info queries
        ProbeRule {
            intent_id: "system.uptime",
            keywords: &["uptime"],
            negative_keywords: &[],
            probes: &["uptime"],
            description: "System uptime",
        },
        ProbeRule {
            intent_id: "system.kernel",
            keywords: &["kernel"],
            negative_keywords: &["install"],
            probes: &["uname", "installed_kernels"],
            description: "Kernel info",
        },
        ProbeRule {
            intent_id: "system.os",
            keywords: &["os", "version"],
            negative_keywords: &[],
            probes: &["os_release", "uname"],
            description: "OS version",
        },
        ProbeRule {
            intent_id: "system.distro",
            keywords: &["distro"],
            negative_keywords: &[],
            probes: &["os_release"],
            description: "Distribution info",
        },
    ]
}
