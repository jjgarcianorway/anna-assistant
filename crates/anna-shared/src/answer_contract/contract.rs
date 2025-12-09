//! AnswerContract struct and methods (v0.0.209).

use serde::{Deserialize, Serialize};

use super::types::{RequestedField, Verbosity};

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
        if query_lower.contains("just")
            || query_lower.contains("only")
            || query_lower.contains("exactly")
        {
            verbosity = Verbosity::Minimal;
        }
        if query_lower.contains("explain")
            || query_lower.contains("teach")
            || query_lower.contains("why")
        {
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
