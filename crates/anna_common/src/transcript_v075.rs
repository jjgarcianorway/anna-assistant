//! Transcript Rendering v0.0.75
//!
//! Enhanced fly-on-the-wall transcript rendering:
//! - Human mode: Meaningful dialogue without evidence IDs or tool names
//!   - Evidence referenced by short plain descriptions
//!   - Reliability score and timing summary retained
//! - Debug mode: Full canonical translator lines, tool names, evidence IDs
//!
//! This module provides the translation layer between internal events and
//! transcript output.

use crate::evidence_topic::EvidenceTopic;
use crate::humanizer::{DepartmentTag, HumanLabel, MessageTone};
use crate::transcript_events::TranscriptMode;
use serde::{Deserialize, Serialize};

/// Evidence description for human mode (no IDs or tool names)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanEvidence {
    /// Short plain description (e.g., "Hardware snapshot: CPU model and core count")
    pub description: String,
    /// Source category (e.g., "hardware snapshot", "service snapshot", "journal scan")
    pub source: String,
    /// Optional key finding from this evidence
    pub key_finding: Option<String>,
}

/// Convert internal evidence to human-readable format
pub fn humanize_evidence(
    tool_name: &str,
    topic: Option<EvidenceTopic>,
    summary: &str,
) -> HumanEvidence {
    let (description, source) = match topic {
        Some(EvidenceTopic::CpuInfo) => (
            format!("Hardware snapshot: {}", summary),
            "hardware snapshot".to_string(),
        ),
        Some(EvidenceTopic::MemoryInfo) => (
            format!("Memory status: {}", summary),
            "memory snapshot".to_string(),
        ),
        Some(EvidenceTopic::KernelVersion) => (
            format!("System info: {}", summary),
            "kernel info".to_string(),
        ),
        Some(EvidenceTopic::DiskFree) => (
            format!("Storage status: {}", summary),
            "storage snapshot".to_string(),
        ),
        Some(EvidenceTopic::NetworkStatus) => (
            format!("Network status: {}", summary),
            "network snapshot".to_string(),
        ),
        Some(EvidenceTopic::AudioStatus) => (
            format!("Audio stack: {}", summary),
            "audio snapshot".to_string(),
        ),
        Some(EvidenceTopic::ServiceState) => (
            format!("Service status: {}", summary),
            "service snapshot".to_string(),
        ),
        Some(EvidenceTopic::BootTime) => (
            format!("Boot analysis: {}", summary),
            "boot timeline".to_string(),
        ),
        Some(EvidenceTopic::GraphicsStatus) => (
            format!("Graphics status: {}", summary),
            "graphics snapshot".to_string(),
        ),
        Some(EvidenceTopic::RecentErrors) => (
            format!("Journal scan: {}", summary),
            "error journal".to_string(),
        ),
        Some(EvidenceTopic::Alerts) => (
            format!("Active alerts: {}", summary),
            "alert check".to_string(),
        ),
        Some(EvidenceTopic::PackagesChanged) => (
            format!("Package history: {}", summary),
            "package log".to_string(),
        ),
        _ => {
            // Fallback: derive from tool name
            let source = tool_name_to_source(tool_name);
            (format!("{}: {}", source, summary), source)
        }
    };

    HumanEvidence {
        description,
        source,
        key_finding: if !summary.is_empty() {
            Some(summary.to_string())
        } else {
            None
        },
    }
}

/// Convert tool name to human-readable source
fn tool_name_to_source(tool_name: &str) -> String {
    match tool_name {
        "hw_snapshot_summary" | "hw_snapshot" => "hardware snapshot".to_string(),
        "memory_info" | "mem_summary" => "memory snapshot".to_string(),
        "kernel_version" | "uname_summary" => "kernel info".to_string(),
        "mount_usage" | "disk_usage" => "storage snapshot".to_string(),
        "network_status" | "nm_summary" | "net_routes_summary" => "network snapshot".to_string(),
        "audio_status" | "audio_services_summary" => "audio snapshot".to_string(),
        "service_status" | "systemd_service_probe_v1" => "service snapshot".to_string(),
        "boot_time_summary" | "boot_time_trend" => "boot timeline".to_string(),
        "recent_errors_summary" | "journal_warnings" => "error journal".to_string(),
        "proactive_alerts_summary" => "alert check".to_string(),
        "recent_installs" | "what_changed" => "package history".to_string(),
        _ => {
            // Try to extract meaningful name
            let name = tool_name
                .replace('_', " ")
                .replace("summary", "")
                .replace("v1", "")
                .trim()
                .to_string();
            if name.is_empty() {
                "system snapshot".to_string()
            } else {
                format!("{} snapshot", name)
            }
        }
    }
}

