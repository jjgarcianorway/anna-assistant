//! Evidence Topic v0.0.63 - Deterministic Evidence Router + Strict Validation
//!
//! Ensures answers match the question:
//! - Disk queries return disk info, not CPU/GPU
//! - Kernel queries return kernel version
//! - Each topic has required fields and answer template
//!
//! v0.0.63: Strict validation
//! - Caps reliability at 40% if answer doesn't contain expected data
//! - Evidence freshness tracking (age in seconds)
//! - Enhanced answer content validation per topic
//!
//! The topic selector runs BEFORE the LLM translator and produces deterministic
//! routing. LLM output is combined but deterministic wins on conflict.

use serde::{Deserialize, Serialize};

// ============================================================================
// Evidence Topic Enum
// ============================================================================

/// Closed set of evidence topics for deterministic routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTopic {
    /// CPU model, cores, threads, frequency
    CpuInfo,
    /// Total memory, available memory, usage
    MemoryInfo,
    /// Kernel release version string
    KernelVersion,
    /// Free disk space on / and key mounts
    DiskFree,
    /// Network connectivity, interfaces, routes, DNS
    NetworkStatus,
    /// Audio/sound stack status (PipeWire/PulseAudio)
    AudioStatus,
    /// Systemd service state (is X running)
    ServiceState,
    /// Recent journal errors/warnings
    RecentErrors,
    /// Boot time and startup analysis
    BootTime,
    /// Recent package changes
    PackagesChanged,
    /// GPU/graphics status
    GraphicsStatus,
    /// Proactive alerts from Anna
    Alerts,
    /// Unknown - requires LLM classification
    Unknown,
}

impl EvidenceTopic {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvidenceTopic::CpuInfo => "cpu_info",
            EvidenceTopic::MemoryInfo => "memory_info",
            EvidenceTopic::KernelVersion => "kernel_version",
            EvidenceTopic::DiskFree => "disk_free",
            EvidenceTopic::NetworkStatus => "network_status",
            EvidenceTopic::AudioStatus => "audio_status",
            EvidenceTopic::ServiceState => "service_state",
            EvidenceTopic::RecentErrors => "recent_errors",
            EvidenceTopic::BootTime => "boot_time",
            EvidenceTopic::PackagesChanged => "packages_changed",
            EvidenceTopic::GraphicsStatus => "graphics_status",
            EvidenceTopic::Alerts => "alerts",
            EvidenceTopic::Unknown => "unknown",
        }
    }

    /// Human-readable description for transcript
    pub fn human_label(&self) -> &'static str {
        match self {
            EvidenceTopic::CpuInfo => "CPU information",
            EvidenceTopic::MemoryInfo => "memory usage",
            EvidenceTopic::KernelVersion => "kernel version",
            EvidenceTopic::DiskFree => "disk space",
            EvidenceTopic::NetworkStatus => "network status",
            EvidenceTopic::AudioStatus => "audio status",
            EvidenceTopic::ServiceState => "service status",
            EvidenceTopic::RecentErrors => "recent system errors",
            EvidenceTopic::BootTime => "boot time",
            EvidenceTopic::PackagesChanged => "package changes",
            EvidenceTopic::GraphicsStatus => "graphics status",
            EvidenceTopic::Alerts => "system alerts",
            EvidenceTopic::Unknown => "system information",
        }
    }

    /// Working message for human transcript
    pub fn working_message(&self) -> &'static str {
        match self {
            EvidenceTopic::CpuInfo => "Checking your CPU information...",
            EvidenceTopic::MemoryInfo => "Checking memory usage...",
            EvidenceTopic::KernelVersion => "Checking your running kernel version...",
            EvidenceTopic::DiskFree => "Checking disk space on your filesystems...",
            EvidenceTopic::NetworkStatus => "Checking network connectivity...",
            EvidenceTopic::AudioStatus => "Checking audio stack status...",
            EvidenceTopic::ServiceState => "Checking service status...",
            EvidenceTopic::RecentErrors => "Checking recent system errors...",
            EvidenceTopic::BootTime => "Checking boot time...",
            EvidenceTopic::PackagesChanged => "Checking recent package changes...",
            EvidenceTopic::GraphicsStatus => "Checking graphics configuration...",
            EvidenceTopic::Alerts => "Checking active alerts...",
            EvidenceTopic::Unknown => "Gathering system information...",
        }
    }
}

// ============================================================================
// Topic Metadata
// ============================================================================

/// Configuration for a topic
#[derive(Debug, Clone)]
pub struct TopicConfig {
    /// Topic enum value
    pub topic: EvidenceTopic,
    /// Required tools that must be executed
    pub required_tools: Vec<&'static str>,
    /// Optional supplementary tools
    pub optional_tools: Vec<&'static str>,
    /// Required fields in the evidence data (for validation)
    pub required_fields: Vec<&'static str>,
    /// Human-readable evidence description (for transcript)
    pub evidence_description: &'static str,
}

