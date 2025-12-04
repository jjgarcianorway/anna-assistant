//! Department Protocol v0.0.66 - Structured Investigation Pattern
//!
//! Defines the contract that all departments (doctors) must implement
//! for structured investigations. This enables:
//! - Consistent interface across all specialist departments
//! - Type-safe routing from Service Desk
//! - Multi-department escalation with clear handoff
//!
//! Each department runs a structured investigation:
//! 1. Can it handle this ticket? (can_handle)
//! 2. What evidence topics does it need? (required_topics)
//! 3. Run investigation with collected evidence (investigate)
//! 4. Return findings, recommendations, and reliability hint

use serde::{Deserialize, Serialize};

use crate::case_lifecycle::{ActionRisk, CaseFileV2, Department, TicketType};
use crate::evidence_record::EvidenceBundle;
use crate::evidence_topic::EvidenceTopic;
use crate::service_desk::Ticket;

// ============================================================================
// Department Names (for routing)
// ============================================================================

/// Department name enum for type-safe routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepartmentName {
    ServiceDesk,
    Networking,
    Storage,
    Audio,
    Boot,
    Graphics,
    Security,
    Performance,
    InfoDesk,
}

impl DepartmentName {
    pub fn as_str(&self) -> &'static str {
        match self {
            DepartmentName::ServiceDesk => "service_desk",
            DepartmentName::Networking => "networking",
            DepartmentName::Storage => "storage",
            DepartmentName::Audio => "audio",
            DepartmentName::Boot => "boot",
            DepartmentName::Graphics => "graphics",
            DepartmentName::Security => "security",
            DepartmentName::Performance => "performance",
            DepartmentName::InfoDesk => "info_desk",
        }
    }

    pub fn human_label(&self) -> &'static str {
        match self {
            DepartmentName::ServiceDesk => "Service Desk",
            DepartmentName::Networking => "Networking",
            DepartmentName::Storage => "Storage",
            DepartmentName::Audio => "Audio",
            DepartmentName::Boot => "Boot",
            DepartmentName::Graphics => "Graphics",
            DepartmentName::Security => "Security",
            DepartmentName::Performance => "Performance",
            DepartmentName::InfoDesk => "Info Desk",
        }
    }

    /// Convert from case_lifecycle::Department
    pub fn from_department(dept: Department) -> Self {
        match dept {
            Department::ServiceDesk => DepartmentName::ServiceDesk,
            Department::Networking => DepartmentName::Networking,
            Department::Storage => DepartmentName::Storage,
            Department::Boot => DepartmentName::Boot,
            Department::Audio => DepartmentName::Audio,
            Department::Graphics => DepartmentName::Graphics,
            Department::Security => DepartmentName::Security,
            Department::Performance => DepartmentName::Performance,
        }
    }

    /// Convert to case_lifecycle::Department
    pub fn to_department(&self) -> Department {
        match self {
            DepartmentName::ServiceDesk | DepartmentName::InfoDesk => Department::ServiceDesk,
            DepartmentName::Networking => Department::Networking,
            DepartmentName::Storage => Department::Storage,
            DepartmentName::Boot => Department::Boot,
            DepartmentName::Audio => Department::Audio,
            DepartmentName::Graphics => Department::Graphics,
            DepartmentName::Security => Department::Security,
            DepartmentName::Performance => Department::Performance,
        }
    }
}

impl std::fmt::Display for DepartmentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.human_label())
    }
}

// ============================================================================
// Routing Decision
// ============================================================================

/// Routing decision from Service Desk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Target department
    pub department: DepartmentName,
    /// Confidence in routing (0-100)
    pub confidence: u8,
    /// Human-readable reason for routing
    pub reason: String,
    /// Keywords that triggered this routing
    pub matched_keywords: Vec<String>,
}

impl RoutingDecision {
    pub fn new(department: DepartmentName, confidence: u8, reason: &str) -> Self {
        Self {
            department,
            confidence,
            reason: reason.to_string(),
            matched_keywords: Vec::new(),
        }
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.matched_keywords = keywords;
        self
    }
}

// ============================================================================
// Work Order
// ============================================================================

/// Work order from Service Desk to Department
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrder {
    /// Target department
    pub department: DepartmentName,
    /// Goals for this investigation
    pub goals: Vec<String>,
    /// Required evidence topics to collect
    pub required_topics: Vec<EvidenceTopic>,
    /// Optional secondary topics
    pub optional_topics: Vec<EvidenceTopic>,
    /// Maximum departments to consult (for escalation)
    pub max_escalations: u8,
    /// Current escalation count
    pub escalation_count: u8,
}

