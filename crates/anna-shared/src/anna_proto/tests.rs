//! Acceptance Tests (Part H) - v0.0.436.
//!
//! Integration tests verifying the complete protocol pipeline.

#[cfg(test)]
mod acceptance_tests {
    use crate::anna_proto::*;
    use crate::anna_proto::decoder::{DecodeResult, DecodeError, ProtoDecoder};
    use crate::anna_proto::envelope::{
        ModelResultEnvelope, ModelRole, Claim, Action, EvidenceRef, ModelError, ErrorCode,
        EvidenceKind,
    };
    use crate::anna_proto::fallback::{EvidenceFallback, GatheredEvidence};
    use crate::anna_proto::framing::{PROTO_START, PROTO_END, create_frame};
    use crate::anna_proto::stats::{PeriodStats, TicketOutcome, outcome_from_decode};
    use crate::anna_proto::streaming::{StreamBuffer, StreamState};

    // ========================================
    // Acceptance Test 1: Valid framed response
    // ========================================
    #[test]
    fn test_valid_framed_response_decodes_successfully() {
        let envelope = ModelResultEnvelope::success(ModelRole::Junior, "DSK-001", "Boot is 15s", 0.9)
            .with_claim(Claim::with_support("Boot is slow", vec!["ev_boot".to_string()]))
            .with_evidence(EvidenceRef::probe("ev_boot", "Boot analysis"));

        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let framed = create_frame(&json);

        let decoder = ProtoDecoder::new();
        let result = decoder.decode(&framed);

        assert!(result.is_success());
        let decoded = result.envelope().unwrap();
        assert!(decoded.ok);
        assert_eq!(decoded.ticket_id, "DSK-001");
        assert_eq!(decoded.claims.len(), 1);
    }

    // ========================================
    // Acceptance Test 2: Framed with preamble
    // ========================================
    #[test]
    fn test_framed_with_preamble_decodes() {
        let json = r#"{"ok": true, "role": "junior", "ticket_id": "DSK-002", "confidence": 0.8, "summary": "Test", "claims": [], "next_actions": [], "evidence_used": [], "errors": []}"#;

        let output = format!(
            "Let me think about this...\nAnalyzing...\n{}\n{}\n{}\nDone!",
            PROTO_START, json, PROTO_END
        );

        let decoder = ProtoDecoder::new();
        let result = decoder.decode(&output);

        assert!(result.is_success(), "Should extract frame from preamble");
    }

    // ========================================
    // Acceptance Test 3: Trailing comma repair
    // ========================================
    #[test]
    fn test_trailing_comma_is_repaired() {
        let json_with_comma = r#"{
            "ok": true,
            "role": "junior",
            "ticket_id": "DSK-003",
            "confidence": 0.75,
            "summary": "Fixed trailing comma",
            "claims": [],
            "next_actions": [],
            "evidence_used": [],
            "errors": [],
        }"#;

        let framed = create_frame(json_with_comma);
        let decoder = ProtoDecoder::new();
        let result = decoder.decode(&framed);

