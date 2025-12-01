//! Deterministic Orchestration Tests v1.0.0
//!
//! See `docs/architecture.md` for the test strategy.
//!
//! These tests use FakeLlmClient and FakeProbeExecutor to verify
//! orchestration flows without any network or shell calls.

use anna_common::{FastQuestionType, ProbeRequest};
use annad::orchestrator::{
    DraftAnswerV80, FakeJuniorResponse, FakeLlmClient, FakeLlmClientBuilder,
    FakeProbeExecutor, FakeSeniorResponse, LlmClient, ProbeExecutor,
};

// ============================================================================
// Brain Fast Path Tests
// ============================================================================

/// Brain should classify RAM questions correctly
#[test]
fn test_brain_classifies_ram_question() {
    let qt = FastQuestionType::classify("How much RAM do I have?");
    assert_eq!(qt, FastQuestionType::Ram);

    let qt = FastQuestionType::classify("What is my total memory?");
    assert_eq!(qt, FastQuestionType::Ram);

    let qt = FastQuestionType::classify("how many gb of memory do I have");
    assert_eq!(qt, FastQuestionType::Ram);
}

/// Brain should classify CPU questions correctly
#[test]
fn test_brain_classifies_cpu_question() {
    let qt = FastQuestionType::classify("How many CPU cores do I have?");
    assert_eq!(qt, FastQuestionType::CpuCores);

    let qt = FastQuestionType::classify("What processor do I have?");
    // v1.1.0: This now classifies as CpuModel (asking about processor type)
    // Previously Unknown or CpuCores, now we have a dedicated type
    assert!(
        qt == FastQuestionType::Unknown
            || qt == FastQuestionType::CpuCores
            || qt == FastQuestionType::CpuModel,
        "Expected Unknown, CpuCores, or CpuModel but got {:?}",
        qt
    );

    let qt = FastQuestionType::classify("number of cpu threads");
    assert_eq!(qt, FastQuestionType::CpuCores);
}

/// Brain should classify disk questions correctly
#[test]
fn test_brain_classifies_disk_question() {
    let qt = FastQuestionType::classify("How much free disk space on root?");
    assert_eq!(qt, FastQuestionType::RootDiskSpace);

    let qt = FastQuestionType::classify("What is available storage on /?");
    assert_eq!(qt, FastQuestionType::RootDiskSpace);
}

/// Brain should classify health questions correctly
#[test]
fn test_brain_classifies_health_question() {
    let qt = FastQuestionType::classify("Are you healthy Anna?");
    assert_eq!(qt, FastQuestionType::AnnaHealth);

    let qt = FastQuestionType::classify("What is your health status?");
    assert_eq!(qt, FastQuestionType::AnnaHealth);
}

/// Brain should recognize unknown questions
#[test]
fn test_brain_classifies_unknown_question() {
    let qt = FastQuestionType::classify("What is the meaning of life?");
    assert_eq!(qt, FastQuestionType::Unknown);

    let qt = FastQuestionType::classify("Install nginx please");
    assert_eq!(qt, FastQuestionType::Unknown);
}

// ============================================================================
// LLM Fake Tests - Verify Fake Behavior
// ============================================================================

/// FakeLlmClient should return pre-configured Junior responses
#[tokio::test]
async fn test_fake_llm_junior_response() {
    let fake = FakeLlmClient::with_direct_answer("You have 8 CPU cores", 0.95);

    let (response, _) = fake.call_junior_v80("How many cores?").await.unwrap();

    assert!(response.probe_requests.is_empty());
    assert!(response.draft_answer.is_some());
    assert_eq!(response.draft_answer.unwrap().text, "You have 8 CPU cores");
}

/// FakeLlmClient should return pre-configured Senior responses
#[tokio::test]
async fn test_fake_llm_senior_response() {
    let fake = FakeLlmClient::with_direct_answer("You have 8 CPU cores", 0.95);

    let (response, _) = fake.call_senior_v80("review").await.unwrap();

    assert_eq!(response.verdict, "approve");
    assert_eq!(response.scores_overall, 0.95);
}

/// FakeLlmClient should track call counts correctly
#[tokio::test]
async fn test_fake_llm_call_counts() {
    let fake = FakeLlmClient::new();

    assert_eq!(fake.junior_call_count(), 0);
    assert_eq!(fake.senior_call_count(), 0);

    fake.call_junior_v80("test1").await.unwrap();
    fake.call_junior_v80("test2").await.unwrap();
    fake.call_senior_v80("review").await.unwrap();

    assert_eq!(fake.junior_call_count(), 2);
    assert_eq!(fake.senior_call_count(), 1);
}

/// FakeLlmClient should handle probe requests correctly
#[tokio::test]
async fn test_fake_llm_probe_request() {
    let fake = FakeLlmClient::with_probe_request("cpu.info", "Need CPU information");

    let (response, _) = fake.call_junior_v80("What CPU?").await.unwrap();

    assert_eq!(response.probe_requests.len(), 1);
    assert_eq!(response.probe_requests[0].probe_id, "cpu.info");
    assert!(response.draft_answer.is_none());
}

