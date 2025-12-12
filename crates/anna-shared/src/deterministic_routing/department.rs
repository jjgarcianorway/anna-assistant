//! Department Ownership Rules (Part F) - v0.0.439.
//!
//! Stop cross-department nonsense. Each department owns specific domains:
//! - Storage: filesystems, mounts, SMART, btrfs, du/df/lsblk
//! - Hardware: CPU/GPU sensors, kernel modules, PCI devices, drivers
//! - Services: systemd services, journald, pacman locks, timers
//! - Performance: CPU/mem top consumers, iowait, boot performance
//! - Network: WiFi disconnects, DNS, routing, DHCP
//! - Desktop: editors, shells, dotfiles, DE/WM configs
//! - Security: firewall, permissions, vuln checks
//!
//! If translator outputs a conflicting department, we override and log.

use super::intent_map::IntentMapTable;
use super::intent_schema::{CanonicalIntent, Department, TicketIntentSchema};

/// Department ownership definition.
#[derive(Debug, Clone)]
pub struct DepartmentOwnership {
    /// Department.
    pub department: Department,
    /// Topics this department owns.
    pub owns_topics: Vec<&'static str>,
    /// Keywords that indicate this department.
    pub keywords: Vec<&'static str>,
}

/// Department ownership rules.
pub struct DepartmentRules {
    /// Ownership definitions.
    ownerships: Vec<DepartmentOwnership>,
    /// Intent map for authoritative lookups.
    intent_map: IntentMapTable,
}

impl DepartmentRules {
    /// Create new rules.
    pub fn new() -> Self {
        let ownerships = vec![
            DepartmentOwnership {
                department: Department::Storage,
                owns_topics: vec![
                    "filesystems",
                    "mounts",
                    "partitions",
                    "SMART",
                    "btrfs",
                    "ext4",
                    "du",
                    "df",
                    "lsblk",
                    "disk space",
                    "storage",
                    "SSD",
                    "HDD",
                    "NVMe",
                ],
                keywords: vec![
                    "disk",
                    "storage",
                    "mount",
                    "partition",
                    "filesystem",
                    "btrfs",
                    "ext4",
                    "xfs",
                    "fat",
                    "ntfs",
                    "smart",
                    "lsblk",
                    "df",
                    "du",
                ],
            },
            DepartmentOwnership {
                department: Department::Hardware,
                owns_topics: vec![
                    "CPU hardware",
                    "GPU sensors",
                    "kernel modules",
                    "PCI devices",
                    "drivers",
                    "temperature",
                    "fans",
                    "USB devices",
                    "audio hardware",
                ],
                keywords: vec![
                    "gpu",
                    "graphics",
                    "driver",
                    "nvidia",
                    "amd",
                    "intel",
                    "radeon",
                    "nouveau",
                    "kernel module",
                    "lspci",
                    "lsusb",
                    "sensors",
                    "temperature",
                    "fan",
                    "hardware",
                    "cpu info",
                    "lscpu",
                    "audio",
                    "sound",
                    "alsa",
                    "pulseaudio",
                    "pipewire",
                ],
            },
            DepartmentOwnership {
                department: Department::Services,
                owns_topics: vec![
                    "systemd services",
                    "journald",
                    "timers",
                    "sockets",
                    "targets",
                    "pacman database locks",
                    "service status",
                ],
                keywords: vec![
                    "service",
                    "systemd",
                    "systemctl",
                    "journalctl",
                    "failed service",
                    "timer",
                    "unit",
                    "daemon",
                    "pacman",
                    "package",
                ],
            },
            DepartmentOwnership {
                department: Department::Performance,
                owns_topics: vec![
                    "CPU load",
                    "memory usage",
                    "top consumers",
                    "iowait",
                    "boot performance",
                    "startup time",
                    "load average",
                    "processes",
                ],
                keywords: vec![
                    "boot",
                    "startup",
                    "slow",
                    "fast",
                    "load",
                    "memory",
                    "ram",
                    "cpu load",
                    "top",
                    "htop",
                    "process",
                    "consumer",
                    "iowait",
                    "performance",
                    "uptime",
                    "free memory",
                    "available memory",
                ],
            },
            DepartmentOwnership {
                department: Department::Network,
                owns_topics: vec![
                    "WiFi",
                    "Ethernet",
                    "DNS",
                    "routing",
                    "DHCP",
                    "IP address",
                    "network interface",
                    "connectivity",
                ],
                keywords: vec![
                    "wifi",
                    "wireless",
                    "ethernet",
                    "network",
                    "dns",
                    "route",
                    "routing",
                    "dhcp",
                    "ip address",
                    "connectivity",
                    "internet",
                    "ping",
                    "nmcli",
                    "networkmanager",
                    "iw",
                    "iwconfig",
                ],
            },
            DepartmentOwnership {
                department: Department::Desktop,
                owns_topics: vec![
                    "editors",
                    "shells",
                    "dotfiles",
                    "DE/WM configs",
                    "themes",
                    "keybindings",
                    "terminal emulators",
                ],
                keywords: vec![
                    "editor",
                    "vim",
                    "neovim",
                    "emacs",
                    "shell",
                    "bash",
                    "zsh",
                    "fish",
                    "dotfile",
                    "config",
                    "theme",
                    "wallpaper",
                    "desktop",
                    "gnome",
                    "kde",
                    "i3",
                    "sway",
                    "hyprland",
                    "terminal",
                ],
            },
            DepartmentOwnership {
                department: Department::Security,
                owns_topics: vec![
                    "firewall",
                    "permissions",
                    "vulnerabilities",
                    "audit",
                    "access control",
                    "encryption",
                ],
                keywords: vec![
                    "firewall",
                    "iptables",
                    "nftables",
                    "ufw",
                    "permission",
                    "chmod",
                    "chown",
                    "security",
                    "vulnerability",
                    "audit",
                    "selinux",
                    "apparmor",
                ],
            },
        ];

        Self {
            ownerships,
            intent_map: IntentMapTable::build(),
        }
    }

