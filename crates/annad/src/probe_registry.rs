//! Probe ID registry - maps probe IDs to actual commands.
//!
//! Extracted from translator.rs (v0.0.164) for modularization.

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
];

/// Map probe ID to actual command
pub fn probe_id_to_command(id: &str) -> Option<&'static str> {
    match id {
        "top_memory" => Some("ps aux --sort=-%mem"),
        "top_cpu" => Some("ps aux --sort=-%cpu"),
        "cpu_info" | "lscpu" => Some("lscpu"),
        "memory_info" | "free" => Some("free -h"),
        "disk_usage" | "df" => Some("df -h"),
        "block_devices" => Some("lsblk"),
        "network_addrs" => Some("ip addr show"),
        "network_routes" => Some("ip route"),
        "listening_ports" => Some("ss -tulpn"),
        "failed_services" | "failed_units" | "systemctl" => Some("systemctl --failed --no-pager"),
        "system_logs" => Some("journalctl -p warning..alert -n 200 --no-pager"),
        // v0.0.35: SystemTriage fast-path probes
        "journal_errors" => Some("journalctl -p 3 -b --no-pager"),
        "journal_warnings" => Some("journalctl -p 4 -b --no-pager"),
        "boot_time" => Some("systemd-analyze"),
        // v0.45.8: Audio probes
        "lspci_audio" => Some("lspci | grep -i audio"),
        "pactl_cards" => Some("pactl list cards"),
        // v0.0.56: Hardware probes
        "sensors" => Some("sensors"),
        "lspci_gpu" => Some("lspci | grep -i vga"),
        "pacman_count" => Some("pacman -Qq | wc -l"),
        // v0.0.59: Editor probes for ConfigureEditor (expanded list with hx)
        "command_v_vim" => Some("sh -lc 'command -v vim'"),
        "command_v_nvim" => Some("sh -lc 'command -v nvim'"),
        "command_v_nano" => Some("sh -lc 'command -v nano'"),
        "command_v_emacs" => Some("sh -lc 'command -v emacs'"),
        "command_v_micro" => Some("sh -lc 'command -v micro'"),
        "command_v_helix" => Some("sh -lc 'command -v helix'"),
        "command_v_hx" => Some("sh -lc 'command -v hx'"),
        "command_v_code" => Some("sh -lc 'command -v code'"),
        "command_v_kate" => Some("sh -lc 'command -v kate'"),
        "command_v_gedit" => Some("sh -lc 'command -v gedit'"),
        // v0.0.77: System probes
        "uname" => Some("uname -a"),
        // v0.0.122: New system probes
        "package_updates" => Some("checkupdates 2>/dev/null || pacman -Qu 2>/dev/null"),
        "timedatectl" => Some("timedatectl"),
        "uptime" => Some("uptime -p"),
        // v0.0.123: New system probes
        "who" => Some("who"),
        "battery" => Some("upower -i $(upower -e | grep battery) 2>/dev/null || cat /sys/class/power_supply/BAT*/capacity 2>/dev/null"),
        "load_average" => Some("cat /proc/loadavg"),
        "last_boot" => Some("who -b"),
        // v0.0.124: New system probes
        "hostname" => Some("hostname"),
        "os_release" => Some("cat /etc/os-release"),
        "ping_check" => Some("ping -c 1 -W 2 8.8.8.8 2>/dev/null"),
        "findmnt" => Some("findmnt -l -o TARGET,SOURCE,FSTYPE,SIZE,USED -t notmpfs,nodevtmpfs,nosquashfs"),
        "lsusb" => Some("lsusb"),
        // v0.0.125: New system probes
        "running_services" => Some("systemctl list-units --type=service --state=running --no-pager --no-legend | head -30"),
        "current_user" => Some("id"),
        "arch" => Some("uname -m"),
        "env_vars" => Some("env | head -30"),
        // v0.0.126: New system and network probes
        "pstree" => Some("pstree -p 2>/dev/null | head -40"),
        "dns_servers" => Some("cat /etc/resolv.conf 2>/dev/null | grep -E '^nameserver'"),
        "default_gateway" => Some("ip route | grep default | head -1"),
        "open_files" => Some("lsof 2>/dev/null | wc -l"),
        "locale" => Some("locale"),
        // v0.0.127: Hardware and storage probes
        "installed_kernels" => Some("pacman -Q linux linux-lts linux-zen linux-hardened 2>/dev/null || ls /boot/vmlinuz-* 2>/dev/null"),
        "cpu_frequency" => Some("cat /proc/cpuinfo | grep 'cpu MHz' | head -1 || lscpu | grep 'CPU MHz'"),
        "memory_slots" => Some("sudo dmidecode -t memory 2>/dev/null | grep -E 'Size:|Locator:|Type:' | head -20 || echo 'Requires root access'"),
        "zfs_status" => Some("zpool status 2>/dev/null || echo 'ZFS not installed'"),
        // v0.0.128: Security and admin probes
        "boot_loader" => Some("bootctl status 2>/dev/null || cat /boot/grub/grub.cfg 2>/dev/null | head -10 || echo 'Boot loader not detected'"),
        "firewall_status" => Some("iptables -L -n 2>/dev/null | head -20 || nft list ruleset 2>/dev/null | head -20 || ufw status 2>/dev/null || echo 'No firewall detected'"),
        "systemd_units" => Some("systemctl list-units --no-pager --no-legend | head -30"),
        "crontabs" => Some("crontab -l 2>/dev/null || echo 'No crontab for current user'"),
        "ssh_connections" => Some("who | grep -E 'pts|tty' 2>/dev/null || ss -tn state established '( dport = :22 or sport = :22 )' 2>/dev/null | head -10"),
        // v0.0.129: Docker and logging probes
        "docker_containers" => Some("docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Image}}' 2>/dev/null || echo 'Docker not available'"),
        "docker_images" => Some("docker images --format 'table {{.Repository}}\t{{.Tag}}\t{{.Size}}' 2>/dev/null | head -20 || echo 'Docker not available'"),
        "systemd_timers" => Some("systemctl list-timers --no-pager --no-legend | head -20"),
        "last_logins" => Some("last -n 10 2>/dev/null || echo 'Login history not available'"),
        "failed_logins" => Some("journalctl -u sshd --no-pager -n 20 2>/dev/null | grep -i 'failed\\|invalid' | head -10 || lastb -n 10 2>/dev/null || echo 'Failed login data not available'"),
        // v0.0.130: System and security probes
        "systemd_journal" => Some("journalctl -n 30 --no-pager 2>/dev/null || echo 'Journal not available'"),
        "network_namespaces" => Some("ip netns list 2>/dev/null || echo 'No network namespaces'"),
        "available_shells" => Some("cat /etc/shells 2>/dev/null || echo 'Shell list not available'"),
        "sudoers_info" => Some("sudo -l 2>/dev/null || echo 'Sudo access not available'"),
        "installed_desktops" => Some("pacman -Qs 'gnome-shell\\|plasma-desktop\\|xfce4-session\\|cinnamon\\|mate-session\\|budgie-desktop\\|lxqt-session\\|sway\\|hyprland' 2>/dev/null | grep -E '^local/' | head -10 || echo 'Desktop info not available'"),
        // v0.0.131: Virtualization and security probes
        "virtualization_info" => Some("systemd-detect-virt 2>/dev/null || echo 'none'"),
        "selinux_status" => Some("sestatus 2>/dev/null || echo 'SELinux not installed'"),
        "apparmor_status" => Some("aa-status 2>/dev/null || cat /sys/module/apparmor/parameters/enabled 2>/dev/null || echo 'AppArmor not installed'"),
        "systemd_slices" => Some("systemd-cgls --no-pager 2>/dev/null | head -40 || echo 'Cgroups not available'"),
        "coredump_list" => Some("coredumpctl list --no-pager 2>/dev/null | head -20 || echo 'No coredumps or coredumpctl not available'"),
        // v0.0.132: Kernel and network probes
        "kernel_modules" => Some("lsmod | head -30"),
        "systemd_targets" => Some("systemctl list-units --type=target --no-pager --no-legend | head -20"),
        "ip_routes" => Some("ip route show 2>/dev/null || route -n 2>/dev/null"),
        "arp_table" => Some("ip neigh show 2>/dev/null || arp -a 2>/dev/null"),
        "iptables_rules" => Some("iptables -L -n --line-numbers 2>/dev/null | head -40 || echo 'iptables not available or requires root'"),
        // v0.0.133: System and user probes
        "pci_devices" => Some("lspci 2>/dev/null | head -30"),
        "dmesg_errors" => Some("dmesg --level=err,warn 2>/dev/null | tail -20 || dmesg | grep -iE 'error|warn|fail' | tail -20"),
        "systemd_sockets" => Some("systemctl list-sockets --no-pager --no-legend | head -20"),
        "tmp_files" => Some("ls -la /tmp 2>/dev/null | head -30"),
        "user_groups" => Some("groups && id"),
        // v0.0.134: Storage and hardware probes
        "lvm_status" => Some("lvs 2>/dev/null && vgs 2>/dev/null || echo 'LVM not installed or no volumes'"),
        "raid_status" => Some("cat /proc/mdstat 2>/dev/null || mdadm --detail --scan 2>/dev/null || echo 'No RAID detected'"),
        "ntp_status" => Some("timedatectl show 2>/dev/null || chronyc tracking 2>/dev/null || ntpq -p 2>/dev/null || echo 'NTP status not available'"),
        "sensors_temp" => Some("sensors 2>/dev/null || echo 'lm-sensors not installed'"),
        "gpu_memory" => Some("nvidia-smi --query-gpu=memory.total,memory.used,memory.free --format=csv 2>/dev/null || echo 'nvidia-smi not available'"),
        "xorg_log" => Some("tail -50 /var/log/Xorg.0.log 2>/dev/null | grep -iE 'error|warn|EE|WW' | head -20 || echo 'Xorg log not found'"),
        // v0.0.135: Peripheral and audio probes
        "bluetooth_devices" => Some("bluetoothctl devices 2>/dev/null || echo 'Bluetooth not available'"),
        "wireless_networks" => Some("nmcli device wifi list 2>/dev/null | head -20 || iwlist scan 2>/dev/null | grep -E 'ESSID|Quality' | head -20 || echo 'WiFi scanning not available'"),
        "printer_status" => Some("lpstat -p -d 2>/dev/null || echo 'No printers configured'"),
        "audio_devices" => Some("pactl list sinks short 2>/dev/null && pactl list sources short 2>/dev/null || aplay -l 2>/dev/null || echo 'Audio devices not available'"),
        "systemd_paths" => Some("systemctl list-units --type=path --no-pager --no-legend | head -20"),
        // v0.0.136: System configuration probes
        "systemctl_mask" => Some("systemctl list-unit-files --state=masked --no-pager --no-legend | head -20"),
        "hosts_file" => Some("cat /etc/hosts 2>/dev/null | grep -v '^#' | grep -v '^$' | head -30"),
        "fstab_entries" => Some("cat /etc/fstab 2>/dev/null | grep -v '^#' | grep -v '^$'"),
        "sysctl_settings" => Some("sysctl -a 2>/dev/null | head -40 || cat /etc/sysctl.conf 2>/dev/null"),
        "loginctl_sessions" => Some("loginctl list-sessions --no-pager --no-legend 2>/dev/null || echo 'loginctl not available'"),
        // v0.0.139: System and network probes
        "environment_variables" => Some("printenv | sort | head -50"),
        "systemd_scopes" => Some("systemctl list-units --type=scope --no-pager --no-legend | head -20"),
        "kernel_cmdline" => Some("cat /proc/cmdline"),
        "module_params" => Some("lsmod | head -20 | awk 'NR>1 {print $1}' | xargs -I{} sh -c 'echo \"=== {} ===\"; modinfo {} 2>/dev/null | grep -E \"^(parm|description):\" | head -5'"),
        "network_bonding" => Some("cat /proc/net/bonding/* 2>/dev/null || echo 'No network bonding configured'"),
        // v0.0.141: System and network probes
        "swap_files" => Some("cat /proc/swaps"),
        "cpu_governor" => Some("cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null | sort | uniq -c || echo 'CPU frequency scaling not available'"),
        "systemd_mounts" => Some("systemctl list-units --type=mount --no-pager --no-legend | head -20"),
        "loaded_firmware" => Some("dmesg 2>/dev/null | grep -i 'firmware\\|microcode' | tail -20 || echo 'Firmware info not available'"),
        "network_stats" => Some("cat /proc/net/dev"),
        _ => None,
    }
}

/// Filter probe IDs to only valid ones
pub fn filter_valid_probes(probes: Vec<String>) -> Vec<String> {
    probes
        .into_iter()
        .filter(|p| PROBE_IDS.contains(&p.as_str()) || probe_id_to_command(p).is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_id_to_command() {
        assert_eq!(
            probe_id_to_command("top_memory"),
            Some("ps aux --sort=-%mem")
        );
        assert_eq!(probe_id_to_command("invalid"), None);
    }

    #[test]
    fn test_filter_valid_probes() {
        let probes = vec![
            "top_memory".to_string(),
            "invalid".to_string(),
            "cpu_info".to_string(),
        ];
        let filtered = filter_valid_probes(probes);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&"top_memory".to_string()));
        assert!(!filtered.contains(&"invalid".to_string()));
    }

    #[test]
    fn test_probe_ids_list() {
        // Ensure PROBE_IDS contains the core probes
        assert!(PROBE_IDS.contains(&"top_memory"));
        assert!(PROBE_IDS.contains(&"cpu_info"));
        assert!(PROBE_IDS.contains(&"memory_info"));
        assert!(PROBE_IDS.contains(&"disk_usage"));
    }
}
