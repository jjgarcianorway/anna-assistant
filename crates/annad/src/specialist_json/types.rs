//! Types for JSON specialist handling

use anna_shared::specialist_contract::SpecialistResponse;
use crate::response_renderer::RenderedResponse;

/// Result of JSON specialist processing
#[derive(Debug, Clone)]
pub struct JsonSpecialistResult {
    /// The parsed specialist response (if successful)
    pub response: Option<SpecialistResponse>,
    /// Rendered output with personality
    pub rendered: RenderedResponse,
    /// Whether LLM was called or deterministic path used
    pub used_llm: bool,
    /// Raw LLM output (for debugging)
    pub raw_output: Option<String>,
}
