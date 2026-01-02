//! Section rendering methods for the Hollywood renderer.

use super::super::styles::{self, labels};
use super::super::types::{HollywoodTranscript, InternalComm, ProbeResult, ProbeStatus, TranscriptOutcome};
use super::core::HollywoodRenderer;
use super::extractors::RendererExtractors;
use crate::transcript_segment::SegmentKind;

/// Trait for section rendering methods on HollywoodRenderer
pub(super) trait RendererSections {
    fn render_internal_comms(&self, comms: &[InternalComm]) -> String;
    fn render_probes(&self, probes: &[ProbeResult]) -> String;
    fn render_answer(&self, transcript: &HollywoodTranscript) -> String;
    fn render_footer(&self, transcript: &HollywoodTranscript) -> String;
    fn render_debug_section(&self, transcript: &HollywoodTranscript) -> String;
}

impl RendererSections for HollywoodRenderer {
    /// Render internal comms section
    fn render_internal_comms(&self, comms: &[InternalComm]) -> String {
        let mut output = String::new();
        output.push_str(&styles::section_header(labels::INTERNAL));

        for comm in comms {
            output.push_str(&styles::internal_comm_line(
                comm.relative_secs,
                &comm.staff_display(),
                &comm.message,
                self.options.show_timestamps,
            ));
            output.push('\n');
        }

        output
    }

    /// Render probes section
    fn render_probes(&self, probes: &[ProbeResult]) -> String {
        let mut output = String::new();
        output.push_str(&format!("\n{}\n", labels::PROBES));

        for probe in probes {
            output.push_str(&styles::probe_line(
                &probe.name,
                probe.status.display(),
                probe.duration_ms,
            ));
            output.push('\n');
        }

        output
    }

    /// Render answer section
    fn render_answer(&self, transcript: &HollywoodTranscript) -> String {
        let mut output = String::new();
        output.push_str(&format!("\n{}\n", labels::ANNA));

        match transcript.outcome {
            TranscriptOutcome::Success | TranscriptOutcome::Partial => {
                if let Some(ref answer) = transcript.final_answer {
                    output.push_str(answer);
                    output.push('\n');
                } else {
                    output.push_str("No response available.\n");
                }
            }
            TranscriptOutcome::Failed => {
                output.push_str("I wasn't able to complete this request.\n");
                if let Some(error) = self.find_error_message(transcript) {
                    output.push_str(&format!("\nReason: {}\n", error));
                }
            }
            TranscriptOutcome::ParseError => {
                output.push_str(
                    "Something went wrong interpreting the technician's report for this request.\n",
                );
                output.push_str(
                    "I did collect system data, but I couldn't turn it into a reliable answer.\n",
                );
            }
            TranscriptOutcome::Cancelled => {
                output.push_str("Request was cancelled.\n");
            }
        }

        output
    }

    /// Render status footer
    fn render_footer(&self, transcript: &HollywoodTranscript) -> String {
        let status = match transcript.outcome {
            TranscriptOutcome::Success => "System Status",
            TranscriptOutcome::Partial => "Partial Answer",
            TranscriptOutcome::Failed => "Request Failed",
            TranscriptOutcome::ParseError => "Parse Error",
            TranscriptOutcome::Cancelled => "Cancelled",
        };

        let handler = transcript.handled_by.as_ref().map(|h| {
            if let Some(ref dept) = transcript.department {
                format!("{} ({})", h, dept)
            } else {
                h.clone()
            }
        });

        styles::status_footer(
            status,
            transcript.confidence,
            handler.as_deref(),
            !transcript.evidence_sources.is_empty(),
        )
    }

    /// Render debug section
    fn render_debug_section(&self, transcript: &HollywoodTranscript) -> String {
        let mut output = String::new();
        output.push_str(&format!("\n{}\n", labels::DEBUG_SECTION));

        // Raw probes
        let probes = self.extract_probes(transcript);
        if !probes.is_empty() {
            output.push_str("\n[probes raw]\n");
            for probe in probes {
                let cmd = probe.command.as_deref().unwrap_or("-");
                output.push_str(&format!(
                    "  {:30} -> exit={} ({}ms)\n",
                    styles::truncate(&probe.name, 30),
                    if probe.status == ProbeStatus::Ok {
                        "0"
                    } else {
                        "1"
                    },
                    probe.duration_ms
                ));
                if !cmd.is_empty() && cmd != "-" {
                    output.push_str(&format!("    cmd: {}\n", cmd));
                }
                if let Some(ref raw) = probe.raw_output {
                    let truncated = styles::truncate(raw.lines().next().unwrap_or(""), 60);
                    output.push_str(&format!("    out: {}\n", truncated));
                }
            }
        }

        // Evidence
        if !transcript.evidence_sources.is_empty() {
            output.push_str("\n[evidence]\n");
            for source in &transcript.evidence_sources {
                output.push_str(&format!("  {}\n", source));
            }
        }

        // Debug JSON segments
        for segment in transcript.segments() {
            if segment.kind == SegmentKind::DebugJson {
                let label = segment
                    .meta
                    .get("label")
                    .map(|s| s.as_str())
                    .unwrap_or("json");
                output.push_str(&format!("\n[debug] {}:\n", label));
                // Truncate large JSON
                let content = if segment.content.len() > 500 {
                    format!("{}... (truncated)", &segment.content[..500])
                } else {
                    segment.content.clone()
                };
                output.push_str(&styles::indent(&content, 2));
                output.push('\n');
            }
        }

        // Processing info
        output.push_str("\n[processing]\n");
        output.push_str(&format!("  request_id: {}\n", transcript.inner.request_id));
        output.push_str(&format!(
            "  duration: {}ms\n",
            transcript.processing_time_ms
        ));
        output.push_str(&format!("  outcome: {:?}\n", transcript.outcome));

        output
    }
}
