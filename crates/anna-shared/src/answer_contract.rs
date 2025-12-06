//! Answer contract - enforces that answers contain only what was asked.
//!
//! v0.0.74: Implements answer shaping to prevent over-sharing facts.
//!
//! # Design
//! - Translator output includes `requested_fields` and `verbosity`
//! - Final answers are validated against the contract
//! - Extra facts are trimmed unless teaching mode is enabled

use serde::{Deserialize, Serialize};

/// Verbosity level for answers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verbosity {
    /// Minimal: only the exact answer requested
    Minimal,
    /// Normal: answer with brief context
    #[default]
    Normal,
    /// Teach: explain reasoning and provide educational context
    Teach,
}

impl std::fmt::Display for Verbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verbosity::Minimal => write!(f, "minimal"),
            Verbosity::Normal => write!(f, "normal"),
            Verbosity::Teach => write!(f, "teach"),
        }
    }
}

/// Requested field type - what the user asked for
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestedField {
    /// CPU core count only
    CpuCores,
    /// CPU model name only
    CpuModel,
    /// CPU temperature
    CpuTemp,
    /// Total RAM
    RamTotal,
    /// Free/available RAM
    RamFree,
    /// Used RAM
    RamUsed,
    /// Disk usage (specific mount or all)
    DiskUsage(Option<String>),
    /// Disk free space
    DiskFree(Option<String>),
    /// Sound card / audio device
    SoundCard,
    /// GPU info
    GpuInfo,
    /// Network interfaces
    NetworkInterfaces,
    /// Service status (specific service)
    ServiceStatus(String),
    /// Process list (top by resource)
    ProcessList,
    /// Package count
    PackageCount,
    /// Tool existence check
    ToolExists(String),
    /// Generic query (needs full answer)
    Generic,
}

impl std::fmt::Display for RequestedField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestedField::CpuCores => write!(f, "cpu_cores"),
            RequestedField::CpuModel => write!(f, "cpu_model"),
            RequestedField::CpuTemp => write!(f, "cpu_temp"),
            RequestedField::RamTotal => write!(f, "ram_total"),
            RequestedField::RamFree => write!(f, "ram_free"),
            RequestedField::RamUsed => write!(f, "ram_used"),
            RequestedField::DiskUsage(m) => {
                if let Some(mount) = m {
                    write!(f, "disk_usage:{}", mount)
                } else {
                    write!(f, "disk_usage")
                }
            }
            RequestedField::DiskFree(m) => {
                if let Some(mount) = m {
                    write!(f, "disk_free:{}", mount)
                } else {
                    write!(f, "disk_free")
                }
            }
            RequestedField::SoundCard => write!(f, "sound_card"),
            RequestedField::GpuInfo => write!(f, "gpu_info"),
            RequestedField::NetworkInterfaces => write!(f, "network_interfaces"),
            RequestedField::ServiceStatus(s) => write!(f, "service_status:{}", s),
            RequestedField::ProcessList => write!(f, "process_list"),
            RequestedField::PackageCount => write!(f, "package_count"),
            RequestedField::ToolExists(t) => write!(f, "tool_exists:{}", t),
            RequestedField::Generic => write!(f, "generic"),
        }
    }
}

/// Answer contract - defines what the answer should contain
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnswerContract {
    /// Fields explicitly requested by the user
    pub requested_fields: Vec<RequestedField>,
    /// Verbosity level
    pub verbosity: Verbosity,
    /// Whether teaching mode is enabled (allows extra context)
    pub teaching_mode: bool,
    /// Original query for reference
    pub original_query: String,
}