/// Render evidence for human mode transcript
pub fn render_human_evidence(
    evidence_list: &[(String, String, Option<EvidenceTopic>)],
) -> Vec<String> {
    let mut lines = Vec::new();

    for (tool_name, summary, topic) in evidence_list {
        let human = humanize_evidence(tool_name, *topic, summary);
        lines.push(format!("  {}", human.description));
    }

    lines
}

/// Render evidence for debug mode transcript (includes IDs and tool names)
pub fn render_debug_evidence(
    evidence_list: &[(String, String, String, Option<EvidenceTopic>)],
) -> Vec<String> {
    let mut lines = Vec::new();

    for (evidence_id, tool_name, summary, _topic) in evidence_list {
        lines.push(format!("  [{}] {} -> {}", evidence_id, tool_name, summary));
    }

    lines
}

/// Human mode staff message (role talking like a person)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanStaffMessage {
    /// Role tag (e.g., "service desk", "network", "storage")
    pub tag: String,
    /// The message text
    pub text: String,
    /// Tone (affects rendering)
    pub tone: MessageTone,
    /// Whether this is a side-thread message (indented)
    pub is_side_thread: bool,
}

impl HumanStaffMessage {
    /// Create a new staff message
    pub fn new(tag: &str, text: &str) -> Self {
        Self {
            tag: tag.to_string(),
            text: text.to_string(),
            tone: MessageTone::Neutral,
            is_side_thread: false,
        }
    }

    /// Set tone
    pub fn with_tone(mut self, tone: MessageTone) -> Self {
        self.tone = tone;
        self
    }

    /// Mark as side thread
    pub fn as_side_thread(mut self) -> Self {
        self.is_side_thread = true;
        self
    }

    /// Format for display
    pub fn format(&self) -> String {
        if self.is_side_thread {
            format!("  [{}] {}", self.tag, self.text)
        } else {
            format!("[{}] {}", self.tag, self.text)
        }
    }
}

/// Generate opening message for human mode
pub fn human_case_open() -> HumanStaffMessage {
    HumanStaffMessage::new("service desk", "Let me look into that for you.")
}

/// Generate triage message for human mode
pub fn human_triage(primary: DepartmentTag, supporting: &[DepartmentTag]) -> HumanStaffMessage {
    let text = if supporting.is_empty() {
        format!("I'll have {} look into this.", primary.display_name())
    } else {
        let support_names: Vec<_> = supporting.iter().map(|d| d.display_name()).collect();
        format!(
            "I'll have {} look into this, with help from {}.",
            primary.display_name(),
            support_names.join(" and ")
        )
    };
    HumanStaffMessage::new("service desk", &text)
}

/// Generate finding message with confidence-based prefix
pub fn human_finding(dept: DepartmentTag, finding: &str, confidence: u8) -> HumanStaffMessage {
    let prefix = if confidence >= 90 {
        ""
    } else if confidence >= 75 {
        "It looks like "
    } else if confidence >= 60 {
        "I think "
    } else {
        "I'm not certain, but "
    };
    let text = format!("{}{}", prefix, finding);
    HumanStaffMessage::new(dept.tag(), &text)
}

/// Generate missing evidence message
pub fn human_missing_evidence(dept: DepartmentTag, topic: &str) -> HumanStaffMessage {
    let text = match dept {
        DepartmentTag::Network => format!("Can't confirm {} without more network data.", topic),
        DepartmentTag::Storage => format!("No clear evidence for {} in storage snapshot.", topic),
        DepartmentTag::Audio => format!("Audio stack doesn't show data for {}.", topic),
        DepartmentTag::Boot => format!("Boot timeline doesn't have {} info.", topic),
        DepartmentTag::Graphics => format!("Graphics snapshot doesn't cover {}.", topic),
        _ => format!("Can't confirm {} with available evidence.", topic),
    };
    HumanStaffMessage::new(dept.tag(), &text)
        .with_tone(MessageTone::Skeptical)
        .as_side_thread()
}

/// Generate reliability footer
pub fn human_reliability_footer(score: u8, reason: &str) -> String {
    let desc = if score >= 80 {
        "good evidence coverage"
    } else if score >= 60 {
        "some evidence gaps"
    } else {
        "limited evidence"
    };
    format!("Reliability: {}% ({}, {})", score, desc, reason)
}

/// Format timing summary for human mode
pub fn human_timing_summary(total_ms: u64, phases: &[(String, u64)]) -> String {
    let total_secs = total_ms as f64 / 1000.0;
    if total_secs < 1.0 {
        format!("Completed in {}ms", total_ms)
    } else {
        format!("Completed in {:.1}s", total_secs)
    }
}

