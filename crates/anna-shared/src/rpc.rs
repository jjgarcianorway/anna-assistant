//! Simplified RPC types for daemon communication.

use serde::{Deserialize, Serialize};

/// Intent categories for question classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntentCategory {
    /// Simple factual question ("what is my kernel version?", "how much RAM?")
    Factual,
    /// How-to question ("how do I install X?", "how to configure Y?")
    HowTo,
    /// Troubleshooting problem ("X not working", "error when Y")
    Troubleshoot,
    /// Multiple questions combined ("what's my disk AND how do I install Y?")
    Multi,
    /// Unclear or ambiguous question requiring clarification
    Unclear,
}

/// Result of LLM-based intent classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    /// Primary intent category
    pub category: IntentCategory,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// For MULTI: decomposed sub-questions
    pub sub_questions: Option<Vec<String>>,
    /// For UNCLEAR: suggested clarification question
    pub clarification: Option<String>,
    /// Detected entities (packages, services, files mentioned)
    pub entities: Vec<String>,
    /// Detected topic (network, audio, storage, etc.)
    pub topic: Option<String>,
}

/// Deep understanding of a user request - makes Anna think before acting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepUnderstanding {
    /// What Anna thinks the user is asking (paraphrase for verification)
    pub interpreted_as: String,
    /// What information is needed to answer this question
    pub required_info: Vec<String>,
    /// Critical details that are missing from the request
    pub missing_info: Vec<String>,
    /// Alternative valid interpretations of the question
    pub ambiguities: Vec<String>,
    /// Confidence in understanding (0.0 - 1.0)
    pub confidence: f32,
    /// The intent category
    pub category: IntentCategory,
    /// Detected entities (packages, services, files mentioned)
    pub entities: Vec<String>,
    /// Topic area (network, audio, storage, etc.)
    pub topic: Option<String>,
    /// For MULTI: decomposed sub-questions
    pub sub_questions: Option<Vec<String>>,
    /// Suggested clarification if understanding is uncertain
    pub clarification_needed: Option<String>,
    /// Whether Anna should confirm its understanding before proceeding
    pub needs_confirmation: bool,
}

/// RPC methods supported by the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcMethod {
    /// Send a question to be answered (non-streaming, returns full result)
    Ask,
    /// Send a question with streaming response
    AskStreaming,
    /// Get daemon status
    Status,
}

/// Streaming response types (JSON lines format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamingResponse {
    /// A dialogue step (user question, command, output, etc.)
    #[serde(rename = "step")]
    Step { step: DialogueStep },
    /// A token from the LLM (for streaming final answer)
    #[serde(rename = "token")]
    Token { token: String },
    /// Final result with complete answer
    #[serde(rename = "done")]
    Done { result: AskResult },
    /// Error occurred
    #[serde(rename = "error")]
    Error { message: String },
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
    /// If true, the question was unclear and needs clarification
    #[serde(default)]
    pub needs_clarification: bool,
    /// The clarification question to ask the user (when needs_clarification is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clarification_question: Option<String>,
    /// If true, this answer was returned from cache (instant)
    #[serde(default)]
    pub cached: bool,
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
    /// Intent classification being performed
    IntentClassifying,
    /// Intent classification result
    IntentResult,
    /// Searching Arch Wiki
    WikiSearch,
    /// Wiki search results
    WikiResults,
    /// Commands extracted from wiki
    WikiCommands,
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
    /// Clarification question for user
    ClarificationQuestion,
    /// User's clarification response
    ClarificationResponse,
    /// Sub-question being processed (for MULTI intent)
    SubQuestion,
    /// Sub-question result
    SubQuestionResult,
    /// Anna's understanding of the request (paraphrase)
    UnderstandingCheck,
    /// Anna asking for confirmation of understanding
    ConfirmationRequest,
    /// Missing information detected
    MissingInfo,
}
