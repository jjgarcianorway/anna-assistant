//! Role-Based Phrasing v0.0.73
//!
//! Natural, role-appropriate phrases for Human transcript mode.
//! Makes each department sound like a real IT professional.
//!
//! NEVER fabricates actions or results - only changes how real events are described.

use super::roles::DepartmentTag;

/// Get role-appropriate phrase for case opening
pub fn phrase_case_open(dept: Option<DepartmentTag>) -> &'static str {
    match dept {
        Some(DepartmentTag::Network) => "Looking at this network request now.",
        Some(DepartmentTag::Storage) => "Taking a look at the storage situation.",
        Some(DepartmentTag::Performance) => "Reviewing the system performance request.",
        Some(DepartmentTag::Audio) => "Checking what's going on with audio.",
        Some(DepartmentTag::Graphics) => "Looking into the graphics issue.",
        Some(DepartmentTag::Boot) => "Let me check boot-related details.",
        Some(DepartmentTag::Security) => "Reviewing the security request.",
        Some(DepartmentTag::InfoDesk) => "Let me find that information for you.",
        None => "I'm triaging this request and deciding who should handle it.",
    }
}

/// Get role-appropriate phrase for evidence gathering start
pub fn phrase_evidence_start(dept: DepartmentTag) -> &'static str {
    match dept {
        DepartmentTag::Network => "Looking at link state and active connections.",
        DepartmentTag::Storage => "Checking disk and filesystem status.",
        DepartmentTag::Performance => "Checking the latest hardware and load snapshot.",
        DepartmentTag::Audio => "Checking audio server and device status.",
        DepartmentTag::Graphics => "Looking at display and GPU state.",
        DepartmentTag::Boot => "Reviewing boot timeline and init status.",
        DepartmentTag::Security => "Checking security state and policies.",
        DepartmentTag::InfoDesk => "Looking up the relevant information.",
    }
}

/// Get role-appropriate phrase for taking ownership
pub fn phrase_taking_ownership(dept: DepartmentTag) -> String {
    match dept {
        DepartmentTag::Network => "Network team is taking this case.".to_string(),
        DepartmentTag::Storage => "Storage team is taking this case.".to_string(),
        DepartmentTag::Performance => "Performance team is handling this.".to_string(),
        DepartmentTag::Audio => "Audio team is on it.".to_string(),
        DepartmentTag::Graphics => "Graphics team is looking into this.".to_string(),
        DepartmentTag::Boot => "Boot team is reviewing this.".to_string(),
        DepartmentTag::Security => "Security team is handling this request.".to_string(),
        DepartmentTag::InfoDesk => "Info desk is finding the answer.".to_string(),
    }
}

/// Get role-appropriate phrase for what will be checked first
pub fn phrase_first_check(dept: DepartmentTag) -> &'static str {
    match dept {
        DepartmentTag::Network => "We'll start by checking interface state and connectivity.",
        DepartmentTag::Storage => "First, we'll look at mount points and disk usage.",
        DepartmentTag::Performance => "Starting with CPU, memory, and load readings.",
        DepartmentTag::Audio => "Checking PipeWire/PulseAudio status first.",
        DepartmentTag::Graphics => "Starting with GPU driver and display status.",
        DepartmentTag::Boot => "Looking at the most recent boot sequence first.",
        DepartmentTag::Security => "Checking access logs and policy state first.",
        DepartmentTag::InfoDesk => "Searching documentation and knowledge base.",
    }
}

/// Get role-appropriate phrase for summarizing
pub fn phrase_summarizing() -> &'static str {
    "Summarizing what we know and what we don't."
}

/// Get role-appropriate phrase for confidence-based finding
pub fn phrase_finding_prefix(confidence: u8) -> &'static str {
    if confidence >= 85 {
        "" // High confidence - direct statement
    } else if confidence >= 70 {
        "Based on the evidence, " // Good confidence
    } else if confidence >= 50 {
        "From what I can see, " // Medium confidence
    } else {
        "I'm not certain, but " // Low confidence
    }
}

