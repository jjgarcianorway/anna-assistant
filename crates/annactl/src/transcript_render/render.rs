//! Main render functions (v0.0.179).
//! v0.0.305: Auto-detect debug mode from LLM call presence in transcript.

use anna_shared::rpc::ServiceDeskResult;
use anna_shared::transcript::TranscriptEventKind;

use crate::output::OutputMode;
use crate::theatre_render;

use super::debug_render::render_debug;

/// v0.0.144: Simple render - always theatre mode with internal comms
/// v0.0.305: Detect debug mode from transcript LLM calls
pub fn render(result: &ServiceDeskResult) {
    // v0.0.305: Check if debug mode was enabled by looking for LLM calls in transcript
    let has_llm_calls = result
        .transcript
        .events
        .iter()
        .any(|e| matches!(e.kind, TranscriptEventKind::LlmCall { .. }));

    if has_llm_calls {
        // Debug mode was enabled - show detailed LLM call view
        let output_mode = OutputMode::detect();
        render_debug(result, output_mode);
    } else {
        // Normal theatre mode (cinematic experience) with internal comms visible
        theatre_render::render_theatre(result, true);
    }
}

/// Render with explicit internal communications option (kept for compatibility)
#[allow(dead_code)]
pub fn render_with_options(result: &ServiceDeskResult, debug_mode: bool, show_internal: bool) {
    if debug_mode {
        let output_mode = OutputMode::detect();
        render_debug(result, output_mode);
    } else {
        // v0.0.81: Use theatre renderer for cinematic experience
        theatre_render::render_theatre(result, show_internal);
    }
}
