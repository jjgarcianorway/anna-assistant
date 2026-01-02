//! Full transcript and segment rendering functions.

use crate::transcript_segment::{SegmentKind, Transcript, TranscriptMode, TranscriptSegment};
use crate::ui::colors;

use super::config::RenderConfig;
use super::segment_renderers::*;

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
                    colors::DIM,
                    colors::RESET
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
}
