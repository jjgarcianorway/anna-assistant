//! Live Renderer - Real-time transcript rendering with spinner (v0.0.413).
//!
//! Handles streaming output, spinner animation, and progressive display
//! of transcript segments as they arrive from annad.

use anna_shared::transcript_render::{render_segment, RenderConfig};
use anna_shared::transcript_segment::{SegmentKind, Transcript, TranscriptSegment};
use anna_shared::ui::colors;
use anna_shared::ui_config::{SpinnerStyle, UiConfig, UiState};
use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Live renderer state
pub struct LiveRenderer {
    /// Render configuration
    config: RenderConfig,
    /// UI state
    state: UiState,
    /// Spinner frames
    spinner_frames: &'static [&'static str],
    /// Last spinner update time
    last_spinner_tick: Instant,
    /// Spinner tick interval
    spinner_interval: Duration,
    /// Currently showing spinner
    spinner_active: bool,
    /// Segments already rendered
    rendered_count: usize,
    /// Probes collected (for grouping)
    probes_buffer: Vec<String>,
    /// In internal comms section
    in_internal_section: bool,
    /// Output buffer
    output: String,
}

impl LiveRenderer {
    /// Create new renderer with config
    pub fn new(ui_config: &UiConfig) -> Self {
        let render_config = RenderConfig {
            mode: ui_config.mode,
            show_internal_comms: ui_config.show_internal_comms,
            show_tips: ui_config.show_tips,
            show_probes: ui_config.show_probes,
            show_timestamps: ui_config.show_timestamps,
            width: terminal_width(),
        };

        Self {
            config: render_config,
            state: UiState::new(),
            spinner_frames: ui_config.spinner_frames(),
            last_spinner_tick: Instant::now(),
            spinner_interval: Duration::from_millis(100),
            spinner_active: false,
            rendered_count: 0,
            probes_buffer: Vec::new(),
            in_internal_section: false,
            output: String::new(),
        }
    }

    /// Render user input immediately
    pub fn render_user_input(&mut self, query: &str) {
        let line = format!(
            "\n{}[you]{} {}\n",
            colors::CYAN, colors::RESET, query
        );
        self.print(&line);
    }

    /// Start the spinner
    pub fn start_spinner(&mut self) {
        self.spinner_active = true;
        self.last_spinner_tick = Instant::now();
        self.render_spinner();
    }

    /// Stop the spinner and clear its line
    pub fn stop_spinner(&mut self) {
        if self.spinner_active {
            self.clear_line();
            self.spinner_active = false;
        }
    }

    /// Tick the spinner (call in event loop)
    pub fn tick(&mut self) -> bool {
        if !self.spinner_active {
            return false;
        }

        if self.last_spinner_tick.elapsed() >= self.spinner_interval {
            self.render_spinner();
            self.last_spinner_tick = Instant::now();
            return true;
        }
        false
    }

    /// Render new segments from transcript
    pub fn render_new_segments(&mut self, transcript: &Transcript) {
        // Stop spinner before rendering content
        self.stop_spinner();

        for segment in transcript.segments.iter().skip(self.rendered_count) {
            self.render_segment_live(segment);
            self.rendered_count += 1;
        }

        // Resume spinner if not done
        if !self.is_complete(transcript) {
            self.start_spinner();
        }
    }

    /// Render a single segment with proper grouping
    fn render_segment_live(&mut self, segment: &TranscriptSegment) {
        // Handle probe grouping
        if segment.kind == SegmentKind::ProbeRun {
            if let Some(probe_id) = segment.meta.get("probe_id") {
                self.probes_buffer.push(probe_id.clone());
            }
            return; // Buffer probes, don't render individually
        }

        // Flush probes buffer if we're past probe section
        if !self.probes_buffer.is_empty() && segment.kind != SegmentKind::ProbeRun {
            self.flush_probes();
        }

        // Handle internal comms section header
        if segment.kind == SegmentKind::InternalComms && !self.in_internal_section {
            if self.config.show_internal_comms {
                self.print(&format!(
                    "\n{}--- internal comms ---{}\n",
                    colors::DIM, colors::RESET
                ));
                self.in_internal_section = true;
            }
        } else if segment.kind != SegmentKind::InternalComms && self.in_internal_section {
            self.in_internal_section = false;
        }

        // Render the segment
        if let Some(rendered) = render_segment(segment, &self.config) {
            self.print(&rendered);
        }
    }

