//! Integration Tests for v0.0.74 Direct Answers
//!
//! Tests the full flow from classification to direct answers:
//! - "how much memory do i have" → returns direct answer, uses memory tool
//! - "what kernel version" → uses kernel tool
//! - "how much disk space" → uses disk tool
//! - "network status" → uses network summary or doctor
//! - Regression: none produce "Proposed action plan"

use crate::case_engine::IntentType;
use crate::direct_answer::{generate_direct_answer, DirectAnswer};
use crate::evidence_topic::{detect_topic, EvidenceTopic};
use crate::intent_taxonomy::{classify_intent, IntentClassification};
use crate::system_query_router::{detect_target, QueryTarget};
use crate::tools::ToolResult;

// ============================================================================
// Classification Tests (Part A verification)
// ============================================================================

#[test]
fn test_v074_memory_query_is_system_query() {
    let queries = [
        "how much memory do i have",
        "how much ram do i have",
        "what is my ram",
        "tell me my memory",
    ];
    for query in queries {
        let result = classify_intent(query);
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "Query '{}' should be SystemQuery, got {:?}",
            query,
            result.intent
        );
    }
}

#[test]
fn test_v074_kernel_query_is_system_query() {
    let queries = [
        "what kernel version am i using",
        "what kernel am i running",
        "kernel version",
        "linux version",
    ];
    for query in queries {
        let result = classify_intent(query);
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "Query '{}' should be SystemQuery, got {:?}",
            query,
            result.intent
        );
    }
}

#[test]
fn test_v074_disk_query_is_system_query() {
    let queries = [
        "how much disk space is free",
        "disk space",
        "how much space do i have",
        "what is my disk usage",
    ];
    for query in queries {
        let result = classify_intent(query);
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "Query '{}' should be SystemQuery, got {:?}",
            query,
            result.intent
        );
    }
}

#[test]
fn test_v074_network_status_is_system_query() {
    let queries = [
        "what is my network status",
        "am i connected",
        "is my network up",
    ];
    for query in queries {
        let result = classify_intent(query);
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "Query '{}' should be SystemQuery, got {:?}",
            query,
            result.intent
        );
    }
}

#[test]
fn test_v074_service_check_is_system_query() {
    let queries = [
        "is docker running",
        "is nginx running",
        "is systemd running",
    ];
    for query in queries {
        let result = classify_intent(query);
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "Query '{}' should be SystemQuery, got {:?}",
            query,
            result.intent
        );
    }
}

// ============================================================================
// Topic Detection Tests (Part B verification)
// ============================================================================

#[test]
fn test_v074_memory_query_detects_memory_topic() {
    let detection = detect_topic("how much memory do i have");
    assert_eq!(
        detection.topic,
        EvidenceTopic::MemoryInfo,
        "Should detect MemoryInfo topic"
    );
    assert!(detection.confidence >= 80);
}

#[test]
fn test_v074_kernel_query_detects_kernel_topic() {
    let detection = detect_topic("what kernel version am i using");
    assert_eq!(
        detection.topic,
        EvidenceTopic::KernelVersion,
        "Should detect KernelVersion topic"
    );
}

#[test]
fn test_v074_disk_query_detects_disk_topic() {
    let detection = detect_topic("how much disk space is free");
    assert_eq!(
        detection.topic,
        EvidenceTopic::DiskFree,
        "Should detect DiskFree topic"
    );
}

#[test]
fn test_v074_network_query_detects_network_topic() {
    let detection = detect_topic("what is my network status");
    assert_eq!(
        detection.topic,
        EvidenceTopic::NetworkStatus,
        "Should detect NetworkStatus topic"
    );
}

#[test]
fn test_v074_service_query_detects_service_topic() {
    let detection = detect_topic("is docker running");
    assert_eq!(
        detection.topic,
        EvidenceTopic::ServiceState,
        "Should detect ServiceState topic"
    );
    assert_eq!(detection.service_name, Some("docker".to_string()));
}

