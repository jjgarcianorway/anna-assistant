//! RPC type definitions for daemon communication.

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
    /// Phase 26: True if answer was abstained due to low confidence
    #[serde(default)]
    pub abstained: bool,
    /// Phase 26: Final confidence value (for outcome recording)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_confidence: Option<f32>,
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

/// A single step in the dialogue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueStep {
    /// Type of step
    pub step_type: StepType,
    /// Content of this step
    pub content: String,
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

/// Types of dialogue steps
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// v0.3.55: Phase 22 heartbeat for long operations
    Heartbeat,
    /// v0.3.57: Phase 24 policy decision basis (Debug only)
    PolicyBasis,
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
