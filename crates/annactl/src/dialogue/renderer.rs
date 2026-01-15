//! Dialogue Renderer - Renders timeline dialogue to terminal.
//!
//! Phase 11: Human-readable internal dialogue display.

use anna_shared::timeline::{DialogueLine, DialogueTimeline, RedactionMode};
use anna_shared::timeline::narrator::narrate_timeline;

use crate::display::{print_colored, println_colored, BOLD, CYAN, DIM, GREEN, MAGENTA, RESET, WHITE, YELLOW};

/// Render a dialogue line to the terminal.
pub fn render_line(line: &DialogueLine) {
    let time_str = format!("[{:.1}s]", line.offset_ms as f64 / 1000.0);

    print_colored(&time_str, DIM);
    print!(" ");

    if line.speaker == "---" {
        // Narration line
        println_colored(&line.message, DIM);
    } else if let Some(recipient) = &line.recipient {
        // Dialogue line
        print_colored(&line.speaker, speaker_color(&line.speaker));
        print_colored(" -> ", DIM);
        print_colored(recipient, speaker_color(recipient));
        print!(": ");
        println!("{}", line.message);
    } else {
        // Speaker-only (rare)
        print_colored(&line.speaker, speaker_color(&line.speaker));
        print!(": ");
        println!("{}", line.message);
    }
}

/// Render a complete timeline.
pub fn render_timeline(timeline: &DialogueTimeline, mode: RedactionMode) {
    let include_internal = matches!(mode, RedactionMode::Debug);
    let lines = narrate_timeline(timeline, include_internal);

    if lines.is_empty() {
        println_colored("(no dialogue recorded)", DIM);
        return;
    }

    for line in &lines {
        render_line(line);
    }
}

/// Render the internal comms header.
pub fn render_internal_comms_header() {
    println!();
    println_colored("--- internal comms ---", DIM);
}

/// Render a resolution summary.
pub fn render_resolution(specialist: &str, confidence: f32, learned: bool) {
    println!();
    print_colored("Resolved by ", DIM);
    print_colored(specialist, GREEN);
    print_colored(&format!(" ({:.0}% confidence)", confidence * 100.0), DIM);
    println!();

    if learned {
        print_colored("  ", DIM);
        println_colored("New recipe learned.", CYAN);
    }
}

/// Render a completion message when request could not be fulfilled.
/// v0.3.45: Use calm, professional language (no alarmism).
pub fn render_incomplete(reason: &str) {
    println!();
    print_colored("Unable to complete: ", YELLOW);
    println!("{}", reason);
}

/// Get color for a speaker based on their role.
fn speaker_color(speaker: &str) -> &'static str {
    if speaker == "Anna" {
        GREEN
    } else if speaker.contains("Sr") || speaker.contains("Senior") {
        MAGENTA
    } else if speaker.contains("Jr") || speaker.contains("Junior") {
        CYAN
    } else if speaker.starts_with('[') {
        DIM
    } else {
        WHITE
    }
}

/// Format a timeline for logging (no colors).
pub fn format_for_log(timeline: &DialogueTimeline, mode: RedactionMode) -> String {
    let include_internal = matches!(mode, RedactionMode::Debug);
    let lines = narrate_timeline(timeline, include_internal);

    lines
        .iter()
        .map(|line| {
            let time = format!("[{:.1}s]", line.offset_ms as f64 / 1000.0);
            match &line.recipient {
                Some(r) => format!("{} {} -> {}: {}", time, line.speaker, r, line.message),
                None => format!("{} {}", time, line.message),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::timeline::types::EntryKind;

    #[test]
    fn test_speaker_color() {
        assert_eq!(speaker_color("Anna"), GREEN);
        assert_eq!(speaker_color("James (Jr, System)"), CYAN);
        assert_eq!(speaker_color("Lisa (Sr, System)"), MAGENTA);
        assert_eq!(speaker_color("[debug]"), DIM);
    }

    #[test]
    fn test_format_for_log() {
        let mut timeline = DialogueTimeline::new("CN-001", "test");
        timeline.add(EntryKind::SpecialistAssigned {
            specialist_id: "sys-jr".to_string(),
            specialist_name: "James".to_string(),
            level: "Junior".to_string(),
            department: "System".to_string(),
        });

        let log = format_for_log(&timeline, RedactionMode::Normal);
        assert!(log.contains("James"));
        assert!(log.contains("Anna"));
    }
}
