//! Command validation for LLM-suggested commands

/// Validate that a string looks like a valid bash command, not garbage
pub fn is_valid_command(cmd: &str) -> bool {
    let cmd = cmd.trim();

    // Too short or too long
    if cmd.len() < 2 || cmd.len() > 300 {
        return false;
    }

    // Contains LLM prompt tokens or non-ASCII
    if cmd.contains("<|") || cmd.contains("|>") {
        return false;
    }

    // Reject commands with non-ASCII characters (Chinese, etc)
    if !cmd.chars().all(|c| c.is_ascii() || c == '/' || c == '-' || c == '_') {
        return false;
    }

    // Starts with common English words (not commands)
    if starts_with_english_word(cmd) {
        return false;
    }

    // First word should look like a command
    let first_word = cmd.split_whitespace().next().unwrap_or("");
    if first_word.is_empty() {
        return false;
    }

    // Valid command patterns: starts with letter, or ./ or /
    let first_char = first_word.chars().next().unwrap_or(' ');
    if !first_char.is_ascii_alphabetic() && first_char != '.' && first_char != '/' {
        return false;
    }

    // Check if first word exactly matches a valid command
    let base_cmd = first_word.split('/').last().unwrap_or(first_word);
    if is_known_command(base_cmd) {
        return true;
    }

    // Also allow absolute paths to common locations
    first_word.starts_with("/usr/bin/") ||
    first_word.starts_with("/bin/") ||
    first_word.starts_with("/sbin/") ||
    first_word.starts_with("./")
}

/// Check if string starts with common English words (not commands)
fn starts_with_english_word(cmd: &str) -> bool {
    const ENGLISH_STARTS: &[&str] = &[
        "Please", "Could", "Would", "Can", "The", "This", "That", "It", "If",
        "To", "For", "With", "From", "I ", "You", "We", "They", "What", "How",
        "Why", "When", "Where", "Is", "Are", "Was", "Were", "Been", "Being",
        "Have", "Has", "Had", "Do", "Does", "Did", "Will", "Shall", "May",
        "Might", "Must", "Should", "A ", "An ", "Based", "Here", "Let",
    ];

    for word in ENGLISH_STARTS {
        if cmd.starts_with(word) {
            return true;
        }
    }
    false
}

/// Check if command matches known valid commands
fn is_known_command(cmd: &str) -> bool {
    // EXACT valid commands (not prefixes - prevents systemd-analyzeblade)
    // v0.2.6: Expanded command list
    const VALID_COMMANDS: &[&str] = &[
        // Core utils
        "ls", "cat", "head", "tail", "grep", "awk", "sed", "find", "df", "du",
        "wc", "sort", "uniq", "cut", "tr", "tee", "xargs", "basename", "dirname",
        "cp", "mv", "touch", "mkdir", "rm", "ln", "readlink",
        // System info
        "free", "ps", "uptime", "uname", "lscpu", "lspci", "lsblk", "lsusb", "lsof",
        "hostnamectl", "timedatectl", "localectl", "locale", "hwinfo",
        // Storage
        "mount", "umount", "findmnt", "swapon", "swapoff", "mkswap",
        "fdisk", "gdisk", "parted", "blkid", "smartctl", "hdparm",
        "zpool", "zfs", "btrfs", "cryptsetup", "lvm", "mdadm", "lvs", "vgs", "pvs",
        // Systemd
        "systemctl", "journalctl", "systemd-analyze", "loginctl", "coredumpctl",
        // Network
        "ip", "ss", "ping", "curl", "wget", "traceroute", "dig", "nslookup", "host",
        "nmcli", "iwctl", "rfkill", "iw", "ethtool", "netstat", "arp",
        "nft", "iptables", "firewall-cmd", "ufw",
        // Packages
        "pacman", "yay", "paru", "makepkg", "pkgfile", "pacsearch",
        // Hardware
        "nvidia-smi", "glxinfo", "vulkaninfo", "vainfo", "vdpauinfo",
        "lsmod", "modinfo", "modprobe", "dmesg", "sensors", "acpi", "dmidecode",
        "upower", "powertop", "tlp-stat", "cpupower", "turbostat",
        // Audio
        "pactl", "pipewire", "pw-cli", "pw-dump", "wpctl", "aplay", "arecord", "amixer",
        // Display
        "xrandr", "wlr-randr", "swaymsg", "hyprctl", "xdpyinfo", "xwininfo",
        // Users/Auth
        "id", "whoami", "groups", "passwd", "chown", "chmod", "chsh",
        "getent", "last", "lastlog", "w", "who", "users",
        // Environment
        "printenv", "env", "echo", "printf", "test", "true", "false", "set", "export",
        // Other system
        "sudo", "which", "whereis", "file", "type", "stat", "date", "cal",
        "logger", "xdg-open", "fwupdmgr", "bluetoothctl",
        // Printing
        "cupsd", "lpstat", "lpq", "lp", "cancel",
        // Monitoring (interactive but useful output)
        "top", "htop", "btop", "iotop", "nethogs", "iftop",
    ];

    VALID_COMMANDS.contains(&cmd)
}
