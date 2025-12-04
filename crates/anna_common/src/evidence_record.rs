//! Evidence Record v0.0.65 - Typed Evidence System
//!
//! Forces relevance in evidence collection:
//! - Each EvidenceRecord has a topic, human summary, and debug source
//! - EvidenceBundle collects related records for a query
//! - Strict validation ensures answers match evidence topics
//!
//! v0.0.65: Purposeful, typed evidence that matches questions

use crate::evidence_topic::EvidenceTopic;
use serde::{Deserialize, Serialize};

// ============================================================================
// Probe Kind - Passive vs Active
// ============================================================================

/// Whether a probe is passive (reads state) or active (generates traffic)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeKind {
    /// Reads system state without side effects (cat /proc/*, uname, etc.)
    Passive,
    /// Generates network traffic or has side effects (ping, curl)
    Active,
}

impl ProbeKind {
    pub fn human_label(&self) -> &'static str {
        match self {
            ProbeKind::Passive => "reading system state",
            ProbeKind::Active => "running connectivity check",
        }
    }
}

// ============================================================================
// Evidence Record
// ============================================================================

/// A single piece of evidence with topic, human summary, and debug trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    /// The topic this evidence addresses
    pub topic: EvidenceTopic,
    /// Human-readable summary (no tool names, no IDs)
    pub human_summary: String,
    /// Debug label: tool name(s) used
    pub debug_source: String,
    /// Evidence ID for citation (E1, E2, etc.)
    pub evidence_id: String,
    /// Raw data (optional, for debug mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_data: Option<serde_json::Value>,
    /// Unix timestamp when evidence was collected
    pub timestamp: u64,
    /// Whether collection succeeded
    pub success: bool,
    /// Probe kind (passive vs active)
    pub probe_kind: ProbeKind,
    /// Duration to collect (milliseconds)
    pub duration_ms: u64,
}

impl EvidenceRecord {
    /// Create a new evidence record
    pub fn new(
        topic: EvidenceTopic,
        human_summary: String,
        debug_source: &str,
        evidence_id: &str,
    ) -> Self {
        Self {
            topic,
            human_summary,
            debug_source: debug_source.to_string(),
            evidence_id: evidence_id.to_string(),
            raw_data: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            success: true,
            probe_kind: ProbeKind::Passive,
            duration_ms: 0,
        }
    }

    /// Set raw data for debug mode
    pub fn with_raw_data(mut self, data: serde_json::Value) -> Self {
        self.raw_data = Some(data);
        self
    }

    /// Set probe kind
    pub fn with_probe_kind(mut self, kind: ProbeKind) -> Self {
        self.probe_kind = kind;
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Mark as failed
    pub fn failed(mut self) -> Self {
        self.success = false;
        self
    }
}

// ============================================================================
// Evidence Bundle
// ============================================================================

/// A collection of evidence records for a single query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    /// All evidence records collected
    pub records: Vec<EvidenceRecord>,
    /// Primary topic being answered
    pub primary_topic: EvidenceTopic,
    /// Whether all required evidence was collected
    pub complete: bool,
    /// Missing topics if incomplete
    pub missing_topics: Vec<EvidenceTopic>,
}

impl EvidenceBundle {
    /// Create a new empty bundle
    pub fn new(primary_topic: EvidenceTopic) -> Self {
        Self {
            records: Vec::new(),
            primary_topic,
            complete: false,
            missing_topics: Vec::new(),
        }
    }

    /// Create an empty bundle (v0.0.66: for department protocol)
    pub fn empty() -> Self {
        Self {
            records: Vec::new(),
            primary_topic: EvidenceTopic::Unknown,
            complete: false,
            missing_topics: Vec::new(),
        }
    }

    /// Add an evidence record
    pub fn add(&mut self, record: EvidenceRecord) {
        self.records.push(record);
    }

    /// Check if bundle has a specific topic
    pub fn has_topic(&self, topic: EvidenceTopic) -> bool {
        self.records.iter().any(|r| r.topic == topic && r.success)
    }

    /// Get record for a specific topic
    pub fn get_topic(&self, topic: EvidenceTopic) -> Option<&EvidenceRecord> {
        self.records.iter().find(|r| r.topic == topic && r.success)
    }

    /// Get all successful records
    pub fn successful_records(&self) -> Vec<&EvidenceRecord> {
        self.records.iter().filter(|r| r.success).collect()
    }

    /// Mark bundle as complete with missing topics check
    pub fn finalize(&mut self, required_topics: &[EvidenceTopic]) {
        self.missing_topics = required_topics
            .iter()
            .filter(|t| !self.has_topic(**t))
            .copied()
            .collect();
        self.complete = self.missing_topics.is_empty();
    }

