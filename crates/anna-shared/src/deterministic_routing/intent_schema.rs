//! Ticket Intent Schema (Part A) - v0.0.439.
//!
//! Canonical schema for translator output. Translator must output exactly this
//! JSON structure with temperature=0, max tokens 200, no prose.

use serde::{Deserialize, Serialize};

/// Canonical intent types that map deterministically to departments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalIntent {
    // Performance intents
    BootPerf,
    MemStatus,
    CpuLoad,
    IoWait,

    // Storage intents
    DiskUsage,
    MountHealth,
    SmartStatus,
    BtrfsHealth,

    // Services intents
    SvcFailed,
    SvcHealth,
    SvcStatus,
    LogsRecentErrors,
    TimerStatus,

    // Network intents
    NetHealth,
    DnsHealth,
    WifiStatus,
    RouteStatus,

    // Hardware intents
    GpuInfo,
    GpuDriver,
    HardwareSensors,
    CpuInfo,
    AudioHealth,
    UsbDevices,
    PciDevices,

    // Desktop intents
    SessionDesktop,
    EditorConfig,
    ShellConfig,
    ThemeConfig,

    // Security intents
    SecurityFirewall,
    PermissionCheck,
    VulnCheck,

    // Package intents
    PkgInventory,
    PkgUpdates,
    PkgSearch,

    // Fallback
    Unknown,
}

impl CanonicalIntent {
    /// Parse from string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().replace('-', "_").as_str() {
            "boot_perf" | "bootperf" | "boot" => Self::BootPerf,
            "mem_status" | "memstatus" | "memory" | "ram" => Self::MemStatus,
            "cpu_load" | "cpuload" => Self::CpuLoad,
            "io_wait" | "iowait" => Self::IoWait,

            "disk_usage" | "diskusage" | "disk" => Self::DiskUsage,
            "mount_health" | "mounthealth" | "mounts" => Self::MountHealth,
            "smart_status" | "smartstatus" | "smart" => Self::SmartStatus,
            "btrfs_health" | "btrfshealth" | "btrfs" => Self::BtrfsHealth,

            "svc_failed" | "svcfailed" | "failed_services" => Self::SvcFailed,
            "svc_health" | "svchealth" | "service_health" => Self::SvcHealth,
            "svc_status" | "svcstatus" | "service" => Self::SvcStatus,
            "logs_recent_errors" | "logsrecenterrors" | "errors" | "logs" => Self::LogsRecentErrors,
            "timer_status" | "timerstatus" | "timers" => Self::TimerStatus,

            "net_health" | "nethealth" | "network" => Self::NetHealth,
            "dns_health" | "dnshealth" | "dns" => Self::DnsHealth,
            "wifi_status" | "wifistatus" | "wifi" => Self::WifiStatus,
            "route_status" | "routestatus" | "routing" => Self::RouteStatus,

            "gpu_info" | "gpuinfo" | "gpu" => Self::GpuInfo,
            "gpu_driver" | "gpudriver" => Self::GpuDriver,
            "hardware_sensors" | "hardwaresensors" | "sensors" | "temperature" => Self::HardwareSensors,
            "cpu_info" | "cpuinfo" => Self::CpuInfo,
            "audio_health" | "audiohealth" | "audio" | "sound" => Self::AudioHealth,
            "usb_devices" | "usbdevices" | "usb" => Self::UsbDevices,
            "pci_devices" | "pcidevices" | "pci" => Self::PciDevices,

            "session_desktop" | "sessiondesktop" | "desktop" => Self::SessionDesktop,
            "editor_config" | "editorconfig" | "editor" => Self::EditorConfig,
            "shell_config" | "shellconfig" | "shell" => Self::ShellConfig,
            "theme_config" | "themeconfig" | "theme" => Self::ThemeConfig,

            "security_firewall" | "securityfirewall" | "firewall" => Self::SecurityFirewall,
            "permission_check" | "permissioncheck" | "permissions" => Self::PermissionCheck,
            "vuln_check" | "vulncheck" | "vulnerabilities" => Self::VulnCheck,

            "pkg_inventory" | "pkginventory" | "packages" => Self::PkgInventory,
            "pkg_updates" | "pkgupdates" | "updates" => Self::PkgUpdates,
            "pkg_search" | "pkgsearch" => Self::PkgSearch,