    /// Get the authoritative department for an intent.
    /// This is the CANONICAL source - overrides any translator suggestion.
    pub fn get_authoritative_department(&self, intent: CanonicalIntent) -> Department {
        self.intent_map.get_department(intent)
    }

    /// Check if translator department conflicts with canonical mapping.
    pub fn check_conflict(
        &self,
        intent: CanonicalIntent,
        translator_dept: Department,
    ) -> Option<DepartmentConflict> {
        let canonical = self.get_authoritative_department(intent);

        if canonical != translator_dept {
            Some(DepartmentConflict {
                intent,
                translator_suggested: translator_dept,
                canonical_department: canonical,
            })
        } else {
            None
        }
    }

    /// Override translator department if it conflicts with canonical mapping.
    /// Returns the corrected schema and optional conflict log.
    pub fn enforce_ownership(
        &self,
        mut schema: TicketIntentSchema,
    ) -> (TicketIntentSchema, Option<DepartmentConflict>) {
        let canonical = self.get_authoritative_department(schema.intent);

        if schema.department != canonical {
            let conflict = DepartmentConflict {
                intent: schema.intent,
                translator_suggested: schema.department,
                canonical_department: canonical,
            };
            schema.department = canonical;
            (schema, Some(conflict))
        } else {
            (schema, None)
        }
    }

    /// Get department that owns a keyword.
    pub fn department_for_keyword(&self, keyword: &str) -> Option<Department> {
        let keyword_lower = keyword.to_lowercase();
        for ownership in &self.ownerships {
            for kw in &ownership.keywords {
                if keyword_lower.contains(kw) || kw.contains(&keyword_lower) {
                    return Some(ownership.department);
                }
            }
        }
        None
    }

    /// Get ownership info for a department.
    pub fn get_ownership(&self, dept: Department) -> Option<&DepartmentOwnership> {
        self.ownerships.iter().find(|o| o.department == dept)
    }

    /// List all topics owned by a department.
    pub fn topics_for_department(&self, dept: Department) -> Vec<&str> {
        self.ownerships
            .iter()
            .find(|o| o.department == dept)
            .map(|o| o.owns_topics.clone())
            .unwrap_or_default()
    }
}

impl Default for DepartmentRules {
    fn default() -> Self {
        Self::new()
    }
}

/// A conflict between translator suggestion and canonical mapping.
#[derive(Debug, Clone)]
pub struct DepartmentConflict {
    /// The intent in question.
    pub intent: CanonicalIntent,
    /// What translator suggested.
    pub translator_suggested: Department,
    /// The canonical (correct) department.
    pub canonical_department: Department,
}

impl DepartmentConflict {
    /// Format as log message.
    pub fn log_message(&self) -> String {
        format!(
            "[route] Translator suggested {} but mapping says {}, overridden.",
            self.translator_suggested.label(),
            self.canonical_department.label()
        )
    }
}

/// Route result after applying ownership rules.
#[derive(Debug, Clone)]
pub struct RouteResult {
    /// Final schema with correct department.
    pub schema: TicketIntentSchema,
    /// Whether department was overridden.
    pub was_overridden: bool,
    /// Conflict details if overridden.
    pub conflict: Option<DepartmentConflict>,
    /// Required probes from intent map.
    pub required_probes: Vec<String>,
    /// Optional probes from intent map.
    pub optional_probes: Vec<String>,
}

