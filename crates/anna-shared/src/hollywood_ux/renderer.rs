//! Hollywood terminal renderer (v0.0.431).
//!
//! Renders transcripts in cinematic IT department style.

use super::styles::{self, labels};
use super::types::{
    HollywoodTranscript, InternalComm, ProbeResult, ProbeStatus, RenderOptions, TranscriptOutcome,
};
use crate::transcript_segment::SegmentKind;

/// Hollywood renderer for transcripts
pub struct HollywoodRenderer {
    options: RenderOptions,
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

        let handler = transcript
            .handled_by
            .as_ref()
            .map(|h| {
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
                    if probe.status == ProbeStatus::Ok { "0" } else { "1" },
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
                let label = segment.meta.get("label").map(|s| s.as_str()).unwrap_or("json");
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
        output.push_str(&format!(
            "  request_id: {}\n",
            transcript.inner.request_id
        ));
        output.push_str(&format!(
            "  duration: {}ms\n",
            transcript.processing_time_ms
        ));
        output.push_str(&format!("  outcome: {:?}\n", transcript.outcome));

        output
    }

    /// Extract internal comms from transcript
    fn extract_internal_comms(&self, transcript: &HollywoodTranscript) -> Vec<InternalComm> {
        transcript
            .segments()
            .iter()
            .filter(|s| s.kind == SegmentKind::InternalComms)
            .map(|s| InternalComm::from_actor(&s.actor, &s.content, s.relative_secs))
            .collect()
    }

    /// Extract probes from transcript
    fn extract_probes(&self, transcript: &HollywoodTranscript) -> Vec<ProbeResult> {
        transcript
            .segments()
            .iter()
            .filter(|s| s.kind == SegmentKind::ProbeRun)
            .map(|s| {
                let name = s.meta.get("probe_id").map(|s| s.as_str()).unwrap_or("probe");
                let status = s.meta.get("status").map(|s| s.as_str()).unwrap_or("ok");
                let duration: u64 = s
                    .meta
                    .get("duration_ms")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);

                let probe_status = match status {
                    "ok" | "success" => ProbeStatus::Ok,
                    "failed" | "error" => ProbeStatus::Failed,
                    "timeout" => ProbeStatus::Timeout,
                    _ => ProbeStatus::Ok,
                };

                ProbeResult {
                    name: name.to_string(),
                    command: s.meta.get("command").cloned(),
                    status: probe_status,
                    duration_ms: duration,
                    summary: Some(s.content.clone()),
                    raw_output: s.meta.get("raw_output").cloned(),
                }
            })
            .collect()
    }

    /// Find first error message in transcript
    fn find_error_message(&self, transcript: &HollywoodTranscript) -> Option<String> {
        transcript
            .segments()
            .iter()
            .find(|s| s.kind == SegmentKind::Error)
            .map(|s| s.content.clone())
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_transcript() {
        let mut t = HollywoodTranscript::new("REQ-001", "how much free ram?");
        t.set_answer("You have 17.0 GiB free out of 31.0 GiB total (54% available).");
        t.add_evidence("/proc/meminfo");
        t.set_confidence(0.95);
        t.set_handler("Sofia", "Desktop");

        let rendered = render_cinematic(&t);

        assert!(rendered.contains("[you]"));
        assert!(rendered.contains("[anna]"));
        assert!(rendered.contains("17.0 GiB"));
        assert!(rendered.contains("Evidence:"));
        assert!(rendered.contains("95%"));
    }

    #[test]
    fn test_render_with_internal_comms() {
        use crate::transcript_segment::{staff, TranscriptSegment};

        let mut t = HollywoodTranscript::new("REQ-002", "why is my boot slow?");
        t.add(TranscriptSegment::internal_comms(
            staff::sofia(),
            "Checking boot services...",
        ));
        t.add(TranscriptSegment::internal_comms(
            staff::tomas(),
            "Found slow service: NetworkManager",
        ));
        t.set_answer("Your boot is slow due to NetworkManager taking 2.5s.");
        t.add_evidence("systemd-analyze");
        t.set_confidence(0.90);

        let rendered = render_cinematic(&t);

        assert!(rendered.contains("internal comms"));
        assert!(rendered.contains("Sofia"));
        assert!(rendered.contains("Tomas"));
        assert!(rendered.contains("NetworkManager"));
    }

    #[test]
    fn test_format_simple_answer() {
        let output = format_simple_answer(
            "what time is it?",
            "The current time is 14:32.",
            &["system clock"],
            Some(1.0),
        );

        assert!(output.contains("[you]"));
        assert!(output.contains("14:32"));
        assert!(output.contains("Evidence:"));
        assert!(output.contains("100%"));
    }
}
