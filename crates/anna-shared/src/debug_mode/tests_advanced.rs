//! Advanced acceptance tests for debug mode (v0.0.446).
//!
//! Tests for:
//! - TraceBlock canonical structures
//! - Redaction and sanitization
//! - LLM trace structures
//! - Probe trace structures
//! - Timeout info
//! - Gate results
//! - Parse error info

use super::block::*;
use super::config::*;
use super::reason_codes::*;
use super::redact::*;
use super::sanitize::*;
use super::trace::*;
use crate::reliability_metrics::CanonicalOutcome;

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

/// Test: Debug config parses 4 levels from TOML.
#[test]
fn test_config_parses_all_levels() {
    let toml0 = r#"level = "off""#;
    let config0: DebugConfig = toml::from_str(toml0).unwrap();
    assert_eq!(config0.level, DebugLevel::Off);

    let toml1 = r#"level = "summary""#;
    let config1: DebugConfig = toml::from_str(toml1).unwrap();
    assert_eq!(config1.level, DebugLevel::Summary);

    let toml2 = r#"level = "trace""#;
    let config2: DebugConfig = toml::from_str(toml2).unwrap();
    assert_eq!(config2.level, DebugLevel::Trace);

    let toml3 = r#"level = "full""#;
    let config3: DebugConfig = toml::from_str(toml3).unwrap();
    assert_eq!(config3.level, DebugLevel::Full);
}
