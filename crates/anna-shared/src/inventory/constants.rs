//! Inventory constants (v0.0.188).

/// Default inventory TTL in seconds (10 minutes) - v0.0.41
pub const INVENTORY_TTL_SECS: u64 = 600;

/// VIP tools to check on inventory refresh
pub const VIP_TOOLS: &[&str] = &[
    "vim",
    "vi",
    "nano",
    "emacs",
    "nvim",
    "code",
    "micro", // Editors
    "git",
    "hg",
    "svn", // VCS
    "pacman",
    "yay",
    "paru", // Arch package managers
    "systemctl",
    "journalctl", // Systemd
    "ip",
    "nmcli",
    "iwctl",
    "ping", // Network
    "docker",
    "podman", // Containers
    "ssh",
    "rsync", // Remote
    "python",
    "python3",
    "node",
    "npm",
    "cargo",
    "rustc", // Languages
];

/// Desktop environment packages to detect
pub const DESKTOP_PACKAGES: &[&str] = &[
    "gnome-shell",
    "plasma-desktop",
    "xfce4-session",
    "cinnamon",
    "mate-session-manager",
    "budgie-desktop",
    "lxqt-session",
    "sway",
    "i3",
];
