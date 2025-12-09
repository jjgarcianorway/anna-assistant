//! Answer validation functions (v0.0.209).

use serde::{Deserialize, Serialize};

use super::contract::AnswerContract;
use super::types::{RequestedField, Verbosity};

/// Validation result for an answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerValidation {
    /// Whether the answer is valid
    pub valid: bool,
    /// Fields that were requested but missing
    pub missing_fields: Vec<RequestedField>,
    /// Fields that were included but not requested (only in minimal mode)
    pub extra_fields: Vec<String>,
    /// Suggested trimmed answer (if trimming is possible)
    pub trimmed_answer: Option<String>,
}

/// Validate and optionally trim an answer against a contract
/// Returns validation result with trimming suggestions
pub fn validate_answer(answer: &str, contract: &AnswerContract) -> AnswerValidation {
    let answer_lower = answer.to_lowercase();

    // Check for missing requested fields
    let mut missing_fields = Vec::new();
    for field in &contract.requested_fields {
        if !field_present_in_answer(&answer_lower, field) {
            missing_fields.push(field.clone());
        }
    }

    // In minimal mode, check for extra fields
    let mut extra_fields = Vec::new();
    if contract.verbosity == Verbosity::Minimal && !contract.teaching_mode {
        // Detect common extra info patterns
        if answer_lower.contains("model") && !contract.allows_field(&RequestedField::CpuModel) {
            extra_fields.push("cpu_model".to_string());
        }
        if answer_lower.contains("total") && !contract.allows_field(&RequestedField::RamTotal) {
            extra_fields.push("ram_total".to_string());
        }
    }

    let valid = missing_fields.is_empty()
        && (contract.verbosity != Verbosity::Minimal || extra_fields.is_empty());

    AnswerValidation {
        valid,
        missing_fields,
        extra_fields,
        trimmed_answer: None, // Trimming is complex, handled separately
    }
}

/// Check if a field's information is present in the answer
pub fn field_present_in_answer(answer: &str, field: &RequestedField) -> bool {
    match field {
        RequestedField::CpuCores => {
            answer.contains("core")
                || answer.contains("thread")
                || answer.chars().any(|c| c.is_ascii_digit())
        }
        RequestedField::CpuModel => {
            answer.contains("intel") || answer.contains("amd") || answer.contains("cpu")
        }
        RequestedField::CpuTemp => answer.contains("°") || answer.contains("temp"),
        RequestedField::RamFree => answer.contains("free") || answer.contains("available"),
        RequestedField::RamTotal => {
            answer.contains("total") || answer.contains("gb") || answer.contains("mb")
        }
        RequestedField::RamUsed => answer.contains("used"),
        RequestedField::DiskUsage(_) => answer.contains("%") || answer.contains("used"),
        RequestedField::DiskFree(_) => answer.contains("free") || answer.contains("available"),
        RequestedField::SoundCard => answer.contains("audio") || answer.contains("sound"),
        RequestedField::GpuInfo => {
            answer.contains("gpu")
                || answer.contains("graphics")
                || answer.contains("nvidia")
                || answer.contains("amd")
        }
        RequestedField::NetworkInterfaces => {
            answer.contains("eth") || answer.contains("wlan") || answer.contains("interface")
        }
        RequestedField::ServiceStatus(_) => {
            answer.contains("running") || answer.contains("stopped") || answer.contains("active")
        }
        RequestedField::ProcessList => answer.contains("process") || answer.contains("pid"),
        RequestedField::PackageCount => {
            answer.chars().any(|c| c.is_ascii_digit()) && answer.contains("package")
        }
        RequestedField::ToolExists(_) => {
            answer.contains("installed") || answer.contains("found") || answer.contains("not found")
        }
        RequestedField::Generic => true, // Generic always passes
    }
}