/// Get configuration for a topic
pub fn get_topic_config(topic: EvidenceTopic) -> TopicConfig {
    match topic {
        EvidenceTopic::CpuInfo => TopicConfig {
            topic,
            required_tools: vec!["hw_snapshot_summary"],
            optional_tools: vec![],
            required_fields: vec!["cpu_model", "cores"],
            evidence_description: "hardware inventory (CPU model, cores)",
        },
        EvidenceTopic::MemoryInfo => TopicConfig {
            topic,
            required_tools: vec!["memory_info"],
            optional_tools: vec![],
            required_fields: vec!["total_gib", "available_gib"],
            evidence_description: "memory usage snapshot",
        },
        EvidenceTopic::KernelVersion => TopicConfig {
            topic,
            required_tools: vec!["kernel_version"],
            optional_tools: vec![],
            required_fields: vec!["kernel_release"],
            evidence_description: "kernel version info",
        },
        EvidenceTopic::DiskFree => TopicConfig {
            topic,
            required_tools: vec!["mount_usage"],
            optional_tools: vec!["disk_usage"],
            required_fields: vec!["root"],
            evidence_description: "disk usage snapshot",
        },
        EvidenceTopic::NetworkStatus => TopicConfig {
            topic,
            required_tools: vec!["network_status"],
            optional_tools: vec![],
            required_fields: vec!["has_default_route"],
            evidence_description: "network status snapshot",
        },
        EvidenceTopic::AudioStatus => TopicConfig {
            topic,
            required_tools: vec!["audio_status"],
            optional_tools: vec![],
            required_fields: vec!["pipewire_running"],
            evidence_description: "audio stack status",
        },
        EvidenceTopic::ServiceState => TopicConfig {
            topic,
            required_tools: vec!["service_status"],
            optional_tools: vec!["systemd_service_probe_v1"],
            required_fields: vec!["active"],
            evidence_description: "service status check",
        },
        EvidenceTopic::RecentErrors => TopicConfig {
            topic,
            required_tools: vec!["journal_warnings"],
            optional_tools: vec![],
            required_fields: vec![],
            evidence_description: "recent system log entries",
        },
        EvidenceTopic::BootTime => TopicConfig {
            topic,
            required_tools: vec!["boot_time_trend"],
            optional_tools: vec![],
            required_fields: vec![],
            evidence_description: "boot time analysis",
        },
        EvidenceTopic::PackagesChanged => TopicConfig {
            topic,
            required_tools: vec!["recent_installs", "what_changed"],
            optional_tools: vec![],
            required_fields: vec![],
            evidence_description: "recent package changes",
        },
        EvidenceTopic::GraphicsStatus => TopicConfig {
            topic,
            required_tools: vec!["hw_snapshot_summary"],
            optional_tools: vec![],
            required_fields: vec!["gpu"],
            evidence_description: "graphics hardware status",
        },
        EvidenceTopic::Alerts => TopicConfig {
            topic,
            required_tools: vec!["proactive_alerts_summary"],
            optional_tools: vec![],
            required_fields: vec![],
            evidence_description: "active system alerts",
        },
        EvidenceTopic::Unknown => TopicConfig {
            topic,
            required_tools: vec![],
            optional_tools: vec![],
            required_fields: vec![],
            evidence_description: "system information",
        },
    }
}

// ============================================================================
// Deterministic Topic Detection (Pre-LLM)
// ============================================================================

/// Topic detection result
#[derive(Debug, Clone)]
pub struct TopicDetection {
    /// Primary detected topic
    pub topic: EvidenceTopic,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Secondary topic (if ambiguous)
    pub secondary: Option<EvidenceTopic>,
    /// Service name if ServiceState detected
    pub service_name: Option<String>,
    /// Whether this looks like a diagnostic (problem) vs informational query
    pub is_diagnostic: bool,
}

