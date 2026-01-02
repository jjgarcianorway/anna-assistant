//! Tests for specialist response schema.

#[cfg(test)]
mod tests {
    use crate::specialist_v3::schema::*;

    #[test]
    fn test_success_response() {
        let resp = SpecialistResponse::success("DSK-001", "Memory is healthy")
            .with_specialist("Sofia", "System Admin", "desktop")
            .with_confidence(0.95)
            .with_finding(Finding::new("mem_available_mb", "17024").with_evidence("probe:free"));

        assert_eq!(resp.status, ResponseStatus::Success);
        assert!(resp.is_usable());
        assert!(resp.validate().is_ok());
    }

    #[test]
    fn test_error_response() {
        let resp = SpecialistResponse::error("DSK-002", ErrorKind::Timeout, "Request timed out");

        assert_eq!(resp.status, ResponseStatus::Error);
        assert!(!resp.is_usable());
        assert!(resp.validate().is_ok());
    }

    #[test]
    fn test_validation_failures() {
        let resp = SpecialistResponse {
            ticket_id: String::new(), // Missing
            status: ResponseStatus::Success,
            summary: String::new(), // Missing
            confidence: 1.5,        // Out of range
            ..Default::default()
        };

        let errors = resp.validate().unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_status_semantics() {
        assert!(ResponseStatus::Success.is_success());
        assert!(ResponseStatus::Partial.is_success());
        assert!(!ResponseStatus::NoData.is_success());
        assert!(ResponseStatus::Unsupported.should_reroute());
        assert!(ResponseStatus::Error.is_error());
    }

    #[test]
    fn test_json_roundtrip() {
        let resp = SpecialistResponse::success("DSK-003", "All clear")
            .with_finding(Finding::new("uptime", "3 days"));

        let json = serde_json::to_string(&resp).unwrap();
        let parsed: SpecialistResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.ticket_id, "DSK-003");
        assert_eq!(parsed.findings.len(), 1);
    }
}
