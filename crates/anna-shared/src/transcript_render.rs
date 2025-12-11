//! Transcript Renderer - Cinematic and Debug mode rendering (v0.0.413).
//!
//! Renders TranscriptSegments to terminal output with proper styling.
//! Supports both Hollywood IT department view and developer debug view.

use crate::transcript_segment::{SegmentKind, Transcript, TranscriptMode, TranscriptSegment};
use crate::ui::colors;

/// Render configuration
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Display mode
    pub mode: TranscriptMode,
    /// Show internal comms section
    pub show_internal_comms: bool,
    /// Show tips
    pub show_tips: bool,
    /// Show probe details
    pub show_probes: bool,
    /// Show timestamps
    pub show_timestamps: bool,
    /// Terminal width (for wrapping)
    pub width: usize,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            mode: TranscriptMode::Cinematic,
            show_internal_comms: true,
            show_tips: true,
            show_probes: true,
            show_timestamps: true,
            width: 80,
        }
    }
}

impl RenderConfig {
    pub fn cinematic() -> Self {
        Self::default()
    }

    pub fn debug() -> Self {
        Self {
            mode: TranscriptMode::Debug,
            ..Self::default()
        }
    }

    pub fn minimal() -> Self {
        Self {
            show_internal_comms: false,
            show_probes: false,
            show_tips: false,
            show_timestamps: false,
            ..Self::default()
        }
    }
}

/// Render a single segment
pub fn render_segment(segment: &TranscriptSegment, config: &RenderConfig) -> Option<String> {
    // Skip debug-only segments in cinematic mode
    if config.mode == TranscriptMode::Cinematic && !segment.kind.show_in_cinematic() {
        return None;
    }

    // Skip internal comms if disabled
    if !config.show_internal_comms && segment.kind == SegmentKind::InternalComms {
        return None;
    }

    // Skip tips if disabled
    if !config.show_tips && segment.kind == SegmentKind::Tip {
        return None;
    }

    // Skip probes if disabled
    if !config.show_probes && segment.kind == SegmentKind::ProbeRun {
        return None;
    }

    match segment.kind {
        SegmentKind::UserInput => Some(render_user_input(segment, config)),
        SegmentKind::SystemInfo => Some(render_system_info(segment, config)),
        SegmentKind::TicketHeader => Some(render_ticket_header(segment, config)),
        SegmentKind::InternalComms => Some(render_internal_comms(segment, config)),
        SegmentKind::ProbeRun => Some(render_probe_run(segment, config)),
        SegmentKind::SpecialistMessage => Some(render_specialist_message(segment, config)),
        SegmentKind::Answer => Some(render_answer(segment, config)),
        SegmentKind::Error => Some(render_error(segment, config)),
        SegmentKind::Tip => Some(render_tip(segment, config)),
        SegmentKind::StatsSnippet => Some(render_stats(segment, config)),
        SegmentKind::DebugJson => Some(render_debug_json(segment, config)),
        SegmentKind::Progress => Some(render_progress(segment, config)),
    }
}

/// Render full transcript
pub fn render_transcript(transcript: &Transcript, config: &RenderConfig) -> String {
    let mut output = String::new();
    let mut in_internal_section = false;
    let mut in_probe_section = false;
    let mut probes_collected: Vec<String> = Vec::new();

    for segment in &transcript.segments {
        // Group internal comms together
        if segment.kind == SegmentKind::InternalComms {
            if !in_internal_section && config.show_internal_comms {
                output.push_str(&format!(
                    "\n{}--- internal comms ---{}\n",
                    colors::DIM, colors::RESET
                ));
                in_internal_section = true;
            }
        } else if in_internal_section && segment.kind != SegmentKind::ProbeRun {
            in_internal_section = false;
        }

        // Collect probes for compact display
        if segment.kind == SegmentKind::ProbeRun {
            if let Some(probe_id) = segment.meta.get("probe_id") {
                probes_collected.push(probe_id.clone());
            }
            if !in_probe_section && config.show_probes {
                in_probe_section = true;
            }
            continue; // Don't render individual probes, show summary
        } else if in_probe_section {
            // End of probe section, render summary
            if !probes_collected.is_empty() && config.show_probes {
                output.push_str(&format!(
                    "\n{}[probes]{}\n  {}\n",
                    colors::DIM,
                    colors::RESET,
                    probes_collected.join("\n  ")
                ));
                probes_collected.clear();
            }
            in_probe_section = false;
        }

        if let Some(rendered) = render_segment(segment, config) {
            output.push_str(&rendered);
        }
    }

    // Flush any remaining probes
    if !probes_collected.is_empty() && config.show_probes {
        output.push_str(&format!(
            "\n{}[probes]{}\n  {}\n",
            colors::DIM,
            colors::RESET,
            probes_collected.join("\n  ")
        ));
    }

    output
}

// Individual segment renderers

fn render_user_input(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    format!(
        "\n{}[you]{} {}\n",
        colors::CYAN, colors::RESET, segment.content
    )
}

fn render_system_info(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    if config.mode == TranscriptMode::Debug {
        format!(
            "{}[system]{} {}\n",
            colors::DIM, colors::RESET, segment.content
        )
    } else {
        // In cinematic mode, system info is very subtle
        String::new()
    }
}

