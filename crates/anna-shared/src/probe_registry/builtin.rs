//! Built-in probe definitions.

use super::types::{ProbeCost, ProbeDef};
use crate::evidence_engine::{EvidenceDomain, EvidenceIntent};

/// Built-in probe definitions
pub(super) fn builtin_probes() -> Vec<ProbeDef> {
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
