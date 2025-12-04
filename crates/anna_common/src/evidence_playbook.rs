//! Evidence Playbook v0.0.67 - Department Evidence Plans
//!
//! Provides structured evidence gathering for department investigations:
//! - PlaybookTopic: What to check and why it matters
//! - PlaybookEvidence: Human + debug summaries with raw refs
//! - PlaybookBundle: Collection with coverage scoring
//! - CauseCategory: Root cause classification per domain
//!
//! Human Mode never shows IDs, tool names, or raw output.
//! It shows topic titles and human summaries only.

use serde::{Deserialize, Serialize};

// ============================================================================
// Playbook Topic - What to check
// ============================================================================

/// A topic in an evidence playbook (what to check and why)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookTopic {
    /// Topic ID (e.g., "net_link", "storage_mount")
    pub id: String,
    /// Human-readable title (e.g., "Link State")
    pub title: String,
    /// Why this matters for diagnosis
    pub why_it_matters: String,
    /// Tool steps to gather this evidence
    pub tool_steps: Vec<String>,
    /// Is this topic required or optional?
    pub required: bool,
}

impl PlaybookTopic {
    pub fn required(id: &str, title: &str, why: &str, tools: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            why_it_matters: why.to_string(),
            tool_steps: tools.iter().map(|s| s.to_string()).collect(),
            required: true,
        }
    }

    pub fn optional(id: &str, title: &str, why: &str, tools: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            why_it_matters: why.to_string(),
            tool_steps: tools.iter().map(|s| s.to_string()).collect(),
            required: false,
        }
    }
}

// ============================================================================
// Playbook Evidence - What was found
// ============================================================================

/// Evidence collected for a playbook topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookEvidence {
    /// Topic ID this evidence addresses
    pub topic_id: String,
    /// Human-readable summary (no tool names, no IDs, no raw output)
    pub summary_human: String,
    /// Debug summary (tool names, more detail)
    pub summary_debug: String,
    /// Raw references (commands/paths used, debug only)
    pub raw_refs: Vec<String>,
    /// Whether collection succeeded
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl PlaybookEvidence {
    pub fn success(topic_id: &str, human: &str, debug: &str) -> Self {
        Self {
            topic_id: topic_id.to_string(),
            summary_human: human.to_string(),
            summary_debug: debug.to_string(),
            raw_refs: Vec::new(),
            success: true,
            duration_ms: 0,
        }
    }

    pub fn failed(topic_id: &str, reason: &str) -> Self {
        Self {
            topic_id: topic_id.to_string(),
            summary_human: format!("Could not check: {}", reason),
            summary_debug: format!("FAILED: {}", reason),
            raw_refs: Vec::new(),
            success: false,
            duration_ms: 0,
        }
    }

    pub fn with_refs(mut self, refs: Vec<String>) -> Self {
        self.raw_refs = refs;
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

// ============================================================================
// Playbook Bundle - Collection of evidence
// ============================================================================

/// Bundle of evidence from a playbook investigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookBundle {
    /// All evidence items collected
    pub items: Vec<PlaybookEvidence>,
    /// Coverage score (0-100): how many required topics were covered
    pub coverage_score: u8,
    /// Missing required topics
    pub missing_topics: Vec<String>,
    /// Department that collected this
    pub department: String,
}

impl PlaybookBundle {
    pub fn new(department: &str) -> Self {
        Self {
            items: Vec::new(),
            coverage_score: 0,
            missing_topics: Vec::new(),
            department: department.to_string(),
        }
    }

    pub fn add(&mut self, evidence: PlaybookEvidence) {
        self.items.push(evidence);
    }

    pub fn get(&self, topic_id: &str) -> Option<&PlaybookEvidence> {
        self.items
            .iter()
            .find(|e| e.topic_id == topic_id && e.success)
    }

    pub fn has_topic(&self, topic_id: &str) -> bool {
        self.items
            .iter()
            .any(|e| e.topic_id == topic_id && e.success)
    }

    /// Finalize bundle with coverage calculation
    pub fn finalize(&mut self, required_topics: &[PlaybookTopic]) {
        let required_ids: Vec<_> = required_topics
            .iter()
            .filter(|t| t.required)
            .map(|t| t.id.as_str())
            .collect();

        let covered = required_ids.iter().filter(|id| self.has_topic(id)).count();

        self.coverage_score = if required_ids.is_empty() {
            100
        } else {
            ((covered * 100) / required_ids.len()) as u8
        };

        self.missing_topics = required_ids
            .iter()
            .filter(|id| !self.has_topic(id))
            .map(|s| s.to_string())
            .collect();
    }

    /// Human-readable summary (no tool names, no IDs)
    pub fn human_summary(&self) -> String {
        if self.items.is_empty() {
            return "No evidence collected.".to_string();
        }

        self.items
            .iter()
            .filter(|e| e.success)
            .map(|e| e.summary_human.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Debug summary (with tool names)
    pub fn debug_summary(&self) -> String {
        self.items
            .iter()
            .map(|e| {
                let status = if e.success { "OK" } else { "FAIL" };
                let refs = if e.raw_refs.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", e.raw_refs.join(", "))
                };
                format!("[{}] {}: {}{}", status, e.topic_id, e.summary_debug, refs)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ============================================================================
// Cause Categories - Root cause classification
// ============================================================================

/// Network cause categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkCauseCategory {
    /// Physical/wireless link is down
    Link,
    /// DHCP or static IP configuration issue
    Dhcp,
    /// DNS resolution issue
    Dns,
    /// Network manager conflict (NM vs iwd vs systemd-networkd)
    ManagerConflict,
    /// Driver or firmware issue
    DriverFirmware,
    /// Unable to determine cause
    Unknown,
}

impl NetworkCauseCategory {
    pub fn human_label(&self) -> &'static str {
        match self {
            NetworkCauseCategory::Link => "link/connection issue",
            NetworkCauseCategory::Dhcp => "IP addressing issue",
            NetworkCauseCategory::Dns => "DNS resolution issue",
            NetworkCauseCategory::ManagerConflict => "network manager conflict",
            NetworkCauseCategory::DriverFirmware => "driver/firmware issue",
            NetworkCauseCategory::Unknown => "unknown cause",
        }
    }
}

/// Storage risk signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageRiskLevel {
    /// No issues detected
    None,
    /// Minor issues (low disk space warning)
    Low,
    /// Moderate issues (filesystem warnings)
    Medium,
    /// Critical issues (device errors, imminent failure)
    High,
}

impl StorageRiskLevel {
    pub fn human_label(&self) -> &'static str {
        match self {
            StorageRiskLevel::None => "no issues detected",
            StorageRiskLevel::Low => "minor concerns",
            StorageRiskLevel::Medium => "moderate risk signals",
            StorageRiskLevel::High => "critical risk - attention required",
        }
    }
}

// ============================================================================
// Department Diagnosis Results
// ============================================================================

/// Networking department diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingDiagnosis {
    /// Evidence collected
    pub evidence: PlaybookBundle,
    /// Findings (human-readable)
    pub findings: Vec<String>,
    /// Most likely cause category
    pub cause_category: NetworkCauseCategory,
    /// Confidence in diagnosis (0-100)
    pub confidence: u8,
}

impl NetworkingDiagnosis {
    pub fn new(evidence: PlaybookBundle) -> Self {
        Self {
            evidence,
            findings: Vec::new(),
            cause_category: NetworkCauseCategory::Unknown,
            confidence: 0,
        }
    }

    pub fn with_findings(mut self, findings: Vec<String>) -> Self {
        self.findings = findings;
        self
    }

    pub fn with_cause(mut self, cause: NetworkCauseCategory, confidence: u8) -> Self {
        self.cause_category = cause;
        self.confidence = confidence;
        self
    }
}

/// Storage department diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDiagnosis {
    /// Evidence collected
    pub evidence: PlaybookBundle,
    /// Findings (human-readable)
    pub findings: Vec<String>,
    /// Risk level
    pub risk_level: StorageRiskLevel,
    /// Specific risk signals
    pub risk_signals: Vec<String>,
    /// Confidence in diagnosis (0-100)
    pub confidence: u8,
}

