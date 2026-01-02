//! Core types for trace blocks.
//!
//! Basic enums and structs used throughout the trace system.

use serde::{Deserialize, Serialize};

/// Route type for how the request was handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteType {
    /// Handled by deterministic probes only (no LLM)
    Deterministic,
    /// Handled by LLM specialist
    LlmSpecialist,
    /// Fell back to generic LLM response
    LlmFallback,
    /// Needed clarification from user
    Clarification,
}

impl std::fmt::Display for RouteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deterministic => write!(f, "deterministic"),
            Self::LlmSpecialist => write!(f, "llm_specialist"),
            Self::LlmFallback => write!(f, "llm_fallback"),
            Self::Clarification => write!(f, "clarification"),
        }
    }
}

/// Outcome of a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TraceOutcome {
    Success,
    FailedNoEvidence,
    FailedTimeout,
    FailedParse,
    FailedLowConfidence,
    FailedAmbiguousQuery,
    FailedContractViolation,
    FailedNoClaims,
    FailedGenericAnswer,
    FailedProbes,
}

impl std::fmt::Display for TraceOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "SUCCESS"),
            Self::FailedNoEvidence => write!(f, "FAILED_NO_EVIDENCE"),
            Self::FailedTimeout => write!(f, "FAILED_TIMEOUT"),
            Self::FailedParse => write!(f, "FAILED_PARSE"),
            Self::FailedLowConfidence => write!(f, "FAILED_LOW_CONFIDENCE"),
            Self::FailedAmbiguousQuery => write!(f, "FAILED_AMBIGUOUS_QUERY"),
            Self::FailedContractViolation => write!(f, "FAILED_CONTRACT_VIOLATION"),
            Self::FailedNoClaims => write!(f, "FAILED_NO_CLAIMS"),
            Self::FailedGenericAnswer => write!(f, "FAILED_GENERIC_ANSWER"),
            Self::FailedProbes => write!(f, "FAILED_PROBES"),
        }
    }
}

/// Timing breakdown.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimingTrace {
    pub translator_ms: u64,
    pub probes_ms: u64,
    pub specialist_ms: u64,
    pub gate_ms: u64,
    pub total_ms: u64,
}

/// Timeout information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutInfo {
    /// Which stage timed out
    pub stage: String,
    /// Configured timeout (ms)
    pub timeout_ms: u64,
    /// Elapsed time when timeout occurred
    pub elapsed_ms: u64,
    /// Partial output captured (level 3)
    pub partial_output: Option<String>,
}

/// Failure detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetail {
    /// Which check failed
    pub check: String,
    /// Why it failed
    pub reason: String,
    /// Additional context
    pub context: Option<String>,
}

/// Reliability gate result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Pass or fail
    pub passed: bool,
    /// Individual check results
    pub checks: Vec<GateCheck>,
}

/// Individual gate check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheck {
    pub name: String,
    pub passed: bool,
    pub details: Option<String>,
}
