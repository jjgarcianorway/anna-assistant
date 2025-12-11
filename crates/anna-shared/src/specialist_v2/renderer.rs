//! Response renderer for specialist outputs (v0.0.421).
//!
//! Converts SpecialistResponseV2 to user-friendly output.
//! Never exposes:
//! - Raw JSON
//! - Parse errors
//! - Internal errors

use super::answer::FindingSeverity;
use super::schema::{SpecialistResponseV2, SpecialistStatus};

/// Rendered answer ready for display
#[derive(Debug, Clone)]
pub struct RenderedAnswer {
    /// Main answer text (headline)
    pub headline: String,
    /// Key findings formatted as a list
    pub findings: Vec<String>,
    /// Recommended actions formatted
    pub actions: Vec<String>,
    /// Source citations
    pub sources: Vec<String>,
    /// Extra notes (if any)
    pub notes: Option<String>,
    /// Status indicator for UI
    pub status_indicator: StatusIndicator,
}

/// Status indicator for UI rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusIndicator {
    /// Everything worked
    Success,
    /// Partial answer
    Partial,
    /// Couldn't answer
    Failed,
}

impl RenderedAnswer {
    /// Format as simple text for terminal output
    pub fn as_text(&self) -> String {
        let mut output = String::new();

        // Headline
        output.push_str(&self.headline);
        output.push('\n');

        // Findings
        if !self.findings.is_empty() {
            output.push('\n');
            for finding in &self.findings {
                output.push_str("  • ");
                output.push_str(finding);
                output.push('\n');
            }
        }

        // Actions
        if !self.actions.is_empty() {
            output.push_str("\nNext steps:\n");
            for action in &self.actions {
                output.push_str("  → ");
                output.push_str(action);
                output.push('\n');
            }
        }

        // Sources
        if !self.sources.is_empty() {
            output.push_str("\nSources: ");
            output.push_str(&self.sources.join(", "));
            output.push('\n');
        }

        // Notes
        if let Some(notes) = &self.notes {
            output.push_str("\n");
            output.push_str(notes);
            output.push('\n');
        }

        output
    }

    /// Format as markdown
    pub fn as_markdown(&self) -> String {
        let mut output = String::new();

        // Headline
        output.push_str(&self.headline);
        output.push_str("\n\n");

        // Findings as table if multiple
        if self.findings.len() > 1 {
            output.push_str("| Finding | Value |\n|---------|-------|\n");
            for finding in &self.findings {
                // Try to split on ": " for table format
                if let Some((label, value)) = finding.split_once(": ") {
                    output.push_str(&format!("| {} | {} |\n", label, value));
                } else {
                    output.push_str(&format!("| {} | |\n", finding));
                }
            }
        } else if !self.findings.is_empty() {
            for finding in &self.findings {
                output.push_str("- ");
                output.push_str(finding);
                output.push('\n');
            }
        }

        // Actions
        if !self.actions.is_empty() {
            output.push_str("\n**Next steps:**\n");
            for action in &self.actions {
                output.push_str("- ");
                output.push_str(action);
                output.push('\n');
            }
        }

        // Sources
        if !self.sources.is_empty() {
            output.push_str("\n*Sources:* ");
            output.push_str(&self.sources.join(", "));
            output.push('\n');
        }

        output
    }
}

/// Render a specialist response into user-friendly output
pub fn render_response(response: &SpecialistResponseV2) -> RenderedAnswer {
    let headline = render_headline(response);
    let findings = render_findings(response);
    let actions = render_actions(response);
    let sources = render_sources(response);
    let notes = response.notes.clone();

    let status_indicator = match response.status {
        SpecialistStatus::Ok if response.has_direct_answer() => StatusIndicator::Success,
        SpecialistStatus::Ok => StatusIndicator::Partial,
        SpecialistStatus::InsufficientEvidence => StatusIndicator::Partial,
        SpecialistStatus::Error => StatusIndicator::Failed,
    };

    RenderedAnswer {
        headline,
        findings,
        actions,
        sources,
        notes,
        status_indicator,
    }
}

/// Render the headline from direct_answer or construct from findings
fn render_headline(response: &SpecialistResponseV2) -> String {
    // Prefer direct_answer.short_text
    if let Some(ref answer) = response.direct_answer {
        if !answer.short_text.is_empty() {
            return answer.short_text.clone();
        }
    }

    // Fall back to first key finding
    if let Some(finding) = response.key_findings.first() {
        return format!("{}: {}", finding.label, finding.value);
    }

    // Fall back to notes
    if let Some(ref notes) = response.notes {
        return notes.clone();
    }

    // Last resort
    match response.status {
        SpecialistStatus::Ok => "Request completed.".to_string(),
        SpecialistStatus::InsufficientEvidence => {
            "I couldn't find enough information to answer this.".to_string()
        }
        SpecialistStatus::Error => "Something went wrong while processing your request.".to_string(),
    }
}

