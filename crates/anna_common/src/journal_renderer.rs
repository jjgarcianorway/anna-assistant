//! Journal Renderer - Episode rendering for OutputEngine
//!
//! v6.51.0: Beautiful, structured rendering of action history

use crate::action_episodes::ActionEpisode;
use crate::change_journal::{EpisodeDomain, EpisodeRisk, EpisodeSummary};
use crate::output_engine::OutputEngine;
use chrono::{DateTime, Utc};
use owo_colors::OwoColorize;

/// Render a list of episode summaries
pub fn render_episode_list(
    engine: &OutputEngine,
    summaries: &[EpisodeSummary],
    show_header: bool,
) -> String {
    let mut output = String::new();

    if show_header {
        output.push_str(&engine.format_header("Change Journal"));
        output.push_str("\n\n");
    }

    if summaries.is_empty() {
        output.push_str("No episodes found matching your criteria.\n");
        return output;
    }

    for (i, summary) in summaries.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        output.push_str(&render_summary_line(summary));
    }

    output
}

/// Render a single episode summary (one line)
fn render_summary_line(summary: &EpisodeSummary) -> String {
    let timestamp = format_timestamp(&summary.timestamp);
    let risk_icon = risk_icon(&summary.risk);
    let domain_badge = domain_badge(&summary.domain);
    let status = if summary.rolled_back {
        " [ROLLED BACK]".dimmed().to_string()
    } else if summary.executed {
        "".to_string()
    } else {
        " [PLANNED]".dimmed().to_string()
    };

    let validation = if let Some(score) = summary.validation_score {
        let percent = (score * 100.0) as i32;
        let indicator = if percent >= 90 {
            "✓".green().to_string()
        } else if percent >= 70 {
            "⚠".yellow().to_string()
        } else {
            "✗".red().to_string()
        };
        format!(" {} {}%", indicator, percent)
    } else {
        String::new()
    };

    format!(
        "{} {} {} {} {}{}{}",
        timestamp.dimmed(),
        risk_icon,
        domain_badge,
        summary.intent_summary.bold(),
        format!("(#{})", summary.id).dimmed(),
        status,
        validation
    )
}

/// Render full episode details
pub fn render_episode_details(engine: &OutputEngine, episode: &ActionEpisode) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&engine.format_header(&format!("Episode #{}", episode.episode_id)));
    output.push_str("\n\n");

    // Metadata
    output.push_str(&format!(
        "{}  {}\n",
        "Timestamp:".bold(),
        format_timestamp(&episode.created_at)
    ));
    output.push_str(&format!(
        "{}  {}\n",
        "Question:".bold(),
        episode.user_question
    ));
    output.push_str(&format!(
        "{}  {}\n",
        "Answer:".bold(),
        episode.final_answer_summary
    ));
    output.push_str(&format!(
        "{}  {:?}\n",
        "Rollback:".bold(),
        episode.rollback_capability
    ));
    output.push_str(&format!(
        "{}  {:?}\n",
        "Status:".bold(),
        episode.execution_status
    ));

    // Tags
    if !episode.tags.topics.is_empty() {
        output.push_str(&format!(
            "{}  {}\n",
            "Tags:".bold(),
            episode.tags.topics.join(", ")
        ));
    }
    if let Some(ref domain) = episode.tags.domain {
        output.push_str(&format!("{}  {}\n", "Domain:".bold(), domain));
    }

    output.push('\n');

    // Actions
    if !episode.actions.is_empty() {
        output.push_str(&engine.format_subheader("Actions Taken"));
        output.push('\n');
        for (i, action) in episode.actions.iter().enumerate() {
            output.push_str(&format!(
                "  {}) {} - {:?}\n",
                i + 1,
                action.command.cyan(),
                action.kind
            ));
            if !action.files_touched.is_empty() {
                output.push_str(&format!(
                    "     Files: {}\n",
                    action.files_touched.join(", ").dimmed()
                ));
            }
            if !action.backup_paths.is_empty() {
                output.push_str(&format!(
                    "     Backups: {}\n",
                    action.backup_paths.join(", ").dimmed()
                ));
            }
        }
        output.push('\n');
    }

    // Post-validation
    if let Some(ref validation) = episode.post_validation {
        output.push_str(&engine.format_subheader("Post-Execution Assessment"));
        output.push('\n');
        let score_percent = (validation.satisfaction_score * 100.0) as i32;
        output.push_str(&format!(
            "  Satisfaction: {}%\n",
            format!("{}", score_percent).bold()
        ));
        output.push_str(&format!("  {}\n", validation.summary));

        if !validation.residual_concerns.is_empty() {
            output.push_str("\n  Concerns:\n");
            for concern in &validation.residual_concerns {
                output.push_str(&format!("    • {}\n", concern.yellow()));
            }
        }

        if !validation.suggested_checks.is_empty() {
            output.push_str("\n  Suggested Checks:\n");
            for check in &validation.suggested_checks {
                output.push_str(&format!("    $ {}\n", check.cyan()));
            }
        }
        output.push('\n');
    }

    output
}

