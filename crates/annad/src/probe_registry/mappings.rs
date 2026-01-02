//! Probe ID to command mappings
//!
//! Extracted from probe_registry.rs for modularization.

/// Map probe ID to actual command
pub fn probe_id_to_command(id: &str) -> Option<&'static str> {
    match id {
        "top_memory" => Some("ps aux --sort=-%mem"),
        "top_cpu" => Some("ps aux --sort=-%cpu"),
        "cpu_info" | "lscpu" => Some("lscpu"),
        "memory_info" | "free" => Some("free -h"),
        "disk_usage" | "df" => Some("df -h"),
        // v0.0.399: Fast largest directories
        // v0.0.808: Use df for overview + fast first-level scan
        // v0.0.814: DRILL DOWN into large directories to show actual content folders
        // User wants to know WHAT is using space, not just that /home is 313G
        "largest_dirs" => Some(
            "echo '=== DISK OVERVIEW ===' && df -h / 2>/dev/null | tail -1 && \
             echo '=== TOP 20 LARGEST FOLDERS ===' && \
             (timeout 8 sh -c 'du -h --max-depth=2 /home /var /usr /opt 2>/dev/null | sort -rh | head -25' || echo 'SCAN_PARTIAL')"
        ),
        // v0.0.814: Now scans deeper into home subdirs
        "largest_home" => Some(
            "echo '=== HOME DIRECTORY ===' && \
             (timeout 5 du -h --max-depth=2 $HOME 2>/dev/null | sort -rh | head -20 || echo 'HOME_TIMEOUT')"
        ),
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
        // v0.0.388: Enhanced with multi-distro support
        "package_updates" => Some(
            "checkupdates 2>/dev/null | head -30 || \
             pacman -Qu 2>/dev/null | head -30 || \
             apt list --upgradable 2>/dev/null | head -30 || \
             dnf check-update 2>/dev/null | head -30 || \
             echo 'NO_UPDATES_AVAILABLE'"
        ),
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
        // v0.0.403: Added bluetooth_service for service status checks
        "bluetooth_service" => Some("systemctl status bluetooth.service 2>&1 | head -20"),
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
        // v0.0.309: Desktop wallpaper probe - tries multiple desktop environments
        "desktop_wallpaper" => Some(
            "gsettings get org.gnome.desktop.background picture-uri 2>/dev/null || \
             gsettings get org.cinnamon.desktop.background picture-uri 2>/dev/null || \
             gsettings get org.mate.background picture-filename 2>/dev/null || \
             qdbus org.kde.plasmashell /PlasmaShell evaluateScript 'print(desktops()[0].wallpaperPlugin)' 2>/dev/null || \
             cat ~/.config/hypr/hyprpaper.conf 2>/dev/null | grep -E '^wallpaper' || \
             echo 'UNKNOWN_DE'"
        ),
        // v0.0.318: Display server detection (Xorg vs Wayland)
        "display_server" => Some(
            "echo \"XDG_SESSION_TYPE=$XDG_SESSION_TYPE\" && \
             echo \"WAYLAND_DISPLAY=$WAYLAND_DISPLAY\" && \
             echo \"DISPLAY=$DISPLAY\" && \
             loginctl show-session $(loginctl | grep $(whoami) | awk '{print $1}') -p Type 2>/dev/null || \
             echo 'session_type=unknown'"
        ),
        // v0.0.318: CUDA and GPU compute detection
        "cuda_installed" => Some(
            "nvcc --version 2>/dev/null || \
             pacman -Q cuda 2>/dev/null || \
             ls /opt/cuda/version.txt 2>/dev/null || \
             echo 'CUDA_NOT_FOUND'"
        ),
        // v0.0.318: Driver detection
        "gpu_drivers" => Some(
            "lspci -k | grep -A 3 -E 'VGA|3D' 2>/dev/null || \
             lsmod | grep -E 'nvidia|amdgpu|i915|nouveau' 2>/dev/null || \
             echo 'NO_GPU_DRIVERS'"
        ),
        // v0.0.318: Config file content probes
        "vimrc_content" => Some(
            "cat ~/.vimrc 2>/dev/null || cat ~/.vim/vimrc 2>/dev/null || cat ~/.config/nvim/init.vim 2>/dev/null || echo 'NO_VIMRC_FOUND'"
        ),
        "nvim_config" => Some(
            "cat ~/.config/nvim/init.lua 2>/dev/null || cat ~/.config/nvim/init.vim 2>/dev/null || echo 'NO_NVIM_CONFIG_FOUND'"
        ),
        "bashrc_content" => Some(
            "cat ~/.bashrc 2>/dev/null | head -100 || echo 'NO_BASHRC_FOUND'"
        ),
        "zshrc_content" => Some(
            "cat ~/.zshrc 2>/dev/null | head -100 || echo 'NO_ZSHRC_FOUND'"
        ),
        // v0.0.321: Hardware acceleration probes (browser/video)
        "vaapi_status" => Some(
            "vainfo 2>/dev/null | head -30 || echo 'VA-API_NOT_AVAILABLE'"
        ),
        "vdpau_status" => Some(
            "vdpauinfo 2>/dev/null | head -30 || echo 'VDPAU_NOT_AVAILABLE'"
        ),
        "vulkan_status" => Some(
            "vulkaninfo --summary 2>/dev/null | head -20 || echo 'VULKAN_NOT_AVAILABLE'"
        ),
        "glxinfo_renderer" => Some(
            "glxinfo 2>/dev/null | grep -E 'OpenGL renderer|OpenGL vendor|direct rendering' | head -5 || echo 'GLX_NOT_AVAILABLE'"
        ),
        "libva_driver" => Some(
            "ls /usr/lib/dri/*_drv_video.so 2>/dev/null | xargs -I{} basename {} _drv_video.so || echo 'NO_VA_DRIVERS'"
        ),
        "firefox_hw_accel" => Some(
            "cat ~/.mozilla/firefox/*/prefs.js 2>/dev/null | grep -E 'webrender|vaapi|gfx.webrender' | head -10 || echo 'FIREFOX_PREFS_NOT_FOUND'"
        ),
        "chromium_gpu_flags" => Some(
            "cat ~/.config/chromium-flags.conf 2>/dev/null || cat ~/.config/chrome-flags.conf 2>/dev/null || echo 'NO_CHROMIUM_FLAGS'"
        ),
        // v0.0.388: Package management probes (distro-aware)
        "installed_packages" => Some(
            "pacman -Q 2>/dev/null | head -50 || \
             dpkg -l 2>/dev/null | grep '^ii' | head -50 || \
             rpm -qa 2>/dev/null | head -50 || \
             apk info 2>/dev/null | head -50 || \
             echo 'PACKAGE_LIST_NOT_AVAILABLE'"
        ),
        "package_count" => Some(
            "pacman -Qq 2>/dev/null | wc -l || \
             dpkg -l 2>/dev/null | grep -c '^ii' || \
             rpm -qa 2>/dev/null | wc -l || \
             apk info 2>/dev/null | wc -l || \
             echo '0'"
        ),
        // v0.0.405: New domain-specific probes
        "boot_blame" => Some("systemd-analyze blame | head -15"),
        "audio_server" => Some(
            "systemctl --user status pipewire pipewire-pulse 2>/dev/null || \
             systemctl --user status pulseaudio 2>/dev/null || \
             pactl info 2>/dev/null | head -10 || \
             echo 'NO_AUDIO_SERVER'"
        ),
        "gpu_info" => Some(
            "lspci | grep -E 'VGA|3D' && \
             (glxinfo 2>/dev/null | grep -E 'OpenGL renderer|vendor' || echo 'NO_GLX') && \
             (nvidia-smi -L 2>/dev/null || echo 'NO_NVIDIA')"
        ),
        "display_info" => Some(
            "xrandr 2>/dev/null || wlr-randr 2>/dev/null || \
             hyprctl monitors 2>/dev/null || \
             echo 'DISPLAY_INFO_NOT_AVAILABLE'"
        ),
        "desktop_session" => Some(
            "echo \"XDG_CURRENT_DESKTOP=$XDG_CURRENT_DESKTOP\" && \
             echo \"XDG_SESSION_DESKTOP=$XDG_SESSION_DESKTOP\" && \
             echo \"DESKTOP_SESSION=$DESKTOP_SESSION\""
        ),
        // v0.0.801: Device type detection (laptop/desktop)
        "hostnamectl" => Some("hostnamectl"),
        // v0.0.802: Webcam/camera detection
        "webcam_devices" => Some(
            "lsusb 2>/dev/null | grep -iE 'webcam|camera|video|cam' || \
             ls /dev/video* 2>/dev/null || \
             v4l2-ctl --list-devices 2>/dev/null || \
             echo 'NO_WEBCAM_FOUND'"
        ),
        // v0.0.805: Screen/display resolution (xrandr with fallbacks for Wayland)
        "xrandr" => Some(
            "xrandr --query 2>/dev/null || \
             wlr-randr 2>/dev/null || \
             hyprctl monitors 2>/dev/null || \
             cat /sys/class/drm/*/modes 2>/dev/null | head -5 || \
             echo 'DISPLAY_INFO_NOT_AVAILABLE'"
        ),
        _ => None,
    }
}
