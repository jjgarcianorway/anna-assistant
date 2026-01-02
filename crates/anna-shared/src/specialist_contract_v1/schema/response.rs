//! Main SpecialistResponseV1 struct for SRC v1.

use serde::{Deserialize, Serialize};

use super::action::SrcAction;
use super::assessment::SrcAssessment;
use super::citation::SrcCitation;
use super::constants::{MAX_ACTIONS, MAX_CITATIONS};
use super::types::{SrcDepartment, SrcRisk};

/// The Specialist Response Contract v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistResponseV1 {
    /// Case ID (must match the ticket).
    pub case_id: String,
    /// Department that handled this.
    pub department: SrcDepartment,
    /// Assessment of the situation.
    pub assessment: SrcAssessment,
    /// Proposed actions (max 5).
    #[serde(default)]
    pub actions: Vec<SrcAction>,
    /// Citations (max 5).
    #[serde(default)]
    pub citations: Vec<SrcCitation>,
}

impl SpecialistResponseV1 {
    /// Create a new response.
    pub fn new(
        case_id: &str,
        department: SrcDepartment,
        summary: &str,
        confidence: f64,
        risk: SrcRisk,
    ) -> Self {
        Self {
            case_id: case_id.to_string(),
            department,
            assessment: SrcAssessment::new(summary, confidence, risk),
            actions: Vec::new(),
            citations: Vec::new(),
        }
    }

    /// Add an action.
    pub fn with_action(mut self, action: SrcAction) -> Self {
        if self.actions.len() < MAX_ACTIONS {
            self.actions.push(action);
        }
        self
    }

    /// Add a citation.
    pub fn with_citation(mut self, citation: SrcCitation) -> Self {
        if self.citations.len() < MAX_CITATIONS {
            self.citations.push(citation);
        }
        self
    }

    /// Validate the entire response.
    pub fn validate(&self, expected_case_id: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Case ID must match
        if self.case_id != expected_case_id {
            errors.push(format!(
                "case_id mismatch: expected '{}', got '{}'",
                expected_case_id, self.case_id
            ));
        }

        // Validate assessment
        if let Err(assessment_errors) = self.assessment.validate() {
            errors.extend(assessment_errors);
        }

        // Validate actions count
        if self.actions.len() > MAX_ACTIONS {
            errors.push(format!(
                "too many actions: {} > {}",
                self.actions.len(),
                MAX_ACTIONS
            ));
        }

        // Validate each action
        for (i, action) in self.actions.iter().enumerate() {
            if let Err(action_errors) = action.validate() {
                for e in action_errors {
                    errors.push(format!("action[{}]: {}", i, e));
                }
            }
        }

        // Validate citations count
        if self.citations.len() > MAX_CITATIONS {
            errors.push(format!(
                "too many citations: {} > {}",
                self.citations.len(),
                MAX_CITATIONS
            ));
        }

        // Validate each citation
        for (i, citation) in self.citations.iter().enumerate() {
            if let Err(citation_errors) = citation.validate() {
                for e in citation_errors {
                    errors.push(format!("citation[{}]: {}", i, e));
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

#[cfg(test)]
mod tests {
    use super::super::constants::MAX_ACTIONS;
    use super::super::types::{SrcCitationSource, SrcDepartment, SrcRisk};
    use super::*;

    #[test]
    fn test_src_response_validation() {
        let response = SpecialistResponseV1::new(
            "DSK-0101",
            SrcDepartment::Performance,
            "Boot time is 7.5 seconds.",
            0.9,
            SrcRisk::ReadOnly,
        );

        assert!(response.validate("DSK-0101").is_ok());
        assert!(response.validate("DSK-0102").is_err()); // Wrong case_id
    }

    #[test]
    fn test_src_response_max_actions() {
        let mut response = SpecialistResponseV1::new(
            "DSK-0101",
            SrcDepartment::Performance,
            "Summary",
            0.9,
            SrcRisk::ReadOnly,
        );

        // Add 10 actions - should only keep 5
        for i in 0..10 {
            response = response.with_action(SrcAction::explain(
                &format!("Action {}", i),
                "why",
                "expected",
            ));
        }

        assert_eq!(response.actions.len(), MAX_ACTIONS);
    }

    #[test]
    fn test_json_serialization() {
        let response = SpecialistResponseV1::new(
            "DSK-0101",
            SrcDepartment::Performance,
            "Boot time is 7.5 seconds.",
            0.9,
            SrcRisk::ReadOnly,
        )
        .with_citation(SrcCitation::new(
            SrcCitationSource::Man,
            "systemd-analyze(1)",
            "Analyze and debug system manager",
        ));

        let json = response.to_json().unwrap();
        assert!(json.contains("DSK-0101"));
        assert!(json.contains("Performance"));
    }
}
