//! Intent classification enums: category, subject, scope, timeframe, precision.

use serde::{Deserialize, Serialize};

/// Category of question - determines answer shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IntentCategory {
    /// Simple fact retrieval (RAM, CPU model, etc.).
    Fact,
    /// System status check (service running, enabled, etc.).
    Status,
    /// Problem diagnosis requiring reasoning.
    Diagnosis,
    /// Explanation of how/why something works.
    Explanation,
    /// Request to perform an action.
    ActionRequest,
    /// Unknown - requires clarification.
    #[default]
    Unknown,
}

impl IntentCategory {
    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Status => "status",
            Self::Diagnosis => "diagnosis",
            Self::Explanation => "explanation",
            Self::ActionRequest => "action_request",
            Self::Unknown => "unknown",
        }
    }

    /// Whether this category allows tutorial/help text.
    pub fn allows_tutorials(&self) -> bool {
        matches!(self, Self::Explanation | Self::ActionRequest)
    }

    /// Whether this category requires synthesis.
    pub fn requires_synthesis(&self) -> bool {
        matches!(self, Self::Diagnosis | Self::Explanation)
    }

    /// Whether this category needs a conclusion.
    pub fn needs_conclusion(&self) -> bool {
        matches!(self, Self::Diagnosis)
    }
}

/// Subject domain of the question.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Subject {
    Cpu,
    Memory,
    Disk,
    Service,
    Network,
    Audio,
    Gpu,
    Boot,
    Packages,
    Desktop,
    Security,
    Kernel,
    Drivers,
    Power,
    Time,
    Users,
    Processes,
    /// Questions about Anna itself.
    Meta,
    /// Multiple subjects.
    Multiple,
    /// Unknown - requires clarification.
    #[default]
    Unknown,
}

impl Subject {
    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Memory => "memory",
            Self::Disk => "disk",
            Self::Service => "service",
            Self::Network => "network",
            Self::Audio => "audio",
            Self::Gpu => "gpu",
            Self::Boot => "boot",
            Self::Packages => "packages",
            Self::Desktop => "desktop",
            Self::Security => "security",
            Self::Kernel => "kernel",
            Self::Drivers => "drivers",
            Self::Power => "power",
            Self::Time => "time",
            Self::Users => "users",
            Self::Processes => "processes",
            Self::Meta => "meta",
            Self::Multiple => "multiple",
            Self::Unknown => "unknown",
        }
    }

    /// Common fields for this subject.
    pub fn common_fields(&self) -> Vec<&'static str> {
        match self {
            Self::Memory => vec![
                "total",
                "used",
                "free",
                "available",
                "cached",
                "swap_total",
                "swap_used",
            ],
            Self::Cpu => vec![
                "model",
                "cores",
                "threads",
                "frequency",
                "usage",
                "temperature",
            ],
            Self::Disk => vec![
                "total",
                "used",
                "free",
                "filesystem",
                "mount_point",
                "usage_percent",
            ],
            Self::Service => vec!["name", "status", "enabled", "active", "description"],
            Self::Network => vec![
                "interface",
                "ip",
                "mac",
                "status",
                "speed",
                "gateway",
                "dns",
            ],
            Self::Gpu => vec!["model", "driver", "memory", "temperature", "usage"],
            Self::Boot => vec!["time", "services", "kernel_time", "userspace_time"],
            Self::Packages => vec!["name", "version", "installed", "repository"],
            Self::Audio => vec!["device", "driver", "volume", "muted", "default"],
            Self::Power => vec!["battery", "charging", "time_remaining", "power_profile"],
            _ => vec![],
        }
    }
}

/// Scope of the answer - how many items expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Single value expected (e.g., "how much free RAM").
    #[default]
    Single,
    /// List of items expected (e.g., "which services are failed").
    List,
    /// Summary/overview expected (e.g., "system health").
    Summary,
    /// Boolean yes/no expected (e.g., "is zram enabled").
    Boolean,
}

impl Scope {
    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::List => "list",
            Self::Summary => "summary",
            Self::Boolean => "boolean",
        }
    }
}

/// Timeframe the question refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    /// Current state (default for most questions).
    #[default]
    Now,
    /// Since last boot.
    LastBoot,
    /// Today only.
    Today,
    /// Since system was installed.
    SinceInstall,
    /// Historical (needs more context).
    Historical,
}

/// Precision level required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Precision {
    /// Exact value required (3.14159).
    #[default]
    Exact,
    /// Approximate is acceptable (about 3).
    Approximate,
}
