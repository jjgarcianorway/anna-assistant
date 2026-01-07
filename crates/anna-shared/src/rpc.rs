//! Simplified RPC types for daemon communication.

use serde::{Deserialize, Serialize};

/// RPC methods supported by the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcMethod {
    /// Send a question to be answered
    Ask,
    /// Get daemon status
    Status,
}

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: RpcMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: String,
}

impl RpcRequest {
    pub fn new(method: RpcMethod, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: String,
}

impl RpcResponse {
    pub fn success(id: &str, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: id.to_string(),
        }
    }

    pub fn error(id: &str, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError {
                code,
                message: message.to_string(),
            }),
            id: id.to_string(),
        }
    }
}

/// JSON-RPC 2.0 error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

/// Result of asking a question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskResult {
    /// The final answer to show the user
    pub answer: String,
    /// Whether the answer was successfully validated
    pub success: bool,
    /// Number of iterations it took
    pub iterations: u32,
    /// Commands that were executed (for transparency)
    pub commands_executed: Vec<String>,
    /// Full dialogue for transparency
    pub dialogue: Vec<DialogueStep>,
}

/// A single step in the dialogue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueStep {
    /// Type of step
    pub step_type: StepType,
    /// Content of this step
    pub content: String,
}

/// Types of dialogue steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    /// User's original question
    UserQuestion,
    /// Prompt sent to LLM asking for commands
    AnnaToLlm,
    /// LLM's response with commands
    LlmCommands,
    /// Command being executed
    CommandExec,
    /// Output from command execution
    CommandOutput,
    /// Prompt sent to LLM for validation
    ValidationPrompt,
    /// LLM's validation response
    ValidationResponse,
    /// Final answer prompt sent to LLM
    FinalPrompt,
    /// Final answer from LLM
    FinalAnswer,
}
