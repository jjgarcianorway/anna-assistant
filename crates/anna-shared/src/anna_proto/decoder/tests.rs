//! Tests for decoder functionality.

#[cfg(test)]
mod tests {
    use super::super::core::ProtoDecoder;
    use super::super::types::{DecodeError, DecodeResult};
    use super::super::super::framing::{PROTO_END, PROTO_START};

    fn valid_envelope_json() -> String {
        r#"{
            "ok": true,
            "role": "junior",
            "ticket_id": "DSK-001",
            "confidence": 0.85,
            "summary": "Boot time is 15 seconds",
            "claims": [],
            "next_actions": [],
            "evidence_used": [],
            "errors": []
        }"#
        .to_string()
    }

    #[test]
    fn test_decode_valid_framed() {
        let decoder = ProtoDecoder::new();
        let output = format!("{}\n{}\n{}", PROTO_START, valid_envelope_json(), PROTO_END);

        let result = decoder.decode(&output);
        assert!(result.is_success());

        let envelope = result.envelope().unwrap();
        assert!(envelope.ok);
        assert_eq!(envelope.ticket_id, "DSK-001");
    }

    #[test]
    fn test_decode_with_preamble() {
        let decoder = ProtoDecoder::new();
        let output = format!(
            "Some thinking text...\n{}\n{}\n{}\nMore text",
            PROTO_START,
            valid_envelope_json(),
            PROTO_END
        );

        let result = decoder.decode(&output);
        assert!(result.is_success());
    }

    #[test]
    fn test_decode_no_frame_with_recovery() {
        let decoder = ProtoDecoder::new();
        // Raw JSON without framing
        let output = valid_envelope_json();

        let result = decoder.decode(&output);
        assert!(result.is_success(), "Should recover JSON without frame");
    }

    #[test]
    fn test_decode_trailing_comma() {
        let decoder = ProtoDecoder::new();
        let json_with_comma = r#"{
            "ok": true,
            "role": "junior",
            "ticket_id": "DSK-001",
            "confidence": 0.85,
            "summary": "Test",
            "claims": [],
            "next_actions": [],
            "evidence_used": [],
            "errors": [],
        }"#;

        let output = format!("{}\n{}\n{}", PROTO_START, json_with_comma, PROTO_END);
        let result = decoder.decode(&output);
        assert!(result.is_success(), "Should repair trailing comma");
    }

    #[test]
    fn test_decode_empty_output() {
        let decoder = ProtoDecoder::new();
        let result = decoder.decode("");

        assert!(!result.is_success());
        assert!(matches!(result.error(), Some(DecodeError::EmptyOutput)));
    }

    #[test]
    fn test_decode_incomplete_frame() {
        let decoder = ProtoDecoder::new();
        let output = format!("{}\n{}", PROTO_START, valid_envelope_json());
        // No end marker

        let result = decoder.decode(&output);
        // Should still succeed because JSON is complete
        assert!(
            result.is_success(),
            "Should parse complete JSON even without end marker"
        );
    }

    #[test]
    fn test_decode_multiple_frames() {
        let decoder = ProtoDecoder::new();
        let output = format!(
            "{}\n{{}}\n{}\n{}\n{{}}\n{}",
            PROTO_START, PROTO_END, PROTO_START, PROTO_END
        );

        let result = decoder.decode(&output);
        assert!(!result.is_success());
        assert!(matches!(result.error(), Some(DecodeError::MultipleFrames)));
    }

    #[test]
    fn test_timeout_error() {
        let result = ProtoDecoder::timeout_error(12000, Some("partial...".to_string()));

        assert!(!result.is_success());
        if let Some(DecodeError::ModelTimeout {
            timeout_ms,
            partial_output,
        }) = result.error()
        {
            assert_eq!(*timeout_ms, 12000);
            assert!(partial_output.is_some());
            assert!(result.error().unwrap().is_timeout());
            assert!(!result.error().unwrap().is_parse_error());
        } else {
            panic!("Expected ModelTimeout error");
        }
    }

    #[test]
    fn test_decode_error_display() {
        let err = DecodeError::ModelTimeout {
            timeout_ms: 12000,
            partial_output: None,
        };
        assert!(err.message().contains("12000"));

        let err = DecodeError::NoFrame {
            raw_output: "test".to_string(),
        };
        assert!(err.message().contains("frame"));
    }
}
