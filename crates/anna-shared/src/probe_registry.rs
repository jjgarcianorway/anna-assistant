//! Probe Registry - Composable system probes (v0.0.410).
//!
//! Centralized definitions for all probes Anna can run.
//! Each probe has:
//! - Unique ID
//! - Shell command
//! - Domain/tags for matching
//! - Cost (cheap/medium/expensive)
//! - Selection predicates

use crate::evidence_engine::{EvidenceDomain, EvidenceIntent};
use serde::{Deserialize, Serialize};

/// Cost of running a probe
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeCost {
    /// Fast, no disk/network (< 100ms)
    Cheap,
    /// May take a moment (< 1s)
    Medium,
    /// Slow or resource-intensive (> 1s)
    Expensive,
}

/// A probe definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeDef {
    /// Unique identifier (e.g., "probe:df_root")
    pub id: String,
    /// Shell command to run
    pub command: String,
    /// Human description
    pub description: String,
    /// Applicable domains
    pub domains: Vec<EvidenceDomain>,
    /// Matching tags
    pub tags: Vec<String>,
    /// Cost classification
    pub cost: ProbeCost,
    /// Required intents (empty = any)
    pub intents: Vec<EvidenceIntent>,
    /// Output parser hint
    pub parse_hint: Option<String>,
}

impl ProbeDef {
    /// Check if this probe matches a request
    pub fn matches(&self, domain: EvidenceDomain, intent: EvidenceIntent, tags: &[String]) -> bool {
        // Domain must match (or be related)
        let domain_match = self.domains.contains(&domain);

        // Intent must match if specified
        let intent_match = self.intents.is_empty() || self.intents.contains(&intent);

        // At least one tag must match
        let tag_match = tags.iter().any(|t| {
            let t_lower = t.to_lowercase();
            self.tags.iter().any(|pt| pt.to_lowercase() == t_lower)
        });

        domain_match && intent_match && tag_match
    }

    /// Score relevance (higher = more relevant)
    pub fn relevance_score(&self, tags: &[String]) -> u32 {
        let mut score = 0u32;
        for tag in tags {
            let t_lower = tag.to_lowercase();
            if self.tags.iter().any(|pt| pt.to_lowercase() == t_lower) {
                score += 10;
            }
        }
        // Cheaper probes get slight boost
        match self.cost {
            ProbeCost::Cheap => score += 3,
            ProbeCost::Medium => score += 1,
            ProbeCost::Expensive => {}
        }
        score
    }
}

/// The probe registry
pub struct ProbeRegistry {
    probes: Vec<ProbeDef>,
}

impl Default for ProbeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProbeRegistry {
    /// Create registry with built-in probes
    pub fn new() -> Self {
        Self {
            probes: builtin_probes(),
        }
    }

    /// Select probes for a request
    pub fn select(
        &self,
        domain: EvidenceDomain,
        intent: EvidenceIntent,
        tags: &[String],
        max_probes: usize,
    ) -> Vec<&ProbeDef> {
        let mut matches: Vec<_> = self
            .probes
            .iter()
            .filter(|p| p.matches(domain, intent, tags))
            .collect();

        // Sort by relevance (desc) then cost (asc)
        matches.sort_by(|a, b| {
            let score_a = a.relevance_score(tags);
            let score_b = b.relevance_score(tags);
            match score_b.cmp(&score_a) {
                std::cmp::Ordering::Equal => a.cost.cmp(&b.cost),
                other => other,
            }
        });

        matches.truncate(max_probes);
        matches
    }

    /// Get probe by ID
    pub fn get(&self, id: &str) -> Option<&ProbeDef> {
        self.probes.iter().find(|p| p.id == id)
    }

    /// Add a custom probe
    pub fn add(&mut self, probe: ProbeDef) {
        self.probes.push(probe);
    }

    /// List all probes for a domain
    pub fn for_domain(&self, domain: EvidenceDomain) -> Vec<&ProbeDef> {
        self.probes
            .iter()
            .filter(|p| p.domains.contains(&domain))
            .collect()
    }
}