        assert!(result.is_success(), "Should repair trailing comma");
    }

    // ========================================
    // Acceptance Test 4: Recovery without frame
    // ========================================
    #[test]
    fn test_recovery_without_frame_markers() {
        let raw_json = r#"{"ok": true, "role": "senior", "ticket_id": "DSK-004", "confidence": 0.95, "summary": "Recovered", "claims": [], "next_actions": [], "evidence_used": [], "errors": []}"#;

        let decoder = ProtoDecoder::new();
        let result = decoder.decode(raw_json);

        assert!(result.is_success(), "Should recover JSON without frame");
    }

    // ========================================
    // Acceptance Test 5: Timeout is not parse error
    // ========================================
    #[test]
    fn test_timeout_distinct_from_parse_error() {
        let timeout_result = ProtoDecoder::timeout_error(12000, Some("partial".to_string()));

        if let DecodeResult::Failed(error) = timeout_result {
            assert!(error.is_timeout());
            assert!(!error.is_parse_error());
            assert!(error.message().contains("12000"));
        } else {
            panic!("Expected failure");
        }
    }

    // ========================================
    // Acceptance Test 6: Stats only count valid success
    // ========================================
    #[test]
    fn test_stats_count_valid_success_only() {
        let mut stats = PeriodStats::new();

        // Valid success
        let success = DecodeResult::Success(ModelResultEnvelope::success(
            ModelRole::Junior, "DSK-005", "Success", 0.9,
        ));
        let outcome = outcome_from_decode(&success);
        assert_eq!(outcome, TicketOutcome::Resolved);
        stats.record(outcome, 100);

        // Parse failure
        let failure = DecodeResult::Failed(DecodeError::NoFrame {
            raw_output: "garbage".to_string(),
        });
        let outcome = outcome_from_decode(&failure);
        assert_eq!(outcome, TicketOutcome::InternalFailure);
        stats.record(outcome, 150);

        // Timeout
        let timeout = DecodeResult::Failed(DecodeError::ModelTimeout {
            timeout_ms: 12000,
            partial_output: None,
        });
        let outcome = outcome_from_decode(&timeout);
        assert_eq!(outcome, TicketOutcome::InternalFailure);
        stats.record(outcome, 12000);

        // Verify stats integrity
        assert_eq!(stats.resolved, 1);
        assert_eq!(stats.internal_failures, 2);
        assert!((stats.resolution_rate() - 0.333).abs() < 0.01);
    }

    // ========================================
    // Acceptance Test 7: Evidence fallback on failure
    // ========================================
    #[test]
    fn test_evidence_fallback_on_model_failure() {
        let error = DecodeError::ModelTimeout {
            timeout_ms: 12000,
            partial_output: None,
        };

        let mut builder = EvidenceFallback::new("DSK-006", ModelRole::Junior, error);
        builder.add_evidence(GatheredEvidence::new(
            "ev_boot",
            EvidenceKind::Probe,
            "Boot Analysis",
            "Boot took 15 seconds",
        ));

        let response = builder.build();

        assert_eq!(response.ticket_id, "DSK-006");
        assert!(!response.evidence.is_empty());
        assert!(response.confidence <= 0.5);
        assert!(!response.message.is_empty());
        assert!(!response.suggested_probes.is_empty());
    }

    // ========================================
    // Acceptance Test 8: Stream buffer detects complete frame
    // ========================================
    #[test]
    fn test_stream_buffer_frame_detection() {
        let mut buffer = StreamBuffer::new();

        // Simulate streaming chunks
        buffer.append("Thinking...\n");
        assert_eq!(buffer.state(), StreamState::Receiving);

        buffer.append(PROTO_START);
        assert_eq!(buffer.state(), StreamState::FrameStarted);

        buffer.append(r#"{"ok": true}"#);
        assert_eq!(buffer.state(), StreamState::FrameStarted);

        buffer.append(PROTO_END);
        assert_eq!(buffer.state(), StreamState::FrameComplete);
        assert!(buffer.has_complete_frame());
    }

    // ========================================
    // Acceptance Test 9: Envelope validation
    // ========================================
    #[test]
    fn test_envelope_validation() {
        // Valid success
        let valid = ModelResultEnvelope::success(ModelRole::Junior, "DSK-007", "Summary", 0.8);
        assert!(valid.validate().is_valid());

        // Invalid: success with empty summary
        let invalid_summary = ModelResultEnvelope {
            ok: true,
            role: ModelRole::Junior,
            ticket_id: "DSK-008".to_string(),
            confidence: 0.8,
            summary: String::new(), // Empty!
            claims: vec![],
            next_actions: vec![],
            evidence_used: vec![],
            errors: vec![],
        };
        assert!(!invalid_summary.validate().is_valid());

        // Invalid: failure with no errors
        let invalid_failure = ModelResultEnvelope {
            ok: false,
            role: ModelRole::Senior,
            ticket_id: "DSK-009".to_string(),
            confidence: 0.0,
            summary: String::new(),
            claims: vec![],
            next_actions: vec![],
            evidence_used: vec![],
            errors: vec![], // No errors!
        };
        assert!(!invalid_failure.validate().is_valid());
    }

    // ========================================
    // Acceptance Test 10: Full pipeline
    // ========================================
    #[test]
    fn test_full_pipeline_success() {
        // 1. Create envelope
        let envelope = ModelResultEnvelope::success(ModelRole::Junior, "DSK-010", "Boot is 15s", 0.85)
            .with_claim(Claim::with_support("Boot slow", vec!["ev_1".to_string()]))
            .with_action(Action::probe("sys.boot.breakdown"))
            .with_evidence(EvidenceRef::probe("ev_1", "Boot timing"));

        // 2. Serialize and frame
        let json = serde_json::to_string(&envelope).unwrap();
        let framed = create_frame(&json);

        // 3. Simulate streaming
        let mut buffer = StreamBuffer::new();
        buffer.append(&framed);
        assert!(buffer.has_complete_frame());

        // 4. Decode
        let decoder = ProtoDecoder::new();
        let result = decoder.decode(buffer.content());
        assert!(result.is_success());

        // 5. Update stats
        let mut stats = PeriodStats::new();
        let outcome = outcome_from_decode(&result);
        assert_eq!(outcome, TicketOutcome::Resolved);
        stats.record(outcome, 500);

        assert_eq!(stats.resolved, 1);
        assert_eq!(stats.internal_failures, 0);
    }

    // ========================================
    // Acceptance Test 11: Full pipeline failure
    // ========================================
    #[test]
    fn test_full_pipeline_failure() {
        // 1. Simulate garbage output (model went haywire)
        let garbage = "I don't understand the question. Let me try again...";

        // 2. Try streaming
        let mut buffer = StreamBuffer::new();
        buffer.append(garbage);
        buffer.complete(); // Model finished
        assert_eq!(buffer.state(), StreamState::NoFrame);

        // 3. Decode (will fail)
        let decoder = ProtoDecoder::new();
        let result = decoder.decode(buffer.content());
        assert!(!result.is_success());

        // 4. Create fallback
        if let Some(error) = result.error() {
            let mut fallback = EvidenceFallback::new("DSK-011", ModelRole::Junior, error.clone());
            fallback.add_evidence(GatheredEvidence::new(
                "ev_collected",
                EvidenceKind::Probe,
                "Previous Data",
                "Some data collected before failure",
            ));
            let response = fallback.build();

            assert!(!response.message.is_empty());
            assert!(response.confidence <= 0.5);
        }

        // 5. Update stats
        let mut stats = PeriodStats::new();
        let outcome = outcome_from_decode(&result);
        assert_eq!(outcome, TicketOutcome::InternalFailure);
        stats.record(outcome, 5000);

        assert_eq!(stats.resolved, 0);
        assert_eq!(stats.internal_failures, 1);
    }

    // ========================================
    // Acceptance Test 12: Multiple frames rejected
    // ========================================
    #[test]
    fn test_multiple_frames_rejected() {
        let double_frame = format!(
            "{}{{}}{}{}{{}}{}",
            PROTO_START, PROTO_END, PROTO_START, PROTO_END
        );

        let decoder = ProtoDecoder::new();
        let result = decoder.decode(&double_frame);

        assert!(!result.is_success());
        assert!(matches!(result.error(), Some(DecodeError::MultipleFrames)));
    }

    // ========================================
    // Acceptance Test 13: Empty output handled
    // ========================================
    #[test]
    fn test_empty_output_handled() {
        let decoder = ProtoDecoder::new();
        let result = decoder.decode("");

        assert!(!result.is_success());
        assert!(matches!(result.error(), Some(DecodeError::EmptyOutput)));
    }

    // ========================================
    // Acceptance Test 14: Confidence bounds enforced
    // ========================================
    #[test]
    fn test_confidence_bounds() {
        // Envelope clamps confidence
        let envelope = ModelResultEnvelope::success(ModelRole::Junior, "DSK-012", "Test", 1.5);
        assert!(envelope.confidence <= 1.0);

        // Fallback caps at 0.5
        let error = DecodeError::EmptyOutput;
        let builder = EvidenceFallback::new("DSK-013", ModelRole::Junior, error);
        let response = builder.build();
        assert!(response.confidence <= 0.5);
    }

    // ========================================
    // Acceptance Test 15: Action types work
    // ========================================
    #[test]
    fn test_action_types() {
        let probe = Action::probe("sys.boot.analyze");
        assert!(!probe.requires_confirmation);

        let change = Action::propose_change("Restart nginx", "systemctl restart nginx");
        assert!(change.requires_confirmation);

        let ask = Action::ask_user("Should I proceed?");
        assert!(!ask.requires_confirmation);
    }
}
