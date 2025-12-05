//! Rule-based action recommendations (ADVISE).
//!
//! Generates actionable recommendations from report health checks.
//! All rules pinned to evidence thresholds - no LLM speculation.

use crate::report::{HealthItem, HealthSeverity, SystemReport};
use serde::{Deserialize, Serialize};

/// Recommendation severity (matches health severity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationSeverity {
    Warning = 1,
    Critical = 2,
}

impl From<HealthSeverity> for Option<RecommendationSeverity> {
    fn from(severity: HealthSeverity) -> Self {
        match severity {
            HealthSeverity::Warning => Some(RecommendationSeverity::Warning),
            HealthSeverity::Critical => Some(RecommendationSeverity::Critical),
            HealthSeverity::Ok => None,
        }
    }
}

/// A single actionable recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Severity level (Warning or Critical)
    pub severity: RecommendationSeverity,
    /// Short imperative action (e.g., "Clear disk space on /")
    pub action: String,
    /// Reference to the health check that triggered this
    pub evidence_ref: String,
    /// Optional example command to fix the issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_hint: Option<String>,
}

/// Collection of recommendations from a report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Recommendations {
    /// List of recommendations sorted by severity (critical first)
    pub items: Vec<Recommendation>,
}

impl Recommendations {
    /// Check if there are any recommendations
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Count of critical recommendations
    pub fn critical_count(&self) -> usize {
        self.items
            .iter()
            .filter(|r| r.severity == RecommendationSeverity::Critical)
            .count()
    }

    /// Count of warning recommendations
    pub fn warning_count(&self) -> usize {
        self.items
            .iter()
            .filter(|r| r.severity == RecommendationSeverity::Warning)
            .count()
    }
}

/// Generate recommendations from a system report
pub fn generate_recommendations(report: &SystemReport) -> Recommendations {
    let mut items = Vec::new();

    for check in &report.health_checks {
        if let Some(rec) = recommendation_for_check(check) {
            items.push(rec);
        }
    }

    // Sort by severity (critical first), then by evidence_ref for stability
    items.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| a.evidence_ref.cmp(&b.evidence_ref))
    });

    Recommendations { items }
}

/// Generate a recommendation for a single health check
fn recommendation_for_check(check: &HealthItem) -> Option<Recommendation> {
    // Only generate recommendations for non-OK severity
    let severity = match check.severity {
        HealthSeverity::Critical => RecommendationSeverity::Critical,
        HealthSeverity::Warning => RecommendationSeverity::Warning,
        HealthSeverity::Ok => return None,
    };

    // Route to specific recommendation generator based on check ID prefix
    if check.id.starts_with("disk_") {
        Some(disk_recommendation(check, severity))
    } else if check.id == "memory" {
        Some(memory_recommendation(check, severity))
    } else if check.id == "services" {
        Some(services_recommendation(check, severity))
    } else {
        None
    }
}

/// Generate disk space recommendation
fn disk_recommendation(check: &HealthItem, severity: RecommendationSeverity) -> Recommendation {
    // Reconstruct mount path from ID: disk_root -> /, disk_home -> /home
    let mount = check
        .id
        .strip_prefix("disk_")
        .map(|s| {
            if s == "root" {
                "/".to_string()
            } else {
                format!("/{}", s.replace('_', "/"))
            }
        })
        .unwrap_or_else(|| "/".to_string());

    let action = match severity {
        RecommendationSeverity::Critical => format!("Urgently free disk space on {}", mount),
        RecommendationSeverity::Warning => format!("Consider freeing disk space on {}", mount),
    };

    let command_hint = if mount == "/" {
        Some("sudo du -sh /* 2>/dev/null | sort -hr | head -10".to_string())
    } else {
        Some(format!(
            "sudo du -sh {}/* 2>/dev/null | sort -hr | head -10",
            mount
        ))
    };

    Recommendation {
        severity,
        action,
        evidence_ref: check.id.clone(),
        command_hint,
    }
}

/// Generate memory recommendation
fn memory_recommendation(check: &HealthItem, severity: RecommendationSeverity) -> Recommendation {
    Recommendation {
        severity,
        action: "Investigate high memory usage".to_string(),
        evidence_ref: check.id.clone(),
        command_hint: Some("ps aux --sort=-%mem | head -10".to_string()),
    }
}

/// Generate services recommendation
fn services_recommendation(check: &HealthItem, severity: RecommendationSeverity) -> Recommendation {
    // Extract failed service names from the claim
    let action = if check.claim.contains(',') {
        "Restart failed services".to_string()
    } else {
        // Single service - extract name
        let service = check
            .claim
            .split(':')
            .nth(1)
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim())
            .unwrap_or("failed service");
        format!("Investigate and restart {}", service)
    };

    Recommendation {
        severity,
        action,
        evidence_ref: check.id.clone(),
        command_hint: Some("systemctl --failed".to_string()),
    }
}

// =============================================================================
// Format recommendations
// =============================================================================

/// Format recommendations as plain text
pub fn format_recommendations_text(recs: &Recommendations) -> String {
    if recs.is_empty() {
        return "No actions recommended.".to_string();
    }

    let mut out = String::new();
    out.push_str("RECOMMENDED ACTIONS\n");
    out.push_str("===================\n\n");

    for rec in &recs.items {
        let severity_label = match rec.severity {
            RecommendationSeverity::Critical => "[CRITICAL]",
            RecommendationSeverity::Warning => "[WARNING]",
        };

        out.push_str(&format!("{} {}\n", severity_label, rec.action));
        out.push_str(&format!("  Evidence: {}\n", rec.evidence_ref));
        if let Some(hint) = &rec.command_hint {
            out.push_str(&format!("  Try: {}\n", hint));
        }
        out.push('\n');
    }

    out
}

/// Format recommendations as markdown
pub fn format_recommendations_markdown(recs: &Recommendations) -> String {
    if recs.is_empty() {
        return "*No actions recommended.*".to_string();
    }

    let mut out = String::new();
    out.push_str("## Recommended Actions\n\n");

    for rec in &recs.items {
        let severity_label = match rec.severity {
            RecommendationSeverity::Critical => "**CRITICAL**",
            RecommendationSeverity::Warning => "WARNING",
        };

        out.push_str(&format!("- {} {}\n", severity_label, rec.action));
        out.push_str(&format!("  - *Evidence*: `{}`\n", rec.evidence_ref));
        if let Some(hint) = &rec.command_hint {
            out.push_str(&format!("  - *Try*: `{}`\n", hint));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_severity_ordering() {
        assert!(RecommendationSeverity::Critical > RecommendationSeverity::Warning);
    }

    #[test]
    fn test_empty_recommendations() {
        let recs = Recommendations::default();
        assert!(recs.is_empty());
        assert_eq!(recs.critical_count(), 0);
        assert_eq!(recs.warning_count(), 0);
    }

    #[test]
    fn test_format_empty_text() {
        let recs = Recommendations::default();
        let text = format_recommendations_text(&recs);
        assert!(text.contains("No actions recommended"));
    }

    #[test]
    fn test_format_empty_markdown() {
        let recs = Recommendations::default();
        let md = format_recommendations_markdown(&recs);
        assert!(md.contains("No actions recommended"));
    }
}
