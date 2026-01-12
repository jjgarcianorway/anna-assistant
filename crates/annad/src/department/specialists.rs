//! IT Department Specialists - Named team members.
//! v0.0.999: Initial implementation
//!
//! Each specialist has a name, role, and area of expertise.
//! Juniors use lighter/faster models, seniors use deeper thinking models.

use serde::{Deserialize, Serialize};

/// A specialist in the IT department
#[derive(Debug, Clone)]
pub struct Specialist {
    pub id: &'static str,
    pub name: &'static str,
    pub role: SpecialistRole,
    pub department: &'static str,
    pub expertise: &'static [&'static str],
    /// Model preference: "fast" for juniors, "deep" for seniors
    pub model_tier: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpecialistRole {
    Junior,
    Senior,
    Manager,
}

impl std::fmt::Display for SpecialistRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialistRole::Junior => write!(f, "Junior"),
            SpecialistRole::Senior => write!(f, "Senior"),
            SpecialistRole::Manager => write!(f, "Manager"),
        }
    }
}

/// The IT Department structure
pub struct Department {
    pub specialists: Vec<Specialist>,
}

impl Department {
    pub fn new() -> Self {
        Self {
            specialists: vec![
                // Network Team
                Specialist {
                    id: "net-jr",
                    name: "Michael",
                    role: SpecialistRole::Junior,
                    department: "Network",
                    expertise: &["wifi", "ethernet", "dns", "dhcp", "ip", "ping", "connectivity", "firewall"],
                    model_tier: "fast",
                },
                Specialist {
                    id: "net-sr",
                    name: "Sarah",
                    role: SpecialistRole::Senior,
                    department: "Network",
                    expertise: &["routing", "vpn", "iptables", "nftables", "advanced networking", "tunnels"],
                    model_tier: "deep",
                },

                // Desktop Team
                Specialist {
                    id: "desk-jr",
                    name: "Alex",
                    role: SpecialistRole::Junior,
                    department: "Desktop",
                    expertise: &["vim", "neovim", "nano", "editor", "terminal", "shell", "bash", "zsh", "config"],
                    model_tier: "fast",
                },
                Specialist {
                    id: "desk-sr",
                    name: "Emma",
                    role: SpecialistRole::Senior,
                    department: "Desktop",
                    expertise: &["hyprland", "sway", "i3", "gnome", "kde", "window manager", "compositor", "wayland", "x11"],
                    model_tier: "deep",
                },

                // System Team
                Specialist {
                    id: "sys-jr",
                    name: "James",
                    role: SpecialistRole::Junior,
                    department: "System",
                    expertise: &["systemd", "service", "boot", "startup", "process", "memory", "cpu", "disk"],
                    model_tier: "fast",
                },
                Specialist {
                    id: "sys-sr",
                    name: "Lisa",
                    role: SpecialistRole::Senior,
                    department: "System",
                    expertise: &["kernel", "modules", "performance", "optimization", "security", "permissions"],
                    model_tier: "deep",
                },

                // Package Team
                Specialist {
                    id: "pkg-jr",
                    name: "David",
                    role: SpecialistRole::Junior,
                    department: "Packages",
                    expertise: &["pacman", "yay", "aur", "install", "update", "upgrade", "package", "dependency"],
                    model_tier: "fast",
                },
                Specialist {
                    id: "pkg-sr",
                    name: "Nina",
                    role: SpecialistRole::Senior,
                    department: "Packages",
                    expertise: &["makepkg", "pkgbuild", "aur maintenance", "package conflicts", "downgrade"],
                    model_tier: "deep",
                },

                // Hardware Team
                Specialist {
                    id: "hw-jr",
                    name: "Ryan",
                    role: SpecialistRole::Junior,
                    department: "Hardware",
                    expertise: &["gpu", "nvidia", "amd", "intel", "driver", "graphics", "display", "monitor"],
                    model_tier: "fast",
                },
                Specialist {
                    id: "hw-sr",
                    name: "Sophie",
                    role: SpecialistRole::Senior,
                    department: "Hardware",
                    expertise: &["firmware", "bios", "uefi", "acpi", "power management", "sensors", "overclocking"],
                    model_tier: "deep",
                },

                // Audio Team
                Specialist {
                    id: "audio-jr",
                    name: "Chris",
                    role: SpecialistRole::Junior,
                    department: "Audio",
                    expertise: &["pipewire", "pulseaudio", "sound", "audio", "volume", "speaker", "microphone"],
                    model_tier: "fast",
                },
                Specialist {
                    id: "audio-sr",
                    name: "Maria",
                    role: SpecialistRole::Senior,
                    department: "Audio",
                    expertise: &["alsa", "jack", "audio routing", "latency", "pro audio", "midi"],
                    model_tier: "deep",
                },

                // Storage Team
                Specialist {
                    id: "stor-jr",
                    name: "Kevin",
                    role: SpecialistRole::Junior,
                    department: "Storage",
                    expertise: &["disk", "partition", "mount", "fstab", "usb", "drive", "storage", "space"],
                    model_tier: "fast",
                },
                Specialist {
                    id: "stor-sr",
                    name: "Rachel",
                    role: SpecialistRole::Senior,
                    department: "Storage",
                    expertise: &["lvm", "raid", "btrfs", "zfs", "encryption", "luks", "backup", "recovery"],
                    model_tier: "deep",
                },

                // Security Team
                Specialist {
                    id: "sec-jr",
                    name: "Tom",
                    role: SpecialistRole::Junior,
                    department: "Security",
                    expertise: &["password", "ssh", "key", "gpg", "permission", "user", "group"],
                    model_tier: "fast",
                },
                Specialist {
                    id: "sec-sr",
                    name: "Elena",
                    role: SpecialistRole::Senior,
                    department: "Security",
                    expertise: &["selinux", "apparmor", "audit", "hardening", "vulnerability", "firewall rules"],
                    model_tier: "deep",
                },
            ],
        }
    }

