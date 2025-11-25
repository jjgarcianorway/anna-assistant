//! Recommendation engine
//!
//! Analyzes system facts and generates actionable advice with Arch Wiki citations.

use anna_common::{Advice, SystemFacts};
use std::path::Path;
use std::process::Command;

// Submodules
mod applications;
mod desktop;
mod development;
mod hardware;
mod misc;
mod network;
mod packages;
mod security;
mod shell;
mod storage;
mod system;
mod telemetry;

/// Helper function to check command usage in shell history
pub(crate) fn check_command_usage(commands: &[&str]) -> usize {
    let mut count = 0;

    // Try to read bash history
    if let Ok(home) = std::env::var("HOME") {
        let bash_history = Path::new(&home).join(".bash_history");
        if let Ok(contents) = std::fs::read_to_string(bash_history) {
            for cmd in commands {
                count += contents.lines().filter(|line| line.contains(cmd)).count();
            }
        }

        // Also try zsh history
        let zsh_history = Path::new(&home).join(".zsh_history");
        if let Ok(contents) = std::fs::read_to_string(zsh_history) {
            for cmd in commands {
                count += contents.lines().filter(|line| line.contains(cmd)).count();
            }
        }
    }

    count
}

/// Helper function to check if a package is installed
pub(crate) fn is_package_installed(package: &str) -> bool {
    Command::new("pacman")
        .args(&["-Qq", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Generate advice based on system facts
pub fn generate_advice(facts: &SystemFacts) -> Vec<Advice> {
    let mut advice = Vec::new();

    // Telemetry-based checks (beta.35+)
    advice.extend(hardware::check_cpu_temperature(facts));
    advice.extend(hardware::check_disk_health(facts));
    advice.extend(telemetry::check_journal_errors(facts));
    advice.extend(telemetry::check_degraded_services(facts));
    advice.extend(telemetry::check_memory_pressure(facts));
    advice.extend(hardware::check_battery_health(facts));
    advice.extend(telemetry::check_service_crashes(facts));
    advice.extend(telemetry::check_kernel_errors(facts));

    // Network health monitoring (beta.96+)
    advice.extend(telemetry::check_network_health(facts));

    // Disk performance monitoring (beta.99+)
    advice.extend(hardware::check_disk_performance(facts));
    advice.extend(hardware::check_disk_space_prediction(facts));

    // RAM monitoring with leak detection (beta.100+)
    advice.extend(hardware::check_ram_health(facts));

    // CPU monitoring with throttling detection (beta.101+)
    advice.extend(hardware::check_cpu_health(facts));

    // Extended telemetry-based checks (beta.43+)
    advice.extend(hardware::check_microcode_updates(facts));
    advice.extend(hardware::check_battery_optimization(facts));
    advice.extend(security::check_backup_system_presence(facts));
    advice.extend(network::check_bluetooth_setup(facts));
    advice.extend(storage::check_ssd_trim_status(facts));
    advice.extend(storage::check_swap_optimization(facts));
    advice.extend(system::check_locale_configuration(facts));
    advice.extend(packages::check_pacman_hooks_recommendations(facts));

    // Environment-specific recommendations (beta.39+)
    advice.extend(hardware::check_hyprland_nvidia_config(facts));
    advice.extend(hardware::check_wayland_nvidia_config(facts));
    advice.extend(desktop::check_window_manager_recommendations(facts));
    advice.extend(desktop::check_desktop_environment_specific(facts));

    // Hardware checks
    advice.extend(hardware::check_microcode(facts));
    advice.extend(hardware::check_gpu_drivers(facts));
    advice.extend(hardware::check_intel_gpu_support(facts));
    advice.extend(hardware::check_amd_gpu_enhancements(facts));
    advice.extend(hardware::check_gpu_enhancements(facts));

    // Package management
    advice.extend(packages::check_orphan_packages(facts));
    advice.extend(packages::check_system_updates());
    advice.extend(packages::check_pacman_config());
    advice.extend(packages::check_flatpak());
    advice.extend(packages::check_aur_helper());
    advice.extend(packages::check_reflector());
    advice.extend(packages::check_aur_helper_safety());

    // Storage and filesystem
    advice.extend(storage::check_btrfs_maintenance(facts));
    advice.extend(system::check_trim_timer(facts));
    advice.extend(storage::check_swap());
    advice.extend(storage::check_ssd_optimizations(facts));
    advice.extend(storage::check_swap_compression());
    advice.extend(storage::check_filesystem_maintenance(facts));
    advice.extend(storage::check_snapshot_systems(facts));

    // System configuration
    advice.extend(telemetry::check_systemd_health());
    advice.extend(system::check_sysctl_parameters());
    advice.extend(system::check_locale_timezone());
    advice.extend(system::check_kernel_parameters());
    advice.extend(system::check_bootloader_optimization());
    advice.extend(system::check_systemd_timers());
    advice.extend(system::check_journal_size());

    // Network
    advice.extend(network::check_network_manager(facts));
    advice.extend(network::check_firewall());
    advice.extend(network::check_ssh_config());
    advice.extend(network::check_bluetooth());
    advice.extend(network::check_wifi_setup());
    advice.extend(network::check_network_quality(facts));
    advice.extend(network::check_dns_configuration());
    advice.extend(network::check_vpn_tools());
    advice.extend(network::check_network_tools());

    // Shell and CLI tools
    advice.extend(shell::check_shell_enhancements(facts));
    advice.extend(shell::check_essential_cli_tools());
    advice.extend(shell::check_cli_tools(facts));
    advice.extend(shell::check_shell_productivity());
    advice.extend(shell::check_shell_alternatives());
    advice.extend(shell::check_compression_advanced());
    advice.extend(shell::check_archive_tools());
    advice.extend(shell::check_documentation_tools());
    advice.extend(shell::check_dotfile_managers());

    // Desktop environment
    advice.extend(desktop::check_status_bar(facts));
    advice.extend(desktop::check_gaming_setup());
    advice.extend(desktop::check_desktop_environment(facts));
    advice.extend(desktop::check_terminal_and_fonts());
    advice.extend(desktop::check_audio_system(facts));
    advice.extend(desktop::check_gamepad_support());
    advice.extend(desktop::check_audio_enhancements());
    advice.extend(desktop::check_gaming_enhancements());
    advice.extend(desktop::check_terminal_multiplexers());
    advice.extend(desktop::check_audio_production());

    // Development tools
    advice.extend(development::check_docker_support(facts));
    advice.extend(development::check_virtualization_support(facts));
    advice.extend(development::check_golang_dev());
    advice.extend(development::check_java_dev());
    advice.extend(development::check_nodejs_dev());
    advice.extend(development::check_cpp_dev());
    advice.extend(development::check_php_dev());
    advice.extend(development::check_ruby_dev());
    advice.extend(misc::check_databases());
    advice.extend(misc::check_web_servers());
    advice.extend(development::check_git_advanced());
    advice.extend(development::check_container_alternatives());
    advice.extend(misc::check_additional_databases());
    advice.extend(development::check_python_tools());
    advice.extend(development::check_rust_tools());
    advice.extend(development::check_container_orchestration());
    advice.extend(development::check_python_enhancements());
    advice.extend(development::check_git_workflow_tools());

    // Applications
    advice.extend(applications::check_screenshot_tools());
    advice.extend(applications::check_media_tools());
    advice.extend(applications::check_text_editors());
    advice.extend(applications::check_mail_clients());
    advice.extend(applications::check_torrent_clients());
    advice.extend(applications::check_office_suite());
    advice.extend(applications::check_graphics_software());
    advice.extend(applications::check_video_editing());
    advice.extend(applications::check_music_players());
    advice.extend(applications::check_pdf_readers());
    advice.extend(applications::check_code_editors());
    advice.extend(applications::check_image_viewers());
    advice.extend(applications::check_communication_apps());
    advice.extend(applications::check_scientific_tools());
    advice.extend(applications::check_3d_tools());
    advice.extend(applications::check_cad_software());
    advice.extend(applications::check_markdown_tools());
    advice.extend(applications::check_note_taking());
    advice.extend(applications::check_browser_recommendations());
    advice.extend(applications::check_screen_recording());

    // Security
    advice.extend(security::check_password_manager());
    advice.extend(security::check_security_tools());
    advice.extend(security::check_security_hardening());
    advice.extend(security::check_backup_solutions());
    advice.extend(security::check_password_managers());

    // Miscellaneous
    advice.extend(misc::check_config_files());
    advice.extend(misc::check_system_monitors());
    advice.extend(misc::check_arch_specific_tools());
    advice.extend(development::check_git_enhancements());
    advice.extend(system::check_firmware_tools());
    advice.extend(misc::check_power_management());
    advice.extend(misc::check_usb_automount());
    advice.extend(misc::check_printer_support());
    advice.extend(misc::check_monitoring_tools());
    advice.extend(system::check_firmware_updates());
    advice.extend(misc::check_laptop_optimizations(facts));
    advice.extend(misc::check_webcam_support());
    advice.extend(misc::check_dual_boot());
    advice.extend(packages::check_pkgbuild_tools());
    advice.extend(hardware::check_disk_management());
    advice.extend(misc::check_monitor_tools());
    advice.extend(misc::check_system_monitoring_advanced());
    advice.extend(misc::check_file_sharing());
    advice.extend(misc::check_cloud_storage());
    advice.extend(misc::check_android_integration());
    advice.extend(misc::check_remote_desktop());

    advice
}