/// Render key findings as formatted strings
fn render_findings(response: &SpecialistResponseV2) -> Vec<String> {
    response
        .key_findings
        .iter()
        .map(|f| {
            let severity_prefix = match f.severity {
                Some(FindingSeverity::Critical) => "⚠️ ",
                Some(FindingSeverity::Warning) => "⚡ ",
                _ => "",
            };
            format!("{}{}: {}", severity_prefix, f.label, f.value)
        })
        .collect()
}

/// Render recommended actions as formatted strings
fn render_actions(response: &SpecialistResponseV2) -> Vec<String> {
    response
        .recommended_actions
        .iter()
        .map(|a| {
            let risk_suffix = match a.risk_level {
                super::answer::RiskLevel::High => " [high risk]",
                super::answer::RiskLevel::Medium => " [medium risk]",
                super::answer::RiskLevel::Low => "",
            };
            format!("{}{}", a.summary, risk_suffix)
        })
        .collect()
}

/// Render citations as source strings
fn render_sources(response: &SpecialistResponseV2) -> Vec<String> {
    response
        .citations
        .iter()
        .map(|c| {
            // Format probe:name as "probe name"
            if let Some(name) = c.strip_prefix("probe:") {
                format!("probe {}", name)
            } else if let Some(name) = c.strip_prefix("man:") {
                format!("man {}", name)
            } else if let Some(name) = c.strip_prefix("archwiki:") {
                format!("wiki:{}", name)
            } else {
                c.clone()
            }
        })
        .collect()
}

/// Render a friendly error message (never "Failed to parse specialist response")
pub fn render_friendly_error(internal_error: &str, probe_data: &[(&str, &str)]) -> RenderedAnswer {
    // Log the actual error but don't show it
    tracing::warn!("Specialist error (hidden from user): {}", internal_error);

    let headline = if probe_data.is_empty() {
        "I had trouble processing this request. Please try again.".to_string()
    } else {
        format!(
            "I had trouble understanding my specialist's reply. Here's what I found from the probes: {}",
            probe_data.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(", ")
        )
    };

    let findings: Vec<String> = probe_data
        .iter()
        .take(3)
        .map(|(name, value)| {
            let truncated = if value.len() > 100 {
                format!("{}...", &value[..100])
            } else {
                value.to_string()
            };
            format!("{}: {}", name, truncated)
        })
        .collect();

    RenderedAnswer {
        headline,
        findings,
        actions: vec![],
        sources: probe_data.iter().map(|(k, _)| format!("probe {}", k)).collect(),
        notes: None,
        status_indicator: StatusIndicator::Partial,
    }
}

/// Render a timeout message
pub fn render_timeout(probe_data: &[(&str, &str)]) -> RenderedAnswer {
    render_friendly_error("Specialist timed out", probe_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_v2::answer::{DirectAnswer, KeyFinding, RecommendedAction, RiskLevel};

    #[test]
    fn test_render_simple_answer() {
        let response = SpecialistResponseV2::ok()
            .with_direct_answer(DirectAnswer::simple("17.0 GiB available"))
            .with_citation("probe:free");

        let rendered = render_response(&response);
        assert_eq!(rendered.headline, "17.0 GiB available");
        assert_eq!(rendered.status_indicator, StatusIndicator::Success);
    }

    #[test]
    fn test_render_with_findings() {
        let response = SpecialistResponseV2::ok()
            .with_direct_answer(DirectAnswer::no("there are no failed services."))
            .with_finding(KeyFinding::info("services_checked", "42"));

        let rendered = render_response(&response);
        assert!(!rendered.findings.is_empty());
    }

    #[test]
    fn test_render_with_actions() {
        let response = SpecialistResponseV2::ok()
            .with_direct_answer(DirectAnswer::simple("Disk is 95% full"))
            .with_action(RecommendedAction::medium_risk(
                "cleanup",
                "Remove old log files to free space",
            ));

        let rendered = render_response(&response);
        assert!(!rendered.actions.is_empty());
        assert!(rendered.actions[0].contains("[medium risk]"));
    }

    #[test]
    fn test_render_as_text() {
        let response = SpecialistResponseV2::ok()
            .with_direct_answer(DirectAnswer::simple("System uptime: 5 days"))
            .with_citation("probe:uptime");

        let rendered = render_response(&response);
        let text = rendered.as_text();

        assert!(text.contains("System uptime: 5 days"));
        assert!(text.contains("Sources:"));
    }

    #[test]
    fn test_friendly_error() {
        let rendered = render_friendly_error(
            "JSON parse error: unexpected token",
            &[("free", "Mem: 32Gi 15Gi 17Gi")],
        );

        assert!(!rendered.headline.contains("parse"));
        assert!(!rendered.headline.contains("JSON"));
        assert_eq!(rendered.status_indicator, StatusIndicator::Partial);
    }
}
