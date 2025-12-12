//! Probe primitives library (v0.0.435).
//!
//! A limited set of generic probe primitives - no one-off scripts.
//! New primitives require code changes and should be rare.

use serde::{Deserialize, Serialize};

/// Domain for probe categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    /// Boot and startup.
    Boot,
    /// Systemd services.
    Services,
    /// System logs.
    Logs,
    /// Memory.
    Memory,
    /// Disk and storage.
    Disk,
    /// Network.
    Network,
    /// Hardware.
    Hardware,
    /// Performance.
    Performance,
    /// Desktop environment.
    Desktop,
    /// Packages.
    Packages,
    /// Security.
    Security,
}

impl Domain {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Boot => "boot",
            Self::Services => "services",
            Self::Logs => "logs",
            Self::Memory => "memory",
            Self::Disk => "disk",
            Self::Network => "network",
            Self::Hardware => "hardware",
            Self::Performance => "performance",
            Self::Desktop => "desktop",
            Self::Packages => "packages",
            Self::Security => "security",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "boot" | "startup" => Some(Self::Boot),
            "services" | "systemd" => Some(Self::Services),
            "logs" | "journal" => Some(Self::Logs),
            "memory" | "ram" => Some(Self::Memory),
            "disk" | "storage" => Some(Self::Disk),
            "network" | "net" => Some(Self::Network),
            "hardware" | "hw" => Some(Self::Hardware),
            "performance" | "perf" => Some(Self::Performance),
            "desktop" | "gui" => Some(Self::Desktop),
            "packages" | "pkg" => Some(Self::Packages),
            "security" | "sec" => Some(Self::Security),
            _ => None,
        }
    }
}

/// Parser identifier for probe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParserId {
    /// Raw text, no parsing.
    Raw,
    /// Key-value pairs.
    KeyValue,
    /// Table format.
    Table,
    /// JSON output.
    Json,
    /// Time/duration values.
    TimeDuration,
    /// Numeric values.
    Numeric,
}

