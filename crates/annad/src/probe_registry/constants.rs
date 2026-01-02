//! Probe ID constants
//!
//! Extracted from probe_registry.rs for modularization.

/// Probe IDs for translator to select from
pub const PROBE_IDS: &[&str] = &[
    "top_memory",      // ps aux --sort=-%mem
    "top_cpu",         // ps aux --sort=-%cpu
    "cpu_info",        // lscpu
    "memory_info",     // free -h
    "disk_usage",      // df -h
    "block_devices",   // lsblk
    "network_addrs",   // ip addr show
    "network_routes",  // ip route
    "listening_ports", // ss -tulpn
    "failed_services", // systemctl --failed
    "system_logs",     // journalctl -p warning..alert -n 200 --no-pager
    // v0.0.35: SystemTriage fast-path probes
    "journal_errors",   // journalctl -p 3 -b --no-pager (errors only)
    "journal_warnings", // journalctl -p 4 -b --no-pager (warnings only)
    "failed_units",     // systemctl --failed --no-pager
    "boot_time",        // systemd-analyze
    "free",             // free -h (alias for memory_info)
    "df",               // df -h (alias for disk_usage)
    // v0.0.318: Config file probes
    "vimrc_content",  // cat ~/.vimrc or ~/.vim/vimrc
    "nvim_config",    // cat ~/.config/nvim/init.lua
    "bashrc_content", // cat ~/.bashrc (first 100 lines)
    "zshrc_content",  // cat ~/.zshrc (first 100 lines)
    // v0.0.321: Hardware acceleration probes
    "gpu_drivers",      // lspci -k | grep VGA + lsmod
    "vaapi_status",     // vainfo
    "vdpau_status",     // vdpauinfo
    "vulkan_status",    // vulkaninfo --summary
    "glxinfo_renderer", // glxinfo | grep renderer
    // v0.0.395: Storage analysis probes (largest folders)
    // v0.0.814: Now drills down into subdirs (max-depth=2) to show actual large folders
    "largest_dirs", // du -h --max-depth=2 /home /var /usr /opt | sort -rh | head -25
    "largest_home", // du -h --max-depth=2 $HOME | sort -rh | head-20
    // v0.0.403: Service status probes
    "bluetooth_service", // systemctl status bluetooth.service
];