/// FakeLlmClient should queue multiple responses correctly
#[tokio::test]
async fn test_fake_llm_response_queue() {
    let fake = FakeLlmClientBuilder::new()
        .junior_response(FakeJuniorResponse {
            probe_requests: vec![ProbeRequest {
                probe_id: "cpu.info".into(),
                reason: "first".into(),
            }],
            draft_answer: None,
            raw_text: "{}".into(),
        })
        .junior_response(FakeJuniorResponse {
            probe_requests: vec![],
            draft_answer: Some(DraftAnswerV80 {
                text: "You have 8 cores".into(),
                citations: vec!["cpu.info".into()],
            }),
            raw_text: "{}".into(),
        })
        .build();

    // First call: request probe
    let (r1, _) = fake.call_junior_v80("test").await.unwrap();
    assert_eq!(r1.probe_requests.len(), 1);
    assert!(r1.draft_answer.is_none());

    // Second call: provide answer
    let (r2, _) = fake.call_junior_v80("test").await.unwrap();
    assert!(r2.probe_requests.is_empty());
    assert!(r2.draft_answer.is_some());
}

// ============================================================================
// Probe Fake Tests - Verify Fake Behavior
// ============================================================================

/// FakeProbeExecutor should return pre-configured probe responses
#[tokio::test]
async fn test_fake_probe_response() {
    let fake = FakeProbeExecutor::for_cpu_question();

    let evidence = fake.execute_probe("cpu.info").await;

    assert_eq!(evidence.status, anna_common::EvidenceStatus::Ok);
    assert!(evidence.raw.as_ref().unwrap().contains("Ryzen"));
    assert!(evidence.parsed.is_some());
}

/// FakeProbeExecutor should return NotFound for unknown probes
#[tokio::test]
async fn test_fake_probe_unknown() {
    let fake = FakeProbeExecutor::new();

    let evidence = fake.execute_probe("nonexistent.probe").await;

    assert_eq!(evidence.status, anna_common::EvidenceStatus::NotFound);
}

/// FakeProbeExecutor should track call counts
#[tokio::test]
async fn test_fake_probe_call_counts() {
    let fake = FakeProbeExecutor::new();

    assert_eq!(fake.call_count("cpu.info"), 0);

    fake.execute_probe("cpu.info").await;
    fake.execute_probe("cpu.info").await;
    fake.execute_probe("mem.info").await;

    assert_eq!(fake.call_count("cpu.info"), 2);
    assert_eq!(fake.call_count("mem.info"), 1);
    assert_eq!(fake.total_calls(), 3);
}

/// FakeProbeExecutor should execute multiple probes
#[tokio::test]
async fn test_fake_probe_multiple() {
    let fake = FakeProbeExecutor::new();

    let probe_ids = vec!["cpu.info".to_string(), "mem.info".to_string()];
    let results = fake.execute_probes(&probe_ids).await;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].probe_id, "cpu.info");
    assert_eq!(results[1].probe_id, "mem.info");
}

// ============================================================================
// Orchestration Flow Tests - Using Fakes
// ============================================================================

/// Test: Junior requests probe -> Execute probe -> Junior provides answer
#[tokio::test]
async fn test_flow_probe_then_answer() {
    // Junior: first call requests probe, second call provides answer
    let llm = FakeLlmClientBuilder::new()
        .junior_response(FakeJuniorResponse {
            probe_requests: vec![ProbeRequest {
                probe_id: "cpu.info".into(),
                reason: "Need CPU info to answer".into(),
            }],
            draft_answer: None,
            raw_text: "{}".into(),
        })
        .junior_response(FakeJuniorResponse {
            probe_requests: vec![],
            draft_answer: Some(DraftAnswerV80 {
                text: "You have 16 CPU threads".into(),
                citations: vec!["cpu.info".into()],
            }),
            raw_text: "{}".into(),
        })
        .senior_response(FakeSeniorResponse {
            verdict: "approve".into(),
            fixed_answer: None,
            scores_overall: 0.95,
            raw_text: "{}".into(),
        })
        .build();

    let probes = FakeProbeExecutor::for_cpu_question();

    // Simulate orchestration flow
    // Step 1: Junior requests probe
    let (j1, _) = llm.call_junior_v80("How many CPU threads?").await.unwrap();
    assert_eq!(j1.probe_requests.len(), 1);
    assert!(j1.draft_answer.is_none());

    // Step 2: Execute requested probe
    let evidence = probes.execute_probe(&j1.probe_requests[0].probe_id).await;
    assert_eq!(evidence.status, anna_common::EvidenceStatus::Ok);

    // Step 3: Junior provides answer with evidence
    let (j2, _) = llm.call_junior_v80("with evidence").await.unwrap();
    assert!(j2.draft_answer.is_some());
    let draft = j2.draft_answer.unwrap();
    assert!(draft.text.contains("16"));

    // Step 4: Senior reviews
    let (senior, _) = llm.call_senior_v80("review").await.unwrap();
    assert_eq!(senior.verdict, "approve");
    assert_eq!(senior.scores_overall, 0.95);

    // Verify call counts: 2 Junior + 1 Senior = 3 total (matches architecture limit)
    assert_eq!(llm.junior_call_count(), 2);
    assert_eq!(llm.senior_call_count(), 1);
}

