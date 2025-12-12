//! Streaming renderer with spinner (v0.0.431).
//!
//! Live terminal updates for long-running operations.

use super::styles::{self, labels, spinner};
use super::types::{HollywoodTranscript, InternalComm, RenderOptions};
use crate::transcript_segment::{SegmentKind, TranscriptSegment};
use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Streaming state for live updates
pub struct StreamingRenderer {
    /// Render options
    pub(crate) options: RenderOptions,
    /// Number of segments already rendered
    pub(crate) rendered_count: usize,
    /// Internal comms section started
    pub(crate) internal_section_started: bool,
    /// Probes buffer (for grouped display)
    pub(crate) probe_buffer: Vec<String>,
    /// Spinner state
    pub(crate) spinner: SpinnerState,
    /// Header already printed
    pub(crate) header_printed: bool,
    /// Start time
    pub(crate) started_at: Instant,
}

/// Spinner animation state
pub struct SpinnerState {
    /// Current frame index
    pub(crate) frame: usize,
    /// Whether spinner is active
    pub(crate) active: bool,
    /// Last tick time
    pub(crate) last_tick: Instant,
    /// Current message
    pub(crate) message: String,
}

impl Default for SpinnerState {
    fn default() -> Self {
        Self {
            frame: 0,
            active: false,
            last_tick: Instant::now(),
            message: String::new(),
        }
    }
}

impl SpinnerState {
    /// Advance spinner frame if enough time has passed
    pub fn tick(&mut self) -> bool {
        if !self.active {
            return false;
        }

        let elapsed = self.last_tick.elapsed();
        if elapsed >= Duration::from_millis(spinner::INTERVAL_MS) {
            self.frame = (self.frame + 1) % spinner::FRAMES.len();
            self.last_tick = Instant::now();
            true
        } else {
            false
        }
    }

    /// Get current frame character
    pub fn frame_char(&self) -> &'static str {
        spinner::FRAMES[self.frame % spinner::FRAMES.len()]
    }
}

impl StreamingRenderer {
    /// Create new streaming renderer
    pub fn new(options: RenderOptions) -> Self {
        Self {
            options,
            rendered_count: 0,
            internal_section_started: false,
            probe_buffer: Vec::new(),
            spinner: SpinnerState::default(),
            header_printed: false,
            started_at: Instant::now(),
        }
    }

    /// Create with default cinematic options
    pub fn cinematic() -> Self {
        Self::new(RenderOptions::cinematic())
    }

    /// Render user input header
    pub fn render_header(&mut self, query: &str) {
        if self.header_printed {
            return;
        }
        println!("{}", styles::header_block(query, self.options.width));
        self.header_printed = true;
    }

    /// Start spinner with message
    pub fn start_spinner(&mut self, message: &str) {
        self.spinner.active = true;
        self.spinner.message = message.to_string();
        self.spinner.last_tick = Instant::now();
        self.print_spinner_line();
    }

    /// Stop spinner
    pub fn stop_spinner(&mut self) {
        if self.spinner.active {
            self.clear_spinner_line();
            self.spinner.active = false;
        }
    }

    /// Tick spinner animation (call periodically)
    pub fn tick(&mut self) -> bool {
        if self.spinner.tick() {
            self.update_spinner_line();
            true
        } else {
            false
        }
    }

    /// Print spinner line
    fn print_spinner_line(&self) {
        let elapsed = self.started_at.elapsed().as_secs_f32();
        let line = styles::working_status(&self.spinner.message, elapsed, self.spinner.frame);
        print!("\r{}", line);
        let _ = io::stdout().flush();
    }

    /// Update spinner line in place
    fn update_spinner_line(&self) {
        self.clear_line();
        self.print_spinner_line();
    }

    /// Clear current line
    fn clear_line(&self) {
        print!("\r{}\r", " ".repeat(self.options.width));
        let _ = io::stdout().flush();
    }

    /// Clear spinner line
    fn clear_spinner_line(&self) {
        self.clear_line();
    }

    /// Render new segments incrementally
    pub fn render_incremental(&mut self, transcript: &HollywoodTranscript) {
        let segments = transcript.segments();
        let new_segments = &segments[self.rendered_count..];

        for segment in new_segments {
            self.render_segment(segment);
            self.rendered_count += 1;
        }
    }

