//! Tests for strict contract parsing and validation

use super::*;

#[test]
fn test_strict_response_ok() {
    let response = StrictSpecialistResponse::ok(
        "DSK-001",
        "query_metric",
        "Available memory: 17.0 GiB",
        0.95,
    )
    .with_evidence("memory_info", "MemAvailable: 17892232 kB");

    assert!(response.is_valid());
    assert!(response.is_resolved());
}

#[test]
fn test_strict_response_validates_forbidden() {
    let response =
        StrictSpecialistResponse::ok("DSK-001", "check_package", "unknown is installed", 0.9);
    let issues = response.validate();
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.contains("forbidden")));
}

#[test]
fn test_strict_response_validates_evidence() {
    let response =
        StrictSpecialistResponse::ok("DSK-001", "query_metric", "Your RAM is 16GB", 0.95);
    // No evidence but high confidence + ok status
    let issues = response.validate();
    assert!(issues.iter().any(|i| i.contains("no evidence")));
}

#[test]
fn test_parse_clean_json() {
    let json = r#"{"ticket_id":"DSK-001","intent":"query_metric","status":"ok","confidence":0.9,"summary":"You have 16GB RAM","evidence":[{"probe":"memory_info","summary":"MemTotal: 16384000 kB"}]}"#;
    let result = parse_specialist_output(json, "DSK-001", "query_metric");
    assert!(result.is_success());
}

#[test]
fn test_parse_markdown_json() {
    let raw = r#"Here's my analysis:
```json
{"ticket_id":"DSK-001","intent":"query_metric","status":"ok","confidence":0.9,"summary":"16GB RAM available","evidence":[]}
```"#;
    let result = parse_specialist_output(raw, "DSK-001", "query_metric");
    // Should parse but fail validation (no evidence with high confidence)
    match result {
        ParseResult::ValidationFailed { .. } => (), // Expected
        ParseResult::Success(_) => panic!("Should fail validation"),
        other => panic!("Unexpected result: {:?}", other),
    }
}

#[test]
fn test_lenient_parsing() {
    // Old schema format
    let json = r#"{"ticket_id":"DSK-001","status":"ok","answer":{"short":"You have 0 failed services"},"confidence":0.85}"#;
    let result = parse_lenient(json, "DSK-001", "check_status");
    assert!(result.is_ok());
    let r = result.unwrap();
    assert_eq!(r.summary, "You have 0 failed services");
}

#[test]
fn test_is_resolved() {
    let good = StrictSpecialistResponse::ok("DSK-001", "query", "Answer", 0.9)
        .with_evidence("probe", "data");
    assert!(good.is_resolved());

    let low_conf = StrictSpecialistResponse::ok("DSK-001", "query", "Answer", 0.5);
    assert!(!low_conf.is_resolved());

    let partial = StrictSpecialistResponse::partial("DSK-001", "query", "Partial answer");
    assert!(!partial.is_resolved());
}