/// Router that applies all rules.
pub struct DeterministicRouter {
    /// Department rules.
    rules: DepartmentRules,
    /// Intent map.
    intent_map: IntentMapTable,
}

impl DeterministicRouter {
    /// Create new router.
    pub fn new() -> Self {
        Self {
            rules: DepartmentRules::new(),
            intent_map: IntentMapTable::build(),
        }
    }

    /// Route a schema, enforcing all rules.
    pub fn route(&self, schema: TicketIntentSchema) -> RouteResult {
        let (corrected, conflict) = self.rules.enforce_ownership(schema);
        let was_overridden = conflict.is_some();

        // Get probes from intent map
        let required_probes = self
            .intent_map
            .get_required_probes(corrected.intent)
            .into_iter()
            .map(String::from)
            .collect();
        let optional_probes = self
            .intent_map
            .get_optional_probes(corrected.intent)
            .into_iter()
            .map(String::from)
            .collect();

        RouteResult {
            schema: corrected,
            was_overridden,
            conflict,
            required_probes,
            optional_probes,
        }
    }

    /// Get department rules.
    pub fn rules(&self) -> &DepartmentRules {
        &self.rules
    }
}

impl Default for DeterministicRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_routes_to_performance() {
        let rules = DepartmentRules::new();
        let dept = rules.get_authoritative_department(CanonicalIntent::BootPerf);
        assert_eq!(dept, Department::Performance);
    }

    #[test]
    fn test_gpu_routes_to_hardware() {
        let rules = DepartmentRules::new();
        assert_eq!(
            rules.get_authoritative_department(CanonicalIntent::GpuInfo),
            Department::Hardware
        );
        assert_eq!(
            rules.get_authoritative_department(CanonicalIntent::GpuDriver),
            Department::Hardware
        );
    }

    #[test]
    fn test_disk_routes_to_storage() {
        let rules = DepartmentRules::new();
        assert_eq!(
            rules.get_authoritative_department(CanonicalIntent::DiskUsage),
            Department::Storage
        );
    }

    #[test]
    fn test_ram_routes_to_performance() {
        let rules = DepartmentRules::new();
        assert_eq!(
            rules.get_authoritative_department(CanonicalIntent::MemStatus),
            Department::Performance
        );
    }

    #[test]
    fn test_conflict_detection() {
        let rules = DepartmentRules::new();

        // Boot should be Performance, not Desktop
        let conflict = rules.check_conflict(CanonicalIntent::BootPerf, Department::Desktop);
        assert!(conflict.is_some());
        let c = conflict.unwrap();
        assert_eq!(c.translator_suggested, Department::Desktop);
        assert_eq!(c.canonical_department, Department::Performance);
    }

    #[test]
    fn test_no_conflict_when_correct() {
        let rules = DepartmentRules::new();

        // GPU to Hardware is correct
        let conflict = rules.check_conflict(CanonicalIntent::GpuInfo, Department::Hardware);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_enforce_ownership_override() {
        let rules = DepartmentRules::new();

        // Translator wrongly says Desktop for boot
        let schema = TicketIntentSchema::new(
            "why is boot slow?",
            CanonicalIntent::BootPerf,
            Department::Desktop, // WRONG
        );

        let (corrected, conflict) = rules.enforce_ownership(schema);
        assert_eq!(corrected.department, Department::Performance); // Fixed
        assert!(conflict.is_some());
    }

    #[test]
    fn test_router_full_flow() {
        let router = DeterministicRouter::new();

        // Wrong department from translator
        let schema = TicketIntentSchema::new(
            "what GPU driver am I using?",
            CanonicalIntent::GpuDriver,
            Department::Storage, // WRONG
        );

        let result = router.route(schema);
        assert!(result.was_overridden);
        assert_eq!(result.schema.department, Department::Hardware);
        assert!(!result.required_probes.is_empty());
    }

    #[test]
    fn test_department_for_keyword() {
        let rules = DepartmentRules::new();

        assert_eq!(
            rules.department_for_keyword("gpu"),
            Some(Department::Hardware)
        );
        assert_eq!(
            rules.department_for_keyword("boot"),
            Some(Department::Performance)
        );
        assert_eq!(
            rules.department_for_keyword("disk"),
            Some(Department::Storage)
        );
        assert_eq!(
            rules.department_for_keyword("firewall"),
            Some(Department::Security)
        );
    }

    #[test]
    fn test_conflict_log_message() {
        let conflict = DepartmentConflict {
            intent: CanonicalIntent::BootPerf,
            translator_suggested: Department::Desktop,
            canonical_department: Department::Performance,
        };

        let msg = conflict.log_message();
        assert!(msg.contains("Desktop"));
        assert!(msg.contains("Performance"));
        assert!(msg.contains("overridden"));
    }
}