// ============================================================================
// Query Target Tests (Tool routing)
// ============================================================================

#[test]
fn test_v074_memory_routes_to_memory_target() {
    let (target, confidence) = detect_target("how much ram do i have");
    assert_eq!(target, QueryTarget::Memory);
    assert!(confidence >= 80);
}

#[test]
fn test_v074_kernel_routes_to_kernel_target() {
    let (target, confidence) = detect_target("what kernel version");
    assert_eq!(target, QueryTarget::KernelVersion);
    assert!(confidence >= 80);
}

#[test]
fn test_v074_disk_routes_to_disk_target() {
    let (target, confidence) = detect_target("how much disk space");
    assert_eq!(target, QueryTarget::DiskFree);
    assert!(confidence >= 80);
}

#[test]
fn test_v074_network_routes_to_network_target() {
    let (target, confidence) = detect_target("network status");
    assert_eq!(target, QueryTarget::NetworkStatus);
    assert!(confidence >= 80);
}

// ============================================================================
// Direct Answer Generation Tests (Part B verification)
// ============================================================================

fn mock_memory_tool_result() -> ToolResult {
    ToolResult {
        tool_name: "memory_info".to_string(),
        evidence_id: "E1".to_string(),
        data: serde_json::json!({
            "total_gib": "32",
            "available_gib": "16"
        }),
        human_summary: "32 GiB total, 16 GiB available".to_string(),
        success: true,
        error: None,
        timestamp: 0,
    }
}

fn mock_kernel_tool_result() -> ToolResult {
    ToolResult {
        tool_name: "kernel_version".to_string(),
        evidence_id: "E1".to_string(),
        data: serde_json::json!({
            "kernel_release": "6.17.9-arch1-1"
        }),
        human_summary: "Linux 6.17.9-arch1-1".to_string(),
        success: true,
        error: None,
        timestamp: 0,
    }
}

fn mock_disk_tool_result() -> ToolResult {
    ToolResult {
        tool_name: "mount_usage".to_string(),
        evidence_id: "E1".to_string(),
        data: serde_json::json!({
            "root": {
                "avail_human": "100 GiB",
                "use_percent": "60"
            }
        }),
        human_summary: "Root: 100 GiB free (40%)".to_string(),
        success: true,
        error: None,
        timestamp: 0,
    }
}

fn mock_network_tool_result() -> ToolResult {
    ToolResult {
        tool_name: "network_status".to_string(),
        evidence_id: "E1".to_string(),
        data: serde_json::json!({
            "has_default_route": true,
            "primary_interface": "enp5s0"
        }),
        human_summary: "Network up, interface enp5s0".to_string(),
        success: true,
        error: None,
        timestamp: 0,
    }
}

fn mock_service_tool_result() -> ToolResult {
    ToolResult {
        tool_name: "service_status".to_string(),
        evidence_id: "E1".to_string(),
        data: serde_json::json!({
            "active": true,
            "service_name": "docker"
        }),
        human_summary: "docker.service: running".to_string(),
        success: true,
        error: None,
        timestamp: 0,
    }
}

#[test]
fn test_v074_direct_answer_memory() {
    let results = vec![mock_memory_tool_result()];
    let answer = generate_direct_answer("how much memory do i have", &results);

    assert!(
        answer.is_some(),
        "Should generate direct answer for memory query"
    );
    let answer = answer.unwrap();
    assert!(answer.answer.contains("GiB"), "Answer should mention GiB");
    assert!(answer.answer.contains("32") || answer.answer.contains("RAM"));
    assert_eq!(answer.topic, EvidenceTopic::MemoryInfo);
}

#[test]
fn test_v074_direct_answer_kernel() {
    let results = vec![mock_kernel_tool_result()];
    let answer = generate_direct_answer("what kernel version am i using", &results);

    assert!(
        answer.is_some(),
        "Should generate direct answer for kernel query"
    );
    let answer = answer.unwrap();
    assert!(
        answer.answer.contains("6.17.9") || answer.answer.contains("kernel"),
        "Answer should mention kernel version"
    );
    assert_eq!(answer.topic, EvidenceTopic::KernelVersion);
}