/// Built-in probe definitions
fn builtin_probes() -> Vec<ProbeDef> {
    vec![
        // === STORAGE ===
        ProbeDef {
            id: "probe:df_root".into(),
            command: "df -h /".into(),
            description: "Root filesystem usage".into(),
            domains: vec![EvidenceDomain::Storage, EvidenceDomain::Performance],
            tags: vec!["disk", "usage", "filesystem", "root", "space", "full"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![
                EvidenceIntent::Diagnose,
                EvidenceIntent::Inspect,
                EvidenceIntent::Stats,
            ],
            parse_hint: Some("Look for Use% column".into()),
        },
        ProbeDef {
            id: "probe:df_all".into(),
            command: "df -h".into(),
            description: "All filesystem usage".into(),
            domains: vec![EvidenceDomain::Storage],
            tags: vec!["disk", "usage", "filesystem", "mount", "space"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![],
            parse_hint: None,
        },
        ProbeDef {
            id: "probe:lsblk".into(),
            command: "lsblk -o NAME,SIZE,TYPE,MOUNTPOINT".into(),
            description: "Block device layout".into(),
            domains: vec![EvidenceDomain::Storage],
            tags: vec!["disk", "partition", "block", "device", "mount"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![],
            parse_hint: None,
        },
        // === PERFORMANCE ===
        ProbeDef {
            id: "probe:memory".into(),
            command: "free -h".into(),
            description: "Memory usage".into(),
            domains: vec![EvidenceDomain::Performance, EvidenceDomain::System],
            tags: vec!["memory", "ram", "swap", "usage", "free"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![],
            parse_hint: Some("Check Mem: and Swap: lines".into()),
        },
        ProbeDef {
            id: "probe:ps_top_mem".into(),
            command: "ps aux --sort=-%mem | head -15".into(),
            description: "Top memory consumers".into(),
            domains: vec![
                EvidenceDomain::Performance,
                EvidenceDomain::Desktop,
                EvidenceDomain::Services,
            ],
            tags: vec!["memory", "ram", "slow", "process", "heavy"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Diagnose, EvidenceIntent::Inspect],
            parse_hint: None,
        },
        ProbeDef {
            id: "probe:ps_top_cpu".into(),
            command: "ps aux --sort=-%cpu | head -15".into(),
            description: "Top CPU consumers".into(),
            domains: vec![EvidenceDomain::Performance],
            tags: vec!["cpu", "slow", "process", "load", "heavy", "fan"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Diagnose, EvidenceIntent::Inspect],
            parse_hint: None,
        },
        ProbeDef {
            id: "probe:uptime".into(),
            command: "uptime".into(),
            description: "System uptime and load".into(),
            domains: vec![EvidenceDomain::Performance, EvidenceDomain::System],
            tags: vec!["uptime", "load", "average"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![],
            parse_hint: None,
        },
        ProbeDef {
            id: "probe:sensors".into(),
            command: "sensors 2>/dev/null || echo 'sensors not available'".into(),
            description: "Hardware temperatures".into(),
            domains: vec![EvidenceDomain::Hardware, EvidenceDomain::Performance],
            tags: vec![
                "temperature",
                "temp",
                "fan",
                "heat",
                "thermal",
                "hot",
                "cpu_temp",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Diagnose, EvidenceIntent::Inspect],
            parse_hint: None,
        },
        // === SERVICES ===
        ProbeDef {
            id: "probe:systemctl_failed".into(),
            command: "systemctl --failed --no-pager".into(),
            description: "Failed systemd units".into(),
            domains: vec![
                EvidenceDomain::Services,
                EvidenceDomain::System,
                EvidenceDomain::Boot,
            ],
            tags: vec!["service", "failed", "systemd", "unit", "error"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Diagnose, EvidenceIntent::Inspect],
            parse_hint: None,
        },
        ProbeDef {
            id: "probe:systemctl_running".into(),
            command: "systemctl list-units --type=service --state=running --no-pager | head -30"
                .into(),
            description: "Running services".into(),
            domains: vec![EvidenceDomain::Services],
            tags: vec!["service", "running", "active", "systemd"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Inspect, EvidenceIntent::Stats],
            parse_hint: None,
        },
        // === PACKAGES ===
        ProbeDef {
            id: "probe:pacman_count".into(),
            command: "pacman -Qq | wc -l".into(),
            description: "Total installed packages".into(),
            domains: vec![EvidenceDomain::Packages],
            tags: vec!["package", "pacman", "count", "installed", "total"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Stats, EvidenceIntent::Inspect],
            parse_hint: Some("Number only".into()),
        },
        ProbeDef {
            id: "probe:pacman_explicit".into(),
            command: "pacman -Qe | wc -l".into(),
            description: "Explicitly installed packages".into(),
            domains: vec![EvidenceDomain::Packages],
            tags: vec!["package", "pacman", "explicit", "installed"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Stats],
            parse_hint: None,
        },
        ProbeDef {
            id: "probe:pacman_orphans".into(),
            command: "pacman -Qtdq 2>/dev/null | wc -l".into(),
            description: "Orphan packages".into(),
            domains: vec![EvidenceDomain::Packages],
            tags: vec!["package", "pacman", "orphan", "unused", "cleanup"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Medium,
            intents: vec![EvidenceIntent::Diagnose, EvidenceIntent::Stats],
            parse_hint: None,
        },
        // === NETWORK ===
        ProbeDef {
            id: "probe:ip_addr".into(),
            command: "ip -br addr".into(),
            description: "Network interfaces and IPs".into(),
            domains: vec![EvidenceDomain::Network],
            tags: vec!["ip", "address", "interface", "network"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![],
            parse_hint: None,
        },
        ProbeDef {
            id: "probe:ss_listening".into(),
            command: "ss -tlnp 2>/dev/null | head -20".into(),
            description: "Listening TCP ports".into(),
            domains: vec![EvidenceDomain::Network, EvidenceDomain::Security],
            tags: vec!["port", "listen", "tcp", "socket", "network"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Inspect, EvidenceIntent::Diagnose],
            parse_hint: None,
        },
        // === HARDWARE ===
        ProbeDef {
            id: "probe:lscpu".into(),
            command: "lscpu | head -20".into(),
            description: "CPU information".into(),
            domains: vec![EvidenceDomain::Hardware, EvidenceDomain::System],
            tags: vec!["cpu", "processor", "core", "hardware"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Inspect, EvidenceIntent::Explain],
            parse_hint: None,
        },
        ProbeDef {
            id: "probe:lspci_vga".into(),
            command: "lspci | grep -i 'vga\\|3d\\|display'".into(),
            description: "Graphics hardware".into(),
            domains: vec![EvidenceDomain::Hardware, EvidenceDomain::Display],
            tags: vec!["gpu", "graphics", "video", "display", "nvidia", "amd"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![],
            parse_hint: None,
        },
        // === DESKTOP ===
        ProbeDef {
            id: "probe:desktop_env".into(),
            command: "echo $XDG_CURRENT_DESKTOP $DESKTOP_SESSION".into(),
            description: "Desktop environment".into(),
            domains: vec![EvidenceDomain::Desktop],
            tags: vec!["desktop", "environment", "de", "wm"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Inspect],
            parse_hint: None,
        },
        // === AUDIO ===
        ProbeDef {
            id: "probe:pactl_info".into(),
            command: "pactl info 2>/dev/null | head -15".into(),
            description: "Audio server info".into(),
            domains: vec![EvidenceDomain::Audio],
            tags: vec!["audio", "sound", "pulse", "pipewire", "volume"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![],
            parse_hint: None,
        },
        ProbeDef {
            id: "probe:pactl_sinks".into(),
            command: "pactl list sinks short 2>/dev/null".into(),
            description: "Audio output devices".into(),
            domains: vec![EvidenceDomain::Audio],
            tags: vec!["audio", "speaker", "output", "sink", "sound"]
                .into_iter()
                .map(String::from)
                .collect(),
            cost: ProbeCost::Cheap,
            intents: vec![EvidenceIntent::Inspect, EvidenceIntent::Diagnose],
            parse_hint: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_registry_select() {
        let registry = ProbeRegistry::new();
        let probes = registry.select(
            EvidenceDomain::Storage,
            EvidenceIntent::Diagnose,
            &["disk".to_string(), "space".to_string()],
            5,
        );

        assert!(!probes.is_empty());
        assert!(probes.iter().any(|p| p.id == "probe:df_root"));
    }

    #[test]
    fn test_probe_matching() {
        let probe = ProbeDef {
            id: "test".into(),
            command: "test".into(),
            description: "test".into(),
            domains: vec![EvidenceDomain::Storage],
            tags: vec!["disk".into(), "space".into()],
            cost: ProbeCost::Cheap,
            intents: vec![],
            parse_hint: None,
        };

        assert!(probe.matches(
            EvidenceDomain::Storage,
            EvidenceIntent::Diagnose,
            &["disk".into()]
        ));
        assert!(!probe.matches(
            EvidenceDomain::Network,
            EvidenceIntent::Diagnose,
            &["disk".into()]
        ));
    }

    #[test]
    fn test_cost_ordering() {
        let registry = ProbeRegistry::new();
        let probes = registry.select(
            EvidenceDomain::Performance,
            EvidenceIntent::Diagnose,
            &["cpu".to_string()],
            10,
        );

        // Cheap probes should come first
        if probes.len() >= 2 {
            assert!(probes[0].cost <= probes[1].cost);
        }
    }
}