    /// Get specialist by ID
    pub fn get_by_id(&self, id: &str) -> Option<&Specialist> {
        self.specialists.iter().find(|s| s.id == id)
    }

    /// Get all specialists in a department
    pub fn get_by_department(&self, dept: &str) -> Vec<&Specialist> {
        self.specialists.iter()
            .filter(|s| s.department.to_lowercase() == dept.to_lowercase())
            .collect()
    }

    /// Get junior for a department
    pub fn get_junior(&self, dept: &str) -> Option<&Specialist> {
        self.specialists.iter()
            .find(|s| s.department.to_lowercase() == dept.to_lowercase()
                  && s.role == SpecialistRole::Junior)
    }

    /// Get senior for a department
    pub fn get_senior(&self, dept: &str) -> Option<&Specialist> {
        self.specialists.iter()
            .find(|s| s.department.to_lowercase() == dept.to_lowercase()
                  && s.role == SpecialistRole::Senior)
    }

    /// Count active specialists (based on system capabilities)
    pub fn count_active(&self) -> usize {
        // TODO: Filter based on actual system capabilities
        // e.g., no Audio team if no sound hardware
        self.specialists.len()
    }

    /// Get department summary for status display
    pub fn get_summary(&self) -> Vec<(String, usize)> {
        let mut depts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for s in &self.specialists {
            *depts.entry(s.department.to_string()).or_insert(0) += 1;
        }
        let mut result: Vec<_> = depts.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
}

impl Default for Department {
    fn default() -> Self {
        Self::new()
    }
}

/// Global department instance
static DEPARTMENT: std::sync::LazyLock<Department> = std::sync::LazyLock::new(Department::new);

/// Get the global department
pub fn get_department() -> &'static Department {
    &DEPARTMENT
}

/// Find the best specialist for a topic
pub fn get_specialist_for_topic(topic: &str) -> Option<&'static Specialist> {
    let topic_lower = topic.to_lowercase();
    let dept = get_department();

    // First try to find a junior with matching expertise (faster response)
    for specialist in &dept.specialists {
        if specialist.role == SpecialistRole::Junior {
            for exp in specialist.expertise {
                if topic_lower.contains(exp) {
                    return Some(specialist);
                }
            }
        }
    }

    // Then try seniors for more complex topics
    for specialist in &dept.specialists {
        if specialist.role == SpecialistRole::Senior {
            for exp in specialist.expertise {
                if topic_lower.contains(exp) {
                    return Some(specialist);
                }
            }
        }
    }

    // Default to Desktop junior (Alex) for general questions
    dept.get_by_id("desk-jr")
}

/// Determine department from question keywords
pub fn determine_department(question: &str) -> &'static str {
    let q = question.to_lowercase();

    if q.contains("wifi") || q.contains("network") || q.contains("internet")
       || q.contains("ethernet") || q.contains("dns") || q.contains("ip ")
       || q.contains("ping") || q.contains("connection") {
        return "Network";
    }

    if q.contains("sound") || q.contains("audio") || q.contains("volume")
       || q.contains("speaker") || q.contains("microphone") || q.contains("pipewire")
       || q.contains("pulseaudio") {
        return "Audio";
    }

    if q.contains("disk") || q.contains("storage") || q.contains("partition")
       || q.contains("mount") || q.contains("drive") || q.contains("space")
       || q.contains("ssd") || q.contains("hdd") {
        return "Storage";
    }

    if q.contains("gpu") || q.contains("graphics") || q.contains("nvidia")
       || q.contains("amd") || q.contains("driver") || q.contains("display")
       || q.contains("monitor") || q.contains("screen") {
        return "Hardware";
    }

    if q.contains("install") || q.contains("package") || q.contains("pacman")
       || q.contains("yay") || q.contains("aur") || q.contains("update") {
        return "Packages";
    }

    if q.contains("service") || q.contains("systemd") || q.contains("boot")
       || q.contains("memory") || q.contains("cpu") || q.contains("process")
       || q.contains("performance") {
        return "System";
    }

    if q.contains("password") || q.contains("ssh") || q.contains("security")
       || q.contains("permission") || q.contains("user") || q.contains("encrypt") {
        return "Security";
    }

    if q.contains("vim") || q.contains("editor") || q.contains("terminal")
       || q.contains("shell") || q.contains("config") || q.contains("hyprland")
       || q.contains("window") {
        return "Desktop";
    }

    // Default
    "Desktop"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_department() {
        let dept = get_department();
        assert!(dept.specialists.len() >= 14);
    }

    #[test]
    fn test_find_specialist() {
        let specialist = get_specialist_for_topic("wifi not working");
        assert!(specialist.is_some());
        assert_eq!(specialist.unwrap().department, "Network");
    }

    #[test]
    fn test_determine_department() {
        assert_eq!(determine_department("my wifi is slow"), "Network");
        assert_eq!(determine_department("no sound from speakers"), "Audio");
        assert_eq!(determine_department("install neovim"), "Packages");
    }
}
