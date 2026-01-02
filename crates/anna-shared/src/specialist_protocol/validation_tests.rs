//! Tests for validation functionality (v0.0.428).

#[cfg(test)]
mod tests {
    use crate::specialist_protocol::{
        ProbeEvidence, ResponseMeta, ResponseStatus, StrictResponse,
        validation_types::ValidationError,
        validation_core::{validate_response, is_useful_response},
    };

    fn make_meta() -> ResponseMeta {
        ResponseMeta {
            handled_by: "Test".to_string(),
            ticket_id: "TEST-001".to_string(),
            version: 1,
        }
    }

    #[test]
    fn test_valid_response() {
        let response = StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "No failed systemd services detected.",
            vec!["0 failed units".to_string()],
            vec![ProbeEvidence {
                id: "systemctl_failed".to_string(),
                summary: "0 failed units".to_string(),
                raw_reference: None,
            }],
            make_meta(),
        );

        let result = validate_response(&response);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_forbidden_pattern() {
        let response = StrictResponse::success(
            "packages",
            "check_installed",
            "unknown is installed on your system",
            vec![],
            vec![],
            make_meta(),
        );

        let result = validate_response(&response);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::ForbiddenPattern(_))));
        assert_eq!(result.adjusted_status, ResponseStatus::Failure);
    }

    #[test]
    fn test_generic_howto_blocked() {
        let response = StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "Step 1: Run systemctl status. Step 2: Check the logs.",
            vec![],
            vec![],
            make_meta(),
        );

        let result = validate_response(&response);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::GenericHowTo)));
        // Should be downgraded
        assert!(
            result.adjusted_status != ResponseStatus::Success || result.adjusted_confidence < 0.8
        );
    }

    #[test]
    fn test_vague_language_blocked() {
        let response = StrictResponse::success(
            "system",
            "check_memory",
            "Your system might be running low on memory.",
            vec![],
            vec![ProbeEvidence {
                id: "free".to_string(),
                summary: "2GB available".to_string(),
                raw_reference: None,
            }],
            make_meta(),
        );

        let result = validate_response(&response);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::VagueLanguage(_))));
    }

    #[test]
    fn test_missing_evidence_for_high_confidence() {
        let response = StrictResponse::success(
            "packages",
            "check_installed",
            "vim is installed",
            vec!["vim version 9.0".to_string()],
            vec![], // No evidence!
            make_meta(),
        )
        .with_confidence(0.95);

        let result = validate_response(&response);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::MissingEvidence)));
    }

    #[test]
    fn test_useful_response() {
        let good = StrictResponse::success(
            "system",
            "check_ram",
            "You have 16GB RAM available.",
            vec!["16GB available".to_string()],
            vec![ProbeEvidence {
                id: "free".to_string(),
                summary: "16GB available".to_string(),
                raw_reference: None,
            }],
            make_meta(),
        );
        assert!(is_useful_response(&good));

        let bad = StrictResponse::success(
            "system",
            "check_ram",
            "unknown is installed",
            vec![],
            vec![],
            make_meta(),
        );
        assert!(!is_useful_response(&bad));
    }
}