/// Detect topics from user request (runs before LLM)
pub fn detect_topic(request: &str) -> TopicDetection {
    let lower = request.to_lowercase();

    // Disk/storage patterns - highest priority
    let disk_patterns = [
        ("disk space", 98),
        ("disk free", 98),
        ("free space", 95),
        ("storage space", 95),
        ("how much space", 98),
        ("space left", 95),
        ("space available", 95),
        ("space on /", 98),
        ("disk usage", 90),
        ("running out of space", 95),
        ("disk full", 90),
        ("disk is full", 95),
        ("storage left", 95),
        ("how full is", 85),
        ("root partition", 90),
    ];
    for (pattern, confidence) in disk_patterns {
        if lower.contains(pattern) {
            return TopicDetection {
                topic: EvidenceTopic::DiskFree,
                confidence,
                secondary: None,
                service_name: None,
                is_diagnostic: lower.contains("full") || lower.contains("running out"),
            };
        }
    }

    // Kernel version patterns
    let kernel_patterns = [
        ("kernel version", 98),
        ("kernel release", 98),
        ("what kernel", 95),
        ("linux version", 90),
        ("which kernel", 95),
        ("running kernel", 95),
        ("uname", 90),
        ("kernel am i", 95),
    ];
    for (pattern, confidence) in kernel_patterns {
        if lower.contains(pattern) {
            return TopicDetection {
                topic: EvidenceTopic::KernelVersion,
                confidence,
                secondary: None,
                service_name: None,
                is_diagnostic: false,
            };
        }
    }

    // Memory patterns
    let memory_patterns = [
        ("how much memory", 98),
        ("how much ram", 98),
        ("ram available", 95),
        ("memory available", 95),
        ("ram free", 95),
        ("memory free", 95),
        ("total memory", 95),
        ("total ram", 95),
        ("ram usage", 90),
        ("memory usage", 90),
        ("how much mem", 95),
        ("ram do i have", 95),
        ("memory do i have", 95),
    ];
    for (pattern, confidence) in memory_patterns {
        if lower.contains(pattern) {
            return TopicDetection {
                topic: EvidenceTopic::MemoryInfo,
                confidence,
                secondary: None,
                service_name: None,
                is_diagnostic: lower.contains("leak") || lower.contains("high"),
            };
        }
    }

    // CPU patterns
    let cpu_patterns = [
        ("what cpu", 98),
        ("which cpu", 95),
        ("cpu model", 95),
        ("processor model", 90),
        ("cpu info", 90),
        ("processor info", 90),
        ("what processor", 95),
        ("how many cores", 90),
        ("cpu cores", 90),
        ("cpu do i have", 98),
    ];
    for (pattern, confidence) in cpu_patterns {
        if lower.contains(pattern) {
            return TopicDetection {
                topic: EvidenceTopic::CpuInfo,
                confidence,
                secondary: None,
                service_name: None,
                is_diagnostic: lower.contains("high") || lower.contains("load"),
            };
        }
    }

    // Network patterns
    let network_patterns = [
        ("network status", 98),
        ("network connection", 95),
        ("internet connection", 95),
        ("am i connected", 95),
        ("am i online", 95),
        ("is network", 90),
        ("is wifi", 90),
        ("wifi status", 95),
        ("ethernet status", 95),
        ("wifi connected", 95),
        ("wifi working", 95),
        ("connection status", 90),
        ("default route", 90),
        ("network info", 90),
    ];
    for (pattern, confidence) in network_patterns {
        if lower.contains(pattern) {
            let is_diag = lower.contains("disconnect")
                || lower.contains("not working")
                || lower.contains("slow")
                || lower.contains("problem");
            return TopicDetection {
                topic: EvidenceTopic::NetworkStatus,
                confidence,
                secondary: None,
                service_name: None,
                is_diagnostic: is_diag,
            };
        }
    }

    // Audio patterns
    let audio_patterns = [
        ("audio status", 98),
        ("sound status", 95),
        ("is audio", 90),
        ("is sound", 90),
        ("audio working", 95),
        ("sound working", 95),
        ("pipewire status", 95),
        ("pulseaudio status", 95),
        ("pipewire running", 95),
        ("pulseaudio running", 95),
        ("no sound", 90),
        ("no audio", 90),
        ("speakers", 85),
    ];
    for (pattern, confidence) in audio_patterns {
        if lower.contains(pattern) {
            let is_diag = lower.contains("not working")
                || lower.contains("no sound")
                || lower.contains("no audio")
                || lower.contains("problem");
            return TopicDetection {
                topic: EvidenceTopic::AudioStatus,
                confidence,
                secondary: None,
                service_name: None,
                is_diagnostic: is_diag,
            };
        }
    }

    // Service status patterns - "is X running"
    let service_keywords = [
        ("nginx", "nginx"),
        ("docker", "docker"),
        ("sshd", "sshd"),
        ("ssh", "sshd"),
        ("apache", "apache2"),
        ("httpd", "httpd"),
        ("mysql", "mysql"),
        ("mariadb", "mariadb"),
        ("postgresql", "postgresql"),
        ("redis", "redis"),
        ("mongodb", "mongodb"),
        ("systemd", "systemd"),
        ("networkmanager", "NetworkManager"),
        ("bluetooth", "bluetooth"),
    ];
    for (keyword, service) in service_keywords {
        if lower.contains(keyword) {
            if lower.contains("running")
                || lower.contains("status")
                || lower.contains("started")
                || lower.contains("enabled")
                || lower.contains("active")
                || lower.contains("is ")
            {
                return TopicDetection {
                    topic: EvidenceTopic::ServiceState,
                    confidence: 90,
                    secondary: None,
                    service_name: Some(service.to_string()),
                    is_diagnostic: lower.contains("not") || lower.contains("fail"),
                };
            }
        }
    }

    // Boot time patterns
    let boot_patterns = [
        ("boot time", 95),
        ("startup time", 95),
        ("how long to boot", 95),
        ("boot slow", 90),
        ("slow boot", 90),
        ("boot analysis", 90),
    ];
    for (pattern, confidence) in boot_patterns {
        if lower.contains(pattern) {
            return TopicDetection {
                topic: EvidenceTopic::BootTime,
                confidence,
                secondary: None,
                service_name: None,
                is_diagnostic: lower.contains("slow"),
            };
        }
    }

    // Package change patterns
    let package_patterns = [
        ("what changed", 90),
        ("recently installed", 95),
        ("installed recently", 95),
        ("packages changed", 95),
        ("new packages", 90),
        ("updated packages", 90),
    ];
    for (pattern, confidence) in package_patterns {
        if lower.contains(pattern) {
            return TopicDetection {
                topic: EvidenceTopic::PackagesChanged,
                confidence,
                secondary: None,
                service_name: None,
                is_diagnostic: false,
            };
        }
    }

    // Alert patterns
    let alert_patterns = [
        ("show alerts", 98),
        ("what alerts", 98),
        ("any alerts", 95),
        ("any warnings", 95),
        ("show warnings", 95),
        ("any issues", 90),
        ("why warning", 95),
        ("system alerts", 95),
        ("active alerts", 98),
    ];
    for (pattern, confidence) in alert_patterns {
        if lower.contains(pattern) {
            return TopicDetection {
                topic: EvidenceTopic::Alerts,
                confidence,
                secondary: None,
                service_name: None,
                is_diagnostic: true,
            };
        }
    }

    // Graphics patterns
    let graphics_patterns = [
        ("gpu", 90),
        ("graphics card", 95),
        ("video card", 95),
        ("graphics driver", 90),
        ("nvidia", 85),
        ("amd gpu", 90),
        ("intel graphics", 90),
    ];
    for (pattern, confidence) in graphics_patterns {
        if lower.contains(pattern) {
            return TopicDetection {
                topic: EvidenceTopic::GraphicsStatus,
                confidence,
                secondary: None,
                service_name: None,
                is_diagnostic: lower.contains("not working") || lower.contains("problem"),
            };
        }
    }

    // Recent errors
    if lower.contains("error") || lower.contains("errors") {
        if lower.contains("recent") || lower.contains("latest") || lower.contains("show") {
            return TopicDetection {
                topic: EvidenceTopic::RecentErrors,
                confidence: 85,
                secondary: None,
                service_name: None,
                is_diagnostic: true,
            };
        }
    }

    // Unknown - needs LLM
    TopicDetection {
        topic: EvidenceTopic::Unknown,
        confidence: 0,
        secondary: None,
        service_name: None,
        is_diagnostic: false,
    }
}