impl StorageDiagnosis {
    pub fn new(evidence: PlaybookBundle) -> Self {
        Self {
            evidence,
            findings: Vec::new(),
            risk_level: StorageRiskLevel::None,
            risk_signals: Vec::new(),
            confidence: 0,
        }
    }

    pub fn with_findings(mut self, findings: Vec<String>) -> Self {
        self.findings = findings;
        self
    }

    pub fn with_risk(mut self, level: StorageRiskLevel, signals: Vec<String>) -> Self {
        self.risk_level = level;
        self.risk_signals = signals;
        self
    }

    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = confidence;
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playbook_topic() {
        let topic = PlaybookTopic::required(
            "net_link",
            "Link State",
            "Determines if physical/wireless connection exists",
            vec!["ip link show"],
        );
        assert_eq!(topic.id, "net_link");
        assert!(topic.required);
    }

    #[test]
    fn test_playbook_evidence() {
        let evidence = PlaybookEvidence::success(
            "net_link",
            "WiFi connected to home-network",
            "wlan0: UP, SSID=home-network",
        )
        .with_refs(vec!["ip link show".to_string()]);

        assert!(evidence.success);
        assert!(!evidence.summary_human.contains("ip link"));
        assert!(evidence.summary_debug.contains("wlan0"));
    }

    #[test]
    fn test_playbook_bundle_coverage() {
        let topics = vec![
            PlaybookTopic::required("t1", "Topic 1", "Test", vec![]),
            PlaybookTopic::required("t2", "Topic 2", "Test", vec![]),
            PlaybookTopic::optional("t3", "Topic 3", "Test", vec![]),
        ];

        let mut bundle = PlaybookBundle::new("test");
        bundle.add(PlaybookEvidence::success("t1", "Found", "Found"));

        bundle.finalize(&topics);
        assert_eq!(bundle.coverage_score, 50); // 1 of 2 required
        assert!(bundle.missing_topics.contains(&"t2".to_string()));
    }

    #[test]
    fn test_human_summary_no_tool_names() {
        let mut bundle = PlaybookBundle::new("networking");
        bundle.add(
            PlaybookEvidence::success(
                "net_link",
                "WiFi connected",
                "wlan0: UP via iw dev wlan0 link",
            )
            .with_refs(vec!["iw dev wlan0 link".to_string()]),
        );

        let summary = bundle.human_summary();
        assert!(summary.contains("WiFi connected"));
        assert!(!summary.contains("iw dev"));
        assert!(!summary.contains("wlan0:")); // Debug format
    }

    #[test]
    fn test_network_cause_category() {
        assert_eq!(
            NetworkCauseCategory::Dns.human_label(),
            "DNS resolution issue"
        );
    }

    #[test]
    fn test_storage_risk_level() {
        assert_eq!(
            StorageRiskLevel::High.human_label(),
            "critical risk - attention required"
        );
    }
}