impl WorkOrder {
    pub fn new(department: DepartmentName, goals: Vec<String>, topics: Vec<EvidenceTopic>) -> Self {
        Self {
            department,
            goals,
            required_topics: topics,
            optional_topics: Vec::new(),
            max_escalations: 1, // v0.0.66: max 1 additional department
            escalation_count: 0,
        }
    }

    /// Check if escalation is allowed
    pub fn can_escalate(&self) -> bool {
        self.escalation_count < self.max_escalations
    }

    /// Record an escalation
    pub fn record_escalation(&mut self) {
        self.escalation_count += 1;
    }
}

// ============================================================================
// Investigation Context
// ============================================================================

/// Context provided to departments for investigation
#[derive(Debug, Clone)]
pub struct InvestigateCtx {
    /// The ticket being investigated
    pub ticket: Ticket,
    /// Work order from Service Desk
    pub work_order: WorkOrder,
    /// Evidence collected so far
    pub evidence: EvidenceBundle,
    /// Case file for recording findings
    pub case: CaseFileV2,
}

impl InvestigateCtx {
    pub fn new(
        ticket: Ticket,
        work_order: WorkOrder,
        evidence: EvidenceBundle,
        case: CaseFileV2,
    ) -> Self {
        Self {
            ticket,
            work_order,
            evidence,
            case,
        }
    }
}

// ============================================================================
// Department Finding
// ============================================================================

/// Severity of a finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// A finding from a department investigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentFinding {
    /// Human-readable description
    pub description: String,
    /// Severity level
    pub severity: FindingSeverity,
    /// Evidence IDs supporting this finding
    pub evidence_ids: Vec<String>,
    /// Confidence in this finding (0-100)
    pub confidence: u8,
}

impl DepartmentFinding {
    pub fn new(description: &str, severity: FindingSeverity, confidence: u8) -> Self {
        Self {
            description: description.to_string(),
            severity,
            evidence_ids: Vec::new(),
            confidence,
        }
    }

    pub fn with_evidence(mut self, evidence_ids: Vec<String>) -> Self {
        self.evidence_ids = evidence_ids;
        self
    }
}

// ============================================================================
// Recommended Action
// ============================================================================

/// A recommended action from a department
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    /// Human-readable description
    pub description: String,
    /// Risk level
    pub risk: ActionRisk,
    /// Tool to execute (if any)
    pub tool_name: Option<String>,
    /// Parameters for the tool
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
    /// Whether this requires confirmation
    pub needs_confirmation: bool,
}

impl RecommendedAction {
    pub fn read_only(description: &str) -> Self {
        Self {
            description: description.to_string(),
            risk: ActionRisk::ReadOnly,
            tool_name: None,
            parameters: std::collections::HashMap::new(),
            needs_confirmation: false,
        }
    }

    pub fn with_confirmation(description: &str, risk: ActionRisk) -> Self {
        Self {
            description: description.to_string(),
            risk,
            tool_name: None,
            parameters: std::collections::HashMap::new(),
            needs_confirmation: true,
        }
    }
}

// ============================================================================
// Department Result
// ============================================================================

/// Result from a department investigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentResult {
    /// Department that performed the investigation
    pub department: DepartmentName,
    /// Findings from the investigation (human-readable)
    pub findings: Vec<DepartmentFinding>,
    /// Evidence bundle used
    pub evidence_bundle: EvidenceBundle,
    /// Recommended actions (optional)
    pub recommended_actions: Vec<RecommendedAction>,
    /// Whether confirmation is needed for any action
    pub needs_confirmation: bool,
    /// Overall risk level
    pub risk: ActionRisk,
    /// Reliability hint (0-100)
    pub reliability_hint: u8,
    /// Whether this department needs to hand off to another
    pub needs_escalation: bool,
    /// Suggested escalation department (if needs_escalation)
    pub escalation_target: Option<DepartmentName>,
    /// Reason for escalation
    pub escalation_reason: Option<String>,
    /// Human-readable summary for the user
    pub summary: String,
}