// ============================================================================
// Answer Templates
// ============================================================================

/// Generate a structured answer from evidence data
pub fn generate_answer(topic: EvidenceTopic, data: &serde_json::Value) -> Option<String> {
    match topic {
        EvidenceTopic::DiskFree => {
            let root = data.get("root")?;
            let avail = root.get("avail_human")?.as_str()?;
            let use_pct = root.get("use_percent")?.as_str()?;
            let free_pct = 100 - use_pct.parse::<u8>().unwrap_or(0);
            Some(format!("Free space on /: {} ({}% free).", avail, free_pct))
        }
        EvidenceTopic::KernelVersion => {
            let kernel = data.get("kernel_release")?.as_str()?;
            Some(format!("Kernel: {}", kernel))
        }
        EvidenceTopic::MemoryInfo => {
            let total = data.get("total_gib")?.as_str()?;
            let avail = data.get("available_gib")?.as_str()?;
            Some(format!(
                "Memory: {} GiB total, {} GiB available.",
                total, avail
            ))
        }
        EvidenceTopic::CpuInfo => {
            // Try to extract CPU info from hw_snapshot_summary format
            if let Some(cpu_model) = data.get("cpu_model").and_then(|v| v.as_str()) {
                let cores = data.get("cores").and_then(|v| v.as_u64()).unwrap_or(0);
                let threads = data
                    .get("threads")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(cores);
                if cores > 0 {
                    return Some(format!(
                        "CPU: {} ({} cores, {} threads).",
                        cpu_model, cores, threads
                    ));
                }
                return Some(format!("CPU: {}.", cpu_model));
            }
            None
        }
        EvidenceTopic::NetworkStatus => {
            let has_route = data.get("has_default_route")?.as_bool()?;
            let primary = data
                .get("primary_interface")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let status = if has_route {
                "connected"
            } else {
                "disconnected"
            };
            Some(format!(
                "Network: {}, primary interface: {}.",
                status, primary
            ))
        }
        EvidenceTopic::AudioStatus => {
            let pipewire = data.get("pipewire_running")?.as_bool()?;
            let status = if pipewire { "running" } else { "not running" };
            Some(format!("Audio: PipeWire is {}.", status))
        }
        EvidenceTopic::ServiceState => {
            let active = data.get("active")?.as_bool().unwrap_or(false);
            let service_name = data
                .get("service_name")
                .and_then(|v| v.as_str())
                .unwrap_or("service");
            let status = if active { "running" } else { "stopped" };
            Some(format!("Service {}: {}.", service_name, status))
        }
        _ => None,
    }
}

