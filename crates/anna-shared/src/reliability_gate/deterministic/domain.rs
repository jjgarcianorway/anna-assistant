//! Query domain for deterministic routing.

use serde::{Deserialize, Serialize};

/// Query domain for deterministic routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryDomain {
    Ram,
    Cpu,
    Uptime,
    Disk,
    Services,
    Packages,
    Swap,
    Kernel,
    Desktop,
    Network,
    Users,
    Processes,
}

impl QueryDomain {
    /// Parse domain from query.
    pub fn from_query(query: &str) -> Option<Self> {
        let q = query.to_lowercase();

        // RAM/Memory
        if q.contains("ram")
            || q.contains("memory")
            || q.contains("free mem")
            || q.contains("available mem")
        {
            return Some(Self::Ram);
        }

        // CPU
        if q.contains("cpu") || q.contains("processor") || q.contains("cores") {
            return Some(Self::Cpu);
        }

        // Uptime
        if q.contains("uptime") || q.contains("how long") && q.contains("running") {
            return Some(Self::Uptime);
        }

        // Disk
        if q.contains("disk")
            || q.contains("storage")
            || q.contains("filesystem")
            || q.contains("mount")
            || q.contains("partition")
        {
            return Some(Self::Disk);
        }

        // Services
        if q.contains("service")
            || q.contains("daemon")
            || q.contains("systemd")
            || q.contains("unit")
        {
            return Some(Self::Services);
        }

        // Packages
        if q.contains("package")
            || q.contains("installed")
            || q.contains("install")
            || q.contains("pacman")
            || q.contains("apt")
            || q.contains("yum")
        {
            return Some(Self::Packages);
        }

        // Swap
        if q.contains("swap") {
            return Some(Self::Swap);
        }

        // Kernel
        if q.contains("kernel") || q.contains("linux version") || q.contains("uname") {
            return Some(Self::Kernel);
        }

        // Desktop
        if q.contains("desktop")
            || q.contains("de")
            || q.contains("window manager")
            || q.contains("wm")
            || q.contains("gnome")
            || q.contains("kde")
            || q.contains("i3")
            || q.contains("sway")
        {
            return Some(Self::Desktop);
        }

        // Network
        if q.contains("network")
            || q.contains("ip address")
            || q.contains("interface")
            || q.contains("ethernet")
            || q.contains("wifi")
        {
            return Some(Self::Network);
        }

        // Users
        if q.contains("user") || q.contains("logged in") || q.contains("who") {
            return Some(Self::Users);
        }

        // Processes
        if q.contains("process") || q.contains("pid") || q.contains("running program") {
            return Some(Self::Processes);
        }

        None
    }

    /// Get probes for this domain.
    pub fn probes(&self) -> Vec<&'static str> {
        match self {
            Self::Ram => vec!["free -h", "cat /proc/meminfo | head -5"],
            Self::Cpu => vec!["lscpu | head -20", "nproc"],
            Self::Uptime => vec!["uptime -p", "uptime"],
            Self::Disk => vec!["df -h", "lsblk -f"],
            Self::Services => vec![
                "systemctl list-units --failed",
                "systemctl --user list-units --failed",
            ],
            Self::Packages => vec!["pacman -Q 2>/dev/null | wc -l || dpkg -l 2>/dev/null | wc -l"],
            Self::Swap => vec!["swapon --show", "free -h | grep Swap"],
            Self::Kernel => vec!["uname -r", "uname -a"],
            Self::Desktop => vec!["echo $XDG_CURRENT_DESKTOP", "echo $DESKTOP_SESSION"],
            Self::Network => vec!["ip -br addr", "ip route | head -5"],
            Self::Users => vec!["who", "users"],
            Self::Processes => vec!["ps aux --sort=-%mem | head -10"],
        }
    }
}