            _ => Self::Unknown,
        }
    }

    /// Get label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::BootPerf => "boot_perf",
            Self::MemStatus => "mem_status",
            Self::CpuLoad => "cpu_load",
            Self::IoWait => "io_wait",
            Self::DiskUsage => "disk_usage",
            Self::MountHealth => "mount_health",
            Self::SmartStatus => "smart_status",
            Self::BtrfsHealth => "btrfs_health",
            Self::SvcFailed => "svc_failed",
            Self::SvcHealth => "svc_health",
            Self::SvcStatus => "svc_status",
            Self::LogsRecentErrors => "logs_recent_errors",
            Self::TimerStatus => "timer_status",
            Self::NetHealth => "net_health",
            Self::DnsHealth => "dns_health",
            Self::WifiStatus => "wifi_status",
            Self::RouteStatus => "route_status",
            Self::GpuInfo => "gpu_info",
            Self::GpuDriver => "gpu_driver",
            Self::HardwareSensors => "hardware_sensors",
            Self::CpuInfo => "cpu_info",
            Self::AudioHealth => "audio_health",
            Self::UsbDevices => "usb_devices",
            Self::PciDevices => "pci_devices",
            Self::SessionDesktop => "session_desktop",
            Self::EditorConfig => "editor_config",
            Self::ShellConfig => "shell_config",
            Self::ThemeConfig => "theme_config",
            Self::SecurityFirewall => "security_firewall",
            Self::PermissionCheck => "permission_check",
            Self::VulnCheck => "vuln_check",
            Self::PkgInventory => "pkg_inventory",
            Self::PkgUpdates => "pkg_updates",
            Self::PkgSearch => "pkg_search",
            Self::Unknown => "unknown",
        }
    }
}

/// Department that handles a ticket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Department {
    Performance,
    Storage,
    Services,
    Network,
    Security,
    Hardware,
    Desktop,
}

impl Department {
    /// Parse from string.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "performance" | "perf" => Some(Self::Performance),
            "storage" | "disk" => Some(Self::Storage),
            "services" | "service" | "svc" => Some(Self::Services),
            "network" | "net" => Some(Self::Network),
            "security" | "sec" => Some(Self::Security),
            "hardware" | "hw" => Some(Self::Hardware),
            "desktop" | "de" => Some(Self::Desktop),
            _ => None,
        }
    }

    /// Get label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Performance => "Performance",
            Self::Storage => "Storage",
            Self::Services => "Services",
            Self::Network => "Network",
            Self::Security => "Security",
            Self::Hardware => "Hardware",
            Self::Desktop => "Desktop",
        }
    }
}

/// Risk level for the ticket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Read-only operations (probes, status checks).
    ReadOnly,
    /// Safe changes (enabling a service, changing a setting).
    SafeChange,
    /// Risky changes (format, delete, kernel updates).
    RiskyChange,
}

impl RiskLevel {
    /// Get label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::SafeChange => "safe_change",
            Self::RiskyChange => "risky_change",
        }
    }
}

impl Default for RiskLevel {
    fn default() -> Self {
        Self::ReadOnly
    }
}

/// The canonical ticket intent schema output by translator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketIntentSchema {
    /// Original user query.
    pub user_query: String,
    /// Detected intent.
    pub intent: CanonicalIntent,
    /// Department that should handle this.
    pub department: Department,
    /// Required evidence (probe IDs that must succeed).
    pub required_evidence: Vec<String>,
    /// Optional evidence (nice to have).
    #[serde(default)]
    pub optional_evidence: Vec<String>,
    /// Whether clarification is needed.
    #[serde(default)]
    pub need_clarification: bool,
    /// Clarifying question if needed (max 120 chars).
    #[serde(default)]
    pub clarifying_question: Option<String>,
    /// Risk level.
    #[serde(default)]
    pub risk_level: RiskLevel,
}

impl TicketIntentSchema {
    /// Create a new schema with required fields.
    pub fn new(query: &str, intent: CanonicalIntent, department: Department) -> Self {
        Self {
            user_query: query.to_string(),
            intent,
            department,
            required_evidence: Vec::new(),
            optional_evidence: Vec::new(),
            need_clarification: false,
            clarifying_question: None,
            risk_level: RiskLevel::ReadOnly,
        }
    }

    /// Add required evidence.
    pub fn with_required_evidence(mut self, probes: Vec<&str>) -> Self {
        self.required_evidence = probes.into_iter().map(String::from).collect();
        self
    }

    /// Add optional evidence.
    pub fn with_optional_evidence(mut self, probes: Vec<&str>) -> Self {
        self.optional_evidence = probes.into_iter().map(String::from).collect();
        self
    }

    /// Set clarification needed.
    pub fn needs_clarification(mut self, question: &str) -> Self {
        self.need_clarification = true;
        // Truncate to 120 chars
        self.clarifying_question = Some(if question.len() > 120 {
            format!("{}...", &question[..117])
        } else {
            question.to_string()
        });
        self
    }

    /// Set risk level.
    pub fn with_risk(mut self, risk: RiskLevel) -> Self {
        self.risk_level = risk;
        self
    }

