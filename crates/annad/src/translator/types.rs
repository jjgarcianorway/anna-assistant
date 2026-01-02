//! Type definitions for the translator module.

use anna_shared::rpc::TranslatorTicket;
use serde::{Deserialize, Serialize};

/// v0.0.318: Translator result with debug info for LLM call visibility
#[derive(Debug, Clone)]
pub struct TranslatorResult {
    /// The parsed ticket
    pub ticket: TranslatorTicket,
    /// The full prompt sent to the LLM
    pub prompt: String,
    /// The raw response from the LLM
    pub response: String,
    /// Duration of the LLM call in milliseconds
    pub duration_ms: u64,
}

/// Internal JSON structure for LLM output parsing (tolerant of missing fields)
#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct TranslatorOutput {
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub entities: Option<Vec<String>>,
    #[serde(default)]
    pub needs_probes: Option<Vec<String>>,
    #[serde(default)]
    pub clarification_question: Option<String>,
    #[serde(default)]
    pub confidence: Option<f32>,
}

/// Minimal translator input - keeps payload small for fast inference
#[derive(Debug, Clone)]
pub struct TranslatorInput {
    pub query: String,
    pub hw_summary: String, // one line: "CPU cores: 8, RAM: 16GB, GPU: none"
}

impl TranslatorInput {
    /// Create minimal input for translator
    pub fn new(query: &str, cpu_cores: u32, ram_gb: f64, has_gpu: bool) -> Self {
        let gpu_str = if has_gpu { "yes" } else { "none" };
        let hw_summary = format!(
            "CPU cores: {}, RAM: {:.0}GB, GPU: {}",
            cpu_cores, ram_gb, gpu_str
        );
        Self {
            query: query.to_string(),
            hw_summary,
        }
    }
}

/// Maximum allowed translator payload size (8KB)
#[allow(dead_code)]
pub const MAX_TRANSLATOR_PAYLOAD_SIZE: usize = 8192;
