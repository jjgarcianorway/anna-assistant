//! Translator Contract (v0.0.415).
//!
//! Strict schema for translator output.
//! Translator ONLY routes and classifies. It NEVER answers.

use serde::{Deserialize, Serialize};

/// Translator output schema - JSON only
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatorOutput {
    /// Intent classification
    pub intent: TranslatorIntent,

    /// Domain classification
    pub domain: TranslatorDomain,

    /// Probes needed (1-4)
    pub needs_probes: Vec<String>,

    /// Optional follow-up questions (0-2)
    #[serde(default)]
    pub follow_up_questions: Vec<String>,

    /// Whether clarification is needed before proceeding
    #[serde(default)]
    pub needs_clarification: bool,

    /// Priority
    #[serde(default)]
    pub priority: Priority,

    /// Confidence (0.0-1.0)
    #[serde(default)]
    pub confidence: f32,
}

/// Intent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslatorIntent {
    /// Asking for a metric value
    QueryMetric,
    /// Diagnosing a problem
    Diagnose,
    /// Configuring something
    Configure,
    /// Listing items
    List,
    /// Checking status
    CheckStatus,
    /// Explaining a concept
    Explain,
}

impl TranslatorIntent {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "query_metric" | "querymetric" | "metric" => Self::QueryMetric,
            "diagnose" | "debug" | "investigate" => Self::Diagnose,
            "configure" | "config" | "setup" => Self::Configure,
            "list" | "show" | "enumerate" => Self::List,
            "check_status" | "checkstatus" | "status" | "check" => Self::CheckStatus,
            "explain" | "what_is" | "how_does" => Self::Explain,
            _ => Self::QueryMetric, // default
        }
    }
}

impl std::fmt::Display for TranslatorIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::QueryMetric => "query_metric",
            Self::Diagnose => "diagnose",
            Self::Configure => "configure",
            Self::List => "list",
            Self::CheckStatus => "check_status",
            Self::Explain => "explain",
        };
        write!(f, "{}", s)
    }
}

/// Domain types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslatorDomain {
    System,
    Boot,
    Services,
    Network,
    Storage,
    Packages,
    Audio,
    Display,
    Desktop,
    Security,
}

impl TranslatorDomain {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "system" | "memory" | "cpu" | "ram" => Self::System,
            "boot" | "startup" => Self::Boot,
            "services" | "service" | "systemd" | "daemon" => Self::Services,
            "network" | "net" | "wifi" | "ethernet" | "ip" => Self::Network,
            "storage" | "disk" | "space" | "partition" => Self::Storage,
            "packages" | "package" | "pkg" | "install" => Self::Packages,
            "audio" | "sound" | "speaker" | "pipewire" => Self::Audio,
            "display" | "gpu" | "graphics" | "monitor" => Self::Display,
            "desktop" | "de" | "wm" | "hyprland" | "gnome" => Self::Desktop,
            "security" | "firewall" | "ssh" | "login" => Self::Security,
            _ => Self::System, // default
        }
    }
}

impl std::fmt::Display for TranslatorDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::System => "system",
            Self::Boot => "boot",
            Self::Services => "services",
            Self::Network => "network",
            Self::Storage => "storage",
            Self::Packages => "packages",
            Self::Audio => "audio",
            Self::Display => "display",
            Self::Desktop => "desktop",
            Self::Security => "security",
        };
        write!(f, "{}", s)
    }
}

/// Priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    #[default]
    Normal,
    High,
}

impl TranslatorOutput {
    /// Validate translator output
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.needs_probes.is_empty() {
            issues.push("needs_probes is empty".to_string());
        }

        if self.needs_probes.len() > 6 {
            issues.push(format!("too many probes ({} > 6)", self.needs_probes.len()));
        }

        if self.follow_up_questions.len() > 3 {
            issues.push(format!("too many follow-up questions ({} > 3)", self.follow_up_questions.len()));
        }

        if self.confidence < 0.0 || self.confidence > 1.0 {
            issues.push(format!("confidence {} out of range", self.confidence));
        }

        issues
    }
}