/// Validate that human mode output doesn't contain forbidden terms
pub fn validate_human_output(output: &str) -> Result<(), Vec<String>> {
    let forbidden = [
        "[E1]",
        "[E2]",
        "[E3]",
        "[E4]",
        "[E5]",
        "_summary",
        "_probe",
        "_v1",
        "_v2",
        "hw_snapshot",
        "mem_summary",
        "mount_usage",
        "service_status",
        "network_status",
        "deterministic fallback",
        "parse retry",
        "tool_name:",
        "evidence_id:",
    ];

    let mut violations = Vec::new();
    for term in &forbidden {
        if output.contains(term) {
            violations.push(format!("Contains forbidden term: '{}'", term));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Determine transcript mode from environment/config
pub fn get_transcript_mode() -> TranscriptMode {
    // Priority 1: ANNA_DEBUG_TRANSCRIPT env var
    if std::env::var("ANNA_DEBUG_TRANSCRIPT")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return TranscriptMode::Debug;
    }

    // Priority 2: ANNA_UI_TRANSCRIPT_MODE env var
    if let Ok(mode) = std::env::var("ANNA_UI_TRANSCRIPT_MODE") {
        match mode.to_lowercase().as_str() {
            "debug" => return TranscriptMode::Debug,
            "human" => return TranscriptMode::Human,
            _ => {}
        }
    }

    // Priority 3: Config file (would need to read /etc/anna/config.toml)
    // For now, default to Human
    TranscriptMode::Human
}

/// Check if in debug mode
pub fn is_debug_mode() -> bool {
    matches!(get_transcript_mode(), TranscriptMode::Debug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_evidence_cpu() {
        let human = humanize_evidence(
            "hw_snapshot_summary",
            Some(EvidenceTopic::CpuInfo),
            "Intel i9-14900HX",
        );
        assert!(human.description.contains("Hardware snapshot"));
        assert!(human.description.contains("Intel i9-14900HX"));
        assert_eq!(human.source, "hardware snapshot");
    }

    #[test]
    fn test_humanize_evidence_memory() {
        let human = humanize_evidence(
            "memory_info",
            Some(EvidenceTopic::MemoryInfo),
            "32 GiB, 45% used",
        );
        assert!(human.description.contains("Memory status"));
        assert_eq!(human.source, "memory snapshot");
    }

    #[test]
    fn test_humanize_evidence_network() {
        let human = humanize_evidence(
            "network_status",
            Some(EvidenceTopic::NetworkStatus),
            "Connected via enp5s0",
        );
        assert!(human.description.contains("Network status"));
        assert_eq!(human.source, "network snapshot");
    }

    #[test]
    fn test_tool_name_to_source() {
        assert_eq!(
            tool_name_to_source("hw_snapshot_summary"),
            "hardware snapshot"
        );
        assert_eq!(tool_name_to_source("memory_info"), "memory snapshot");
        assert_eq!(tool_name_to_source("mount_usage"), "storage snapshot");
        assert_eq!(tool_name_to_source("network_status"), "network snapshot");
    }

    #[test]
    fn test_human_staff_message() {
        let msg = HumanStaffMessage::new("network", "Link is up on enp5s0");
        assert_eq!(msg.format(), "[network] Link is up on enp5s0");

        let side_msg = msg.as_side_thread();
        assert!(side_msg.format().starts_with("  "));
    }

    #[test]
    fn test_human_finding_confidence() {
        let high = human_finding(DepartmentTag::Network, "Link is up", 95);
        assert!(high.text.starts_with("Link is up"));

        let medium = human_finding(DepartmentTag::Network, "Link is up", 75);
        assert!(medium.text.starts_with("It looks like"));

        let low = human_finding(DepartmentTag::Network, "there's an issue", 55);
        assert!(low.text.starts_with("I'm not certain"));
    }

    #[test]
    fn test_validate_human_output_clean() {
        let clean = "[service desk] Let me look into that for you.
[network] Checking interface status.
  Network status: Connected via enp5s0
Reliability: 85% (good evidence coverage, network snapshot)";

        assert!(validate_human_output(clean).is_ok());
    }

    #[test]
    fn test_validate_human_output_violations() {
        let dirty = "[E1] hw_snapshot_summary -> Intel i9";
        let result = validate_human_output(dirty);
        assert!(result.is_err());
        let violations = result.unwrap_err();
        assert!(violations.iter().any(|v| v.contains("[E1]")));
        assert!(violations.iter().any(|v| v.contains("hw_snapshot")));
    }

    #[test]
    fn test_human_reliability_footer() {
        let high = human_reliability_footer(85, "hardware snapshot");
        assert!(high.contains("85%"));
        assert!(high.contains("good evidence coverage"));

        let medium = human_reliability_footer(65, "network snapshot");
        assert!(medium.contains("some evidence gaps"));

        let low = human_reliability_footer(45, "journal scan");
        assert!(low.contains("limited evidence"));
    }

    #[test]
    fn test_human_timing_summary() {
        let fast = human_timing_summary(250, &[]);
        assert!(fast.contains("250ms"));

        let slow = human_timing_summary(2500, &[]);
        assert!(slow.contains("2.5s"));
    }
}