/// Precondition for running a probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precondition {
    /// Command must exist.
    CommandExists(&'static str),
    /// File must exist.
    FileExists(&'static str),
    /// Systemd must be running.
    SystemdRunning,
    /// Helper must be installed.
    HelperInstalled(&'static str),
}

impl Precondition {
    /// Check if precondition is met.
    pub fn check(&self) -> bool {
        match self {
            Self::CommandExists(cmd) => {
                std::process::Command::new("which")
                    .arg(cmd)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }
            Self::FileExists(path) => std::path::Path::new(path).exists(),
            Self::SystemdRunning => {
                std::process::Command::new("systemctl")
                    .arg("--version")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }
            Self::HelperInstalled(helper) => {
                // Check if helper command exists
                let cmd = match *helper {
                    "lm_sensors" => "sensors",
                    "smartmontools" => "smartctl",
                    "nvme_cli" => "nvme",
                    "ethtool" => "ethtool",
                    _ => helper,
                };
                std::process::Command::new("which")
                    .arg(cmd)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }
        }
    }
}

/// A probe primitive definition.
#[derive(Debug, Clone)]
pub struct ProbePrimitive {
    /// Unique identifier (e.g., "sys.boot.analyze").
    pub id: &'static str,
    /// Domain this probe belongs to.
    pub domain: Domain,
    /// Human-readable purpose.
    pub purpose: &'static str,
    /// Command template to execute.
    pub command_template: &'static str,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Parser for output.
    pub parser: ParserId,
    /// Preconditions that must be met.
    pub preconditions: &'static [Precondition],
    /// Related man page (for documentation lookup).
    pub related_man: Option<&'static str>,
    /// Keywords for matching.
    pub keywords: &'static [&'static str],
}

impl ProbePrimitive {
    /// Check if all preconditions are met.
    pub fn can_run(&self) -> bool {
        self.preconditions.iter().all(|p| p.check())
    }

    /// Get the command to execute.
    pub fn command(&self) -> String {
        self.command_template.to_string()
    }

    /// Check if primitive matches keywords.
    pub fn matches_keywords(&self, query: &[&str]) -> bool {
        for q in query {
            let q_lower = q.to_lowercase();
            if self.keywords.iter().any(|k| k.contains(&q_lower) || q_lower.contains(*k)) {
                return true;
            }
            if self.purpose.to_lowercase().contains(&q_lower) {
                return true;
            }
        }
        false
    }
}

/// The primitive library.
pub struct PrimitiveLibrary {
    primitives: Vec<ProbePrimitive>,
}

impl PrimitiveLibrary {
    /// Create a new library with default primitives.
    pub fn new() -> Self {
        Self::default_library()
    }

    /// Create the default library.
    pub fn default_library() -> Self {
        Self {
            primitives: vec![
                // === Boot ===
                ProbePrimitive {
                    id: "sys.boot.analyze",
                    domain: Domain::Boot,
                    purpose: "Get boot time breakdown",
                    command_template: "systemd-analyze",
                    timeout_ms: 5000,
                    parser: ParserId::TimeDuration,
                    preconditions: &[Precondition::SystemdRunning],
                    related_man: Some("systemd-analyze"),
                    keywords: &["boot", "startup", "time", "slow"],
                },
                ProbePrimitive {
                    id: "sys.boot.blame",
                    domain: Domain::Boot,
                    purpose: "List units by startup time",
                    command_template: "systemd-analyze blame --no-pager | head -50",
                    timeout_ms: 5000,
                    parser: ParserId::Table,
                    preconditions: &[Precondition::SystemdRunning],
                    related_man: Some("systemd-analyze"),
                    keywords: &["boot", "slow", "units", "blame", "startup"],
                },
                ProbePrimitive {
                    id: "sys.boot.critical",
                    domain: Domain::Boot,
                    purpose: "Show critical chain for boot",
                    command_template: "systemd-analyze critical-chain --no-pager",
                    timeout_ms: 5000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::SystemdRunning],
                    related_man: Some("systemd-analyze"),
                    keywords: &["boot", "critical", "chain", "dependency"],
                },
                // === Services ===
                ProbePrimitive {
                    id: "sys.services.failed",
                    domain: Domain::Services,
                    purpose: "List failed systemd units",
                    command_template: "systemctl --failed --no-pager --plain",
                    timeout_ms: 3000,
                    parser: ParserId::Table,
                    preconditions: &[Precondition::SystemdRunning],
                    related_man: Some("systemctl"),
                    keywords: &["failed", "services", "units", "error"],
                },
                ProbePrimitive {
                    id: "sys.services.list",
                    domain: Domain::Services,
                    purpose: "List running services",
                    command_template: "systemctl list-units --type=service --state=running --no-pager --plain",
                    timeout_ms: 3000,
                    parser: ParserId::Table,
                    preconditions: &[Precondition::SystemdRunning],
                    related_man: Some("systemctl"),
                    keywords: &["services", "running", "active", "list"],
                },
                ProbePrimitive {
                    id: "sys.services.timers",
                    domain: Domain::Services,
                    purpose: "List active timers",
                    command_template: "systemctl list-timers --no-pager --plain",
                    timeout_ms: 3000,
                    parser: ParserId::Table,
                    preconditions: &[Precondition::SystemdRunning],
                    related_man: Some("systemctl"),
                    keywords: &["timers", "scheduled", "cron"],
                },
                // === Logs ===
                ProbePrimitive {
                    id: "sys.logs.errors",
                    domain: Domain::Logs,
                    purpose: "Recent error-level log entries",
                    command_template: "journalctl -p err..alert -n 20 --no-pager",
                    timeout_ms: 5000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::SystemdRunning],
                    related_man: Some("journalctl"),
                    keywords: &["errors", "logs", "journal", "alert"],
                },
                ProbePrimitive {
                    id: "sys.logs.boot",
                    domain: Domain::Logs,
                    purpose: "Current boot log entries",
                    command_template: "journalctl -b -n 50 --no-pager",
                    timeout_ms: 5000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::SystemdRunning],
                    related_man: Some("journalctl"),
                    keywords: &["boot", "logs", "journal", "startup"],
                },
                // === Memory ===
                ProbePrimitive {
                    id: "sys.mem.free",
                    domain: Domain::Memory,
                    purpose: "Memory usage from /proc/meminfo",
                    command_template: "cat /proc/meminfo",
                    timeout_ms: 1000,
                    parser: ParserId::KeyValue,
                    preconditions: &[Precondition::FileExists("/proc/meminfo")],
                    related_man: Some("proc"),
                    keywords: &["memory", "ram", "free", "available"],
                },
                ProbePrimitive {
                    id: "sys.mem.usage",
                    domain: Domain::Memory,
                    purpose: "Memory usage summary",
                    command_template: "free -h",
                    timeout_ms: 1000,
                    parser: ParserId::Table,
                    preconditions: &[Precondition::CommandExists("free")],
                    related_man: Some("free"),
                    keywords: &["memory", "ram", "usage", "swap"],
                },
                // === Disk ===
                ProbePrimitive {
                    id: "sys.disk.df",
                    domain: Domain::Disk,
                    purpose: "Filesystem disk usage",
                    command_template: "df -h",
                    timeout_ms: 2000,
                    parser: ParserId::Table,
                    preconditions: &[Precondition::CommandExists("df")],
                    related_man: Some("df"),
                    keywords: &["disk", "space", "filesystem", "usage"],
                },
                ProbePrimitive {
                    id: "sys.disk.inodes",
                    domain: Domain::Disk,
                    purpose: "Inode usage",
                    command_template: "df -i",
                    timeout_ms: 2000,
                    parser: ParserId::Table,
                    preconditions: &[Precondition::CommandExists("df")],
                    related_man: Some("df"),
                    keywords: &["inodes", "disk", "filesystem"],
                },
                ProbePrimitive {
                    id: "sys.disk.mounts",
                    domain: Domain::Disk,
                    purpose: "Current mounts",
                    command_template: "mount | grep -E '^/dev'",
                    timeout_ms: 2000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::CommandExists("mount")],
                    related_man: Some("mount"),
                    keywords: &["mounts", "disk", "filesystem", "partition"],
                },
                // === Network ===
                ProbePrimitive {
                    id: "net.ip.addr",
                    domain: Domain::Network,
                    purpose: "IP addresses and interfaces",
                    command_template: "ip addr",
                    timeout_ms: 2000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::CommandExists("ip")],
                    related_man: Some("ip-address"),
                    keywords: &["ip", "address", "interface", "network"],
                },
                ProbePrimitive {
                    id: "net.ip.route",
                    domain: Domain::Network,
                    purpose: "Routing table",
                    command_template: "ip route",
                    timeout_ms: 2000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::CommandExists("ip")],
                    related_man: Some("ip-route"),
                    keywords: &["route", "gateway", "network"],
                },
                ProbePrimitive {
                    id: "net.dns.resolv",
                    domain: Domain::Network,
                    purpose: "DNS configuration",
                    command_template: "cat /etc/resolv.conf",
                    timeout_ms: 1000,
                    parser: ParserId::KeyValue,
                    preconditions: &[Precondition::FileExists("/etc/resolv.conf")],
                    related_man: Some("resolv.conf"),
                    keywords: &["dns", "nameserver", "resolver"],
                },
                ProbePrimitive {
                    id: "net.connections",
                    domain: Domain::Network,
                    purpose: "Active network connections",
                    command_template: "ss -tuln",
                    timeout_ms: 3000,
                    parser: ParserId::Table,
                    preconditions: &[Precondition::CommandExists("ss")],
                    related_man: Some("ss"),
                    keywords: &["connections", "ports", "listening", "tcp", "udp"],
                },
                // === Hardware ===
                ProbePrimitive {
                    id: "hw.cpu.info",
                    domain: Domain::Hardware,
                    purpose: "CPU information",
                    command_template: "cat /proc/cpuinfo | head -50",
                    timeout_ms: 1000,
                    parser: ParserId::KeyValue,
                    preconditions: &[Precondition::FileExists("/proc/cpuinfo")],
                    related_man: Some("proc"),
                    keywords: &["cpu", "processor", "cores"],
                },
                ProbePrimitive {
                    id: "hw.cpu.temp",
                    domain: Domain::Hardware,
                    purpose: "CPU temperature",
                    command_template: "sensors 2>/dev/null || cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null",
                    timeout_ms: 3000,
                    parser: ParserId::Raw,
                    preconditions: &[],
                    related_man: Some("sensors"),
                    keywords: &["temperature", "temp", "cpu", "thermal", "hot"],
                },
                ProbePrimitive {
                    id: "hw.pci",
                    domain: Domain::Hardware,
                    purpose: "PCI devices",
                    command_template: "lspci -v | head -100",
                    timeout_ms: 3000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::CommandExists("lspci")],
                    related_man: Some("lspci"),
                    keywords: &["pci", "hardware", "devices", "gpu", "network"],
                },
                ProbePrimitive {
                    id: "hw.usb",
                    domain: Domain::Hardware,
                    purpose: "USB devices",
                    command_template: "lsusb",
                    timeout_ms: 2000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::CommandExists("lsusb")],
                    related_man: Some("lsusb"),
                    keywords: &["usb", "devices", "peripheral"],
                },
                // === Performance ===
                ProbePrimitive {
                    id: "perf.load",
                    domain: Domain::Performance,
                    purpose: "System load averages",
                    command_template: "cat /proc/loadavg",
                    timeout_ms: 1000,
                    parser: ParserId::Numeric,
                    preconditions: &[Precondition::FileExists("/proc/loadavg")],
                    related_man: Some("proc"),
                    keywords: &["load", "average", "performance"],
                },
                ProbePrimitive {
                    id: "perf.uptime",
                    domain: Domain::Performance,
                    purpose: "System uptime",
                    command_template: "uptime",
                    timeout_ms: 1000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::CommandExists("uptime")],
                    related_man: Some("uptime"),
                    keywords: &["uptime", "running", "time"],
                },
                ProbePrimitive {
                    id: "perf.top",
                    domain: Domain::Performance,
                    purpose: "Top processes by CPU",
                    command_template: "ps aux --sort=-%cpu | head -15",
                    timeout_ms: 3000,
                    parser: ParserId::Table,
                    preconditions: &[Precondition::CommandExists("ps")],
                    related_man: Some("ps"),
                    keywords: &["processes", "cpu", "top", "usage"],
                },
                // === Packages ===
                ProbePrimitive {
                    id: "pkg.pacman.recent",
                    domain: Domain::Packages,
                    purpose: "Recently installed packages",
                    command_template: "grep -E 'installed|upgraded' /var/log/pacman.log | tail -20",
                    timeout_ms: 2000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::FileExists("/var/log/pacman.log")],
                    related_man: Some("pacman"),
                    keywords: &["packages", "installed", "recent", "pacman"],
                },
                ProbePrimitive {
                    id: "pkg.pacman.orphans",
                    domain: Domain::Packages,
                    purpose: "Orphaned packages",
                    command_template: "pacman -Qdt",
                    timeout_ms: 5000,
                    parser: ParserId::Raw,
                    preconditions: &[Precondition::CommandExists("pacman")],
                    related_man: Some("pacman"),
                    keywords: &["orphans", "packages", "unused"],
                },
            ],
        }
    }

    /// Get primitive by ID.
    pub fn get(&self, id: &str) -> Option<&ProbePrimitive> {
        self.primitives.iter().find(|p| p.id == id)
    }

    /// Get primitives for a domain.
    pub fn for_domain(&self, domain: Domain) -> Vec<&ProbePrimitive> {
        self.primitives.iter().filter(|p| p.domain == domain).collect()
    }

    /// Find primitives matching keywords.
    pub fn find_by_keywords(&self, keywords: &[&str]) -> Vec<&ProbePrimitive> {
        self.primitives
            .iter()
            .filter(|p| p.matches_keywords(keywords))
            .collect()
    }

    /// Find primitives matching a single keyword.
    pub fn find_by_keyword(&self, keyword: &str) -> Vec<&ProbePrimitive> {
        self.find_by_keywords(&[keyword])
    }

    /// Get all primitive IDs.
    pub fn all_ids(&self) -> Vec<&str> {
        self.primitives.iter().map(|p| p.id).collect()
    }

    /// Check if ID exists.
    pub fn exists(&self, id: &str) -> bool {
        self.primitives.iter().any(|p| p.id == id)
    }

    /// Count primitives.
    pub fn count(&self) -> usize {
        self.primitives.len()
    }
}

