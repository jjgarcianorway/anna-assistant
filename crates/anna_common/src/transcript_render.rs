//! Transcript Renderer v0.0.55 - Human-readable transcript formatting
//!
//! Format: [actor] to [actor]: message
//! Evidence citations: [E#] inline
//! Phase markers: --- Phase: X ---

use crate::case_engine::{CaseActor, CaseEvent, CaseEventType, CasePhase, CaseState};
use crate::case_file_v1::CaseFileV1;

// ============================================================================
// Transcript Rendering
// ============================================================================

/// Render a full transcript from CaseState
pub fn render_transcript_from_state(state: &CaseState, terminal_width: usize) -> String {
    let mut lines = Vec::new();
    let width = terminal_width.max(60);

    // Header
    lines.push(format!("=== Case: {} ===", state.case_id));
    lines.push(format!("Request: {}", wrap_text(&state.request, width - 10)));
    lines.push(String::new());

    // Initial user message
    lines.push(format!("[you] to [anna]: {}", wrap_text(&state.request, width - 20)));
    lines.push(String::new());

    // Events by phase
    let mut current_phase: Option<CasePhase> = None;

    for event in &state.events {
        // Phase separator
        if Some(event.phase) != current_phase {
            current_phase = Some(event.phase);
            lines.push(format!("--- {} ---", event.phase));
        }

        // Render event
        let line = render_event(event, width);
        lines.push(line);
    }

    // Final response
    if let Some(answer) = &state.final_answer {
        lines.push(String::new());
        lines.push("[anna] to [you]:".to_string());
        for line in wrap_text(answer, width - 2).lines() {
            lines.push(format!("  {}", line));
        }
    }

    // Footer
    lines.push(String::new());
    if let Some(reliability) = state.reliability_score {
        lines.push(format!("Reliability: {}%", reliability));
    }
    lines.push(format!("Duration: {}ms", state.total_duration_ms()));

    lines.join("\n")
}

/// Render a full transcript from CaseFileV1
pub fn render_transcript_from_file(case: &CaseFileV1, terminal_width: usize) -> String {
    let mut lines = Vec::new();
    let width = terminal_width.max(60);

    // Header
    lines.push(format!("=== Case: {} ===", case.case_id));
    lines.push(format!("Request: {}", wrap_text(&case.request, width - 10)));
    lines.push(format!("Intent: {} ({}%)", case.intent, case.intent_confidence));
    lines.push(String::new());

    // User message
    lines.push(format!("[you] to [anna]: {}", wrap_text(&case.request, width - 20)));
    lines.push(String::new());

    // Triage
    lines.push("--- Triage ---".to_string());
    lines.push(format!("[anna] to [translator]: classify request"));
    lines.push(format!("[translator] to [anna]: {} ({}% confidence)", case.intent, case.intent_confidence));

    // Doctor selection (if applicable)
    if let Some(doctor_id) = &case.doctor_id {
        lines.push(String::new());
        lines.push("--- DoctorSelect ---".to_string());
        lines.push(format!("[anna] to [engine]: select doctor for {:?}", case.problem_domain));
        lines.push(format!("[engine] to [anna]: selected {} ({}%)",
            doctor_id,
            case.doctor_confidence.unwrap_or(0)
        ));
    }

    // Evidence
    if !case.evidence.is_empty() {
        lines.push(String::new());
        lines.push("--- EvidenceGather ---".to_string());
        for e in &case.evidence {
            lines.push(format!("[anna] to [annad]: execute {} -> [{}]", e.tool_name, e.id));
            lines.push(format!("[annad] to [anna]: {}", truncate(&e.summary, width - 20)));
        }
    }

    // Response
    lines.push(String::new());
    lines.push("--- Respond ---".to_string());
    if let Some(answer) = &case.final_answer {
        lines.push("[anna] to [you]:".to_string());
        for line in wrap_text(answer, width - 2).lines() {
            lines.push(format!("  {}", line));
        }
    }

    // Footer
    lines.push(String::new());
    lines.push(format!("Reliability: {}%", case.reliability_score));
    lines.push(format!("Duration: {}ms", case.duration_ms));
    if case.recipe_extracted {
        lines.push(format!("Recipe: {}", case.recipe_id.as_deref().unwrap_or("unknown")));
    }

    lines.join("\n")
}