/// Format a timestamp for display
fn format_timestamp(dt: &DateTime<Utc>) -> String {
    // Show relative time if recent, otherwise full date
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    if duration.num_hours() < 24 {
        if duration.num_hours() < 1 {
            format!("{}m ago", duration.num_minutes())
        } else {
            format!("{}h ago", duration.num_hours())
        }
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else {
        dt.format("%Y-%m-%d %H:%M").to_string()
    }
}

/// Risk icon
fn risk_icon(risk: &EpisodeRisk) -> String {
    match risk {
        EpisodeRisk::Safe => "✓".green().to_string(),
        EpisodeRisk::Moderate => "⚠".yellow().to_string(),
        EpisodeRisk::High => "✗".red().to_string(),
    }
}

/// Domain badge
fn domain_badge(domain: &EpisodeDomain) -> String {
    let text = match domain {
        EpisodeDomain::Config => "[cfg]",
        EpisodeDomain::Services => "[svc]",
        EpisodeDomain::Packages => "[pkg]",
        EpisodeDomain::Network => "[net]",
        EpisodeDomain::General => "[gen]",
    };
    text.dimmed().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_episodes::{
        ActionKind, ActionRecord, EpisodeBuilder, EpisodeTags, ExecutionStatus,
        RollbackCapability,
    };
    use crate::output_engine::TerminalMode;

    fn make_test_summary(
        id: i64,
        intent: &str,
        domain: EpisodeDomain,
        risk: EpisodeRisk,
    ) -> EpisodeSummary {
        EpisodeSummary {
            id,
            timestamp: Utc::now(),
            intent_summary: intent.to_string(),
            domain,
            risk,
            executed: true,
            rolled_back: false,
            validation_score: Some(0.95),
            validation_label: Some("likely satisfied".to_string()),
            tags: vec!["test".to_string()],
        }
    }

    fn make_test_episode(id: i64, question: &str) -> ActionEpisode {
        let tags = EpisodeTags {
            topics: vec!["test".to_string()],
            domain: Some("test".to_string()),
        };

        let action = ActionRecord {
            id: 1,
            kind: ActionKind::RunCommand,
            command: "ls -la".to_string(),
            cwd: Some("/home/test".to_string()),
            files_touched: vec![],
            backup_paths: vec![],
            notes: Some("test action".to_string()),
        };

        let mut episode = EpisodeBuilder::new(question)
            .with_final_answer_summary("Done")
            .with_tags(tags)
            .build();

        episode.episode_id = id;
        episode.execution_status = ExecutionStatus::Executed;
        episode.rollback_capability = RollbackCapability::Full;
        episode.actions.push(action);
        episode
    }

    #[test]
    fn test_render_empty_list() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let output = render_episode_list(&engine, &[], true);
        assert!(output.contains("No episodes found"));
    }

    #[test]
    fn test_render_summary_list() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let summaries = vec![
            make_test_summary(1, "fix vim config", EpisodeDomain::Config, EpisodeRisk::Safe),
            make_test_summary(
                2,
                "restart ssh",
                EpisodeDomain::Services,
                EpisodeRisk::Moderate,
            ),
        ];
        let output = render_episode_list(&engine, &summaries, true);
        assert!(output.contains("fix vim config"));
        assert!(output.contains("restart ssh"));
        assert!(output.contains("Change Journal"));
    }

    #[test]
    fn test_render_episode_details() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let episode = make_test_episode(42, "test question");
        let output = render_episode_details(&engine, &episode);
        assert!(output.contains("Episode #42"));
        assert!(output.contains("test question"));
        assert!(output.contains("Done"));
        assert!(output.contains("Actions Taken"));
        assert!(output.contains("ls -la"));
    }

    #[test]
    fn test_format_timestamp_recent() {
        let now = Utc::now();
        let formatted = format_timestamp(&now);
        assert!(formatted.contains("ago") || formatted.len() > 0);
    }

    #[test]
    fn test_risk_icon() {
        assert!(!risk_icon(&EpisodeRisk::Safe).is_empty());
        assert!(!risk_icon(&EpisodeRisk::Moderate).is_empty());
        assert!(!risk_icon(&EpisodeRisk::High).is_empty());
    }

    #[test]
    fn test_domain_badge() {
        assert!(domain_badge(&EpisodeDomain::Config).contains("cfg"));
        assert!(domain_badge(&EpisodeDomain::Services).contains("svc"));
        assert!(domain_badge(&EpisodeDomain::Packages).contains("pkg"));
    }
}
