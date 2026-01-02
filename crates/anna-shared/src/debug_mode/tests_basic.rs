//! Basic acceptance tests for debug mode (v0.0.446).
//!
//! Tests for:
//! - Debug levels (off, summary, trace, full)
//! - Basic formatting and field inclusion
//! - Config parsing
//! - Simple outcomes and reason codes

use super::block::*;
use super::config::*;
use super::reason_codes::*;
use super::sanitize::*;
use crate::reliability_metrics::CanonicalOutcome;

/// Test: debug.level=1 includes models_used, probes_run, timings, outcome, reason_codes.
#[test]
fn test_trace_level_includes_required_fields() {
    let mut block = DebugBlock::new("TEST-001")
        .with_outcome(CanonicalOutcome::AnsweredVerified)
        .with_topic("storage");

    block.models = ModelsUsedDebug {
        translator: Some("qwen2.5:3b".into()),
        specialist: Some("qwen2.5:7b".into()),
        verifier: None,
    };

    block.probes_required = vec!["df".into(), "free".into()];
    block
        .probes_run
        .push(ProbeDebugInfo::new("df", "df -h", 0, 50).with_status(ProbeStatus::Ok));
    block
        .probes_run
        .push(ProbeDebugInfo::new("free", "free -h", 0, 30).with_status(ProbeStatus::Ok));

    block.timings = TimingDebug {
        total_ms: 1500,
        probe_ms: 80,
        llm_ms: 1200,
        translator_ms: 200,
        specialist_ms: 1000,
    };

    block.add_reason(ReasonCode::Success);

    let output = block.format_trace();

    // Must contain all required fields
    assert!(output.contains("request_id"), "Missing request_id");
    assert!(output.contains("outcome"), "Missing outcome");
    assert!(output.contains("VERIFIED"), "Missing outcome value");
    assert!(output.contains("routed_topic"), "Missing routed_topic");
    assert!(output.contains("models_used"), "Missing models_used");
    assert!(output.contains("qwen2.5:3b"), "Missing translator model");
    assert!(output.contains("qwen2.5:7b"), "Missing specialist model");
    assert!(
        output.contains("probes_required"),
        "Missing probes_required"
    );
    assert!(output.contains("probes_run"), "Missing probes_run");
    assert!(output.contains("df"), "Missing probe df");
    assert!(output.contains("timings"), "Missing timings");
    assert!(output.contains("1500"), "Missing total_ms");
    assert!(output.contains("reason_codes"), "Missing reason_codes");
    assert!(output.contains("SUCCESS"), "Missing reason code");
}

/// Test: timeout produces FAILED_TIMEOUT and no "verified" status text.
#[test]
fn test_timeout_produces_failed_timeout() {
    let mut block = DebugBlock::new("TEST-003")
        .with_outcome(CanonicalOutcome::FailedTimeout)
        .with_topic("network");

    block.timeout = Some(
        TimeoutDebug::new("specialist", 10000, 15000)
            .with_model("qwen2.5:7b")
            .with_partial(500),
    );

    block.add_reason(ReasonCode::LlmTimeoutSpecialist);

    let output = block.format_trace();

    // Must show TIMEOUT outcome, not VERIFIED
    assert!(output.contains("TIMEOUT"), "Missing TIMEOUT outcome");
    assert!(!output.contains("VERIFIED"), "Should not show VERIFIED");

    // Must include timeout details
    assert!(output.contains("specialist"), "Missing timeout stage");
    assert!(output.contains("10000ms"), "Missing configured timeout");
    assert!(output.contains("15000ms"), "Missing elapsed time");
    assert!(output.contains("500 chars"), "Missing partial output info");

    // Must include timeout reason code
    assert!(
        output.contains("LLM_TIMEOUT_SPECIALIST"),
        "Missing timeout reason code"
    );

    // User message should explain timeout
    let user_msg = block.timeout_user_message().unwrap();
    assert!(
        user_msg.contains("timeout"),
        "Missing timeout in user message"
    );
    assert!(
        user_msg.contains("specialist"),
        "Missing stage in user message"
    );
    assert!(
        user_msg.contains("No verified answer"),
        "Missing failure acknowledgment"
    );
}