/// Get role-appropriate phrase for missing evidence
pub fn phrase_missing_evidence(dept: DepartmentTag, topic: &str) -> String {
    match dept {
        DepartmentTag::Network => {
            format!(
                "I can't confirm {} from snapshots alone; I need live signals.",
                topic
            )
        }
        DepartmentTag::Storage => {
            format!(
                "The current snapshot doesn't have {} details; would need a fresh check.",
                topic
            )
        }
        DepartmentTag::Performance => {
            format!(
                "No recent {} data in snapshot; live measurement needed.",
                topic
            )
        }
        _ => format!("Can't determine {} from available evidence.", topic),
    }
}

/// Evidence label with source context for Human mode
/// Shows topic + source without IDs: "CPU model and core count (from latest hardware snapshot)"
pub fn phrase_evidence_label(label: &str, source: &str) -> String {
    format!("{} (from {})", label, source)
}

/// Format evidence labels for human-readable display
pub struct HumanEvidenceLabel;

impl HumanEvidenceLabel {
    /// CPU info label
    pub fn cpu() -> &'static str {
        "CPU model and core count (from latest hardware snapshot)"
    }

    /// Memory info label
    pub fn memory() -> &'static str {
        "Memory capacity and usage (from latest hardware snapshot)"
    }

    /// Disk usage label
    pub fn disk() -> &'static str {
        "Disk usage summary (from storage snapshot)"
    }

    /// Network status label
    pub fn network() -> &'static str {
        "Network interface status (from network snapshot)"
    }

    /// Running services label
    pub fn services() -> &'static str {
        "Running services summary (from service snapshot)"
    }

    /// Boot timing label
    pub fn boot() -> &'static str {
        "Boot timing (from boot timeline)"
    }

    /// Audio status label
    pub fn audio() -> &'static str {
        "Audio server status (from audio snapshot)"
    }

    /// Graphics status label
    pub fn graphics() -> &'static str {
        "Display and GPU status (from graphics snapshot)"
    }

    /// Generic format with source
    pub fn with_source(topic: &str, source: &str) -> String {
        format!("{} (from {})", topic, source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_open_phrases() {
        let service_desk = phrase_case_open(None);
        assert!(service_desk.contains("triaging"));

        let network = phrase_case_open(Some(DepartmentTag::Network));
        assert!(network.contains("network"));
    }

    #[test]
    fn test_evidence_start_phrases() {
        let network = phrase_evidence_start(DepartmentTag::Network);
        assert!(network.contains("link state") || network.contains("connections"));

        let storage = phrase_evidence_start(DepartmentTag::Storage);
        assert!(storage.contains("disk") || storage.contains("filesystem"));
    }

    #[test]
    fn test_taking_ownership() {
        let msg = phrase_taking_ownership(DepartmentTag::Network);
        assert!(msg.contains("Network team"));
        assert!(msg.contains("case"));
    }

    #[test]
    fn test_first_check_phrases() {
        let network = phrase_first_check(DepartmentTag::Network);
        assert!(network.contains("interface") || network.contains("connectivity"));

        let storage = phrase_first_check(DepartmentTag::Storage);
        assert!(storage.contains("mount") || storage.contains("disk"));
    }

    #[test]
    fn test_confidence_prefix() {
        assert_eq!(phrase_finding_prefix(90), "");
        assert!(phrase_finding_prefix(75).contains("evidence"));
        assert!(phrase_finding_prefix(55).contains("see"));
        assert!(phrase_finding_prefix(30).contains("not certain"));
    }

    #[test]
    fn test_human_evidence_labels() {
        let cpu = HumanEvidenceLabel::cpu();
        assert!(cpu.contains("CPU"));
        assert!(cpu.contains("hardware snapshot"));
        assert!(!cpu.contains("[E"));

        let disk = HumanEvidenceLabel::disk();
        assert!(disk.contains("Disk"));
        assert!(disk.contains("storage snapshot"));
    }

    #[test]
    fn test_missing_evidence_phrases() {
        let msg = phrase_missing_evidence(DepartmentTag::Network, "link state");
        assert!(msg.contains("can't confirm"));
        assert!(msg.contains("link state"));
        assert!(msg.contains("live signals"));
    }
}