    /// Flush buffered probes as a group
    fn flush_probes(&mut self) {
        if self.probes_buffer.is_empty() || !self.config.show_probes {
            self.probes_buffer.clear();
            return;
        }

        let probes_str = self.probes_buffer.join("\n  ");
        self.print(&format!(
            "\n{}[probes]{}\n  {}\n",
            colors::DIM, colors::RESET, probes_str
        ));
        self.probes_buffer.clear();
    }

    /// Check if transcript is complete
    fn is_complete(&self, transcript: &Transcript) -> bool {
        transcript.segments.iter().any(|s| {
            matches!(s.kind, SegmentKind::Answer | SegmentKind::Error)
        })
    }

    /// Render the final transcript (for complete results)
    pub fn render_final(&mut self, transcript: &Transcript) {
        self.stop_spinner();

        // Flush any remaining probes
        self.flush_probes();

        // Render any segments we haven't seen
        for segment in transcript.segments.iter().skip(self.rendered_count) {
            if let Some(rendered) = render_segment(segment, &self.config) {
                self.print(&rendered);
            }
        }

        // Trailing newline
        self.print("\n");
    }

    /// Print to stdout with flush
    fn print(&mut self, text: &str) {
        print!("{}", text);
        let _ = io::stdout().flush();
    }

    /// Clear current line (for spinner)
    fn clear_line(&self) {
        print!("\r\x1b[K");
        let _ = io::stdout().flush();
    }

    /// Render spinner frame
    fn render_spinner(&mut self) {
        let frame = self.state.tick_spinner(self.spinner_frames);
        if !frame.is_empty() {
            print!(
                "\r{}{} Working...{}",
                colors::DIM, frame, colors::RESET
            );
            let _ = io::stdout().flush();
        }
    }

    /// Render a tip segment (for long waits)
    pub fn render_tip(&mut self, message: &str) {
        self.stop_spinner();
        self.print(&format!(
            "{}[tip]{} {}\n",
            colors::DIM, colors::RESET, message
        ));
        self.start_spinner();
    }

    /// Get all output (for testing)
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

/// Get terminal width
fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
}

/// Render a complete transcript (non-streaming, for one-shot)
pub fn render_complete(transcript: &Transcript, config: &UiConfig) -> String {
    use anna_shared::transcript_render::render_transcript;

    let render_config = RenderConfig {
        mode: config.mode,
        show_internal_comms: config.show_internal_comms,
        show_tips: config.show_tips,
        show_probes: config.show_probes,
        show_timestamps: config.show_timestamps,
        width: terminal_width(),
    };

    render_transcript(transcript, &render_config)
}

/// Format answer section
pub fn format_answer_section(
    headline: &str,
    body: &str,
    evidence: &[&str],
    quick_actions: Option<&[&str]>,
) -> String {
    anna_shared::transcript_render::format_answer_with_evidence(
        headline,
        body,
        evidence,
        quick_actions,
    )
}

/// Format error section
pub fn format_error_section(
    error_msg: &str,
    collected: &[&str],
    ticket: Option<(&str, &str)>,
    fallback: Option<&str>,
) -> String {
    anna_shared::transcript_render::format_error_with_context(
        error_msg,
        collected,
        ticket,
        fallback,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::transcript_segment::TranscriptMode;

    #[test]
    fn test_live_renderer_creation() {
        let config = UiConfig::default();
        let renderer = LiveRenderer::new(&config);
        assert_eq!(renderer.config.mode, TranscriptMode::Cinematic);
    }

    #[test]
    fn test_format_answer() {
        let answer = format_answer_section(
            "Memory: 17 GiB free",
            "54% of total RAM available",
            &["/proc/meminfo"],
            None,
        );
        assert!(answer.contains("17 GiB"));
        assert!(answer.contains("Evidence:"));
    }
}