/// Render a single event
fn render_event(event: &CaseEvent, width: usize) -> String {
    let actor = format!("[{}]", event.actor);
    let summary = truncate(&event.summary, width - actor.len() - 10);

    match event.event_type {
        CaseEventType::PhaseTransition => {
            format!("  {} {}", actor, summary)
        }
        CaseEventType::ToolExecuted => {
            format!("{} to [annad]: execute {}", actor, summary)
        }
        CaseEventType::EvidenceCollected => {
            format!("[annad] to {}: {}", actor, summary)
        }
        CaseEventType::IntentClassified => {
            format!("[translator] to [anna]: {}", summary)
        }
        CaseEventType::DoctorSelected => {
            format!("[engine] to [anna]: {}", summary)
        }
        CaseEventType::AnswerDrafted => {
            format!("[anna] draft: {}", truncate(&summary, width - 15))
        }
        CaseEventType::VerificationResult => {
            format!("[junior] to [anna]: {}", summary)
        }
        CaseEventType::ResponseSent => {
            format!("[anna] to [you]: response sent")
        }
        CaseEventType::Error => {
            format!("[ERROR] {}: {}", actor, summary)
        }
        _ => {
            format!("{}: {}", actor, summary)
        }
    }
}

// ============================================================================
// Compact Transcript (for status display)
// ============================================================================

/// Render a compact one-line summary
pub fn render_compact_summary(case: &CaseFileV1) -> String {
    let outcome = if case.success { "OK" } else { "FAIL" };
    let request_short = truncate(&case.request, 40);

    format!(
        "[{}] {} | {} | {}% | {}ms",
        case.case_id.chars().take(8).collect::<String>(),
        request_short,
        outcome,
        case.reliability_score,
        case.duration_ms
    )
}

/// Render last N cases as compact list
pub fn render_recent_cases(cases: &[CaseFileV1], max_lines: usize) -> String {
    let mut lines = Vec::new();

    for case in cases.iter().take(max_lines) {
        lines.push(render_compact_summary(case));
    }

    if lines.is_empty() {
        lines.push("(no recent cases)".to_string());
    }

    lines.join("\n")
}

// ============================================================================
// Text Utilities
// ============================================================================

/// Wrap text to fit width
pub fn wrap_text(text: &str, width: usize) -> String {
    let width = width.max(40);
    let mut result = Vec::new();

    for line in text.lines() {
        if line.len() <= width {
            result.push(line.to_string());
        } else {
            let mut current = String::new();
            for word in line.split_whitespace() {
                if current.is_empty() {
                    current = word.to_string();
                } else if current.len() + 1 + word.len() <= width {
                    current.push(' ');
                    current.push_str(word);
                } else {
                    result.push(current);
                    current = word.to_string();
                }
            }
            if !current.is_empty() {
                result.push(current);
            }
        }
    }

    result.join("\n")
}

/// Truncate text with ellipsis
pub fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else if max_len > 3 {
        format!("{}...", &text[..max_len - 3])
    } else {
        text[..max_len].to_string()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_engine::IntentType;

    #[test]
    fn test_wrap_text() {
        // Note: wrap_text has min width of 40, so use a longer test
        let text = "This is a very long line that should definitely be wrapped at some point because it has many words";
        let wrapped = wrap_text(text, 50);
        for line in wrapped.lines() {
            // Lines should be <= 50 chars or be single words (no space to wrap on)
            assert!(line.len() <= 50 || !line.contains(' '), "Line too long: {}", line);
        }
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 2), "hi");
    }

    #[test]
    fn test_compact_summary() {
        let mut case = CaseFileV1::new("test-12345678-abcd", "what cpu do I have");
        case.success = true;
        case.reliability_score = 85;
        case.duration_ms = 150;

        let summary = render_compact_summary(&case);
        assert!(summary.contains("test-123"));
        assert!(summary.contains("OK"));
        assert!(summary.contains("85%"));
    }

    #[test]
    fn test_transcript_rendering() {
        let mut case = CaseFileV1::new("test-render", "what cpu do I have");
        case.set_intent(IntentType::SystemQuery, 95);
        case.add_evidence("E1", "hw_snapshot_cpu", "AMD Ryzen 7 5800X", 50, false);
        case.set_response("You have an AMD Ryzen 7 5800X", 85);
        case.complete(true, None);

        let transcript = render_transcript_from_file(&case, 80);
        assert!(transcript.contains("[you] to [anna]"));
        assert!(transcript.contains("SYSTEM_QUERY"));
        assert!(transcript.contains("AMD Ryzen"));
    }
}