impl DepartmentResult {
    /// Create a successful result with findings
    pub fn success(
        department: DepartmentName,
        findings: Vec<DepartmentFinding>,
        summary: &str,
    ) -> Self {
        let reliability = if findings.is_empty() {
            50
        } else {
            findings.iter().map(|f| f.confidence as u32).sum::<u32>() / findings.len() as u32
        };

        Self {
            department,
            findings,
            evidence_bundle: EvidenceBundle::empty(),
            recommended_actions: Vec::new(),
            needs_confirmation: false,
            risk: ActionRisk::ReadOnly,
            reliability_hint: reliability as u8,
            needs_escalation: false,
            escalation_target: None,
            escalation_reason: None,
            summary: summary.to_string(),
        }
    }

    /// Create a result that needs escalation
    pub fn needs_escalation(
        department: DepartmentName,
        target: DepartmentName,
        reason: &str,
    ) -> Self {
        Self {
            department,
            findings: Vec::new(),
            evidence_bundle: EvidenceBundle::empty(),
            recommended_actions: Vec::new(),
            needs_confirmation: false,
            risk: ActionRisk::ReadOnly,
            reliability_hint: 30,
            needs_escalation: true,
            escalation_target: Some(target),
            escalation_reason: Some(reason.to_string()),
            summary: format!("Escalating to {}: {}", target, reason),
        }
    }

    /// Create an insufficient evidence result
    pub fn insufficient_evidence(department: DepartmentName, missing: &[EvidenceTopic]) -> Self {
        let missing_str: Vec<_> = missing.iter().map(|t| t.human_label()).collect();
        Self {
            department,
            findings: Vec::new(),
            evidence_bundle: EvidenceBundle::empty(),
            recommended_actions: Vec::new(),
            needs_confirmation: false,
            risk: ActionRisk::ReadOnly,
            reliability_hint: 20,
            needs_escalation: false,
            escalation_target: None,
            escalation_reason: None,
            summary: format!("Insufficient evidence. Missing: {}", missing_str.join(", ")),
        }
    }

    pub fn with_evidence(mut self, bundle: EvidenceBundle) -> Self {
        self.evidence_bundle = bundle;
        self
    }

    pub fn with_actions(mut self, actions: Vec<RecommendedAction>) -> Self {
        self.needs_confirmation = actions.iter().any(|a| a.needs_confirmation);
        self.recommended_actions = actions;
        self
    }

    pub fn with_reliability(mut self, reliability: u8) -> Self {
        self.reliability_hint = reliability;
        self
    }
}

// ============================================================================
// Department Trait
// ============================================================================

/// The Department trait - contract for all specialist departments
pub trait DepartmentTrait {
    /// Get the department name
    fn name(&self) -> DepartmentName;

    /// Check if this department can handle the given ticket
    fn can_handle(&self, ticket: &Ticket) -> bool;

    /// Get the evidence topics this department needs
    fn required_topics(&self, ticket: &Ticket) -> Vec<EvidenceTopic>;

    /// Run the investigation
    fn investigate(&self, ctx: &InvestigateCtx) -> DepartmentResult;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_department_name_conversion() {
        let dept = Department::Networking;
        let name = DepartmentName::from_department(dept);
        assert_eq!(name, DepartmentName::Networking);
        assert_eq!(name.to_department(), dept);
    }

    #[test]
    fn test_routing_decision() {
        let decision = RoutingDecision::new(DepartmentName::Networking, 85, "Matched wifi keyword")
            .with_keywords(vec!["wifi".to_string()]);

        assert_eq!(decision.department, DepartmentName::Networking);
        assert_eq!(decision.confidence, 85);
        assert!(decision.matched_keywords.contains(&"wifi".to_string()));
    }

    #[test]
    fn test_work_order_escalation() {
        let mut order = WorkOrder::new(
            DepartmentName::Audio,
            vec!["Check audio stack".to_string()],
            vec![EvidenceTopic::AudioStatus],
        );

        assert!(order.can_escalate());
        order.record_escalation();
        assert!(!order.can_escalate()); // Max 1 escalation
    }

    #[test]
    fn test_department_result() {
        let finding =
            DepartmentFinding::new("Network interface is down", FindingSeverity::Error, 90);

        let result = DepartmentResult::success(
            DepartmentName::Networking,
            vec![finding],
            "Network interface wlan0 is not connected",
        );

        assert_eq!(result.department, DepartmentName::Networking);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.reliability_hint, 90);
        assert!(!result.needs_escalation);
    }

    #[test]
    fn test_escalation_result() {
        let result = DepartmentResult::needs_escalation(
            DepartmentName::Audio,
            DepartmentName::Boot,
            "Audio issue may be related to kernel update",
        );

        assert!(result.needs_escalation);
        assert_eq!(result.escalation_target, Some(DepartmentName::Boot));
    }
}
