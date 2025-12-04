//! Transcript Renderer v0.0.72 - Dual Mode Rendering
//!
//! Both renderers consume the same event stream to ensure they cannot diverge.
//! Human mode shows "fly on the wall" IT department dialogue.
//! Debug mode shows full internal details.

use super::events::{
    EventDataV72, RiskLevelV72, RoleV72, TranscriptEventV72, TranscriptStreamV72,
    WarningCategoryV72,
};
use crate::transcript_events::TranscriptMode;
use owo_colors::OwoColorize;

/// Rendered output line
#[derive(Debug, Clone)]
pub struct RenderedLineV72 {
    pub text: String,
    pub is_progress: bool,
}

impl RenderedLineV72 {
    pub fn new(text: String) -> Self {
        Self {
            text,
            is_progress: false,
        }
    }

    pub fn progress(text: String) -> Self {
        Self {
            text,
            is_progress: true,
        }
    }
}

/// Render event based on mode
pub fn render_event_v72(
    event: &TranscriptEventV72,
    mode: TranscriptMode,
) -> Option<RenderedLineV72> {
    match mode {
        TranscriptMode::Human => render_human_v72(event),
        TranscriptMode::Debug | TranscriptMode::Test => render_debug_v72(event),
    }
}

/// Render event for human mode - no internals exposed
fn render_human_v72(event: &TranscriptEventV72) -> Option<RenderedLineV72> {
    match &event.data {
        EventDataV72::UserMessage { text } => {
            let line = format!("{} {}", style_role_human(RoleV72::Anna, "[you]"), text);
            Some(RenderedLineV72::new(line))
        }

        EventDataV72::StaffMessage {
            role,
            content_human,
            ..
        } => {
            if !role.visible_in_human() {
                return None; // Hide internal roles
            }
            let tag = format!("[{}]", role.tag());
            let line = format!("{} {}", style_role_human(*role, &tag), content_human);
            Some(RenderedLineV72::new(line))
        }

        EventDataV72::Evidence {
            human_label,
            summary_human,
            ..
        } => {
            // Show human label, NOT evidence ID or tool name
            let line = format!(
                "{} Evidence from {}: {}",
                "[anna]".cyan(),
                human_label,
                summary_human
            );
            Some(RenderedLineV72::new(line))
        }

        EventDataV72::ToolCall { action_human, .. } => {
            // Show human action, NOT tool name
            let line = format!("{} I'm {}.", "[anna]".cyan(), action_human);
            Some(RenderedLineV72::new(line))
        }

        EventDataV72::ToolResult {
            result_human,
            success,
            ..
        } => {
            if *success {
                Some(RenderedLineV72::new(format!(
                    "{} {}",
                    "[anna]".cyan(),
                    result_human
                )))
            } else {
                let line = format!("{} Could not retrieve this information.", "[anna]".cyan());
                Some(RenderedLineV72::new(line))
            }
        }

        EventDataV72::Classification {
            understood_human,
            fallback_used,
            ..
        } => {
            // Show humanized classification, hide parse details
            if *fallback_used {
                // Human-friendly fallback message
                let line = format!(
                    "{} Translator struggled to classify this; we used house rules.",
                    "[service desk]".green()
                );
                Some(RenderedLineV72::new(line))
            } else {
                let line = format!("{} {}", "[service desk]".green(), understood_human);
                Some(RenderedLineV72::new(line))
            }
        }

        EventDataV72::Reliability {
            score,
            rationale_short,
            ..
        } => {
            // Show reliability in human-friendly format
            let level = reliability_level(*score);
            let line = format!("Reliability: {}% ({}) - {}", score, level, rationale_short);
            Some(RenderedLineV72::new(line.dimmed().to_string()))
        }

        EventDataV72::Perf { total_ms, .. } => {
            // Simple timing in human mode (no breakdown)
            let line = format!("Completed in {:.1}s", *total_ms as f64 / 1000.0);
            Some(RenderedLineV72::new(line.dimmed().to_string()))
        }

        EventDataV72::Confirmation {
            change_description,
            risk_level,
            confirm_phrase,
            rollback_summary,
            ..
        } => {
            // Humanized but safety-preserving confirmation
            let risk_str = risk_level.display();
            let lines = format!(
                "{} Action Required ({} Risk)\n\
                 What will change: {}\n\
                 Rollback: {}\n\
                 To proceed, type exactly: {}",
                "[service desk]".green(),
                risk_str,
                change_description,
                rollback_summary,
                confirm_phrase.bold()
            );
            Some(RenderedLineV72::new(lines))
        }

        EventDataV72::Warning {
            message_human,
            category,
            ..
        } => {
            // Show humanized warning, hide technical details
            if matches!(
                category,
                WarningCategoryV72::Parse | WarningCategoryV72::Fallback
            ) {
                return None; // Hide parse/fallback warnings in human mode
            }
            let line = format!("{} Note: {}", "[anna]".cyan(), message_human.yellow());
            Some(RenderedLineV72::new(line))
        }

        EventDataV72::Phase { name } => {
            let sep = format!("----- {} -----", name);
            Some(RenderedLineV72::new(sep.dimmed().to_string()))
        }

        EventDataV72::Working { role, message } => {
            if !role.visible_in_human() {
                return None;
            }
            let tag = format!("[{}]", role.tag());
            let line = format!("{} {}", style_role_human(*role, &tag), message.dimmed());
            Some(RenderedLineV72::progress(line))
        }
    }
}

