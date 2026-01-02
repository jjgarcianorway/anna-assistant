//! SpecialistResponse implementations and builder methods.

use super::types::*;

impl Default for SpecialistResponse {
    fn default() -> Self {
        Self {
            ticket_id: String::new(),
            specialist: SpecialistIdentity::default(),
            status: ResponseStatus::default(),
            summary: String::new(),
            confidence: 0.5,
            severity: Severity::default(),
            findings: vec![],
            analysis: vec![],
            recommendations: vec![],
            actions: vec![],
            knowledge_citations: vec![],
            probes_used: vec![],
            error: ErrorInfo::default(),
        }
    }
}

impl SpecialistResponse {
    /// Create a success response
    pub fn success(ticket_id: &str, summary: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Success,
            summary: summary.to_string(),
            confidence: 0.8,
            ..Default::default()
        }
    }

    /// Create a partial response
    pub fn partial(ticket_id: &str, summary: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Partial,
            summary: summary.to_string(),
            confidence: 0.5,
            ..Default::default()
        }
    }

    /// Create a no-data response
    pub fn no_data(ticket_id: &str, reason: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::NoData,
            summary: reason.to_string(),
            confidence: 0.1,
            ..Default::default()
        }
    }

    /// Create an error response
    pub fn error(ticket_id: &str, kind: ErrorKind, message: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Error,
            summary: "An error occurred".to_string(),
            confidence: 0.0,
            error: ErrorInfo {
                message: Some(message.to_string()),
                kind: Some(kind),
                details: None,
            },
            ..Default::default()
        }
    }

    /// Builder: set specialist identity
    pub fn with_specialist(mut self, name: &str, role: &str, department: &str) -> Self {
        self.specialist = SpecialistIdentity {
            name: name.to_string(),
            role: role.to_string(),
            department: department.to_string(),
            seniority: Seniority::Junior,
        };
        self
    }

    /// Builder: set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Builder: set severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Builder: add finding
    pub fn with_finding(mut self, finding: Finding) -> Self {
        self.findings.push(finding);
        self
    }

    /// Builder: add analysis bullet
    pub fn with_analysis(mut self, bullet: &str) -> Self {
        self.analysis.push(bullet.to_string());
        self
    }

    /// Builder: add recommendation
    pub fn with_recommendation(mut self, rec: Recommendation) -> Self {
        self.recommendations.push(rec);
        self
    }

    /// Builder: add action
    pub fn with_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    /// Builder: add probe used
    pub fn with_probe(mut self, probe: ProbeUsed) -> Self {
        self.probes_used.push(probe);
        self
    }

    /// Builder: add knowledge citation
    pub fn with_citation(mut self, citation: KnowledgeCitation) -> Self {
        self.knowledge_citations.push(citation);
        self
    }

    /// Check if this response is usable
    pub fn is_usable(&self) -> bool {
        self.status.is_success() && !self.summary.is_empty()
    }

    /// Get all citation IDs
    pub fn citation_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self
            .knowledge_citations
            .iter()
            .map(|c| c.id.as_str())
            .collect();
        ids.extend(self.probes_used.iter().map(|p| p.id.as_str()));
        ids
    }

    /// Validate the response structure
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = vec![];

        if self.ticket_id.is_empty() {
            errors.push("ticket_id is required".to_string());
        }

        if self.summary.is_empty() && !self.status.is_error() {
            errors.push("summary is required for non-error responses".to_string());
        }

        if self.confidence < 0.0 || self.confidence > 1.0 {
            errors.push("confidence must be between 0.0 and 1.0".to_string());
        }

        if self.status == ResponseStatus::Error && self.error.message.is_none() {
            errors.push("error.message is required when status is error".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
