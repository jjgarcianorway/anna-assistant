//! Individual segment renderers for different segment kinds.

use crate::transcript_segment::TranscriptSegment;
use crate::transcript_segment::TranscriptMode;
use crate::ui::colors;

use super::config::RenderConfig;

pub fn render_user_input(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    format!(
        "\n{}[you]{} {}\n",
        colors::CYAN,
        colors::RESET,
        segment.content
    )
}

pub fn render_system_info(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    if config.mode == TranscriptMode::Debug {
        format!(
            "{}[system]{} {}\n",
            colors::DIM,
            colors::RESET,
            segment.content
        )
    } else {
        // In cinematic mode, system info is very subtle
        String::new()
    }
}

pub fn render_ticket_header(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    let ticket_id = segment
        .meta
        .get("ticket_id")
        .map(|s| s.as_str())
        .unwrap_or("?");
    let domain = segment
        .meta
        .get("domain")
        .map(|s| s.as_str())
        .unwrap_or("?");

    if config.mode == TranscriptMode::Debug {
        format!(
            "{}[ticket]{} {} ({}) - {}\n",
            colors::YELLOW,
            colors::RESET,
            ticket_id,
            domain,
            segment.content
        )
    } else {
        // Cinematic: ticket info is shown in internal comms
        String::new()
    }
}

pub fn render_internal_comms(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    let timestamp = if config.show_timestamps {
        format!(
            "{}[{:.1}s]{} ",
            colors::DIM,
            segment.relative_secs,
            colors::RESET
        )
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

pub fn render_probe_run(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    // Individual probe - usually grouped, but can be rendered alone in debug
    if config.mode == TranscriptMode::Debug {
        let probe_id = segment
            .meta
            .get("probe_id")
            .map(|s| s.as_str())
            .unwrap_or("?");
        format!(
            "{}[probe]{} {} - {}\n",
            colors::DIM,
            colors::RESET,
            probe_id,
            segment.content
        )
    } else {
        String::new()
    }
}

pub fn render_specialist_message(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    let timestamp = if config.show_timestamps {
        format!(
            "{}[{:.1}s]{} ",
            colors::DIM,
            segment.relative_secs,
            colors::RESET
        )
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

pub fn render_answer(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    format!(
        "\n{}[anna]{}\n{}\n",
        colors::OK,
        colors::RESET,
        segment.content
    )
}

pub fn render_error(segment: &TranscriptSegment, config: &RenderConfig) -> String {
    if config.mode == TranscriptMode::Debug {
        format!(
            "\n{}[error]{}\n{}\n",
            colors::ERR,
            colors::RESET,
            segment.content
        )
    } else {
        // Cinematic: errors are presented more gently
        format!(
            "\n{}[anna]{}\n{}\n",
            colors::WARN,
            colors::RESET,
            segment.content
        )
    }
}

pub fn render_tip(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    format!(
        "{}[tip]{} {}\n",
        colors::DIM,
        colors::RESET,
        segment.content
    )
}

pub fn render_stats(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    format!(
        "{}[stats]{} {}\n",
        colors::DIM,
        colors::RESET,
        segment.content
    )
}

pub fn render_debug_json(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    let label = segment
        .meta
        .get("label")
        .map(|s| s.as_str())
        .unwrap_or("json");
    format!(
        "\n{}[debug] {}:{}\n{}\n",
        colors::YELLOW,
        label,
        colors::RESET,
        segment.content
    )
}

pub fn render_progress(segment: &TranscriptSegment, _config: &RenderConfig) -> String {
    format!("{}{}{}", colors::DIM, segment.content, colors::RESET)
}