/// Render event for debug mode - full details
fn render_debug_v72(event: &TranscriptEventV72) -> Option<RenderedLineV72> {
    let ts = event.ts.format("%H:%M:%S%.3f");

    match &event.data {
        EventDataV72::UserMessage { text } => {
            let line = format!("{} {} {}", ts.dimmed(), "[you]".white(), text);
            Some(RenderedLineV72::new(line.to_string()))
        }

        EventDataV72::StaffMessage {
            role,
            content_human,
            content_debug,
            ..
        } => {
            let content = content_debug.as_deref().unwrap_or(content_human);
            let tag = format!("[{}]", role.tag());
            let line = format!(
                "{} {} {}",
                ts.dimmed(),
                style_role_debug(*role, &tag),
                content
            );
            Some(RenderedLineV72::new(line.to_string()))
        }

        EventDataV72::Evidence {
            evidence_id,
            tool_name,
            summary_debug,
            summary_human,
            duration_ms,
            ..
        } => {
            let summary = summary_debug.as_deref().unwrap_or(summary_human);
            let line = format!(
                "{} {} [{}] tool={} ({}ms) {}",
                ts.dimmed(),
                "[evidence]".green(),
                evidence_id.green(),
                tool_name.cyan(),
                duration_ms,
                summary
            );
            Some(RenderedLineV72::new(line.to_string()))
        }

        EventDataV72::ToolCall {
            tool_name, args, ..
        } => {
            let args_str = args
                .as_ref()
                .map(|a| format!(" args={}", a))
                .unwrap_or_default();
            let line = format!(
                "{} {} tool={}{}",
                ts.dimmed(),
                "[tool_call]".blue(),
                tool_name.cyan(),
                args_str
            );
            Some(RenderedLineV72::new(line.to_string()))
        }

        EventDataV72::ToolResult {
            tool_name,
            success,
            result_raw,
            duration_ms,
            ..
        } => {
            let status = if *success { "OK" } else { "FAIL" };
            let raw = result_raw.as_deref().unwrap_or("");
            let line = format!(
                "{} {} tool={} {} ({}ms) {}",
                ts.dimmed(),
                "[tool_result]".blue(),
                tool_name.cyan(),
                status,
                duration_ms,
                truncate(raw, 100)
            );
            Some(RenderedLineV72::new(line.to_string()))
        }

        EventDataV72::Classification {
            canonical_lines,
            parse_attempts,
            fallback_used,
            ..
        } => {
            let mut lines = vec![format!(
                "{} {} parse_attempts={} fallback={}",
                ts.dimmed(),
                "[classification]".yellow(),
                parse_attempts.unwrap_or(1),
                fallback_used
            )];

            if let Some(canonical) = canonical_lines {
                lines.push("CANONICAL:".to_string());
                for line in canonical {
                    lines.push(format!("  {}", line));
                }
            }
            Some(RenderedLineV72::new(lines.join("\n")))
        }

        EventDataV72::Reliability {
            score,
            rationale_full,
            rationale_short,
            uncited_claims,
        } => {
            let rationale = rationale_full.as_deref().unwrap_or(rationale_short);
            let mut line = format!(
                "{} {} score={} {}",
                ts.dimmed(),
                "[reliability]".magenta(),
                score,
                rationale
            );

            if let Some(uncited) = uncited_claims {
                if !uncited.is_empty() {
                    line.push_str(&format!("\n  uncited_claims: {:?}", uncited));
                }
            }
            Some(RenderedLineV72::new(line))
        }

        EventDataV72::Perf {
            total_ms,
            breakdown,
        } => {
            let mut line = format!("{} {} total={}ms", ts.dimmed(), "[perf]".dimmed(), total_ms);

            if let Some(b) = breakdown {
                if let Some(t) = b.translation_ms {
                    line.push_str(&format!(" translation={}ms", t));
                }
                if let Some(t) = b.tool_execution_ms {
                    line.push_str(&format!(" tools={}ms", t));
                }
                if let Some(t) = b.synthesis_ms {
                    line.push_str(&format!(" synthesis={}ms", t));
                }
                if let Some(t) = b.verification_ms {
                    line.push_str(&format!(" verification={}ms", t));
                }
            }
            Some(RenderedLineV72::new(line))
        }

        EventDataV72::Confirmation {
            change_description,
            risk_level,
            confirm_phrase,
            rollback_details,
            rollback_summary,
            ..
        } => {
            let rollback = rollback_details.as_deref().unwrap_or(rollback_summary);
            let line = format!(
                "{} {} risk={} change=\"{}\" confirm=\"{}\" rollback=\"{}\"",
                ts.dimmed(),
                "[confirmation]".red(),
                risk_level.display(),
                change_description,
                confirm_phrase,
                rollback
            );
            Some(RenderedLineV72::new(line))
        }

        EventDataV72::Warning {
            message_human,
            details_debug,
            category,
        } => {
            let details = details_debug.as_deref().unwrap_or(message_human);
            let line = format!(
                "{} {} category={:?} {}",
                ts.dimmed(),
                "WARN:".yellow(),
                category,
                details
            );
            Some(RenderedLineV72::new(line))
        }

        EventDataV72::Phase { name } => {
            let line = format!("{} {} {}", ts.dimmed(), "[phase]".blue(), name);
            Some(RenderedLineV72::new(line))
        }

        EventDataV72::Working { role, message } => {
            let tag = format!("[{}]", role.tag());
            let line = format!(
                "{} {} {}",
                ts.dimmed(),
                style_role_debug(*role, &tag),
                message
            );
            Some(RenderedLineV72::progress(line))
        }
    }
}

