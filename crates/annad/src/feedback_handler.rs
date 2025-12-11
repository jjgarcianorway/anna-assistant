//! User feedback RPC handler (v0.0.401).
//!
//! Handles the SubmitFeedback RPC method to record user feedback about answers.

use anna_shared::rpc::{FeedbackParams, FeedbackResult, RpcResponse};
use tracing::info;

use crate::learning_capture::record_user_feedback;

/// Handle SubmitFeedback request - records user feedback for learning
pub async fn handle_submit_feedback(id: String, params: Option<serde_json::Value>) -> RpcResponse {
    let params: FeedbackParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => {
                return RpcResponse::error(id, -32602, format!("Invalid params: {}", e));
            }
        },
        None => {
            return RpcResponse::error(id, -32602, "Missing params".to_string());
        }
    };

    info!(
        "Feedback received: request_id={}, helpful={}, query='{}'",
        params.request_id,
        params.helpful,
        params.query.chars().take(50).collect::<String>()
    );

    let learning_message = record_user_feedback(&params.query, params.helpful);

    let result = FeedbackResult {
        recorded: true,
        learning_message,
    };

    RpcResponse::success(id, serde_json::to_value(&result).unwrap())
}
