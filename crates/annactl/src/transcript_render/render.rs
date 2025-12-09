//! Main render functions (v0.0.179).

use anna_shared::rpc::ServiceDeskResult;

use crate::output::OutputMode;
use crate::theatre_render;

use super::debug_render::render_debug;

/// v0.0.144: Simple render - always theatre mode with internal comms
pub fn render(result: &ServiceDeskResult) {
    // Always use theatre renderer (cinematic experience) with internal comms visible
    theatre_render::render_theatre(result, true);
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