// ============================================================================
// Validation (v0.0.63: Strict enforcement)
// ============================================================================

/// Validation result for Junior verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicValidation {
    /// Whether the evidence matches the topic
    pub topic_match: bool,
    /// List of missing required fields
    pub missing_fields: Vec<String>,
    /// Whether the answer contains the expected value
    pub answer_contains_value: bool,
    /// Suggested penalty to reliability score
    pub penalty: u8,
    /// v0.0.63: Maximum allowed reliability (40 if answer doesn't match question)
    pub max_reliability: Option<u8>,
    /// v0.0.63: Human-readable critique for narration
    pub critique: String,
    /// v0.0.63: Whether this is a strict topic that must have matching answer
    pub strict_topic: bool,
    /// v0.0.63: Evidence age in seconds (freshness tracking)
    pub evidence_age_secs: Option<u64>,
    /// v0.0.63: Freshness penalty (stale evidence gets penalized)
    pub freshness_penalty: u8,
}

/// v0.0.63: Check if answer contains specific numeric data for a topic
fn answer_has_data_for_topic(topic: EvidenceTopic, answer: &str) -> (bool, &'static str) {
    let answer_lower = answer.to_lowercase();

    match topic {
        EvidenceTopic::DiskFree => {
            // Must have: disk-related number (GiB, GB, %, bytes, etc.)
            let has_disk_keywords = answer_lower.contains("free")
                || answer_lower.contains("space")
                || answer_lower.contains("disk");
            let has_quantity = answer_lower.contains("gib")
                || answer_lower.contains("gb")
                || answer_lower.contains("mib")
                || answer_lower.contains("mb")
                || answer_lower.contains("%")
                || answer.chars().any(|c| c.is_ascii_digit());

            if has_disk_keywords && has_quantity {
                (true, "")
            } else {
                (false, "Answer must include disk space quantity (GiB/GB/%). Got unrelated information.")
            }
        }

        EvidenceTopic::KernelVersion => {
            // Must have: kernel version string like 6.x.x-arch or linux-x.x
            let has_kernel = answer_lower.contains("kernel") || answer_lower.contains("linux");
            let has_version = answer.contains("-arch")
                || answer.contains("-lts")
                || (answer.chars().any(|c| c == '.') && answer.chars().any(|c| c.is_ascii_digit()));

            if has_kernel || has_version {
                (true, "")
            } else {
                (false, "Answer must include kernel version (e.g., 6.x.x-arch1-1). Got unrelated information.")
            }
        }

        EvidenceTopic::MemoryInfo => {
            // Must have: memory quantity
            let has_memory = answer_lower.contains("memory")
                || answer_lower.contains("ram")
                || answer_lower.contains("total")
                || answer_lower.contains("available");
            let has_quantity = answer_lower.contains("gib")
                || answer_lower.contains("gb")
                || answer_lower.contains("mib")
                || answer.chars().any(|c| c.is_ascii_digit());

            if has_memory && has_quantity {
                (true, "")
            } else {
                (
                    false,
                    "Answer must include memory quantity (GiB/GB). Got unrelated information.",
                )
            }
        }

        EvidenceTopic::CpuInfo => {
            // Must have: CPU model or core count
            let has_cpu = answer_lower.contains("cpu")
                || answer_lower.contains("processor")
                || answer_lower.contains("amd")
                || answer_lower.contains("intel")
                || answer_lower.contains("ryzen")
                || answer_lower.contains("core");

            if has_cpu {
                (true, "")
            } else {
                (
                    false,
                    "Answer must include CPU information. Got unrelated information.",
                )
            }
        }

        EvidenceTopic::NetworkStatus => {
            // Must have: network-related status
            let has_network = answer_lower.contains("network")
                || answer_lower.contains("connect")
                || answer_lower.contains("interface")
                || answer_lower.contains("route")
                || answer_lower.contains("ip")
                || answer_lower.contains("link")
                || answer_lower.contains("wifi")
                || answer_lower.contains("ethernet");

            if has_network {
                (true, "")
            } else {
                (
                    false,
                    "Answer must include network status information. Got unrelated information.",
                )
            }
        }

        EvidenceTopic::AudioStatus => {
            // Must have: audio-related status
            let has_audio = answer_lower.contains("audio")
                || answer_lower.contains("sound")
                || answer_lower.contains("pipewire")
                || answer_lower.contains("pulseaudio")
                || answer_lower.contains("wireplumber")
                || answer_lower.contains("sink")
                || answer_lower.contains("source")
                || answer_lower.contains("alsa");

            if has_audio {
                (true, "")
            } else {
                (
                    false,
                    "Answer must include audio status information. Got unrelated information.",
                )
            }
        }

        EvidenceTopic::ServiceState => {
            // Must have: service status
            let has_service = answer_lower.contains("running")
                || answer_lower.contains("stopped")
                || answer_lower.contains("active")
                || answer_lower.contains("inactive")
                || answer_lower.contains("enabled")
                || answer_lower.contains("disabled")
                || answer_lower.contains("service");

            if has_service {
                (true, "")
            } else {
                (
                    false,
                    "Answer must include service status. Got unrelated information.",
                )
            }
        }

        // Other topics don't have strict validation
        _ => (true, ""),
    }
}