/// Render entire stream
pub fn render_stream_v72(
    stream: &TranscriptStreamV72,
    mode: TranscriptMode,
) -> Vec<RenderedLineV72> {
    stream
        .events()
        .iter()
        .filter_map(|e| render_event_v72(e, mode))
        .collect()
}

/// Render stream to string
pub fn render_to_string_v72(stream: &TranscriptStreamV72, mode: TranscriptMode) -> String {
    render_stream_v72(stream, mode)
        .into_iter()
        .map(|l| l.text)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Style role for human mode
fn style_role_human(role: RoleV72, text: &str) -> String {
    match role {
        RoleV72::ServiceDesk => text.green().to_string(),
        RoleV72::Anna => text.cyan().to_string(),
        RoleV72::Network
        | RoleV72::Storage
        | RoleV72::Performance
        | RoleV72::Audio
        | RoleV72::Graphics
        | RoleV72::Boot
        | RoleV72::Security
        | RoleV72::InfoDesk => text.green().to_string(),
        _ => text.dimmed().to_string(),
    }
}

/// Style role for debug mode
fn style_role_debug(role: RoleV72, text: &str) -> String {
    match role {
        RoleV72::Translator => text.yellow().to_string(),
        RoleV72::Junior => text.magenta().to_string(),
        RoleV72::Senior => text.red().to_string(),
        RoleV72::Annad => text.dimmed().to_string(),
        _ => style_role_human(role, text),
    }
}

/// Human-friendly reliability level
fn reliability_level(score: u8) -> &'static str {
    if score >= 85 {
        "High"
    } else if score >= 70 {
        "Good"
    } else if score >= 50 {
        "Moderate"
    } else {
        "Low"
    }
}

