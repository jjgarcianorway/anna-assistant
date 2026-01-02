//! Tests for specialist contract types.

#[cfg(test)]
mod tests {
    use super::super::citation::{CitationKind, KnowledgeCitation};
    use super::super::discovery::Discovery;
    use super::super::response::SpecialistResponse;
    use super::super::types::{Answer, Evidence, Mood, ResponseStatus, Severity, StaffView};

    #[test]
    fn test_parse_specialist_response() {
        let json = r#"{
            "ticket_id": "DSK-0101",
            "status": "ok",
            "answer": {
                "short": "No, there is no active swap configured.",
                "detail": "Both free -h and /proc/swaps show 0B swap."
            },
            "evidence": [
                {
                    "probe": "swap_files",
                    "snippet": "Filename Type Size Used Priority",
                    "interpretation": "No entries listed."
                }
            ],
            "confidence": 0.95,
            "staff_view": {
                "assignee_role": "System Specialist",
                "severity": "info",
                "mood": "confident",
                "short_note": "No swap configured.",
                "complexity": 1
            }
        }"#;

        let response: SpecialistResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, ResponseStatus::Ok);
        assert!(response.answer.short.contains("swap"));
        assert_eq!(response.evidence.len(), 1);
    }

    #[test]
    fn test_parse_needs_more_data() {
        let json = r#"{
            "ticket_id": "DSK-0102",
            "status": "needs_more_data",
            "answer": {
                "short": "I cannot determine if zram is enabled."
            },
            "missing_probes": ["zram_devices"],
            "confidence": 0.3
        }"#;

        let response: SpecialistResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, ResponseStatus::NeedsMoreData);
        assert_eq!(response.missing_probes, vec!["zram_devices"]);
    }

    #[test]
    fn test_parse_with_discovery() {
        let json = r#"{
            "ticket_id": "DSK-0103",
            "status": "ok",
            "answer": {
                "short": "Test answer"
            },
            "evidence": [],
            "confidence": 0.8,
            "discovery": {
                "new_probes": [
                    {
                        "id": "zram_devices",
                        "intent": "Detect zram configuration",
                        "domain": "system",
                        "command": "lsblk | grep zram",
                        "reusable_for": ["is zram enabled", "compressed memory"]
                    }
                ],
                "new_recipes": []
            }
        }"#;

        let response: SpecialistResponse = serde_json::from_str(json).unwrap();
        assert!(response.discovery.is_some());
        let discovery = response.discovery.unwrap();
        assert_eq!(discovery.new_probes.len(), 1);
        assert_eq!(discovery.new_probes[0].id, "zram_devices");
    }

    #[test]
    fn test_validate_forbidden_patterns() {
        let response = SpecialistResponse {
            ticket_id: "DSK-0104".to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: "unknown is installed on your system".to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![],
            confidence: 0.9,
            staff_view: None,
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: true,
            evidence_references: vec![],
            knowledge_used: vec![],
            citations: vec![],
        };

        let errors = response.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("forbidden pattern")));
    }

    #[test]
    fn test_validate_high_confidence_no_evidence() {
        let response = SpecialistResponse {
            ticket_id: "DSK-0105".to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: "vim is installed".to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![], // No evidence!
            confidence: 0.95, // High confidence!
            staff_view: None,
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: true,
            evidence_references: vec![],
            knowledge_used: vec![],
            citations: vec![],
        };

        let errors = response.validate();
        assert!(errors.iter().any(|e| e.contains("no evidence")));
    }

    #[test]
    fn test_validate_valid_response() {
        let response = SpecialistResponse {
            ticket_id: "DSK-0106".to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: "vim is installed at /usr/bin/vim".to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![Evidence {
                probe: "command_v".to_string(),
                snippet: "/usr/bin/vim".to_string(),
                interpretation: "vim binary found".to_string(),
            }],
            confidence: 0.9,
            staff_view: None,
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: true,
            evidence_references: vec!["command_v".to_string()],
            knowledge_used: vec![],
            citations: vec![],
        };

        let errors = response.validate();
        assert!(errors.is_empty());
        assert!(response.is_valid());
    }
}