/// v0.0.63: Check if topic requires strict answer validation
fn is_strict_topic(topic: EvidenceTopic) -> bool {
    matches!(
        topic,
        EvidenceTopic::DiskFree
            | EvidenceTopic::KernelVersion
            | EvidenceTopic::MemoryInfo
            | EvidenceTopic::CpuInfo
            | EvidenceTopic::NetworkStatus
            | EvidenceTopic::AudioStatus
            | EvidenceTopic::ServiceState
    )
}

/// Validate that evidence matches the requested topic (v0.0.63: strict enforcement)
pub fn validate_evidence(
    topic: EvidenceTopic,
    evidence_data: &serde_json::Value,
    answer: &str,
) -> TopicValidation {
    let config = get_topic_config(topic);
    let mut missing = Vec::new();
    let strict = is_strict_topic(topic);

    // Check required fields in evidence
    for field in &config.required_fields {
        if evidence_data.get(*field).is_none() {
            missing.push(field.to_string());
        }
    }

    // v0.0.63: Strict answer content check
    let (answer_ok, critique_msg) = answer_has_data_for_topic(topic, answer);

    let topic_match = missing.is_empty();

    // v0.0.63: Calculate penalty and max reliability cap
    let (penalty, max_reliability, critique) = if !topic_match {
        // Missing evidence - heavy penalty
        (
            60,
            Some(40),
            format!(
                "Missing required evidence: {}. Need to gather {} data.",
                missing.join(", "),
                topic.human_label()
            ),
        )
    } else if !answer_ok && strict {
        // v0.0.63: Answer doesn't contain expected data - CAP at 40%
        (60, Some(40), critique_msg.to_string())
    } else if !answer_ok {
        // Non-strict topic with wrong answer
        (30, None, critique_msg.to_string())
    } else {
        (0, None, String::new())
    };

    TopicValidation {
        topic_match,
        missing_fields: missing,
        answer_contains_value: answer_ok,
        penalty,
        max_reliability,
        critique,
        strict_topic: strict,
        evidence_age_secs: None, // Set by caller with evidence timestamp
        freshness_penalty: 0,    // Set by calculate_freshness_penalty
    }
}

/// v0.0.63: Calculate freshness penalty based on evidence age
/// Fresher evidence = higher reliability
/// Stale evidence (>30 min) starts getting penalized
pub fn calculate_freshness_penalty(evidence_age_secs: u64) -> u8 {
    match evidence_age_secs {
        0..=300 => 0,      // < 5 min: fresh, no penalty
        301..=1800 => 5,   // 5-30 min: slightly stale
        1801..=3600 => 10, // 30-60 min: stale
        3601..=7200 => 15, // 1-2 hours: quite stale
        _ => 20,           // > 2 hours: very stale
    }
}

/// v0.0.63: Update validation with evidence freshness data
pub fn with_evidence_freshness(
    mut validation: TopicValidation,
    evidence_timestamp: u64,
    current_time: u64,
) -> TopicValidation {
    if evidence_timestamp > 0 && current_time >= evidence_timestamp {
        let age_secs = current_time - evidence_timestamp;
        let freshness_penalty = calculate_freshness_penalty(age_secs);

        validation.evidence_age_secs = Some(age_secs);
        validation.freshness_penalty = freshness_penalty;
        validation.penalty = validation.penalty.saturating_add(freshness_penalty);

        // Update critique if evidence is stale
        if freshness_penalty > 0 {
            if !validation.critique.is_empty() {
                validation.critique.push_str("; ");
            }
            let age_desc = match age_secs {
                0..=300 => "fresh",
                301..=1800 => "slightly stale",
                1801..=3600 => "stale",
                3601..=7200 => "quite stale",
                _ => "very stale",
            };
            validation.critique.push_str(&format!(
                "Evidence is {} ({} seconds old, -{} penalty)",
                age_desc, age_secs, freshness_penalty
            ));
        }
    }

    validation
}

