//! Integration tests for daemon-client workflow.
//! v0.0.825: Added comprehensive integration testing.
//!
//! These tests verify the end-to-end flow from client request to daemon response.

use anna_shared::rpc::{RpcMethod, RpcRequest};
use std::time::Duration;
use tokio::time::timeout;

/// Helper to create a mock RPC request
fn create_request(method: RpcMethod, params: Option<serde_json::Value>) -> RpcRequest {
    RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: uuid::Uuid::new_v4().to_string(),
        method,
        params,
    }
}

/// Test that status request returns valid response
#[tokio::test]
async fn test_status_request_format() {
    use annad::state::load_initial_state;

    // Create initial state
    let state = load_initial_state().await;

    // Create status request
    let request = create_request(RpcMethod::Status, None);

    // Handle request
    let response = annad::rpc_handler::handle_request(state, request).await;

    // Verify response
    assert!(response.result.is_some(), "Status should return a result");
    assert!(response.error.is_none(), "Status should not return an error");

    // Verify the result contains expected fields
    let result = response.result.unwrap();
    assert!(
        result.get("state").is_some(),
        "Status should contain 'state'"
    );
    assert!(result.get("llm").is_some(), "Status should contain 'llm'");
}

/// Test that stats request returns valid response
#[tokio::test]
async fn test_stats_request_format() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;
    let request = create_request(RpcMethod::Stats, None);

    let response = annad::rpc_handler::handle_request(state, request).await;

    assert!(response.result.is_some(), "Stats should return a result");
    assert!(response.error.is_none(), "Stats should not return an error");
}

/// Test that progress request returns valid response
#[tokio::test]
async fn test_progress_request_format() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;
    let request = create_request(RpcMethod::Progress, None);

    let response = annad::rpc_handler::handle_request(state, request).await;

    assert!(
        response.result.is_some(),
        "Progress should return a result"
    );
    assert!(
        response.error.is_none(),
        "Progress should not return an error"
    );
}

/// Test that daemon info request returns version info
#[tokio::test]
async fn test_daemon_info_request() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;
    let request = create_request(RpcMethod::GetDaemonInfo, None);

    let response = annad::rpc_handler::handle_request(state, request).await;

    assert!(
        response.result.is_some(),
        "DaemonInfo should return a result"
    );

    let result = response.result.unwrap();
    assert!(
        result.get("version_info").is_some(),
        "DaemonInfo should contain version_info"
    );
    assert!(
        result.get("pid").is_some(),
        "DaemonInfo should contain pid"
    );
    assert!(
        result.get("uptime_secs").is_some(),
        "DaemonInfo should contain uptime_secs"
    );
}

/// Test that request without prompt returns error
#[tokio::test]
async fn test_request_without_prompt_returns_error() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;

    // Request without params
    let request = create_request(RpcMethod::Request, None);
    let response = annad::rpc_handler::handle_request(state.clone(), request).await;

    assert!(
        response.error.is_some(),
        "Request without prompt should error"
    );

    // Request with empty params
    let request = create_request(RpcMethod::Request, Some(serde_json::json!({})));
    let response = annad::rpc_handler::handle_request(state, request).await;

    assert!(
        response.error.is_some(),
        "Request with empty params should error"
    );
}

/// Test that status snapshot returns valid format
#[tokio::test]
async fn test_status_snapshot_format() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;
    let request = create_request(RpcMethod::StatusSnapshot, None);

    let response = annad::rpc_handler::handle_request(state, request).await;

    assert!(
        response.result.is_some(),
        "StatusSnapshot should return a result"
    );
}

/// Test concurrent status requests don't cause issues
#[tokio::test]
async fn test_concurrent_status_requests() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;

    // Spawn multiple concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        let state_clone = state.clone();
        handles.push(tokio::spawn(async move {
            let request = create_request(RpcMethod::Status, None);
            annad::rpc_handler::handle_request(state_clone, request).await
        }));
    }

    // Wait for all to complete
    for handle in handles {
        let response = handle.await.expect("Task should complete");
        assert!(
            response.result.is_some(),
            "Concurrent status should succeed"
        );
    }
}

/// Test that response times are reasonable
#[tokio::test]
async fn test_status_response_time() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;
    let request = create_request(RpcMethod::Status, None);

    // Status should complete within 100ms
    let result = timeout(
        Duration::from_millis(100),
        annad::rpc_handler::handle_request(state, request),
    )
    .await;

    assert!(
        result.is_ok(),
        "Status request should complete within 100ms"
    );
}

/// Test feedback submission
#[tokio::test]
async fn test_feedback_submission() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;

    let params = serde_json::json!({
        "request_id": "test-123",
        "query": "test query",
        "answer": "test answer",
        "helpful": true
    });

    let request = create_request(RpcMethod::SubmitFeedback, Some(params));
    let response = annad::rpc_handler::handle_request(state, request).await;

    assert!(
        response.result.is_some(),
        "Feedback submission should succeed"
    );

    let result = response.result.unwrap();
    assert!(
        result.get("recorded").is_some(),
        "Feedback should return recorded status"
    );
}

/// Test truth ledger status
#[tokio::test]
async fn test_truth_ledger_status() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;
    let request = create_request(RpcMethod::GetTruthLedgerStatus, None);

    let response = annad::rpc_handler::handle_request(state, request).await;

    assert!(
        response.result.is_some(),
        "TruthLedgerStatus should return a result"
    );

    let result = response.result.unwrap();
    assert!(
        result.get("total_claims").is_some(),
        "Should contain total_claims"
    );
}

/// Test truth ledger claims query
#[tokio::test]
async fn test_truth_ledger_claims_query() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;

    // Query without params (should return all)
    let request = create_request(RpcMethod::GetTruthLedgerClaims, None);
    let response = annad::rpc_handler::handle_request(state.clone(), request).await;

    assert!(
        response.result.is_some(),
        "TruthLedgerClaims should return a result"
    );

    // Query with filter
    let params = serde_json::json!({
        "veracity": "verified"
    });
    let request = create_request(RpcMethod::GetTruthLedgerClaims, Some(params));
    let response = annad::rpc_handler::handle_request(state, request).await;

    assert!(
        response.result.is_some(),
        "Filtered TruthLedgerClaims should return a result"
    );
}

/// Test claim feedback submission
#[tokio::test]
async fn test_claim_feedback() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;

    let params = serde_json::json!({
        "claim_text": "Test claim",
        "positive_feedback": true
    });

    let request = create_request(RpcMethod::SubmitClaimFeedback, Some(params));
    let response = annad::rpc_handler::handle_request(state, request).await;

    assert!(
        response.result.is_some(),
        "ClaimFeedback should return a result"
    );
}

/// Test that invalid method params return proper errors
#[tokio::test]
async fn test_invalid_params_return_errors() {
    use annad::state::load_initial_state;

    let state = load_initial_state().await;

    // Invalid feedback params
    let params = serde_json::json!({
        "invalid_field": "value"
    });
    let request = create_request(RpcMethod::SubmitFeedback, Some(params));
    let response = annad::rpc_handler::handle_request(state, request).await;

    assert!(response.error.is_some(), "Invalid params should return error");
    let error = response.error.unwrap();
    assert_eq!(error.code, -32602, "Error code should be -32602 (invalid params)");
}
