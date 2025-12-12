//! Acceptance tests for debug mode (v0.0.446).
//!
//! v0.0.446 acceptance tests:
//! - 4 debug levels (off, summary, trace, full)
//! - TraceBlock canonical structure
//! - Enhanced Redactor with secret patterns
//! - Prompt digest and visibility

use super::block::*;
use super::config::*;
use super::reason_codes::*;
use super::redact::*;
use super::sanitize::*;
use super::trace::*;
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

/// Test: debug.level=2 includes prompt+response but redacts tokens/emails by default.
#[test]
fn test_full_level_redacts_sensitive_data() {
    let sanitizer = Sanitizer::default();

    let prompt = "User email: test@example.com, API_KEY=sk-secret123";
    let response = "Found at 192.168.1.100, password=hunter2";

    let mut block = DebugBlock::new("TEST-002");
    block.llm_calls.push(
        LlmCallDebug::new("specialist", "qwen2.5:7b")
            .with_io(prompt, response, &sanitizer)
            .with_timing(500)
            .with_parse(true, None),
    );

    let output = block.format_full();

    // Must contain LLM call info
    assert!(output.contains("[llm_calls]"), "Missing llm_calls section");
    assert!(output.contains("specialist"), "Missing role");
    assert!(output.contains("qwen2.5:7b"), "Missing model");

    // Must redact sensitive data
    assert!(output.contains("[REDACTED_EMAIL]"), "Email not redacted");
    assert!(output.contains("[REDACTED_SECRET]"), "API key not redacted");
    assert!(output.contains("[REDACTED_IP]"), "IP not redacted");
    assert!(output.contains("[REDACTED]"), "Password not redacted");

    // Must NOT contain raw sensitive data
    assert!(!output.contains("test@example.com"), "Email leaked");
    assert!(!output.contains("sk-secret123"), "API key leaked");
    assert!(!output.contains("192.168.1.100"), "IP leaked");
    assert!(!output.contains("hunter2"), "Password leaked");
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

/// Test: Sanitization does not produce false positives.
#[test]
fn test_sanitization_no_false_positives() {
    let sanitizer = Sanitizer::default();

    let normal_text = "The disk at /dev/sda1 is 80% full. Run df -h for details.";
    let result = sanitizer.sanitize(normal_text);

    assert_eq!(result.redaction_count, 0);
    assert!(result.content.contains("80% full"));
    assert!(result.content.contains("/dev/sda1"));
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

// ============================================================================
// v0.0.446 ACCEPTANCE TESTS
// ============================================================================

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

/// Test: Summary level (1) shows only basic info.
#[test]
fn test_summary_level_output() {
    let mut block = DebugBlock::new("TEST-SUMMARY")
        .with_outcome(CanonicalOutcome::AnsweredVerified)
        .with_topic("storage");

    block.probes_required = vec!["df".into()];
    block
        .probes_run
        .push(ProbeDebugInfo::new("df", "df -h", 0, 50).with_status(ProbeStatus::Ok));
    block.add_reason(ReasonCode::Success);

    let output = block.format_summary();

    // Summary should include basics
    assert!(output.contains("[summary]"), "Missing summary header");
    assert!(output.contains("request_id"), "Missing request_id");
    assert!(output.contains("outcome"), "Missing outcome");
    assert!(output.contains("probes_run"), "Missing probes_run");

    // Summary should NOT include detailed info
    assert!(
        !output.contains("models_used"),
        "Should not have models_used at summary"
    );
    assert!(!output.contains("[debug]"), "Should not have debug header");
}

/// Test: TraceBlock creates canonical structure.
#[test]
fn test_trace_block_canonical_structure_v446() {
    let mut trace = TraceBlock::new("REQ-TRACE", "how much RAM do I have");
    trace.intent = "query_metric".to_string();
    trace.domain = "memory".to_string();
    trace.route = RouteType::Deterministic;
    trace.probes = vec!["free -h".to_string()];
    trace.outcome = TraceOutcome::Success;
    trace.reliability_gate = GateResult {
        passed: true,
        checks: vec![],
    };

    // Verify fields are set
    assert_eq!(trace.request_id, "REQ-TRACE");
    assert_eq!(trace.query, "how much RAM do I have");
    assert_eq!(trace.intent, "query_metric");
    assert_eq!(trace.domain, "memory");
    assert_eq!(trace.route, RouteType::Deterministic);
    assert_eq!(trace.outcome, TraceOutcome::Success);
    assert!(trace.reliability_gate.passed);
}

/// Test: Redactor redacts all mandatory secret patterns.
#[test]
fn test_redactor_mandatory_secrets() {
    let redactor = Redactor::default();

    // API keys and tokens
    let input1 = "API_KEY=sk-secret123 token=abc123";
    let result1 = redactor.redact(input1);
    assert!(!result1.contains("sk-secret123"), "API key leaked");
    assert!(!result1.contains("abc123"), "Token leaked");

    // Passwords
    let input2 = "password=hunter2";
    let result2 = redactor.redact(input2);
    assert!(!result2.contains("hunter2"), "Password leaked");

    // AWS keys
    let input3 = "AKIAIOSFODNN7EXAMPLE";
    let result3 = redactor.redact(input3);
    assert!(!result3.contains("AKIAIOSFODNN7EXAMPLE"), "AWS key leaked");

    // JWT tokens
    let input4 = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.test";
    let result4 = redactor.redact(input4);
    assert!(!result4.contains("eyJhbGciOiJIUzI1NiI"), "JWT leaked");
}

/// Test: PromptDigest captures system and user prompt info.
#[test]
fn test_prompt_digest_creation() {
    let digest = PromptDigest::new("You are a Linux expert.", "How much RAM do I have?");

    // Digest should capture previews
    assert!(digest.system_preview.contains("Linux expert"));
    assert!(digest.user_preview.contains("RAM"));
    assert!(digest.total_chars > 0);
    // Hashes should be non-empty
    assert!(!digest.system_hash.is_empty());
    assert!(!digest.user_hash.is_empty());
}

/// Test: LlmTrace captures info for trace level.
#[test]
fn test_llm_trace_structure() {
    let llm_trace = LlmTrace::new("specialist", "qwen2.5:7b")
        .with_timing(500)
        .with_tokens(100, 50)
        .with_params(0.7, 2048)
        .with_parse(true, None)
        .with_digest("System prompt", "User query");

    assert_eq!(llm_trace.role, "specialist");
    assert_eq!(llm_trace.model, "qwen2.5:7b");
    assert_eq!(llm_trace.duration_ms, 500);
    assert_eq!(llm_trace.input_tokens_est, 100);
    assert_eq!(llm_trace.output_tokens_est, 50);
    assert!(llm_trace.parse_success);
    assert!(llm_trace.prompt_digest.is_some());
}

/// Test: ProbeTrace stores parsed values.
#[test]
fn test_probe_trace_parsed() {
    let mut probe_trace = ProbeTrace::new("free", "free -h", 0, 50);
    probe_trace.add_parsed("available", "8.1Gi");
    probe_trace.add_parsed("total", "15Gi");

    assert_eq!(probe_trace.command, "free -h");
    assert_eq!(probe_trace.exit_code, 0);
    assert_eq!(
        probe_trace.parsed.get("available"),
        Some(&"8.1Gi".to_string())
    );
    assert_eq!(probe_trace.parsed.get("total"), Some(&"15Gi".to_string()));
}

/// Test: TimeoutInfo captures failed stage.
#[test]
fn test_timeout_info_stage() {
    let timeout = TimeoutInfo {
        stage: "specialist".to_string(),
        timeout_ms: 10000,
        elapsed_ms: 15000,
        partial_output: Some("partial...".to_string()),
    };

    assert_eq!(timeout.stage, "specialist");
    assert_eq!(timeout.timeout_ms, 10000);
    assert!(
        timeout.elapsed_ms > timeout.timeout_ms,
        "Elapsed should exceed configured"
    );
    assert!(timeout.partial_output.is_some());
}

/// Test: FailureDetail captures failure reason.
#[test]
fn test_failure_detail_capture() {
    let failure = FailureDetail {
        check: "timeout".to_string(),
        reason: "LLM timeout at specialist stage".to_string(),
        context: None,
    };

    assert_eq!(failure.check, "timeout");
    assert!(failure.reason.contains("LLM timeout"));
    assert!(failure.reason.contains("specialist"));
}

/// Test: Redactor preserves safe environment variables.
#[test]
fn test_redactor_preserves_safe_vars() {
    let redactor = Redactor::default();

    let input = "PATH=/usr/bin HOME=/home/user SECRET_KEY=mysecret LANG=en_US.UTF-8";
    let result = redactor.redact(input);

    // Safe vars preserved
    assert!(result.contains("PATH=/usr/bin"), "PATH should be preserved");
    assert!(
        result.contains("HOME=/home/user"),
        "HOME should be preserved"
    );
    assert!(
        result.contains("LANG=en_US.UTF-8"),
        "LANG should be preserved"
    );

    // Unsafe vars redacted
    assert!(
        !result.contains("mysecret"),
        "SECRET_KEY value should be redacted"
    );
}

/// Test: Sensitive paths are detected.
#[test]
fn test_sensitive_path_detection_v446() {
    // Should be sensitive
    assert!(is_sensitive_path("/etc/shadow"));
    assert!(is_sensitive_path("/home/user/.ssh/id_rsa"));
    assert!(is_sensitive_path("/root/.aws/credentials"));
    assert!(is_sensitive_path("/home/user/.gnupg/private-keys-v1.d"));

    // Should NOT be sensitive
    assert!(!is_sensitive_path("/etc/hostname"));
    assert!(!is_sensitive_path("/var/log/syslog"));
    assert!(!is_sensitive_path("/usr/bin/ssh"));
}

/// Test: GateResult and GateCheck structure.
#[test]
fn test_gate_result_structure() {
    let gate = GateResult {
        passed: true,
        checks: vec![
            GateCheck {
                name: "evidence_present".to_string(),
                passed: true,
                details: None,
            },
            GateCheck {
                name: "claims_bound".to_string(),
                passed: true,
                details: Some("3 claims, 3 bound".to_string()),
            },
        ],
    };

    assert!(gate.passed);
    assert_eq!(gate.checks.len(), 2);
    assert!(gate.checks[0].passed);
    assert!(gate.checks[1].details.is_some());
}

/// Test: RouteType enum values.
#[test]
fn test_route_types() {
    assert_ne!(RouteType::Deterministic, RouteType::LlmSpecialist);
    assert_ne!(RouteType::LlmFallback, RouteType::Deterministic);
    assert_ne!(RouteType::Clarification, RouteType::LlmSpecialist);
}

/// Test: Debug config parses 4 levels from TOML.
#[test]
fn test_config_parses_all_levels() {
    // Off level
    let toml0 = r#"level = "off""#;
    let config0: DebugConfig = toml::from_str(toml0).unwrap();
    assert_eq!(config0.level, DebugLevel::Off);

    // Summary level
    let toml1 = r#"level = "summary""#;
    let config1: DebugConfig = toml::from_str(toml1).unwrap();
    assert_eq!(config1.level, DebugLevel::Summary);

    // Trace level
    let toml2 = r#"level = "trace""#;
    let config2: DebugConfig = toml::from_str(toml2).unwrap();
    assert_eq!(config2.level, DebugLevel::Trace);

    // Full level
    let toml3 = r#"level = "full""#;
    let config3: DebugConfig = toml::from_str(toml3).unwrap();
    assert_eq!(config3.level, DebugLevel::Full);
}

/// Test: ParseErrorInfo captures error context.
#[test]
fn test_parse_error_info() {
    let error = ParseErrorInfo::new("Invalid JSON at line 5").with_location(150, "claims");

    assert!(error.message.contains("Invalid JSON"));
    assert_eq!(error.byte_offset, Some(150));
    assert_eq!(error.field_name, Some("claims".to_string()));
}

/// Test: TraceOutcome enum values.
#[test]
fn test_trace_outcome_values() {
    assert_eq!(format!("{}", TraceOutcome::Success), "SUCCESS");
    assert_eq!(format!("{}", TraceOutcome::FailedTimeout), "FAILED_TIMEOUT");
    assert_eq!(
        format!("{}", TraceOutcome::FailedNoEvidence),
        "FAILED_NO_EVIDENCE"
    );
    assert_eq!(format!("{}", TraceOutcome::FailedParse), "FAILED_PARSE");
}