fn render_ticket_header(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    let ticket_id = segment.meta.get("ticket_id").map(|s| s.as_str()).unwrap_or("?");
    let domain = segment.meta.get("domain").map(|s| s.as_str()).unwrap_or("?");

    if config.mode == TranscriptMode::Debug {
        format!(
            "{}[ticket]{} {} ({}) - {}\n",
            colors::YELLOW, colors::RESET, ticket_id, domain, segment.content
        )
    } else {
        // Cinematic: ticket info is shown in internal comms
        String::new()
    }
}

fn render_internal_comms(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    let timestamp = if config.show_timestamps {
        format!("{}[{:.1}s]{} ", colors::DIM, segment.relative_secs, colors::RESET)
    } else {
        String::new()
    };

    format!(
        "  {}{}{}{}: {}\n",
        timestamp,
        colors::CYAN,
        segment.actor.display(),
        colors::RESET,
        segment.content
    )
}

fn render_probe_run(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    // Individual probe - usually grouped, but can be rendered alone in debug
    if config.mode == TranscriptMode::Debug {
        let probe_id = segment.meta.get("probe_id").map(|s| s.as_str()).unwrap_or("?");
        format!(
            "{}[probe]{} {} - {}\n",
            colors::DIM, colors::RESET, probe_id, segment.content
        )
    } else {
        String::new()
    }
}

fn render_specialist_message(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    let timestamp = if config.show_timestamps {
        format!("{}[{:.1}s]{} ", colors::DIM, segment.relative_secs, colors::RESET)
    } else {
        String::new()
    };

    format!(
        "  {}{}{}{}: {}\n",
        timestamp,
        colors::GREEN,
        segment.actor.display(),
        colors::RESET,
        segment.content
    )
}

fn render_answer(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    format!(
        "\n{}[anna]{}\n{}\n",
        colors::OK, colors::RESET, segment.content
    )
}

fn render_error(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    if config.mode == TranscriptMode::Debug {
        format!(
            "\n{}[error]{}\n{}\n",
            colors::ERR, colors::RESET, segment.content
        )
    } else {
        // Cinematic: errors are presented more gently
        format!(
            "\n{}[anna]{}\n{}\n",
            colors::WARN, colors::RESET, segment.content
        )
    }
}

fn render_tip(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    format!(
        "{}[tip]{} {}\n",
        colors::DIM, colors::RESET, segment.content
    )
}

fn render_stats(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    format!(
        "{}[stats]{} {}\n",
        colors::DIM, colors::RESET, segment.content
    )
}

fn render_debug_json(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    let label = segment.meta.get("label").map(|s| s.as_str()).unwrap_or("json");
    format!(
        "\n{}[debug] {}:{}\n{}\n",
        colors::YELLOW, label, colors::RESET, segment.content
    )
}

fn render_progress(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    format!("{}{}{}", colors::DIM, segment.content, colors::RESET)
}

/// Format an answer with evidence footer
pub fn format_answer_with_evidence(
    headline: &str,
    body: &str,
    evidence: &[&str],
    quick_actions: Option<&[&str]>,
) -> String {
    let mut answer = String::new();

    // Headline
    answer.push_str(headline);
    answer.push_str("\n\n");

    // Body
    answer.push_str(body);

    // Quick actions
    if let Some(actions) = quick_actions {
        if !actions.is_empty() {
            answer.push_str("\n\nQuick actions:\n");
            for (i, action) in actions.iter().enumerate() {
                answer.push_str(&format!("  {}) {}\n", i + 1, action));
            }
        }
    }

    // Evidence footer
    if !evidence.is_empty() {
        answer.push_str(&format!(
            "\n{}Evidence: {}{}",
            colors::DIM,
            evidence.join(", "),
            colors::RESET
        ));
    }

    answer
}

/// Format error with collected evidence
pub fn format_error_with_context(
    error_headline: &str,
    collected_data: &[&str],
    ticket_info: Option<(&str, &str)>,
    fallback_message: Option<&str>,
) -> String {
    let mut error = String::new();

    error.push_str(error_headline);
    error.push('\n');

    if !collected_data.is_empty() {
        error.push_str("\nWhat I collected:\n");
        for data in collected_data {
            error.push_str(&format!("  - {}\n", data));
        }
    }

    if let Some((ticket_id, domain)) = ticket_info {
        error.push_str(&format!("\nTicket: {} ({})\n", ticket_id, domain));
    }

    if let Some(fallback) = fallback_message {
        error.push_str(&format!("\n{}\n", fallback));
    }

    error
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcript_segment::staff;

    #[test]
    fn test_render_cinematic() {
        let mut t = Transcript::new("req-001");
        t.add_user_input("why is nginx failing?");
        t.add(TranscriptSegment::internal_comms(
            staff::hugo(),
            "Checking nginx status.",
        ));
        t.add(TranscriptSegment::answer("nginx has a config error"));

        let config = RenderConfig::cinematic();
        let output = render_transcript(&t, &config);

        assert!(output.contains("[you]"));
        assert!(output.contains("Hugo"));
        assert!(output.contains("[anna]"));
    }

    #[test]
    fn test_format_answer_with_evidence() {
        let answer = format_answer_with_evidence(
            "Memory available: 17.0 GiB",
            "54% of 31.0 GiB total",
            &["/proc/meminfo"],
            None,
        );

        assert!(answer.contains("17.0 GiB"));
        assert!(answer.contains("Evidence:"));
        assert!(answer.contains("/proc/meminfo"));
    }
}
