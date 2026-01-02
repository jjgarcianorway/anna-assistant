//! Core Hollywood renderer implementation.

use super::super::styles::{self, labels};
use super::super::types::{HollywoodTranscript, RenderOptions};
use super::extractors::RendererExtractors;
use super::sections::RendererSections;

/// Hollywood renderer for transcripts
pub struct HollywoodRenderer {
    pub(super) options: RenderOptions,
}

impl HollywoodRenderer {
    /// Create new renderer with options
    pub fn new(options: RenderOptions) -> Self {
        Self { options }
    }

    /// Create cinematic renderer
    pub fn cinematic() -> Self {
        Self::new(RenderOptions::cinematic())
    }

    /// Create debug renderer
    pub fn debug() -> Self {
        Self::new(RenderOptions::debug())
    }

    /// Render complete transcript to string
    pub fn render(&self, transcript: &HollywoodTranscript) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&self.render_header(&transcript.user_query));
        output.push('\n');

        // Internal comms section
        if self.options.show_internal_comms {
            let comms = self.extract_internal_comms(transcript);
            if !comms.is_empty() {
                output.push_str(&self.render_internal_comms(&comms));
            }
        }

        // Probes section
        if self.options.show_probes {
            let probes = self.extract_probes(transcript);
            if !probes.is_empty() {
                output.push_str(&self.render_probes(&probes));
            }
        }

        // Answer section
        output.push_str(&self.render_answer(transcript));

        // Evidence footer
        if self.options.show_evidence && !transcript.evidence_sources.is_empty() {
            output.push_str(&styles::evidence_footer(&transcript.evidence_sources));
            output.push('\n');
        }

        // Status footer
        if self.options.show_footer {
            output.push('\n');
            output.push_str(&self.render_footer(transcript));
            output.push('\n');
        }

        // Debug section (only in debug mode)
        if self.options.is_debug() {
            output.push_str(&self.render_debug_section(transcript));
        }

        output
    }

    /// Render header block
    fn render_header(&self, query: &str) -> String {
        styles::header_block(query, self.options.width)
    }
}
