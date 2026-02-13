//! Early short-circuit handlers for the Ralph streaming loop.
//! These handlers intercept specific request types before the full investigation loop.

use anna_shared::exposure::ExposureGate;
use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::info;

use super::streaming_helpers::{push_and_send, send_done};

/// Handle "remind me in X" requests directly.
pub async fn handle_reminder_request<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::scheduler::{parse_reminder, ScheduledTask, TaskStore};

    let Some((message, when)) = parse_reminder(question) else {
        return Ok(None);
    };

    // Create and save the reminder
    let task = ScheduledTask::reminder(&message, when);
    let mut store = TaskStore::load();
    store.add(task);
    if let Err(e) = store.save() {
        tracing::warn!("Failed to save reminder: {}", e);
    }

    // Format when for display
    let duration = when - chrono::Utc::now();
    let when_str = if duration.num_hours() >= 1 {
        format!("{} hour(s)", duration.num_hours())
    } else {
        format!("{} minute(s)", duration.num_minutes())
    };

    let answer = format!("Got it. I'll remind you to \"{}\" in {}.", message, when_str);

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

    let result = AskResult {
        answer,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(1.0),
    };
    send_done(writer, &result).await?;

    Ok(Some(result))
}

/// Handle morning briefing setup requests.
pub async fn handle_morning_briefing_request<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::scheduler::{parse_morning_briefing, ScheduledTask, TaskStore};
    use chrono::Timelike;

    let Some(time) = parse_morning_briefing(question) else {
        return Ok(None);
    };

    // Create and save the morning briefing task
    let mut store = TaskStore::load();

    // Remove existing morning briefing if any
    store.remove_morning_briefing();

    // v0.3.156: No username in Ralph (CLI context)
    let task = ScheduledTask::morning_briefing(time, None);
    store.add(task);

    if let Err(e) = store.save() {
        tracing::warn!("Failed to save morning briefing: {}", e);
    }

    let answer = format!(
        "Morning briefing set up. I'll send you a daily health check at {:02}:{:02}.",
        time.hour(), time.minute()
    );

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

    let result = AskResult {
        answer,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(1.0),
    };
    send_done(writer, &result).await?;

    Ok(Some(result))
}

/// v0.3.120: Handle natural language system queries (health, problems, status).
pub async fn handle_natural_system_query<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::natural_query::handle_natural_query;

    let Some(answer) = handle_natural_query(question) else {
        return Ok(None);
    };

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

    let result = AskResult {
        answer,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(1.0),
    };
    send_done(writer, &result).await?;

    Ok(Some(result))
}

/// v0.3.123: Handle well-known error patterns with instant, high-confidence answers.
pub async fn handle_pattern_match<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::patterns::{match_error_pattern, format_pattern_answer};

    let Some(pattern) = match_error_pattern(question) else {
        return Ok(None);
    };

    info!("Pattern match: {} (confidence={:.2})", pattern.pattern_id, pattern.confidence);

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;

    // Show that we recognized the pattern
    push_and_send(writer, &mut dialogue, StepType::InvestigationStart,
        format!("Recognized common issue: {}", pattern.pattern_id), gate).await?;

    let answer = format_pattern_answer(&pattern);
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

    let result = AskResult {
        answer,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(pattern.confidence),
    };
    send_done(writer, &result).await?;

    Ok(Some(result))
}

/// v0.3.121: Handle multi-domain questions with parallel agent investigation.
pub async fn handle_multi_agent_query<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::config::AnnaConfig;
    use crate::orchestrator::{should_use_multi_agent, TaskAnalysis};

    let config = AnnaConfig::load().unwrap_or_default();

    // Only proceed if multi-agent mode is enabled and this is a multi-domain question
    if !should_use_multi_agent(question, &config) {
        return Ok(None);
    }

    let analysis = TaskAnalysis::analyze(question, &config);
    info!("Multi-agent mode: {} domains detected, using parallel investigation",
          analysis.domains.len());

    // Build orchestrator and solve
    use crate::agents::build_default_registry;
    use crate::orchestrator::AgentOrchestrator;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let registry = Arc::new(RwLock::new(build_default_registry()));
    let orchestrator = AgentOrchestrator::with_defaults(registry);
    let result = orchestrator.solve(question).await;

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;

    // Report parallel investigation
    let domain_list = analysis.domains.join(", ");
    push_and_send(writer, &mut dialogue, StepType::InvestigationStart,
        format!("Parallel investigation across: {}", domain_list), gate).await?;

    let answer = result.answer.unwrap_or_else(|| "Could not complete investigation.".to_string());
    let confidence = result.confidence;

    push_and_send(writer, &mut dialogue, StepType::InvestigationComplete,
        format!("{} agents contributed", analysis.domains.len()), gate).await?;
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

    let ask_result = AskResult {
        answer,
        success: result.success,
        iterations: 1,
        commands_executed: vec![],
        dialogue,
        needs_clarification: confidence < 0.3,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(confidence),
    };
    send_done(writer, &ask_result).await?;

    Ok(Some(ask_result))
}