impl AnswerContract {
    /// Create a new contract from query analysis
    pub fn from_query(query: &str) -> Self {
        let query_lower = query.to_lowercase();
        let mut fields = Vec::new();
        let mut verbosity = Verbosity::Normal;

        // Parse requested fields from query
        if query_lower.contains("how many cores") || query_lower.contains("core count") {
            fields.push(RequestedField::CpuCores);
        }
        if query_lower.contains("cpu model") || query_lower.contains("processor name") {
            fields.push(RequestedField::CpuModel);
        }
        if query_lower.contains("cpu temp") || query_lower.contains("temperature") {
            fields.push(RequestedField::CpuTemp);
        }
        if query_lower.contains("free ram") || query_lower.contains("available memory") {
            fields.push(RequestedField::RamFree);
        }
        if query_lower.contains("total ram") || query_lower.contains("how much ram") {
            fields.push(RequestedField::RamTotal);
        }
        if query_lower.contains("ram used") || query_lower.contains("memory used") {
            fields.push(RequestedField::RamUsed);
        }
        if query_lower.contains("disk usage") || query_lower.contains("disk space") {
            fields.push(RequestedField::DiskUsage(None));
        }
        if query_lower.contains("disk free") || query_lower.contains("free space") {
            fields.push(RequestedField::DiskFree(None));
        }
        if query_lower.contains("sound card") || query_lower.contains("audio") {
            fields.push(RequestedField::SoundCard);
        }
        if query_lower.contains("gpu") || query_lower.contains("graphics") {
            fields.push(RequestedField::GpuInfo);
        }
        if query_lower.contains("network") || query_lower.contains("ip address") {
            fields.push(RequestedField::NetworkInterfaces);
        }
        if query_lower.contains("packages") && query_lower.contains("how many") {
            fields.push(RequestedField::PackageCount);
        }

        // Detect verbosity hints
        if query_lower.contains("just") || query_lower.contains("only") || query_lower.contains("exactly") {
            verbosity = Verbosity::Minimal;
        }
        if query_lower.contains("explain") || query_lower.contains("teach") || query_lower.contains("why") {
            verbosity = Verbosity::Teach;
        }

        // Default to generic if no specific fields detected
        if fields.is_empty() {
            fields.push(RequestedField::Generic);
        }

        Self {
            requested_fields: fields,
            verbosity,
            teaching_mode: verbosity == Verbosity::Teach,
            original_query: query.to_string(),
        }
    }

    /// Check if a field is allowed in the answer
    pub fn allows_field(&self, field: &RequestedField) -> bool {
        // Teaching mode allows everything
        if self.teaching_mode {
            return true;
        }

        // Generic allows everything
        if self.requested_fields.contains(&RequestedField::Generic) {
            return true;
        }

        // Check if specifically requested
        self.requested_fields.contains(field)
    }

    /// Check if extra context is allowed
    pub fn allows_extra_context(&self) -> bool {
        self.teaching_mode || self.verbosity != Verbosity::Minimal
    }
}

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

    let valid = missing_fields.is_empty() && (contract.verbosity != Verbosity::Minimal || extra_fields.is_empty());

    AnswerValidation {
        valid,
        missing_fields,
        extra_fields,
        trimmed_answer: None, // Trimming is complex, handled separately
    }
}

/// Check if a field's information is present in the answer
fn field_present_in_answer(answer: &str, field: &RequestedField) -> bool {
    match field {
        RequestedField::CpuCores => {
            answer.contains("core") || answer.contains("thread") ||
            answer.chars().any(|c| c.is_ascii_digit())
        }
        RequestedField::CpuModel => answer.contains("intel") || answer.contains("amd") || answer.contains("cpu"),
        RequestedField::CpuTemp => answer.contains("°") || answer.contains("temp"),
        RequestedField::RamFree => answer.contains("free") || answer.contains("available"),
        RequestedField::RamTotal => answer.contains("total") || answer.contains("gb") || answer.contains("mb"),
        RequestedField::RamUsed => answer.contains("used"),
        RequestedField::DiskUsage(_) => answer.contains("%") || answer.contains("used"),
        RequestedField::DiskFree(_) => answer.contains("free") || answer.contains("available"),
        RequestedField::SoundCard => answer.contains("audio") || answer.contains("sound"),
        RequestedField::GpuInfo => answer.contains("gpu") || answer.contains("graphics") || answer.contains("nvidia") || answer.contains("amd"),
        RequestedField::NetworkInterfaces => answer.contains("eth") || answer.contains("wlan") || answer.contains("interface"),
        RequestedField::ServiceStatus(_) => answer.contains("running") || answer.contains("stopped") || answer.contains("active"),
        RequestedField::ProcessList => answer.contains("process") || answer.contains("pid"),
        RequestedField::PackageCount => answer.chars().any(|c| c.is_ascii_digit()) && answer.contains("package"),
        RequestedField::ToolExists(_) => answer.contains("installed") || answer.contains("found") || answer.contains("not found"),
        RequestedField::Generic => true, // Generic always passes
    }
}

