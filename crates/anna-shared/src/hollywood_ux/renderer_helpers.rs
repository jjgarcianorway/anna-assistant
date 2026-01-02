//! Helper functions for rendering transcripts in various formats (v0.0.431).

use super::renderer::HollywoodRenderer;
use super::styles::{self, labels};
use super::types::HollywoodTranscript;

/// Render a transcript with default cinematic options
pub fn render_cinematic(transcript: &HollywoodTranscript) -> String {
    HollywoodRenderer::cinematic().render(transcript)
}

/// Render a transcript with debug options
pub fn render_debug(transcript: &HollywoodTranscript) -> String {
    HollywoodRenderer::debug().render(transcript)
}

/// Format a simple answer without full transcript
pub fn format_simple_answer(
    query: &str,
    answer: &str,
    evidence: &[&str],
    confidence: Option<f32>,
) -> String {
    let width = super::DEFAULT_WIDTH;
    let mut output = String::new();

    output.push_str(&styles::header_block(query, width));
    output.push_str(&format!("\n\n{}\n{}\n", labels::ANNA, answer));

    if !evidence.is_empty() {
        let sources: Vec<String> = evidence.iter().map(|s| s.to_string()).collect();
        output.push_str(&styles::evidence_footer(&sources));
        output.push('\n');
    }

    if let Some(conf) = confidence {
        output.push('\n');
        output.push_str(&styles::status_footer(
            "System Status",
            Some(conf),
            None,
            !evidence.is_empty(),
        ));
        output.push('\n');
    }

    output
}

/// Format error response
pub fn format_error_response(query: &str, error: &str, collected_data: &[&str]) -> String {
    let width = super::DEFAULT_WIDTH;
    let mut output = String::new();

    output.push_str(&styles::header_block(query, width));
    output.push_str(&format!("\n\n{}\n{}\n", labels::ANNA, error));

    if !collected_data.is_empty() {
        output.push_str("\nWhat I collected:\n");
        for data in collected_data {
            output.push_str(&styles::bullet(data));
            output.push('\n');
        }
    }

    output
}