    /// Render a single segment
    fn render_segment(&mut self, segment: &TranscriptSegment) {
        // Pause spinner for output
        let was_active = self.spinner.active;
        if was_active {
            self.stop_spinner();
        }

        match segment.kind {
            SegmentKind::InternalComms => {
                if self.options.show_internal_comms {
                    if !self.internal_section_started {
                        println!("{}", styles::section_header(labels::INTERNAL));
                        self.internal_section_started = true;
                    }
                    let comm = InternalComm::from_actor(
                        &segment.actor,
                        &segment.content,
                        segment.relative_secs,
                    );
                    println!(
                        "{}",
                        styles::internal_comm_line(
                            comm.relative_secs,
                            &comm.staff_display(),
                            &comm.message,
                            self.options.show_timestamps,
                        )
                    );
                }
            }
            SegmentKind::ProbeRun => {
                if self.options.show_probes {
                    if let Some(probe_id) = segment.meta.get("probe_id") {
                        self.probe_buffer.push(probe_id.clone());
                    }
                }
            }
            SegmentKind::Answer => {
                self.flush_probes();
                println!("\n{}\n{}", labels::ANNA, segment.content);
            }
            SegmentKind::Error => {
                self.flush_probes();
                if self.options.is_debug() {
                    println!("\n{}\n{}", labels::ERROR, segment.content);
                } else {
                    println!("\n{}\n{}", labels::ANNA, segment.content);
                }
            }
            SegmentKind::Tip => {
                println!("{} {}", labels::TIP, segment.content);
            }
            SegmentKind::Progress => {
                // Update spinner message
                self.spinner.message = segment.content.clone();
            }
            SegmentKind::DebugJson => {
                if self.options.is_debug() {
                    let label = segment
                        .meta
                        .get("label")
                        .map(|s| s.as_str())
                        .unwrap_or("json");
                    println!(
                        "\n[debug] {}:\n{}",
                        label,
                        styles::indent(&segment.content, 2)
                    );
                }
            }
            _ => {
                // Other segments handled by final render
            }
        }

        // Resume spinner if it was active
        if was_active && !self.is_complete(segment) {
            self.start_spinner(&self.spinner.message.clone());
        }
    }

    /// Check if segment indicates completion
    fn is_complete(&self, segment: &TranscriptSegment) -> bool {
        matches!(segment.kind, SegmentKind::Answer | SegmentKind::Error)
    }

    /// Flush buffered probes
    fn flush_probes(&mut self) {
        if !self.probe_buffer.is_empty() && self.options.show_probes {
            println!("\n{}", labels::PROBES);
            for probe in &self.probe_buffer {
                println!("  {}", probe);
            }
            self.probe_buffer.clear();
        }
    }

    /// Render final output (evidence, footer)
    pub fn render_final(&mut self, transcript: &HollywoodTranscript) {
        self.stop_spinner();
        self.flush_probes();

        // Evidence footer
        if self.options.show_evidence && !transcript.evidence_sources.is_empty() {
            println!("{}", styles::evidence_footer(&transcript.evidence_sources));
        }

        // Status footer
        if self.options.show_footer {
            let handler = transcript.handled_by.as_ref().map(|h| {
                if let Some(ref dept) = transcript.department {
                    format!("{} ({})", h, dept)
                } else {
                    h.clone()
                }
            });

            let status = match transcript.outcome {
                super::types::TranscriptOutcome::Success => "System Status",
                super::types::TranscriptOutcome::Partial => "Partial Answer",
                super::types::TranscriptOutcome::Failed => "Request Failed",
                super::types::TranscriptOutcome::ParseError => "Parse Error",
                super::types::TranscriptOutcome::Cancelled => "Cancelled",
            };

            println!(
                "\n{}",
                styles::status_footer(
                    status,
                    transcript.confidence,
                    handler.as_deref(),
                    !transcript.evidence_sources.is_empty(),
                )
            );
        }
    }

    /// Render tip message (for long waits)
    pub fn render_tip(&mut self, message: &str) {
        let was_active = self.spinner.active;
        if was_active {
            self.stop_spinner();
        }
        println!("{} {}", labels::TIP, message);
        if was_active {
            self.start_spinner(&self.spinner.message.clone());
        }
    }

    /// Check if transcript is complete
    pub fn is_transcript_complete(&self, transcript: &HollywoodTranscript) -> bool {
        transcript
            .segments()
            .iter()
            .any(|s| matches!(s.kind, SegmentKind::Answer | SegmentKind::Error))
    }

    /// Get elapsed time
    pub fn elapsed_secs(&self) -> f32 {
        self.started_at.elapsed().as_secs_f32()
    }

    /// Reset for new request
    pub fn reset(&mut self) {
        self.rendered_count = 0;
        self.internal_section_started = false;
        self.probe_buffer.clear();
        self.spinner = SpinnerState::default();
        self.header_printed = false;
        self.started_at = Instant::now();
    }
}

/// Convenience function for quick streaming display
pub fn stream_with_spinner<F>(query: &str, message: &str, work: F) -> String
where
    F: FnOnce() -> String,
{
    let mut renderer = StreamingRenderer::cinematic();
    renderer.render_header(query);
    renderer.start_spinner(message);

    let result = work();

    renderer.stop_spinner();
    println!("\n{}\n{}", labels::ANNA, result);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_state() {
        let mut spinner = SpinnerState::default();
        assert!(!spinner.active);
        spinner.active = true;
        assert_eq!(spinner.frame_char(), "|");
    }

    #[test]
    fn test_streaming_renderer_creation() {
        let renderer = StreamingRenderer::cinematic();
        assert!(!renderer.header_printed);
        assert!(!renderer.spinner.active);
    }

    #[test]
    fn test_is_complete() {
        use crate::transcript_segment::TranscriptSegment;

        let renderer = StreamingRenderer::cinematic();
        let answer = TranscriptSegment::answer("test");
        assert!(renderer.is_complete(&answer));

        let error = TranscriptSegment::error("test");
        assert!(renderer.is_complete(&error));

        let comms =
            TranscriptSegment::internal_comms(crate::transcript_segment::staff::sofia(), "test");
        assert!(!renderer.is_complete(&comms));
    }
}