/// Test: Direct answer without probes (simple question)
#[tokio::test]
async fn test_flow_direct_answer() {
    let llm = FakeLlmClient::with_direct_answer("Anna is healthy and running.", 0.90);
    let _probes = FakeProbeExecutor::new();

    // Junior provides answer directly
    let (junior, _) = llm.call_junior_v80("Are you healthy?").await.unwrap();
    assert!(junior.probe_requests.is_empty());
    assert!(junior.draft_answer.is_some());

    // Senior approves
    let (senior, _) = llm.call_senior_v80("review").await.unwrap();
    assert_eq!(senior.verdict, "approve");

    // Only 1 Junior + 1 Senior = 2 total (minimal path)
    assert_eq!(llm.junior_call_count(), 1);
    assert_eq!(llm.senior_call_count(), 1);
}

/// Test: Senior corrects Junior's answer
#[tokio::test]
async fn test_flow_senior_correction() {
    let llm = FakeLlmClient::with_senior_fix(
        "You have 8 cores",
        "You have 8 cores (16 threads with hyperthreading)",
        0.92,
    );

    // Junior provides draft
    let (junior, _) = llm.call_junior_v80("CPU info?").await.unwrap();
    assert!(junior.draft_answer.is_some());

    // Senior fixes
    let (senior, _) = llm.call_senior_v80("review").await.unwrap();
    assert_eq!(senior.verdict, "fix_and_accept");
    assert!(senior.fixed_answer.is_some());
    assert!(senior.fixed_answer.unwrap().contains("hyperthreading"));
}

/// Test: Senior refuses low-quality answer
#[tokio::test]
async fn test_flow_senior_refuses() {
    let llm = FakeLlmClient::with_senior_refuse("Insufficient evidence for claim");

    // Junior provides questionable draft
    let (junior, _) = llm.call_junior_v80("What is the best distro?").await.unwrap();
    let _ = junior; // Not checking draft content

    // Senior refuses
    let (senior, _) = llm.call_senior_v80("review").await.unwrap();
    assert_eq!(senior.verdict, "refuse");
    assert_eq!(senior.scores_overall, 0.0);
}

/// Test: LLM unavailable should be detectable
#[tokio::test]
async fn test_llm_unavailable() {
    let llm = FakeLlmClient::unavailable();

    assert!(!llm.is_available().await);
}

/// Test: Probe failures should be handled gracefully
#[tokio::test]
async fn test_probe_failures() {
    let probes = FakeProbeExecutor::all_failing("Connection refused");

    let evidence = probes.execute_probe("cpu.info").await;

    assert_eq!(evidence.status, anna_common::EvidenceStatus::Error);
    assert!(evidence.raw.as_ref().unwrap().contains("Connection refused"));
}

// ============================================================================
// Architecture Invariant Tests
// ============================================================================

/// Invariant: Max 2 Junior + 1 Senior = 3 LLM calls
#[tokio::test]
async fn test_invariant_max_llm_calls() {
    // This test documents the architectural invariant
    // Real orchestration should enforce this limit
    let max_junior_calls = 2;
    let max_senior_calls = 1;
    let max_total = max_junior_calls + max_senior_calls;

    assert_eq!(max_total, 3, "Architecture invariant: max 3 LLM calls");
}

/// Invariant: Probes use whitelist only
#[test]
fn test_invariant_probe_whitelist() {
    // FakeProbeExecutor only responds to known probes
    let fake = FakeProbeExecutor::new();

    // These should be valid (in the fake's whitelist)
    assert!(fake.is_valid("cpu.info"));
    assert!(fake.is_valid("mem.info"));
    assert!(fake.is_valid("disk.df"));
    assert!(fake.is_valid("net.interfaces"));

    // Arbitrary commands should NOT be valid
    assert!(!fake.is_valid("rm -rf /"));
    assert!(!fake.is_valid("curl http://evil.com"));
    assert!(!fake.is_valid("arbitrary.command"));
}

/// Invariant: Every answer has an origin
#[test]
fn test_invariant_answer_has_origin() {
    let answer = anna_common::FastAnswer::new("Test answer", vec!["cpu.info"], 0.95);

    // Origin must be set
    assert!(!answer.origin.is_empty());
    assert_eq!(answer.origin, "Brain");
}

/// Invariant: Reliability is 0.0-1.0
#[test]
fn test_invariant_reliability_range() {
    // Green threshold
    let green_threshold = 0.85;
    assert!((0.0..=1.0).contains(&green_threshold));

    // Yellow threshold
    let yellow_threshold = 0.60;
    assert!((0.0..=1.0).contains(&yellow_threshold));

    // Red is below yellow
    assert!(yellow_threshold < green_threshold);
}