/// Parse translator LLM output
pub fn parse_translator_output(raw: &str) -> Result<TranslatorOutput, String> {
    // Extract JSON
    let json_str = extract_json(raw).ok_or("No JSON found")?;

    // Try strict parse first
    if let Ok(output) = serde_json::from_str::<TranslatorOutput>(&json_str) {
        return Ok(output);
    }

    // Lenient parse
    #[derive(Deserialize, Default)]
    struct Lenient {
        #[serde(default)]
        intent: Option<String>,
        #[serde(default)]
        domain: Option<String>,
        #[serde(default)]
        needs_probes: Option<Vec<String>>,
        #[serde(default)]
        follow_up_questions: Option<Vec<String>>,
        #[serde(default)]
        needs_clarification: Option<bool>,
        #[serde(default)]
        priority: Option<String>,
        #[serde(default)]
        confidence: Option<f32>,
    }

    let l: Lenient = serde_json::from_str(&json_str).map_err(|e| e.to_string())?;

    Ok(TranslatorOutput {
        intent: TranslatorIntent::from_str(l.intent.as_deref().unwrap_or("query_metric")),
        domain: TranslatorDomain::from_str(l.domain.as_deref().unwrap_or("system")),
        needs_probes: l.needs_probes.unwrap_or_default(),
        follow_up_questions: l.follow_up_questions.unwrap_or_default(),
        needs_clarification: l.needs_clarification.unwrap_or(false),
        priority: match l.priority.as_deref() {
            Some("high") => Priority::High,
            _ => Priority::Normal,
        },
        confidence: l.confidence.unwrap_or(0.5).clamp(0.0, 1.0),
    })
}

fn extract_json(raw: &str) -> Option<String> {
    let trimmed = raw.trim();

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed.to_string());
    }

    // Find first { and last }
    let first = trimmed.find('{')?;
    let last = trimmed.rfind('}')?;
    if last > first {
        return Some(trimmed[first..=last].to_string());
    }

    None
}

/// The translator prompt
pub const TRANSLATOR_PROMPT: &str = r#"You are Anna's query classifier. Output ONLY valid JSON.

OUTPUT FORMAT:
{"intent":"query_metric|diagnose|configure|list|check_status|explain","domain":"system|boot|services|network|storage|packages|audio|display|desktop|security","needs_probes":[],"confidence":0.0-1.0}

DOMAIN MAPPING:
- system: CPU, RAM, memory, swap, processes, load, uptime
- boot: startup time, boot errors, systemd-analyze
- services: systemd units, failed services, daemons
- network: IP, DNS, wifi, ethernet, ports
- storage: disk space, partitions, what's taking space
- packages: installed, updates, pacman queries
- audio: sound, PulseAudio, PipeWire
- display: GPU, monitors, resolution
- desktop: WM/DE config, Hyprland, GNOME
- security: firewall, SSH, logins

PROBE MAPPINGS:
system: memory_info, cpu_info, load_average, uptime, swap_files
boot: boot_time, boot_blame, failed_services
services: failed_services, running_services, systemd_timers
network: network_addrs, dns_servers, listening_ports, ping_check
storage: disk_usage, largest_dirs, block_devices
packages: package_count, installed_packages, package_check_<name>
audio: audio_devices, audio_server
display: gpu_info, display_info
desktop: desktop_session
security: firewall_status, ssh_connections

RULES:
1. Output ONLY JSON, no explanation
2. Select 1-4 probes that DIRECTLY answer the query
3. NEVER answer the question yourself
4. For package checks, use "package_check_<name>" format"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clean_json() {
        let json = r#"{"intent":"query_metric","domain":"system","needs_probes":["memory_info"],"confidence":0.9}"#;
        let result = parse_translator_output(json);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.intent, TranslatorIntent::QueryMetric);
        assert_eq!(output.domain, TranslatorDomain::System);
    }

    #[test]
    fn test_parse_lenient() {
        let json = r#"{"intent":"question","domain":"disk","needs_probes":["disk_usage"]}"#;
        let result = parse_translator_output(json);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.domain, TranslatorDomain::Storage); // disk -> storage
    }

    #[test]
    fn test_intent_from_str() {
        assert_eq!(TranslatorIntent::from_str("query_metric"), TranslatorIntent::QueryMetric);
        assert_eq!(TranslatorIntent::from_str("diagnose"), TranslatorIntent::Diagnose);
        assert_eq!(TranslatorIntent::from_str("unknown"), TranslatorIntent::QueryMetric);
    }

    #[test]
    fn test_domain_from_str() {
        assert_eq!(TranslatorDomain::from_str("ram"), TranslatorDomain::System);
        assert_eq!(TranslatorDomain::from_str("disk"), TranslatorDomain::Storage);
        assert_eq!(TranslatorDomain::from_str("wifi"), TranslatorDomain::Network);
    }
}