/// Truncate string for debug output
fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_hides_evidence_id() {
        let event = TranscriptEventV72::new(EventDataV72::Evidence {
            evidence_id: "E1".to_string(),
            tool_name: "hw_snapshot_summary".to_string(),
            human_label: "hardware inventory".to_string(),
            summary_human: "Intel i9-14900HX".to_string(),
            summary_debug: Some("cpu: Intel i9-14900HX, cores=24".to_string()),
            duration_ms: 42,
        });

        let rendered = render_human_v72(&event).unwrap();
        assert!(rendered.text.contains("hardware inventory"));
        assert!(rendered.text.contains("Intel i9-14900HX"));
        assert!(!rendered.text.contains("[E1]"));
        assert!(!rendered.text.contains("hw_snapshot_summary"));
    }

    #[test]
    fn test_debug_shows_evidence_id() {
        let event = TranscriptEventV72::new(EventDataV72::Evidence {
            evidence_id: "E1".to_string(),
            tool_name: "hw_snapshot_summary".to_string(),
            human_label: "hardware inventory".to_string(),
            summary_human: "Intel i9-14900HX".to_string(),
            summary_debug: Some("cpu: Intel i9-14900HX, cores=24".to_string()),
            duration_ms: 42,
        });

        let rendered = render_debug_v72(&event).unwrap();
        let stripped = super::super::validation::strip_ansi(&rendered.text);
        assert!(stripped.contains("[E1]"), "Expected [E1] in: {}", stripped);
        assert!(
            stripped.contains("hw_snapshot_summary"),
            "Expected hw_snapshot_summary in: {}",
            stripped
        );
        assert!(stripped.contains("42ms"), "Expected 42ms in: {}", stripped);
    }

    #[test]
    fn test_human_hides_parse_warnings() {
        let event = TranscriptEventV72::new(EventDataV72::Warning {
            message_human: "Could not fully parse request".to_string(),
            details_debug: Some("Parse error: Invalid Translator output format".to_string()),
            category: WarningCategoryV72::Parse,
        });

        let rendered = render_human_v72(&event);
        assert!(rendered.is_none()); // Parse warnings hidden in human mode
    }

    #[test]
    fn test_debug_shows_parse_warnings() {
        let event = TranscriptEventV72::new(EventDataV72::Warning {
            message_human: "Could not fully parse request".to_string(),
            details_debug: Some("Parse error: Invalid Translator output format".to_string()),
            category: WarningCategoryV72::Parse,
        });

        let rendered = render_debug_v72(&event).unwrap();
        assert!(rendered.text.contains("Parse error"));
    }

    #[test]
    fn test_fallback_humanized() {
        let event = TranscriptEventV72::new(EventDataV72::Classification {
            understood_human: "Looking up system information".to_string(),
            canonical_lines: Some(vec!["intent: question".to_string()]),
            parse_attempts: Some(3),
            fallback_used: true,
        });

        let human = render_human_v72(&event).unwrap();
        assert!(human.text.contains("house rules"));
        assert!(!human.text.contains("deterministic fallback"));
        assert!(!human.text.contains("parse_attempts"));

        let debug = render_debug_v72(&event).unwrap();
        assert!(debug.text.contains("parse_attempts=3"));
        assert!(debug.text.contains("fallback=true"));
    }

    #[test]
    fn test_confirmation_preserves_phrase() {
        let event = TranscriptEventV72::new(EventDataV72::Confirmation {
            change_description: "Install package foo".to_string(),
            risk_level: RiskLevelV72::Medium,
            confirm_phrase: "I understand the risks".to_string(),
            rollback_summary: "pacman -R foo".to_string(),
            rollback_details: Some("pacman -R foo && systemctl restart bar".to_string()),
        });

        let human = render_human_v72(&event).unwrap();
        assert!(human.text.contains("I understand the risks")); // Phrase unchanged
        assert!(human.text.contains("Medium Risk"));

        let debug = render_debug_v72(&event).unwrap();
        assert!(debug.text.contains("I understand the risks")); // Phrase unchanged
    }

    #[test]
    fn test_reliability_format() {
        let event = TranscriptEventV72::new(EventDataV72::Reliability {
            score: 85,
            rationale_short: "good evidence coverage".to_string(),
            rationale_full: Some("Direct evidence from hardware snapshot".to_string()),
            uncited_claims: None,
        });

        let human = render_human_v72(&event).unwrap();
        assert!(human.text.contains("85%"));
        assert!(human.text.contains("High"));
        assert!(human.text.contains("good evidence coverage"));
    }
}