/// Trim answer to only include requested fields (best effort)
/// Returns None if trimming is not possible
pub fn trim_answer(answer: &str, contract: &AnswerContract) -> Option<String> {
    // Don't trim in teaching mode or if generic
    if contract.teaching_mode || contract.requested_fields.contains(&RequestedField::Generic) {
        return Some(answer.to_string());
    }

    // For minimal mode with single specific field, try to extract just that value
    if contract.verbosity == Verbosity::Minimal && contract.requested_fields.len() == 1 {
        match &contract.requested_fields[0] {
            RequestedField::CpuCores => extract_number_with_context(answer, &["core", "thread"]),
            RequestedField::RamFree => extract_size_with_context(answer, &["free", "available"]),
            RequestedField::RamTotal => extract_size_with_context(answer, &["total"]),
            _ => Some(answer.to_string()),
        }
    } else {
        Some(answer.to_string())
    }
}

/// Extract a number with context keywords
fn extract_number_with_context(text: &str, contexts: &[&str]) -> Option<String> {
    let text_lower = text.to_lowercase();

    for context in contexts {
        if let Some(pos) = text_lower.find(context) {
            // Look for number before or after the context word
            let before = &text[..pos];
            let after = &text[pos..];

            // Try to extract number from nearby
            for word in before.split_whitespace().rev().take(3) {
                if let Ok(n) = word.trim_matches(|c: char| !c.is_ascii_digit()).parse::<u32>() {
                    return Some(format!("{} {}", n, context));
                }
            }
            for word in after.split_whitespace().take(5) {
                if let Ok(n) = word.trim_matches(|c: char| !c.is_ascii_digit()).parse::<u32>() {
                    return Some(format!("{} {}", n, context));
                }
            }
        }
    }

    None
}

/// Extract a size value (like "8 GB") with context keywords
fn extract_size_with_context(text: &str, contexts: &[&str]) -> Option<String> {
    let text_lower = text.to_lowercase();

    for context in contexts {
        if text_lower.contains(context) {
            // Look for size patterns like "8 GB", "1.5GB", "512 MB"
            for word in text.split_whitespace() {
                let w = word.to_uppercase();
                if w.ends_with("GB") || w.ends_with("MB") || w.ends_with("TB") {
                    return Some(word.to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_from_query_cores() {
        let contract = AnswerContract::from_query("how many cores does my cpu have?");
        assert!(contract.requested_fields.contains(&RequestedField::CpuCores));
        assert!(!contract.requested_fields.contains(&RequestedField::CpuModel));
    }

    #[test]
    fn test_contract_from_query_free_ram() {
        let contract = AnswerContract::from_query("how much free ram do I have?");
        assert!(contract.requested_fields.contains(&RequestedField::RamFree));
    }

    #[test]
    fn test_contract_minimal_verbosity() {
        let contract = AnswerContract::from_query("just tell me how many cores");
        assert_eq!(contract.verbosity, Verbosity::Minimal);
    }

    #[test]
    fn test_contract_teach_verbosity() {
        let contract = AnswerContract::from_query("explain how many cores I have");
        assert_eq!(contract.verbosity, Verbosity::Teach);
        assert!(contract.teaching_mode);
    }

    #[test]
    fn test_validate_answer_with_required_field() {
        let contract = AnswerContract::from_query("how many cores?");
        let validation = validate_answer("You have 8 cores and 16 threads.", &contract);
        assert!(validation.valid);
        assert!(validation.missing_fields.is_empty());
    }

    #[test]
    fn test_validate_answer_missing_field() {
        let contract = AnswerContract::from_query("what is my cpu temperature?");
        let validation = validate_answer("You have an Intel processor.", &contract);
        assert!(!validation.valid);
        assert!(!validation.missing_fields.is_empty());
    }

    #[test]
    fn test_trim_answer_cores() {
        let contract = AnswerContract {
            requested_fields: vec![RequestedField::CpuCores],
            verbosity: Verbosity::Minimal,
            teaching_mode: false,
            original_query: "just cores".to_string(),
        };
        let trimmed = trim_answer("Your CPU has 8 cores and is an Intel i7.", &contract);
        assert!(trimmed.is_some());
        assert!(trimmed.unwrap().contains("8"));
    }
}