/// v0.0.63: Calculate capped reliability score
pub fn cap_reliability(base_score: u8, validation: &TopicValidation) -> u8 {
    let after_penalty = base_score.saturating_sub(validation.penalty);

    if let Some(max) = validation.max_reliability {
        after_penalty.min(max)
    } else {
        after_penalty
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_detection() {
        let cases = [
            "how much disk space is free",
            "how much free space do I have",
            "disk usage",
            "space left on root",
            "is my disk full",
        ];
        for case in cases {
            let result = detect_topic(case);
            assert_eq!(
                result.topic,
                EvidenceTopic::DiskFree,
                "Expected DiskFree for '{}', got {:?}",
                case,
                result.topic
            );
            assert!(result.confidence >= 85, "Low confidence for '{}'", case);
        }
    }

    #[test]
    fn test_kernel_detection() {
        let cases = [
            "what kernel version am I running",
            "kernel version",
            "which kernel",
            "what linux version",
        ];
        for case in cases {
            let result = detect_topic(case);
            assert_eq!(
                result.topic,
                EvidenceTopic::KernelVersion,
                "Expected KernelVersion for '{}', got {:?}",
                case,
                result.topic
            );
        }
    }

    #[test]
    fn test_memory_detection() {
        let cases = [
            "how much memory do I have",
            "how much ram",
            "ram available",
            "total memory",
        ];
        for case in cases {
            let result = detect_topic(case);
            assert_eq!(
                result.topic,
                EvidenceTopic::MemoryInfo,
                "Expected MemoryInfo for '{}', got {:?}",
                case,
                result.topic
            );
        }
    }

    #[test]
    fn test_cpu_detection() {
        let cases = [
            "what cpu do I have",
            "cpu model",
            "processor info",
            "how many cores",
        ];
        for case in cases {
            let result = detect_topic(case);
            assert_eq!(
                result.topic,
                EvidenceTopic::CpuInfo,
                "Expected CpuInfo for '{}', got {:?}",
                case,
                result.topic
            );
        }
    }

    #[test]
    fn test_service_detection() {
        let result = detect_topic("is docker running");
        assert_eq!(result.topic, EvidenceTopic::ServiceState);
        assert_eq!(result.service_name, Some("docker".to_string()));
    }

    #[test]
    fn test_diagnostic_flag() {
        let disk_problem = detect_topic("disk is full");
        assert!(disk_problem.is_diagnostic);

        let disk_info = detect_topic("how much disk space");
        assert!(!disk_info.is_diagnostic);
    }

    #[test]
    fn test_answer_generation() {
        let disk_data = serde_json::json!({
            "root": {
                "avail_human": "50 GiB",
                "use_percent": "60"
            }
        });
        let answer = generate_answer(EvidenceTopic::DiskFree, &disk_data);
        assert!(answer.is_some());
        let answer = answer.unwrap();
        assert!(answer.contains("50 GiB"));
        assert!(answer.contains("40%"));

        let kernel_data = serde_json::json!({
            "kernel_release": "6.6.1-arch1-1"
        });
        let answer = generate_answer(EvidenceTopic::KernelVersion, &kernel_data);
        assert!(answer.is_some());
        assert!(answer.unwrap().contains("6.6.1-arch1-1"));
    }

    #[test]
    fn test_validation() {
        // Good evidence
        let good_data = serde_json::json!({"kernel_release": "6.6.1-arch1-1"});
        let good_answer = "You are running kernel 6.6.1-arch1-1";
        let result = validate_evidence(EvidenceTopic::KernelVersion, &good_data, good_answer);
        assert!(result.topic_match);
        assert!(result.answer_contains_value);
        assert_eq!(result.penalty, 0);
        assert!(result.max_reliability.is_none());

        // Missing field
        let bad_data = serde_json::json!({"other": "stuff"});
        let result = validate_evidence(EvidenceTopic::KernelVersion, &bad_data, good_answer);
        assert!(!result.topic_match);
        assert!(result
            .missing_fields
            .contains(&"kernel_release".to_string()));
        assert_eq!(result.penalty, 60);
        assert_eq!(result.max_reliability, Some(40)); // v0.0.63: capped at 40%

        // Wrong answer content for strict topic
        let result = validate_evidence(
            EvidenceTopic::KernelVersion,
            &good_data,
            "Your CPU is AMD Ryzen",
        );
        assert!(result.topic_match);
        assert!(!result.answer_contains_value);
        assert_eq!(result.penalty, 60); // v0.0.63: strict topic
        assert_eq!(result.max_reliability, Some(40)); // v0.0.63: capped at 40%
    }

    #[test]
    fn test_strict_topic_caps_at_40() {
        // Test that wrong answers for strict topics cap at 40%
        let good_data = serde_json::json!({"total_gib": "32", "available_gib": "16"});
        let wrong_answer = "Your CPU is AMD Ryzen with 8 cores"; // Memory question, CPU answer

        let result = validate_evidence(EvidenceTopic::MemoryInfo, &good_data, wrong_answer);
        assert!(result.strict_topic);
        assert!(!result.answer_contains_value);
        assert_eq!(result.max_reliability, Some(40));

        // Verify cap_reliability works
        let capped = cap_reliability(90, &result);
        assert!(capped <= 40, "Should be capped at 40, got {}", capped);
    }

    #[test]
    fn test_disk_answer_validation() {
        let disk_data = serde_json::json!({"root": {"avail_human": "50 GiB"}});

        // Good answer
        let good = validate_evidence(
            EvidenceTopic::DiskFree,
            &disk_data,
            "Free space: 50 GiB (40%)",
        );
        assert!(good.answer_contains_value);
        assert_eq!(good.penalty, 0);

        // Bad answer (CPU info for disk question)
        let bad = validate_evidence(
            EvidenceTopic::DiskFree,
            &disk_data,
            "AMD Ryzen 9 5900X processor",
        );
        assert!(!bad.answer_contains_value);
        assert_eq!(bad.max_reliability, Some(40));
    }

    #[test]
    fn test_network_detection() {
        let cases = [
            "what is my network status",
            "am I connected to the internet",
            "is my wifi working",
        ];
        for case in cases {
            let result = detect_topic(case);
            assert_eq!(
                result.topic,
                EvidenceTopic::NetworkStatus,
                "Expected NetworkStatus for '{}', got {:?}",
                case,
                result.topic
            );
        }
    }

    #[test]
    fn test_audio_detection() {
        let cases = ["is my audio working", "audio status", "is pipewire running"];
        for case in cases {
            let result = detect_topic(case);
            assert_eq!(
                result.topic,
                EvidenceTopic::AudioStatus,
                "Expected AudioStatus for '{}', got {:?}",
                case,
                result.topic
            );
        }
    }

    // v0.0.63: Freshness tracking tests
    #[test]
    fn test_freshness_penalty_fresh() {
        // Fresh evidence (< 5 min) - no penalty
        assert_eq!(calculate_freshness_penalty(0), 0);
        assert_eq!(calculate_freshness_penalty(60), 0); // 1 min
        assert_eq!(calculate_freshness_penalty(300), 0); // 5 min exactly
    }

    #[test]
    fn test_freshness_penalty_stale() {
        // Progressively stale evidence
        assert_eq!(calculate_freshness_penalty(600), 5); // 10 min
        assert_eq!(calculate_freshness_penalty(1800), 5); // 30 min
        assert_eq!(calculate_freshness_penalty(3000), 10); // 50 min
        assert_eq!(calculate_freshness_penalty(3600), 10); // 1 hour
        assert_eq!(calculate_freshness_penalty(5400), 15); // 1.5 hours
        assert_eq!(calculate_freshness_penalty(7200), 15); // 2 hours
        assert_eq!(calculate_freshness_penalty(10000), 20); // > 2 hours
    }

    #[test]
    fn test_with_evidence_freshness() {
        let good_data = serde_json::json!({"kernel_release": "6.6.1-arch1-1"});
        let good_answer = "You are running kernel 6.6.1-arch1-1";
        let validation = validate_evidence(EvidenceTopic::KernelVersion, &good_data, good_answer);

        // Fresh evidence - no additional penalty
        let fresh_validation = with_evidence_freshness(validation.clone(), 1000, 1100); // 100 secs
        assert_eq!(fresh_validation.freshness_penalty, 0);
        assert_eq!(fresh_validation.evidence_age_secs, Some(100));

        // Stale evidence (30 min old) - gets penalty
        let stale_validation = with_evidence_freshness(validation.clone(), 1000, 2800); // 1800 secs
        assert_eq!(stale_validation.freshness_penalty, 5);
        assert_eq!(stale_validation.evidence_age_secs, Some(1800));
        assert!(stale_validation.critique.contains("stale"));

        // Very stale evidence (3 hours old)
        let very_stale = with_evidence_freshness(validation.clone(), 1000, 11800); // 10800 secs
        assert_eq!(very_stale.freshness_penalty, 20);
        assert!(very_stale.critique.contains("very stale"));
    }
}