/// Test: All 4 debug levels produce correct output.
#[test]
fn test_debug_block_formatting_at_all_levels() {
    let mut block = DebugBlock::new("TEST-004")
        .with_outcome(CanonicalOutcome::AnsweredVerified)
        .with_topic("storage");

    block.probes_required = vec!["df".into(), "free".into()];
    block
        .probes_run
        .push(ProbeDebugInfo::new("df", "df -h", 0, 50).with_status(ProbeStatus::Ok));
    block.add_reason(ReasonCode::Success);

    // Level 0 (Off): No output
    assert!(block.format(DebugLevel::Off).is_none());

    // Level 1 (Summary): Basic info only
    let summary = block.format(DebugLevel::Summary);
    assert!(summary.is_some());
    let summary_str = summary.unwrap();
    assert!(summary_str.contains("[summary]"), "Missing summary section");
    assert!(summary_str.contains("request_id"), "Missing request_id");
    assert!(summary_str.contains("outcome"), "Missing outcome");
    assert!(
        summary_str.contains("probes_required"),
        "Missing probes_required"
    );

    // Level 2 (Trace): Detailed debug output
    let trace = block.format(DebugLevel::Trace);
    assert!(trace.is_some());
    let trace_str = trace.unwrap();
    assert!(trace_str.contains("[debug]"), "Missing debug section");
    assert!(trace_str.contains("models_used"), "Missing models in trace");
    assert!(trace_str.contains("timings"), "Missing timings in trace");

    // Level 3 (Full): Everything including raw prompts
    let full = block.format(DebugLevel::Full);
    assert!(full.is_some());
    let full_str = full.unwrap();
    assert!(full_str.contains("[debug]"), "Full must include debug");
}

/// Test: Parse errors produce FAILED_PARSE outcome.
#[test]
fn test_parse_error_outcome() {
    let mut block = DebugBlock::new("TEST-005")
        .with_outcome(CanonicalOutcome::FailedParse)
        .with_topic("storage");

    block.add_reason(ReasonCode::LlmInvalidJson);
    block.add_reason(ReasonCode::ValidatorFailSchema);

    let output = block.format_trace();

    assert!(
        output.contains("PARSE_ERROR"),
        "Missing PARSE_ERROR outcome"
    );
    assert!(
        output.contains("LLM_INVALID_JSON"),
        "Missing invalid json reason"
    );
    assert!(
        output.contains("VALIDATOR_FAIL_SCHEMA"),
        "Missing schema fail reason"
    );
}

/// Test: Probe failures produce appropriate reason codes.
#[test]
fn test_probe_failure_reason_codes() {
    let mut block = DebugBlock::new("TEST-006")
        .with_outcome(CanonicalOutcome::FailedProbes)
        .with_topic("storage");

    block.probes_required = vec!["df".into(), "du".into()];
    block
        .probes_run
        .push(ProbeDebugInfo::new("df", "df -h", 1, 50).with_status(ProbeStatus::Fail));

    block.add_reason(ReasonCode::ProbeFailedExit);
    block.add_reason(ReasonCode::ProbeMissingRequired);

    let output = block.format_trace();

    assert!(
        output.contains("PROBE_FAILED"),
        "Missing PROBE_FAILED outcome"
    );
    assert!(
        output.contains("PROBE_FAILED_EXIT"),
        "Missing probe fail reason"
    );
    assert!(
        output.contains("PROBE_MISSING_REQUIRED"),
        "Missing missing probe reason"
    );
    assert!(output.contains("fail"), "Missing probe status");
}

/// Test: Routing transparency shows translator decision.
#[test]
fn test_routing_transparency() {
    let mut block = DebugBlock::new("TEST-007")
        .with_outcome(CanonicalOutcome::AnsweredVerified)
        .with_topic("storage");

    block.translator_decision = Some(TranslatorDecision::new(
        "diagnose",
        "storage",
        vec!["df".into(), "du".into()],
        0.85,
    ));

    let output = block.format_trace();

    assert!(
        output.contains("translator_decision"),
        "Missing translator decision section"
    );
    assert!(output.contains("intent: diagnose"), "Missing intent");
    assert!(output.contains("domain: storage"), "Missing domain");
    assert!(output.contains("probes: [df, du]"), "Missing probes");
    assert!(output.contains("confidence: 0.85"), "Missing confidence");
}

