//! Builder methods and constructors for StrictResponse.

use super::types::*;

impl StrictResponse {
    /// Create a successful response
    pub fn success(
        domain: &str,
        intent: &str,
        summary: &str,
        key_facts: Vec<String>,
        probes: Vec<ProbeEvidence>,
        meta: ResponseMeta,
    ) -> Self {
        Self {
            status: ResponseStatus::Success,
            confidence: 0.9,
            domain: domain.to_string(),
            intent: intent.to_string(),
            summary: summary.to_string(),
            details: ResponseDetails {
                key_facts,
                diagnosis: None,
                recommendations: vec![],
            },
            actions: ResponseActions::default(),
            evidence: ResponseEvidence {
                probes_used: probes,
                arch_wiki_pages: vec![],
                man_pages: vec![],
                help_commands: vec![],
            },
            metrics: ResponseMetrics::default(),
            meta,
        }
    }

    /// Create a partial response
    pub fn partial(
        domain: &str,
        intent: &str,
        summary: &str,
        known_facts: Vec<String>,
        unknown_reason: &str,
        probes: Vec<ProbeEvidence>,
        meta: ResponseMeta,
    ) -> Self {
        Self {
            status: ResponseStatus::Partial,
            confidence: 0.5,
            domain: domain.to_string(),
            intent: intent.to_string(),
            summary: summary.to_string(),
            details: ResponseDetails {
                key_facts: known_facts,
                diagnosis: Some(unknown_reason.to_string()),
                recommendations: vec![],
            },
            actions: ResponseActions::default(),
            evidence: ResponseEvidence {
                probes_used: probes,
                arch_wiki_pages: vec![],
                man_pages: vec![],
                help_commands: vec![],
            },
            metrics: ResponseMetrics::default(),
            meta,
        }
    }

    /// Create a failure response
    pub fn failure(domain: &str, intent: &str, reason: &str, meta: ResponseMeta) -> Self {
        Self {
            status: ResponseStatus::Failure,
            confidence: 0.0,
            domain: domain.to_string(),
            intent: intent.to_string(),
            summary: reason.to_string(),
            details: ResponseDetails::default(),
            actions: ResponseActions::default(),
            evidence: ResponseEvidence::default(),
            metrics: ResponseMetrics::default(),
            meta,
        }
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set latency
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.metrics.latency_ms = latency_ms;
        self
    }

    /// Add recommendations
    pub fn with_recommendations(mut self, recs: Vec<String>) -> Self {
        self.details.recommendations = recs;
        self
    }

    /// Add proposed action
    pub fn with_action(mut self, action: ProposedAction) -> Self {
        self.actions.proposed.push(action);
        self
    }

    /// Check if response is learnable (high confidence success)
    pub fn is_learnable(&self) -> bool {
        self.status == ResponseStatus::Success
            && self.confidence >= super::super::MIN_LEARN_CONFIDENCE
            && !self.evidence.probes_used.is_empty()
    }

    /// Check if actions can be suggested
    pub fn can_suggest_actions(&self) -> bool {
        self.confidence >= super::super::MIN_ACTION_CONFIDENCE
            && matches!(
                self.status,
                ResponseStatus::Success | ResponseStatus::Partial
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_meta(ticket_id: &str) -> ResponseMeta {
        ResponseMeta {
            handled_by: "Test".to_string(),
            ticket_id: ticket_id.to_string(),
            version: 1,
        }
    }

    #[test]
    fn test_success_response() {
        let response = StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "No failed systemd services.",
            vec!["0 failed units".to_string()],
            vec![ProbeEvidence {
                id: "systemctl_failed".to_string(),
                summary: "No failed units found".to_string(),
                raw_reference: None,
            }],
            make_meta("DSK-001"),
        );

        assert_eq!(response.status, ResponseStatus::Success);
        assert!(response.is_learnable());
    }

    #[test]
    fn test_partial_response() {
        let response = StrictResponse::partial(
            "storage.disk",
            "check_disk_usage",
            "Root filesystem is at 97% used.",
            vec!["30 GiB free out of 803 GiB".to_string()],
            "Could not identify which directories are using space (analysis timed out).",
            vec![],
            make_meta("DSK-002"),
        );

        assert_eq!(response.status, ResponseStatus::Partial);
        assert!(!response.is_learnable());
    }

    #[test]
    fn test_failure_response() {
        let response = StrictResponse::failure(
            "system",
            "unknown",
            "Could not determine the answer.",
            make_meta("DSK-003"),
        );

        assert_eq!(response.status, ResponseStatus::Failure);
        assert_eq!(response.confidence, 0.0);
    }

    #[test]
    fn test_serialize_response() {
        let response = StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "No failed services.",
            vec!["All services healthy".to_string()],
            vec![],
            make_meta("DSK-004"),
        );

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"success\""));
        assert!(json.contains("\"domain\":\"services.systemd\""));
    }
}
