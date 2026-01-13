//! Simplified RPC types for daemon communication.

use serde::{Deserialize, Serialize};

/// Intent categories for question classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum IntentCategory {
    /// Simple factual question ("what is my kernel version?", "how much RAM?")
    #[default]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    /// Pre-cached commands for known queries (bypasses LLM command selection)
    #[serde(default)]
    pub suggested_commands: Vec<String>,
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
    /// Reset all statistics and learning data
    Reset,
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
    /// Validation warning (v0.0.889) - issues detected during streaming
    #[serde(rename = "validation")]
    Validation { warning: ValidationWarning },
    /// Final result with complete answer
    #[serde(rename = "done")]
    Done { result: AskResult },
    /// Error occurred
    #[serde(rename = "error")]
    Error { message: String },
}

/// Validation warnings detected during streaming (v0.0.889)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Type of validation issue
    pub issue_type: ValidationIssueType,
    /// Description of the issue
    pub message: String,
    /// Severity: low, medium, high
    pub severity: String,
}

/// Types of validation issues that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssueType {
    /// Answer makes claims not supported by command output
    UnsupportedClaim,
    /// Answer uses uncertain language
    Uncertainty,
    /// Answer contradicts command output
    Contradiction,
    /// Answer is too generic/not specific to the system
    TooGeneric,
    /// Answer references data not in command output
    Hallucination,
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

/// v0.3.6: Citation source for answer grounding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Source name (e.g., "Arch Wiki: Pacman")
    pub source: String,
    /// URL to the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Specific section referenced
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
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
    /// v0.3.6: Sources used to ground the answer
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Citation>,
}

/// Result of reset operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetResult {
    /// Items that were cleared
    pub cleared: Vec<String>,
    /// Backup location (if backup was created)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_path: Option<String>,
}

/// v0.3.20: Reset modes per spec
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResetMode {
    /// Reset memory only (experiences, patterns, clusters)
    Memory,
    /// Reset config only (settings back to defaults)
    Config,
    /// Reset model preferences
    Models,
    /// Reset helper packages tracking
    Helpers,
    /// Reset everything (full factory reset)
    #[default]
    Everything,
}

impl ResetMode {
    /// Get description of what this mode resets
    pub fn description(&self) -> &'static str {
        match self {
            ResetMode::Memory => "memory (experiences, patterns, learned behaviors)",
            ResetMode::Config => "configuration (settings back to defaults)",
            ResetMode::Models => "model preferences (will re-detect on next start)",
            ResetMode::Helpers => "helper tracking (does not uninstall packages)",
            ResetMode::Everything => "everything (full factory reset)",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "memory" | "mem" => Some(ResetMode::Memory),
            "config" | "cfg" => Some(ResetMode::Config),
            "models" | "model" => Some(ResetMode::Models),
            "helpers" | "helper" | "deps" => Some(ResetMode::Helpers),
            "everything" | "all" | "full" => Some(ResetMode::Everything),
            _ => None,
        }
    }
}

/// v0.3.21: Parameters for reset command
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResetParams {
    /// Reset mode (defaults to Everything if not specified)
    #[serde(default)]
    pub mode: ResetMode,
    /// Skip backup (dangerous, default false)
    #[serde(default)]
    pub skip_backup: bool,
    /// Dry run - show what would be reset without actually resetting
    #[serde(default)]
    pub dry_run: bool,
}

impl ResetParams {
    /// Create params for a specific mode
    pub fn with_mode(mode: ResetMode) -> Self {
        Self {
            mode,
            skip_backup: false,
            dry_run: false,
        }
    }
}

/// A single step in the dialogue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueStep {
    /// Type of step
    pub step_type: StepType,
    /// Content of this step
    pub content: String,
}

/// LLM error types for context preservation (v0.0.890)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmErrorType {
    /// Request timed out
    Timeout,
    /// Network/connection error
    Network,
    /// Circuit breaker open (too many failures)
    CircuitOpen,
    /// Invalid/malformed response from LLM
    MalformedResponse,
    /// LLM returned empty response
    EmptyResponse,
    /// HTTP error from Ollama API
    HttpError,
    /// Unknown error
    Unknown,
}

/// LLM error context for debugging and learning (v0.0.890)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmErrorContext {
    /// Type of error
    pub error_type: LlmErrorType,
    /// Error message
    pub message: String,
    /// Number of attempts made
    pub attempts: u32,
    /// What the request was trying to do
    pub purpose: String,
    /// Truncated prompt (for debugging)
    pub prompt_preview: Option<String>,
}

impl LlmErrorContext {
    /// Create error context from an error message
    pub fn from_error(error: &str, purpose: &str, attempts: u32, prompt: Option<&str>) -> Self {
        let error_lower = error.to_lowercase();
        let error_type = if error_lower.contains("timeout") || error_lower.contains("timed out") {
            LlmErrorType::Timeout
        } else if error_lower.contains("circuit breaker") {
            LlmErrorType::CircuitOpen
        } else if error_lower.contains("connection") || error_lower.contains("network") {
            LlmErrorType::Network
        } else if error_lower.contains("empty") {
            LlmErrorType::EmptyResponse
        } else if error_lower.contains("http") || error_lower.contains("status") {
            LlmErrorType::HttpError
        } else if error_lower.contains("parse") || error_lower.contains("json") {
            LlmErrorType::MalformedResponse
        } else {
            LlmErrorType::Unknown
        };

        // Truncate prompt preview to first 200 chars
        let prompt_preview = prompt.map(|p| {
            if p.len() > 200 {
                format!("{}...", &p[..200])
            } else {
                p.to_string()
            }
        });

        Self {
            error_type,
            message: error.to_string(),
            attempts,
            purpose: purpose.to_string(),
            prompt_preview,
        }
    }
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
    /// System alert (proactive issue notification)
    SystemAlert,
    /// LLM error with context (v0.0.890)
    LlmError,
    /// v0.0.999: Ticket created (fly-on-the-wall)
    TicketCreated,
    /// v0.0.999: Anna assigns question to specialist
    TeamAssignment,
    /// v0.0.999: Dialogue between Anna and specialist
    TeamDialogue,
    /// v0.0.999: Specialist escalates to senior
    TeamEscalation,
    /// v0.2.9: Anna dispatches question to specialist
    TeamDispatch,
    /// v0.2.9: Specialist acknowledges and works on question
    SpecialistWorking,
    /// v0.3.29: Investigation mode started with hypothesis
    InvestigationStart,
    /// v0.3.29: Investigation hypothesis being tested
    InvestigationHypothesis,
    /// v0.3.29: Investigation probe being run
    InvestigationProbe,
    /// v0.3.29: Investigation probe result
    InvestigationResult,
    /// v0.3.29: Investigation complete with summary
    InvestigationComplete,
    /// v0.3.29: Experiment started
    ExperimentStart,
    /// v0.3.29: Experiment result
    ExperimentResult,
}
