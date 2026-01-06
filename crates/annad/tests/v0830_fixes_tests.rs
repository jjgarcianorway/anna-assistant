//! Tests for v0.0.830 fixes - internal comms, routing, and prompts.
//!
//! These tests verify the critical fixes from the audit:
//! 1. Internal comms types exist and work
//! 2. Deterministic routing works correctly
//! 3. Specialist prompts are domain-specific

// =============================================================================
// Test 1: Internal comms types
// =============================================================================

#[test]
fn test_internal_comms_types_exist() {
    // Verify InternalComm and CoreLoopResult types exist and are public
    use annad::core_loop::InternalComm;
    use annad::core_loop::CoreLoopResult;
    use annad::core_loop::AnswerSource;

    let comm = InternalComm {
        from: "Anna".to_string(),
        message: "Test message".to_string(),
        elapsed_ms: 100,
    };

    assert_eq!(comm.from, "Anna");
    assert_eq!(comm.message, "Test message");
    assert_eq!(comm.elapsed_ms, 100);
}

#[test]
fn test_core_loop_result_has_internal_comms() {
    use annad::core_loop::{CoreLoopResult, AnswerSource, InternalComm};

    let result = CoreLoopResult {
        answer: "Test answer".to_string(),
        source: AnswerSource::Recipe,
        recipe_id: Some("test_recipe".to_string()),
        reliability: 90,
        elapsed_ms: 150,
        internal_comms: vec![
            InternalComm {
                from: "Anna".to_string(),
                message: "Processing query...".to_string(),
                elapsed_ms: 10,
            },
            InternalComm {
                from: "Specialist".to_string(),
                message: "Found answer".to_string(),
                elapsed_ms: 100,
            },
        ],
    };

    // Verify internal comms are preserved
    assert_eq!(result.internal_comms.len(), 2);
    assert_eq!(result.internal_comms[0].from, "Anna");
    assert_eq!(result.internal_comms[1].from, "Specialist");
}

// =============================================================================
// Test 2: Deterministic routing
// =============================================================================

#[test]
fn test_deterministic_routing_system_update() {
    use annad::router::{classify_query, QueryClass};

    // "update my system" should classify as SystemUpdate
    let class = classify_query("update my system");
    assert_eq!(class, QueryClass::SystemUpdate);
}

#[test]
fn test_deterministic_routing_package_updates() {
    use annad::router::{classify_query, QueryClass};

    // "pending updates" queries should classify as PackageUpdates
    let class = classify_query("any pending updates?");
    assert_eq!(class, QueryClass::PackageUpdates);
}

#[test]
fn test_deterministic_routing_memory() {
    use annad::router::{classify_query, QueryClass};

    // RAM queries classify as RamInfo
    let class = classify_query("how much RAM do I have?");
    assert_eq!(class, QueryClass::RamInfo);

    // Memory usage is a different variant
    let class = classify_query("memory usage");
    assert_eq!(class, QueryClass::MemoryUsage);
}

#[test]
fn test_deterministic_routing_disk() {
    use annad::router::{classify_query, QueryClass};

    let class = classify_query("disk usage");
    assert_eq!(class, QueryClass::DiskUsage);
}

#[test]
fn test_deterministic_routing_unknown() {
    use annad::router::{classify_query, QueryClass};

    // Random queries should be Unknown
    let class = classify_query("what is the meaning of life?");
    assert_eq!(class, QueryClass::Unknown);
}

// =============================================================================
// Test 3: Specialist prompts are domain-specific
// =============================================================================

#[test]
fn test_specialist_prompt_system() {
    use anna_shared::rpc::SpecialistDomain;
    use annad::specialist_prompt::build_specialist_prompt;

    let prompt = build_specialist_prompt(SpecialistDomain::System);

    // Should contain system-specific hints
    assert!(prompt.contains("CPU"), "System prompt should mention CPU");
    assert!(prompt.contains("memory"), "System prompt should mention memory");
    assert!(prompt.contains("swap"), "System prompt should mention swap");
}

#[test]
fn test_specialist_prompt_network() {
    use anna_shared::rpc::SpecialistDomain;
    use annad::specialist_prompt::build_specialist_prompt;

    let prompt = build_specialist_prompt(SpecialistDomain::Network);

    // Should contain network-specific hints
    assert!(prompt.contains("IP"), "Network prompt should mention IP");
    assert!(prompt.contains("DNS") || prompt.contains("gateway"), "Network prompt should mention DNS or gateway");
}

#[test]
fn test_specialist_prompt_storage() {
    use anna_shared::rpc::SpecialistDomain;
    use annad::specialist_prompt::build_specialist_prompt;

    let prompt = build_specialist_prompt(SpecialistDomain::Storage);

    // Should contain storage-specific hints
    assert!(prompt.contains("Disk") || prompt.contains("disk"), "Storage prompt should mention disk");
    assert!(prompt.contains("partition") || prompt.contains("mount"), "Storage prompt should mention partitions or mounts");
}

#[test]
fn test_specialist_prompt_packages() {
    use anna_shared::rpc::SpecialistDomain;
    use annad::specialist_prompt::build_specialist_prompt;

    let prompt = build_specialist_prompt(SpecialistDomain::Packages);

    // Should contain package-specific hints (Arch Linux)
    assert!(prompt.contains("pacman"), "Packages prompt should mention pacman");
}

#[test]
fn test_specialist_prompt_has_schema() {
    use anna_shared::rpc::SpecialistDomain;
    use annad::specialist_prompt::build_specialist_prompt;

    let prompt = build_specialist_prompt(SpecialistDomain::System);

    // All prompts should have the JSON schema
    assert!(prompt.contains("Output format"), "Prompt should have output format");
    assert!(prompt.contains("status"), "Prompt should mention status field");
    assert!(prompt.contains("answer"), "Prompt should mention answer field");
    assert!(prompt.contains("confidence"), "Prompt should mention confidence field");
}

// =============================================================================
// Test 4: Actor mapping for transcript
// =============================================================================

#[test]
fn test_actor_enum_variants() {
    use anna_shared::transcript::Actor;

    // Test that we can create all the Actor variants we need
    let anna = Actor::Anna;
    let specialist = Actor::Specialist;
    let junior = Actor::Junior;
    let senior = Actor::Senior;
    let you = Actor::You;

    // They should all be different
    assert_ne!(anna, specialist);
    assert_ne!(junior, senior);
    assert_ne!(anna, you);
}

// =============================================================================
// Test 5: Progress events have internal comms variant
// =============================================================================

#[test]
fn test_progress_event_internal_comms() {
    use anna_shared::progress::{ProgressEvent, RequestStage};

    // Create an internal comms progress event
    let event = ProgressEvent::internal_comms(
        RequestStage::Specialist,
        "Anna",
        "Processing your query",
        100,
    );

    // Verify it was created successfully
    assert_eq!(event.stage, RequestStage::Specialist);
    assert_eq!(event.elapsed_ms, 100);
}

// =============================================================================
// Test 6: Transcript events work
// =============================================================================

#[test]
fn test_transcript_message_event() {
    use anna_shared::transcript::{Actor, Transcript, TranscriptEvent};

    let mut transcript = Transcript::new();

    // Add a message
    let event = TranscriptEvent::message(
        100,
        Actor::Specialist,
        Actor::You,
        "Here is my answer",
    );
    transcript.push(event);

    // Add final answer
    let event = TranscriptEvent::final_answer(200, "Your system is healthy");
    transcript.push(event);

    assert_eq!(transcript.len(), 2);
}