    /// Generate human-readable summary (no tool names, no IDs)
    pub fn human_summary(&self) -> String {
        if self.records.is_empty() {
            return "No evidence collected.".to_string();
        }

        let summaries: Vec<&str> = self
            .successful_records()
            .iter()
            .map(|r| r.human_summary.as_str())
            .collect();

        summaries.join("; ")
    }

    /// Generate debug summary (with tool names)
    pub fn debug_summary(&self) -> String {
        self.records
            .iter()
            .map(|r| {
                let status = if r.success { "OK" } else { "FAIL" };
                format!(
                    "[{}] {} ({}) - {}",
                    r.evidence_id, r.debug_source, status, r.human_summary
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ============================================================================
// Evidence Schema (strict output validation)
// ============================================================================

/// Schema definition for validating evidence output
#[derive(Debug, Clone)]
pub struct EvidenceSchema {
    /// Topic this schema validates
    pub topic: EvidenceTopic,
    /// Required fields in the output
    pub required_fields: Vec<&'static str>,
    /// Human description of what this evidence shows
    pub description: &'static str,
}

/// Get schema for a topic
pub fn get_evidence_schema(topic: EvidenceTopic) -> EvidenceSchema {
    match topic {
        EvidenceTopic::KernelVersion => EvidenceSchema {
            topic,
            required_fields: vec!["kernel_release"],
            description: "kernel version string",
        },
        EvidenceTopic::MemoryInfo => EvidenceSchema {
            topic,
            required_fields: vec!["total_gib", "available_gib"],
            description: "memory usage",
        },
        EvidenceTopic::DiskFree => EvidenceSchema {
            topic,
            required_fields: vec!["root"],
            description: "disk space on filesystems",
        },
        EvidenceTopic::NetworkStatus => EvidenceSchema {
            topic,
            required_fields: vec!["has_default_route"],
            description: "network connectivity",
        },
        EvidenceTopic::AudioStatus => EvidenceSchema {
            topic,
            required_fields: vec!["pipewire_running"],
            description: "audio stack status",
        },
        EvidenceTopic::CpuInfo => EvidenceSchema {
            topic,
            required_fields: vec!["cpu_model"],
            description: "CPU information",
        },
        EvidenceTopic::ServiceState => EvidenceSchema {
            topic,
            required_fields: vec!["active"],
            description: "service status",
        },
        _ => EvidenceSchema {
            topic,
            required_fields: vec![],
            description: topic.human_label(),
        },
    }
}

/// Validate evidence data against schema
pub fn validate_evidence_data(
    topic: EvidenceTopic,
    data: &serde_json::Value,
) -> (bool, Vec<String>) {
    let schema = get_evidence_schema(topic);
    let mut missing = Vec::new();

    for field in &schema.required_fields {
        if data.get(*field).is_none() {
            missing.push(field.to_string());
        }
    }

    (missing.is_empty(), missing)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_record_creation() {
        let record = EvidenceRecord::new(
            EvidenceTopic::DiskFree,
            "Root: 433 GiB free (87% free)".to_string(),
            "mount_usage",
            "E1",
        );
        assert_eq!(record.topic, EvidenceTopic::DiskFree);
        assert!(record.success);
        assert_eq!(record.probe_kind, ProbeKind::Passive);
    }

    #[test]
    fn test_evidence_bundle() {
        let mut bundle = EvidenceBundle::new(EvidenceTopic::DiskFree);

        bundle.add(EvidenceRecord::new(
            EvidenceTopic::DiskFree,
            "Root: 433 GiB free".to_string(),
            "mount_usage",
            "E1",
        ));

        assert!(bundle.has_topic(EvidenceTopic::DiskFree));
        assert!(!bundle.has_topic(EvidenceTopic::MemoryInfo));

        bundle.finalize(&[EvidenceTopic::DiskFree]);
        assert!(bundle.complete);
        assert!(bundle.missing_topics.is_empty());
    }

    #[test]
    fn test_evidence_validation() {
        let good_data = serde_json::json!({
            "kernel_release": "6.6.1-arch1-1"
        });
        let (valid, missing) = validate_evidence_data(EvidenceTopic::KernelVersion, &good_data);
        assert!(valid);
        assert!(missing.is_empty());

        let bad_data = serde_json::json!({});
        let (valid, missing) = validate_evidence_data(EvidenceTopic::KernelVersion, &bad_data);
        assert!(!valid);
        assert!(missing.contains(&"kernel_release".to_string()));
    }

    #[test]
    fn test_human_summary() {
        let mut bundle = EvidenceBundle::new(EvidenceTopic::NetworkStatus);

        bundle.add(EvidenceRecord::new(
            EvidenceTopic::NetworkStatus,
            "Network connected via WiFi".to_string(),
            "network_status",
            "E1",
        ));

        let summary = bundle.human_summary();
        assert!(summary.contains("Network connected"));
        assert!(!summary.contains("network_status")); // No tool name
        assert!(!summary.contains("E1")); // No evidence ID
    }
}