    /// Validate the schema.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.user_query.is_empty() {
            errors.push("user_query cannot be empty".to_string());
        }

        if self.need_clarification {
            if self.clarifying_question.is_none() {
                errors.push("need_clarification=true but no clarifying_question".to_string());
            } else if let Some(q) = &self.clarifying_question {
                if q.len() > 120 {
                    errors.push(format!("clarifying_question exceeds 120 chars: {}", q.len()));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// Serialize to pretty JSON.
    pub fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

/// Parser for translator JSON output.
pub struct IntentSchemaParser;

impl IntentSchemaParser {
    /// Parse raw JSON to TicketIntentSchema.
    pub fn parse(raw: &str) -> Result<TicketIntentSchema, ParseError> {
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return Err(ParseError::Empty);
        }

        // Try direct parse
        match serde_json::from_str::<TicketIntentSchema>(trimmed) {
            Ok(schema) => {
                if let Err(issues) = schema.validate() {
                    Err(ParseError::ValidationFailed { issues })
                } else {
                    Ok(schema)
                }
            }
            Err(e) => {
                // Try to extract JSON from mixed content
                if let Some(json_str) = Self::extract_json(trimmed) {
                    match serde_json::from_str::<TicketIntentSchema>(&json_str) {
                        Ok(schema) => {
                            if let Err(issues) = schema.validate() {
                                Err(ParseError::ValidationFailed { issues })
                            } else {
                                Ok(schema)
                            }
                        }
                        Err(_) => Err(ParseError::InvalidJson {
                            message: e.to_string(),
                        }),
                    }
                } else {
                    Err(ParseError::InvalidJson {
                        message: e.to_string(),
                    })
                }
            }
        }
    }

    /// Extract JSON object from mixed content.
    fn extract_json(text: &str) -> Option<String> {
        let first_brace = text.find('{')?;
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, c) in text[first_brace..].char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => depth += 1,
                '}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(text[first_brace..first_brace + i + 1].to_string());
                    }
                }
                _ => {}
            }
        }

        None
    }
}

/// Parse error types.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Empty input.
    Empty,
    /// Invalid JSON.
    InvalidJson { message: String },
    /// Validation failed.
    ValidationFailed { issues: Vec<String> },
}

impl ParseError {
    /// Get error message.
    pub fn message(&self) -> String {
        match self {
            Self::Empty => "Empty translator output".to_string(),
            Self::InvalidJson { message } => format!("Invalid JSON: {}", message),
            Self::ValidationFailed { issues } => {
                format!("Validation failed: {}", issues.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_intent_from_str() {
        assert_eq!(CanonicalIntent::from_str_loose("boot_perf"), CanonicalIntent::BootPerf);
        assert_eq!(CanonicalIntent::from_str_loose("GPU_INFO"), CanonicalIntent::GpuInfo);
        assert_eq!(CanonicalIntent::from_str_loose("unknown_thing"), CanonicalIntent::Unknown);
    }

    #[test]
    fn test_department_from_str() {
        assert_eq!(Department::from_str_loose("Performance"), Some(Department::Performance));
        assert_eq!(Department::from_str_loose("hardware"), Some(Department::Hardware));
        assert_eq!(Department::from_str_loose("bogus"), None);
    }

    #[test]
    fn test_schema_creation() {
        let schema = TicketIntentSchema::new("how much RAM?", CanonicalIntent::MemStatus, Department::Performance)
            .with_required_evidence(vec!["meminfo", "free_h"]);

        assert_eq!(schema.intent, CanonicalIntent::MemStatus);
        assert_eq!(schema.department, Department::Performance);
        assert_eq!(schema.required_evidence.len(), 2);
    }

    #[test]
    fn test_clarification_truncation() {
        let long_question = "a".repeat(200);
        let schema = TicketIntentSchema::new("query", CanonicalIntent::Unknown, Department::Performance)
            .needs_clarification(&long_question);

        assert!(schema.clarifying_question.unwrap().len() <= 120);
    }

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{
            "user_query": "how much RAM?",
            "intent": "mem_status",
            "department": "Performance",
            "required_evidence": ["meminfo"],
            "optional_evidence": [],
            "need_clarification": false,
            "clarifying_question": null,
            "risk_level": "read_only"
        }"#;

        let result = IntentSchemaParser::parse(json);
        assert!(result.is_ok());
        let schema = result.unwrap();
        assert_eq!(schema.intent, CanonicalIntent::MemStatus);
    }

    #[test]
    fn test_parse_with_preamble() {
        let raw = r#"Let me analyze...
        {"user_query": "test", "intent": "disk_usage", "department": "Storage", "required_evidence": [], "need_clarification": false, "risk_level": "read_only"}"#;

        let result = IntentSchemaParser::parse(raw);
        assert!(result.is_ok());
    }
}
