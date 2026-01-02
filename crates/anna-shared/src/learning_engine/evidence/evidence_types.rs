//! Evidence types and entry structures.

use serde::{Deserialize, Serialize};

use super::utils::{extract_keywords, infer_domain, now_epoch, truncate};

/// Type of evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Output from a probe
    ProbeOutput,
    /// Man page excerpt
    ManPage,
    /// Help output from command
    HelpOutput,
    /// Arch Wiki excerpt
    ArchWiki,
    /// Prior successful ticket summary
    TicketSummary,
    /// Documentation from /usr/share/doc
    LocalDoc,
}

/// An evidence entry in the cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEntry {
    /// Unique entry ID
    pub id: String,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Associated ticket ID (if any)
    #[serde(default)]
    pub ticket_id: Option<String>,
    /// Domain/category
    pub domain: String,
    /// Intent (if known)
    #[serde(default)]
    pub intent: Option<String>,
    /// Raw content (probe output, help text, etc.)
    pub content: String,
    /// Citation (e.g., "man:systemctl", "help:pacman", "wiki:systemd")
    #[serde(default)]
    pub citation: Option<String>,
    /// Keywords extracted from this evidence
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Whether this contributed to a learned recipe
    #[serde(default)]
    pub used_for_learning: bool,
}

impl EvidenceEntry {
    /// Create a new probe output entry
    pub fn probe_output(probe_id: &str, output: &str, domain: &str) -> Self {
        Self {
            id: format!("probe:{}:{}", probe_id, now_epoch()),
            evidence_type: EvidenceType::ProbeOutput,
            ticket_id: None,
            domain: domain.to_string(),
            intent: None,
            content: truncate(output, 2000),
            citation: Some(format!("probe:{}", probe_id)),
            keywords: extract_keywords(output),
            timestamp: now_epoch(),
            used_for_learning: false,
        }
    }

    /// Create a man page entry
    pub fn man_page(command: &str, excerpt: &str) -> Self {
        Self {
            id: format!("man:{}:{}", command, now_epoch()),
            evidence_type: EvidenceType::ManPage,
            ticket_id: None,
            domain: infer_domain(command),
            intent: None,
            content: truncate(excerpt, 2000),
            citation: Some(format!("man:{}", command)),
            keywords: extract_keywords(excerpt),
            timestamp: now_epoch(),
            used_for_learning: false,
        }
    }

    /// Create a help output entry
    pub fn help_output(command: &str, output: &str) -> Self {
        Self {
            id: format!("help:{}:{}", command, now_epoch()),
            evidence_type: EvidenceType::HelpOutput,
            ticket_id: None,
            domain: infer_domain(command),
            intent: None,
            content: truncate(output, 2000),
            citation: Some(format!("help:{}", command)),
            keywords: extract_keywords(output),
            timestamp: now_epoch(),
            used_for_learning: false,
        }
    }

    /// Create a ticket summary entry
    pub fn ticket_summary(ticket_id: &str, domain: &str, intent: &str, summary: &str) -> Self {
        Self {
            id: format!("ticket:{}:{}", ticket_id, now_epoch()),
            evidence_type: EvidenceType::TicketSummary,
            ticket_id: Some(ticket_id.to_string()),
            domain: domain.to_string(),
            intent: Some(intent.to_string()),
            content: truncate(summary, 1000),
            citation: None,
            keywords: extract_keywords(summary),
            timestamp: now_epoch(),
            used_for_learning: false,
        }
    }

    /// Mark as used for learning
    pub fn mark_used(&mut self) {
        self.used_for_learning = true;
    }
}
