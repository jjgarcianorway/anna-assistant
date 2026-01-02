//! Dialogue renderers - Phase 89
//!
//! Functions for rendering dialogues in various formats.

use super::types::{Dialogue, DialogueMood, Speaker};

/// Render a dialogue for display
pub fn render_dialogue(dialogue: &Dialogue, show_internal: bool) -> String {
    let mut lines = Vec::new();

    // Header
    if let Some(subject) = &dialogue.subject {
        lines.push(format!("--- {} ---", subject));
        lines.push(String::new());
    }

    let reset = "\x1b[0m";

    for turn in &dialogue.turns {
        // Skip internal if not showing
        if turn.internal && !show_internal {
            continue;
        }

        // Internal marker
        if turn.internal && show_internal {
            lines.push("--- Internal communication ---".to_string());
        }

        // Speaker line
        let speaker_display = if let Some(name) = &turn.speaker_name {
            if let Some(dept) = &turn.department {
                format!("{} ({})", name, dept)
            } else {
                name.clone()
            }
        } else {
            turn.speaker.name().to_string()
        };

        let color = turn.speaker.color_code();
        let prefix = turn.mood.prefix();

        lines.push(format!(
            "{}{}{}: {}{}",
            color, speaker_display, reset, prefix, turn.content
        ));
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Render dialogue without colors
pub fn render_dialogue_plain(dialogue: &Dialogue, show_internal: bool) -> String {
    let mut lines = Vec::new();

    if let Some(subject) = &dialogue.subject {
        lines.push(format!("--- {} ---", subject));
        lines.push(String::new());
    }

    for turn in &dialogue.turns {
        if turn.internal && !show_internal {
            continue;
        }

        if turn.internal && show_internal {
            lines.push("--- Internal communication ---".to_string());
        }

        let speaker_display = if let Some(name) = &turn.speaker_name {
            if let Some(dept) = &turn.department {
                format!("{} ({})", name, dept)
            } else {
                name.clone()
            }
        } else {
            turn.speaker.name().to_string()
        };

        let prefix = turn.mood.prefix();
        lines.push(format!("{}: {}{}", speaker_display, prefix, turn.content));
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Render dialogue compact (one line per turn)
pub fn render_dialogue_compact(dialogue: &Dialogue) -> String {
    dialogue
        .external_turns()
        .iter()
        .map(|t| {
            let name = t.speaker_name.as_deref().unwrap_or(t.speaker.name());
            format!("[{}] {}", name, truncate(&t.content, 50))
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Truncate string
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
