//! Miscellaneous probe rules (Docker, logs, updates, security).

use crate::deterministic_probes::types::ProbeRule;

pub fn misc_rules() -> Vec<ProbeRule> {
    vec![
        // Docker queries
        ProbeRule {
            intent_id: "docker.containers",
            keywords: &["docker", "container"],
            negative_keywords: &["install"],
            probes: &["docker_containers"],
            description: "Docker containers",
        },
        ProbeRule {
            intent_id: "docker.images",
            keywords: &["docker", "image"],
            negative_keywords: &["install"],
            probes: &["docker_images"],
            description: "Docker images",
        },
        // Updates queries
        ProbeRule {
            intent_id: "updates.available",
            keywords: &["update"],
            negative_keywords: &["install"],
            probes: &["package_updates"],
            description: "Available updates",
        },
        ProbeRule {
            intent_id: "updates.packages",
            keywords: &["packages", "update"],
            negative_keywords: &[],
            probes: &["package_updates"],
            description: "Package updates",
        },
        // Journal/Logs queries
        ProbeRule {
            intent_id: "logs.errors",
            keywords: &["error", "log"],
            negative_keywords: &[],
            probes: &["journal_errors", "dmesg_errors"],
            description: "Error logs",
        },
        ProbeRule {
            intent_id: "logs.warnings",
            keywords: &["warning", "log"],
            negative_keywords: &[],
            probes: &["journal_warnings"],
            description: "Warning logs",
        },
        ProbeRule {
            intent_id: "logs.recent",
            keywords: &["recent", "log"],
            negative_keywords: &[],
            probes: &["systemd_journal"],
            description: "Recent logs",
        },
        // Security queries
        ProbeRule {
            intent_id: "security.firewall",
            keywords: &["firewall"],
            negative_keywords: &[],
            probes: &["firewall_status", "iptables_rules"],
            description: "Firewall status",
        },
        ProbeRule {
            intent_id: "security.logins",
            keywords: &["login", "failed"],
            negative_keywords: &[],
            probes: &["failed_logins", "last_logins"],
            description: "Failed logins",
        },
        ProbeRule {
            intent_id: "security.ssh",
            keywords: &["ssh", "connection"],
            negative_keywords: &[],
            probes: &["ssh_connections", "listening_ports"],
            description: "SSH connections",
        },
    ]
}
