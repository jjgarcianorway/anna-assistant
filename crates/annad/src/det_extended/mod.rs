//! Extended deterministic answer functions (v0.0.175).
//!
//! Modularized from det_extended.rs (3138 lines) into domain-focused modules.

mod hardware;
mod kernel;
mod meta;
mod network;
mod peripherals;
mod security;
mod services;
mod storage;
mod system;
mod tickets;
mod users;

// Re-export all functions for backwards compatibility
pub use hardware::{
    answer_battery_status, answer_cpu_frequency, answer_cpu_governor, answer_gpu_memory,
    answer_memory_slots, answer_pci_devices, answer_sensors_temp, answer_usb_devices,
};

pub use kernel::{
    answer_dmesg_errors, answer_kernel_cmdline, answer_kernel_modules, answer_kernel_version,
    answer_loaded_firmware, answer_module_params, answer_xorg_log,
};

pub use meta::{answer_config_file_location, answer_meta_small_talk};

pub use network::{
    answer_arp_table, answer_default_gateway, answer_dns_servers, answer_hosts_file,
    answer_ip_routes, answer_listening_ports, answer_network_bonding, answer_network_connectivity,
    answer_network_namespaces, answer_network_stats, answer_wireless_networks,
};

pub use peripherals::{answer_audio_devices, answer_bluetooth_devices, answer_printer_status};

pub use security::{
    answer_apparmor_status, answer_failed_logins, answer_firewall_status, answer_iptables_rules,
    answer_last_logins, answer_selinux_status, answer_ssh_connections, answer_sudoers_info,
    answer_sysctl_settings,
};

pub use services::{
    answer_crontabs, answer_docker_containers, answer_docker_images, answer_loginctl_sessions,
    answer_ntp_status, answer_running_services, answer_systemctl_mask, answer_systemd_journal,
    answer_systemd_paths, answer_systemd_scopes, answer_systemd_slices, answer_systemd_sockets,
    answer_systemd_targets, answer_systemd_timers, answer_systemd_units,
};

pub use storage::{
    answer_block_devices, answer_boot_loader, answer_fstab_entries, answer_installed_kernels,
    answer_lvm_status, answer_mounted_filesystems, answer_raid_status, answer_swap_files,
    answer_systemd_mounts, answer_zfs_status,
};

pub use system::{
    answer_coredump_list, answer_hostname, answer_last_boot, answer_open_files, answer_os_info,
    answer_package_updates, answer_process_tree, answer_swap_info, answer_system_architecture,
    answer_system_load, answer_system_locale, answer_system_uptime, answer_timezone_info,
    answer_tmp_files, answer_virtualization_info,
};

pub use tickets::{answer_staff_roster, answer_ticket_history};

pub use users::{
    answer_available_shells, answer_current_user, answer_environment_variables,
    answer_environment_vars, answer_installed_desktops, answer_logged_in_users, answer_user_groups,
};