/// Test: Low confidence triggers appropriate reason code.
#[test]
fn test_low_confidence_reason_code() {
    let mut block = DebugBlock::new("TEST-008");

    block.translator_decision = Some(TranslatorDecision::new(
        "unknown",
        "general",
        vec![],
        0.3, // Low confidence
    ));

    block.add_reason(ReasonCode::RouteLowConfidence);
    block.add_reason(ReasonCode::RouteNoProbes);

    let output = block.format_trace();

    assert!(
        output.contains("ROUTE_LOW_CONFIDENCE"),
        "Missing low confidence reason"
    );
    assert!(
        output.contains("ROUTE_NO_PROBES"),
        "Missing no probes reason"
    );
}

/// Test: Debug config parsing.
#[test]
fn test_debug_config_parsing() {
    let toml = r#"
level = "trace"

[redact]
redact_private_ips = false
redact_emails = true
max_probe_lines = 100
"#;

    let config: DebugConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.level, DebugLevel::Trace);
    assert!(!config.redact.redact_private_ips);
    assert!(config.redact.redact_emails);
    assert_eq!(config.redact.max_probe_lines, 100);
}

/// Test: Fallback reason code when fallback is used.
#[test]
fn test_fallback_reason_code() {
    let mut block = DebugBlock::new("TEST-009")
        .with_outcome(CanonicalOutcome::AnsweredPartial)
        .with_topic("network");

    block.add_reason(ReasonCode::LlmTimeoutSpecialist);
    block.add_reason(ReasonCode::FallbackUsed);

    let output = block.format_trace();

    assert!(output.contains("FALLBACK_USED"), "Missing fallback reason");
    assert!(output.contains("PARTIAL"), "Should show PARTIAL outcome");
}

/// Test: Evidence coverage in debug output.
#[test]
fn test_evidence_debug() {
    let mut block = DebugBlock::new("TEST-010");
    block.evidence = EvidenceDebug {
        claim_count: 5,
        claims_with_evidence: 3,
        evidence_coverage: 0.6,
        evidence_ids: vec!["ev1".into(), "ev2".into()],
    };

    let output = block.format_trace();

    assert!(output.contains("claims:5"), "Missing claim count");
    assert!(output.contains("with_evidence:3"), "Missing evidence count");
    assert!(output.contains("60%"), "Missing coverage percentage");
}

/// Test: Multiple reason codes are all shown.
#[test]
fn test_multiple_reason_codes() {
    let mut codes = ReasonCodes::new();
    codes.add(ReasonCode::RouteLowConfidence);
    codes.add(ReasonCode::ProbeEmptyOutput);
    codes.add(ReasonCode::RetryAttempted);
    codes.add(ReasonCode::FallbackUsed);

    let display = codes.display();

    assert!(display.contains("ROUTE_LOW_CONFIDENCE"));
    assert!(display.contains("PROBE_EMPTY_OUTPUT"));
    assert!(display.contains("RETRY_ATTEMPTED"));
    assert!(display.contains("FALLBACK_USED"));
}

/// Test: Four debug levels are available and correctly ordered.
#[test]
fn test_four_debug_levels() {
    // Level ordering
    assert!(DebugLevel::Off < DebugLevel::Summary);
    assert!(DebugLevel::Summary < DebugLevel::Trace);
    assert!(DebugLevel::Trace < DebugLevel::Full);

    // Level values
    assert_eq!(DebugLevel::Off.as_u8(), 0);
    assert_eq!(DebugLevel::Summary.as_u8(), 1);
    assert_eq!(DebugLevel::Trace.as_u8(), 2);
    assert_eq!(DebugLevel::Full.as_u8(), 3);

    // Level inclusion checks
    assert!(!DebugLevel::Off.includes_summary());
    assert!(DebugLevel::Summary.includes_summary());
    assert!(DebugLevel::Trace.includes_summary());
    assert!(DebugLevel::Full.includes_summary());

    assert!(!DebugLevel::Summary.includes_trace());
    assert!(DebugLevel::Trace.includes_trace());
    assert!(DebugLevel::Full.includes_trace());

    assert!(!DebugLevel::Trace.includes_full());
    assert!(DebugLevel::Full.includes_full());
}
