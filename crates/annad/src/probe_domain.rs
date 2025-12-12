//! Domain to probes mapping (v0.0.405).
//!
//! Maps each specialist domain to its recommended probes.
//! This is the source of truth for what probes should be run for each domain.

use anna_shared::rpc::SpecialistDomain;

/// Get recommended probes for a domain
/// These are the default probes that should be run for questions in each domain.
/// The translator can add/remove probes based on specific query analysis.
pub fn probes_for_domain(domain: SpecialistDomain) -> Vec<&'static str> {
    match domain {
        SpecialistDomain::System => vec![
            "memory_info",
            "cpu_info",
            "top_cpu",
            "top_memory",
            "failed_services",
            "load_average",
        ],
        SpecialistDomain::Boot => vec![
            "boot_time",
            "boot_blame",
            "failed_services",
            "journal_errors",
        ],
        SpecialistDomain::Services => vec!["failed_services", "running_services", "systemd_units"],
        SpecialistDomain::Network => vec![
            "network_addrs",
            "network_routes",
            "listening_ports",
            "dns_servers",
            "ping_check",
        ],
        SpecialistDomain::Storage => vec!["disk_usage", "block_devices", "findmnt", "largest_dirs"],
        SpecialistDomain::Packages => {
            vec!["installed_packages", "package_count", "package_updates"]
        }
        SpecialistDomain::Audio => vec!["audio_devices", "audio_server", "pactl_cards"],
        SpecialistDomain::Display => vec![
            "gpu_info",
            "display_info",
            "display_server",
            "glxinfo_renderer",
        ],
        SpecialistDomain::Desktop => {
            vec!["desktop_session", "display_server", "installed_desktops"]
        }
        SpecialistDomain::Security => vec![
            "firewall_status",
            "listening_ports",
            "failed_logins",
            "ssh_connections",
        ],
    }
}

/// Get all probes that are valid for a domain (superset for query refinement)
pub fn all_probes_for_domain(domain: SpecialistDomain) -> Vec<&'static str> {
    match domain {
        SpecialistDomain::System => vec![
            "memory_info",
            "cpu_info",
            "top_cpu",
            "top_memory",
            "failed_services",
            "load_average",
            "uptime",
            "uname",
            "pstree",
            "sensors_temp",
            "cpu_frequency",
            "cpu_governor",
        ],
        SpecialistDomain::Boot => vec![
            "boot_time",
            "boot_blame",
            "failed_services",
            "journal_errors",
            "kernel_cmdline",
            "boot_loader",
            "dmesg_errors",
        ],
        SpecialistDomain::Services => vec![
            "failed_services",
            "running_services",
            "systemd_units",
            "systemd_timers",
            "systemd_sockets",
            "systemd_targets",
            "systemctl_mask",
        ],
        SpecialistDomain::Network => vec![
            "network_addrs",
            "network_routes",
            "listening_ports",
            "dns_servers",
            "ping_check",
            "default_gateway",
            "arp_table",
            "network_stats",
            "wireless_networks",
            "network_bonding",
        ],
        SpecialistDomain::Storage => vec![
            "disk_usage",
            "block_devices",
            "findmnt",
            "largest_dirs",
            "largest_home",
            "fstab_entries",
            "lvm_status",
            "raid_status",
            "zfs_status",
            "swap_files",
        ],
        SpecialistDomain::Packages => vec![
            "installed_packages",
            "package_count",
            "package_updates",
            "pacman_count",
        ],
        SpecialistDomain::Audio => vec![
            "audio_devices",
            "audio_server",
            "pactl_cards",
            "lspci_audio",
        ],
        SpecialistDomain::Display => vec![
            "gpu_info",
            "display_info",
            "display_server",
            "glxinfo_renderer",
            "gpu_drivers",
            "vaapi_status",
            "vulkan_status",
            "xorg_log",
            "lspci_gpu",
            "gpu_memory",
        ],
        SpecialistDomain::Desktop => vec![
            "desktop_session",
            "display_server",
            "installed_desktops",
            "desktop_wallpaper",
            "loginctl_sessions",
        ],
        SpecialistDomain::Security => vec![
            "firewall_status",
            "listening_ports",
            "failed_logins",
            "ssh_connections",
            "iptables_rules",
            "last_logins",
            "sudoers_info",
            "selinux_status",
            "apparmor_status",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe_registry::probe_id_to_command;

    #[test]
    fn test_probes_for_domain() {
        // All domains should have at least 2 probes
        for domain in SpecialistDomain::ALL {
            let probes = probes_for_domain(*domain);
            assert!(!probes.is_empty(), "Domain {:?} has no probes", domain);
            assert!(probes.len() >= 2, "Domain {:?} has < 2 probes", domain);
            // All probes should be valid
            for probe in &probes {
                assert!(
                    probe_id_to_command(probe).is_some(),
                    "Probe {} for {:?} is not valid",
                    probe,
                    domain
                );
            }
        }
    }

    #[test]
    fn test_all_probes_for_domain_superset() {
        // all_probes should contain probes_for_domain as subset
        for domain in SpecialistDomain::ALL {
            let basic = probes_for_domain(*domain);
            let all = all_probes_for_domain(*domain);
            for probe in basic {
                assert!(
                    all.contains(&probe),
                    "Basic probe {} not in all probes for {:?}",
                    probe,
                    domain
                );
            }
        }
    }
}