#[test]
fn test_v074_direct_answer_disk() {
    let results = vec![mock_disk_tool_result()];
    let answer = generate_direct_answer("how much disk space is free", &results);

    assert!(
        answer.is_some(),
        "Should generate direct answer for disk query"
    );
    let answer = answer.unwrap();
    assert!(
        answer.answer.contains("100") || answer.answer.contains("free"),
        "Answer should mention disk space"
    );
    assert_eq!(answer.topic, EvidenceTopic::DiskFree);
}

#[test]
fn test_v074_direct_answer_network() {
    let results = vec![mock_network_tool_result()];
    let answer = generate_direct_answer("what is my network status", &results);

    assert!(
        answer.is_some(),
        "Should generate direct answer for network query"
    );
    let answer = answer.unwrap();
    assert!(
        answer.answer.contains("connected") || answer.answer.contains("enp5s0"),
        "Answer should mention connection status"
    );
    assert_eq!(answer.topic, EvidenceTopic::NetworkStatus);
}

#[test]
fn test_v074_direct_answer_service() {
    let results = vec![mock_service_tool_result()];
    let answer = generate_direct_answer("is docker running", &results);

    assert!(
        answer.is_some(),
        "Should generate direct answer for service query"
    );
    let answer = answer.unwrap();
    assert!(
        answer.answer.contains("running") || answer.answer.contains("docker"),
        "Answer should mention service status"
    );
    assert_eq!(answer.topic, EvidenceTopic::ServiceState);
}

// ============================================================================
// Regression Tests (No "Proposed action plan" for system queries)
// ============================================================================

#[test]
fn test_v074_no_action_plan_for_memory_query() {
    let result = classify_intent("how much memory do i have");
    assert_ne!(
        result.intent,
        IntentType::ActionRequest,
        "Memory query should NOT be ActionRequest"
    );
}

#[test]
fn test_v074_no_action_plan_for_kernel_query() {
    let result = classify_intent("what version of linux kernel am i using");
    assert_ne!(
        result.intent,
        IntentType::ActionRequest,
        "Kernel query should NOT be ActionRequest"
    );
}

#[test]
fn test_v074_no_action_plan_for_disk_query() {
    let result = classify_intent("how much disk space is free");
    assert_ne!(
        result.intent,
        IntentType::ActionRequest,
        "Disk query should NOT be ActionRequest"
    );
}

#[test]
fn test_v074_no_action_plan_for_network_status() {
    let result = classify_intent("what is my network status");
    assert_ne!(
        result.intent,
        IntentType::ActionRequest,
        "Network status query should NOT be ActionRequest"
    );
}

#[test]
fn test_v074_no_action_plan_for_service_check() {
    let result = classify_intent("is docker running");
    assert_ne!(
        result.intent,
        IntentType::ActionRequest,
        "Service check should NOT be ActionRequest"
    );
}

// ============================================================================
// Human Mode Verification (No tool names or IDs leaked)
// ============================================================================

#[test]
fn test_v074_direct_answer_human_format_no_ids() {
    let results = vec![mock_memory_tool_result()];
    let answer = generate_direct_answer("how much memory do i have", &results).unwrap();
    let human_output = answer.format_human();

    assert!(
        !human_output.contains("[E1]"),
        "Human output should not contain evidence IDs"
    );
    assert!(
        !human_output.contains("memory_info"),
        "Human output should not contain tool names"
    );
}

#[test]
fn test_v074_direct_answer_source_is_human_readable() {
    let results = vec![mock_memory_tool_result()];
    let answer = generate_direct_answer("how much memory do i have", &results).unwrap();

    // Source should be human-readable, not tool name
    assert!(
        answer.source.contains("memory") || answer.source.contains("snapshot"),
        "Source should be human-readable: {}",
        answer.source
    );
    assert!(
        !answer.source.contains("memory_info"),
        "Source should not be raw tool name"
    );
}
