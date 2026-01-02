//! Intent mapping functions.

use std::collections::HashMap;
use super::types::CanonicalIntent;

/// Get the intent-to-topics mapping
pub fn intent_to_topics() -> &'static HashMap<&'static str, Vec<&'static str>> {
    static MAP: once_cell::sync::Lazy<HashMap<&'static str, Vec<&'static str>>> =
        once_cell::sync::Lazy::new(|| {
            let mut m = HashMap::new();

            // Memory
            m.insert(
                "check_free_ram",
                vec!["ram_usage", "free_command", "proc_meminfo"],
            );
            m.insert(
                "check_swap_presence",
                vec!["swap_configuration", "proc_swaps"],
            );

            // Storage
            m.insert(
                "check_disk_usage",
                vec!["df_command", "disk_usage", "filesystem"],
            );
            m.insert(
                "check_trim_service",
                vec!["fstrim", "systemd_timers", "ssd_trim"],
            );

            // Services
            m.insert(
                "check_failed_services",
                vec!["systemctl_failed", "systemd_units"],
            );
            m.insert("check_service_status", vec!["systemctl", "systemd_units"]);

            // Boot
            m.insert(
                "check_boot_time",
                vec!["systemd_analyze", "boot_performance"],
            );
            m.insert(
                "diagnose_slow_boot",
                vec!["systemd_analyze", "systemd_blame", "boot_performance"],
            );

            // Network
            m.insert(
                "check_network_connectivity",
                vec!["ip_command", "network_interfaces", "ping"],
            );
            m.insert(
                "check_dns_health",
                vec!["dns_resolver", "resolvectl", "nsswitch"],
            );

            // Packages
            m.insert("check_package_installed", vec!["pacman", "package_query"]);
            m.insert("check_updates", vec!["checkupdates", "pacman_sync"]);

            m
        });
    &MAP
}

/// Map translator output to canonical intent
pub fn translator_to_canonical(intent: &str, domain: &str, probes: &[String]) -> CanonicalIntent {
    // First, try to infer from probes (most specific)
    for probe in probes {
        match probe.as_str() {
            "memory_info" | "swap_files" if intent.contains("check") => {
                if probe == "swap_files" {
                    return CanonicalIntent::CheckSwapPresence;
                }
                return CanonicalIntent::CheckFreeRam;
            }
            "disk_usage" => return CanonicalIntent::CheckDiskUsage,
            "failed_services" => return CanonicalIntent::CheckFailedServices,
            "boot_time" => return CanonicalIntent::CheckBootTime,
            "boot_blame" => return CanonicalIntent::DiagnoseSlowBoot,
            "listening_ports" => return CanonicalIntent::CheckListeningPorts,
            "uptime" => return CanonicalIntent::CheckUptime,
            _ => {}
        }
    }

    // Fall back to domain + intent mapping
    match (domain, intent) {
        ("system", "query_metric") if probes.iter().any(|p| p.contains("memory")) => {
            CanonicalIntent::CheckFreeRam
        }
        ("system", "query_metric") => CanonicalIntent::CheckUptime,
        ("storage", "query_metric") => CanonicalIntent::CheckDiskUsage,
        ("services", "check_status") => CanonicalIntent::CheckFailedServices,
        ("boot", "query_metric") => CanonicalIntent::CheckBootTime,
        ("boot", "diagnose") => CanonicalIntent::DiagnoseSlowBoot,
        ("network", "check_status") => CanonicalIntent::CheckNetworkConnectivity,
        ("packages", "check_status") => CanonicalIntent::CheckPackageInstalled,
        (_, "explain") => CanonicalIntent::ExplainConcept,
        _ => CanonicalIntent::GeneralQuery,
    }
}