impl Default for PrimitiveLibrary {
    fn default() -> Self {
        Self::default_library()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_labels() {
        assert_eq!(Domain::Boot.label(), "boot");
        assert_eq!(Domain::Services.label(), "services");
    }

    #[test]
    fn test_domain_from_str() {
        assert_eq!(Domain::from_str("boot"), Some(Domain::Boot));
        assert_eq!(Domain::from_str("services"), Some(Domain::Services));
        assert_eq!(Domain::from_str("systemd"), Some(Domain::Services));
        assert_eq!(Domain::from_str("unknown"), None);
    }

    #[test]
    fn test_primitive_library() {
        let lib = PrimitiveLibrary::default_library();

        assert!(lib.count() > 0);
        assert!(lib.exists("sys.boot.analyze"));
        assert!(lib.exists("sys.mem.free"));
        assert!(!lib.exists("nonexistent"));
    }

    #[test]
    fn test_find_by_keywords() {
        let lib = PrimitiveLibrary::default_library();

        let boot_probes = lib.find_by_keywords(&["boot", "slow"]);
        assert!(!boot_probes.is_empty());
        assert!(boot_probes.iter().any(|p| p.id == "sys.boot.analyze"));

        let memory_probes = lib.find_by_keywords(&["memory", "ram"]);
        assert!(!memory_probes.is_empty());
    }

    #[test]
    fn test_for_domain() {
        let lib = PrimitiveLibrary::default_library();

        let boot_probes = lib.for_domain(Domain::Boot);
        assert!(!boot_probes.is_empty());
        assert!(boot_probes.iter().all(|p| p.domain == Domain::Boot));
    }

    #[test]
    fn test_primitive_matches_keywords() {
        let lib = PrimitiveLibrary::default_library();
        let probe = lib.get("sys.boot.analyze").unwrap();

        assert!(probe.matches_keywords(&["boot"]));
        assert!(probe.matches_keywords(&["slow"]));
        assert!(!probe.matches_keywords(&["network"]));
    }
}
