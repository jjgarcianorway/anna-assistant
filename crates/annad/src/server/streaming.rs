//! Streaming request handling for real-time responses.
//! v0.0.993: Added automatic fix detection and offer
//! v0.0.998: Added configuration recipes
//! v0.0.998: Added Hollywood IT teams experience

use anna_shared::rpc::{DialogueStep, RpcRequest, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

use anna_shared::config::AnnaConfig;
use crate::autofix::{
    find_autofix, check_autofix_needed, format_autofix_offer, execute_autofix,
    set_pending_autofix, take_pending_autofix, is_yes_response, is_no_response,
    get_fix_history_summary,
};
use crate::core_loop::execute_question_streaming;
use crate::ralph;
use crate::recipes;
use crate::state::SharedState;
use crate::team_speak;

use super::alerts::get_pending_alerts;

/// Track pending recipe confirmations by session
use std::collections::HashMap;
use std::sync::RwLock;

static PENDING_RECIPES: RwLock<Option<HashMap<String, String>>> = RwLock::new(None);

fn set_pending_recipe(session_id: &str, recipe_id: &str) {
    if let Ok(mut guard) = PENDING_RECIPES.write() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(session_id.to_string(), recipe_id.to_string());
    }
}

fn take_pending_recipe(session_id: &str) -> Option<String> {
    if let Ok(mut guard) = PENDING_RECIPES.write() {
        if let Some(map) = guard.as_mut() {
            return map.remove(session_id);
        }
    }
    None
}

/// Handle a streaming AskStreaming request
pub async fn handle_streaming_request(
    request: RpcRequest,
    state: SharedState,
    mut writer: tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    let question = request
        .params
        .as_ref()
        .and_then(|p| p.get("question"))
        .and_then(|q| q.as_str())
        .unwrap_or("");

    // Extract session_id from params (client generates it)
    let session_id = request
        .params
        .as_ref()
        .and_then(|p| p.get("session_id"))
        .and_then(|s| s.as_str())
        .unwrap_or("default");

    // v0.2.8: Track response time for RPG stats
    let start_time = std::time::Instant::now();

    if question.is_empty() {
        let response = StreamingResponse::Error {
            message: "Missing 'question' parameter".to_string(),
        };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    }

    // v0.0.994: Check if this is a response to a pending autofix
    if let Some(pending_fix) = take_pending_autofix(session_id) {
        if is_yes_response(question) {
            info!("Executing autofix {} (user confirmed)", pending_fix.id);

            // Save fix_cmd before moving pending_fix
            let fix_cmd = pending_fix.fix_cmd.to_string();

            // Show what we're doing
            let step = DialogueStep {
                step_type: StepType::UnderstandingCheck,
                content: format!("Running fix: {}", fix_cmd),
            };
            let response = StreamingResponse::Step { step };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            // Execute the fix
            let result_msg = match execute_autofix(pending_fix) {
                Ok(msg) => msg,
                Err(e) => format!("Fix failed: {}", e),
            };

            // Return the result
            let result = anna_shared::rpc::AskResult {
                answer: result_msg,
                success: true,
                iterations: 0,
                commands_executed: vec![fix_cmd],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        } else if is_no_response(question) {
            info!("Autofix {} cancelled by user", pending_fix.id);

            // Show cancellation message
            let step = DialogueStep {
                step_type: StepType::FinalAnswer,
                content: "No problem, I won't make any changes.".to_string(),
            };
            let response = StreamingResponse::Step { step };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            let result = anna_shared::rpc::AskResult {
                answer: "No problem, I won't make any changes.".to_string(),
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        }
        // Not a yes/no - continue with normal processing
    }

    // v0.0.997: Check if user is asking about fix history
    if is_fix_history_question(question) {
        info!("User asking about fix history");
        let summary = get_fix_history_summary();

        let step = DialogueStep {
            step_type: StepType::FinalAnswer,
            content: summary.clone(),
        };
        let response = StreamingResponse::Step { step };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;

        let result = anna_shared::rpc::AskResult {
            answer: summary,
            success: true,
            iterations: 0,
            commands_executed: vec![],
            dialogue: vec![],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
        };
        let done = StreamingResponse::Done { result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    }

    // v0.0.998: Check if this is a response to a pending recipe
    if let Some(pending_recipe_id) = take_pending_recipe(session_id) {
        if is_yes_response(question) {
            info!("Executing recipe {} (user confirmed)", pending_recipe_id);
            let result = recipes::execute_confirmed_recipe(&pending_recipe_id);

            let step = DialogueStep {
                step_type: StepType::FinalAnswer,
                content: result.message.clone(),
            };
            let response = StreamingResponse::Step { step };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            let ask_result = anna_shared::rpc::AskResult {
                answer: result.message,
                success: result.success,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
            };
            let done = StreamingResponse::Done { result: ask_result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        } else if is_no_response(question) {
            info!("Recipe {} cancelled by user", pending_recipe_id);
            let step = DialogueStep {
                step_type: StepType::FinalAnswer,
                content: "No problem, I won't make any changes.".to_string(),
            };
            let response = StreamingResponse::Step { step };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            let result = anna_shared::rpc::AskResult {
                answer: "No problem, I won't make any changes.".to_string(),
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        }
        // Not yes/no - fall through to normal processing
    }

    // v0.0.998: Check if this matches a configuration recipe
    if let Some(recipe_result) = recipes::try_recipe(question) {
        info!("Recipe matched for: {}", question);

        let step = DialogueStep {
            step_type: if recipe_result.needs_confirmation {
                StepType::ConfirmationRequest
            } else {
                StepType::FinalAnswer
            },
            content: recipe_result.message.clone(),
        };
        let response = StreamingResponse::Step { step };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;

        if recipe_result.needs_confirmation {
            // Extract recipe ID from the pending recipe system
            // The recipe modules store their pending state internally
            let recipe_id = extract_recipe_id(question);
            set_pending_recipe(session_id, &recipe_id);

            let result = anna_shared::rpc::AskResult {
                answer: recipe_result.message,
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: true,
                clarification_question: recipe_result.confirmation_prompt,
                cached: false,
                citations: vec![],
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        } else {
            let result = anna_shared::rpc::AskResult {
                answer: recipe_result.message,
                success: recipe_result.success,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        }
    }

    // Check for pending critical system alerts and notify user
    if let Some(alerts) = get_pending_alerts() {
        for alert in alerts {
            let step = DialogueStep {
                step_type: StepType::SystemAlert,
                content: alert,
            };
            let response = StreamingResponse::Step { step };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
        }
    }

    // v0.2.0: DISABLED pattern-based autofix shortcut
    // The LLM now investigates and suggests fixes based on actual findings
    // rather than keyword matching. Keeping code commented for reference.
    /*
    if let Some(autofix) = find_autofix(question) {
        if check_autofix_needed(autofix) {
            // ... autofix shortcut logic removed ...
        }
    }
    */

    // Check cache for identical recent question
    {
        let state_guard = state.read().await;
        if let Some(cached_answer) = state_guard.get_cached_answer(question) {
            info!("Returning cached answer for: {}", question);
            // Send cached answer as a quick streaming response
            let step = DialogueStep {
                step_type: StepType::FinalAnswer,
                content: cached_answer.clone(),
            };
            let response = StreamingResponse::Step { step: step.clone() };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            // Send done with AskResult
            let result = anna_shared::rpc::AskResult {
                answer: cached_answer,
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![step],
                needs_clarification: false,
                clarification_question: None,
                cached: true,
                citations: vec![],
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        }
    }

    // Get session context and expand question with references
    let (expanded_question, session_context) = {
        let mut state_guard = state.write().await;
        let session = state_guard.get_or_create_session(session_id);
        let expanded = session.expand_question(question);
        let context = if session.history.is_empty() {
            None
        } else {
            Some(session.get_context_for_llm())
        };
        (expanded, context)
    };

    // Get model from state
    let model = {
        let state_guard = state.read().await;
        match &state_guard.model {
            Some(m) => m.clone(),
            None => {
                let response = StreamingResponse::Error {
                    message: "Daemon not ready - no model available".to_string(),
                };
                let json = serde_json::to_string(&response)?;
                writer.write_all(format!("{}\n", json).as_bytes()).await?;
                return Ok(());
            }
        }
    };

    // Execute with streaming (use expanded question if different)
    let question_to_use = if expanded_question != question {
        info!(
            "Expanded question with session context: {} -> {}",
            question, expanded_question
        );
        &expanded_question
    } else {
        question
    };

    // v0.0.905: Check answer cache before running LLM
    {
        let state_guard = state.read().await;
        if let Some(cached_answer) = state_guard.get_cached_answer(question_to_use) {
            info!("Returning cached answer for: {}", question_to_use);

            // Send cached response with dialogue showing it's cached
            let step = DialogueStep {
                step_type: StepType::UserQuestion,
                content: question_to_use.to_string(),
            };
            let json = serde_json::to_string(&StreamingResponse::Step { step: step.clone() })?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            let step = DialogueStep {
                step_type: StepType::FinalAnswer,
                content: cached_answer.clone(),
            };
            let json = serde_json::to_string(&StreamingResponse::Step { step: step.clone() })?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;

            let result = anna_shared::rpc::AskResult {
                answer: cached_answer,
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: true,
                citations: vec![],
            };
            let json = serde_json::to_string(&StreamingResponse::Done { result })?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(());
        }
    }

    // v0.1.1: Check if Ralph loop is enabled (simpler, more robust)
    let use_ralph = AnnaConfig::load()
        .map(|c| c.use_ralph_loop)
        .unwrap_or(true);

    let result = if use_ralph {
        info!("Using Ralph loop for question: {}", question_to_use);
        ralph::ralph_loop_streaming(&model, question_to_use, &mut writer).await
    } else {
        execute_question_streaming(
            &model,
            question_to_use,
            session_context.as_deref(),
            &mut writer,
        )
        .await
    };

    // v0.0.892: Record full turn to session after execution
    match &result {
        Ok(ask_result) => {
            let mut state_guard = state.write().await;
            if let Some(session) = state_guard.sessions.sessions.get_mut(session_id) {
                // Record the full turn: question, answer, and commands
                session.add_turn(
                    question,
                    &ask_result.answer,
                    ask_result.commands_executed.clone(),
                );
            }
            // v0.0.905: Cache successful answers (only if not a clarification)
            if ask_result.success && !ask_result.needs_clarification && !ask_result.answer.is_empty()
            {
                state_guard.cache_answer(question_to_use, &ask_result.answer);
                debug!("Cached answer for: {}", question_to_use);
            }
            // Cleanup old sessions periodically (also triggers periodic save to disk)
            state_guard.cleanup_sessions();

            // v0.2.8: Record RPG stats
            let elapsed = start_time.elapsed();
            let response_ms = elapsed.as_millis() as u64;
            let answer_type = if ask_result.iterations == 0 {
                anna_shared::stats::AnswerType::Instant
            } else if ask_result.cached {
                anna_shared::stats::AnswerType::Memory
            } else {
                anna_shared::stats::AnswerType::Llm
            };
            if let Ok(mut stats) = anna_shared::stats::PersistentStats::load() {
                stats.record_answer(response_ms, answer_type);
                let _ = stats.save();
            }
        }
        Err(e) => {
            let response = StreamingResponse::Error {
                message: format!("Execution error: {}", e),
            };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
        }
    }

    Ok(())
}

/// v0.0.997: Check if question is asking about fix history
fn is_fix_history_question(question: &str) -> bool {
    let q = question.to_lowercase();

    // Two-word patterns
    let two_word = [
        ("fix", "history"),
        ("what", "fixed"),
        ("fixes", "done"),
        ("show", "fixes"),
        ("list", "fixes"),
        ("recent", "fixes"),
        ("repair", "history"),
        ("auto", "fixes"),
    ];

    for (a, b) in &two_word {
        if q.contains(*a) && q.contains(*b) {
            return true;
        }
    }

    // Three-word patterns
    if (q.contains("what") && q.contains("anna") && q.contains("fix"))
        || (q.contains("what") && q.contains("have") && q.contains("fix"))
    {
        return true;
    }

    false
}

/// v0.0.998: Extract recipe ID from question for pending recipe tracking
fn extract_recipe_id(question: &str) -> String {
    let q = question.to_lowercase();

    // Vim recipes
    if q.contains("vim") || q.contains("neovim") {
        if q.contains("dark") {
            return "vim-dark-mode".to_string();
        }
        if q.contains("syntax") {
            return "vim-syntax".to_string();
        }
        if q.contains("line") && q.contains("number") {
            return "vim-line-numbers".to_string();
        }
        if q.contains("mouse") {
            return "vim-mouse".to_string();
        }
        if q.contains("tab") || q.contains("indent") {
            return "vim-tabs".to_string();
        }
    }

    // Git recipes
    if q.contains("git") {
        if q.contains("email") {
            return "git-email".to_string();
        }
        if q.contains("name") {
            return "git-name".to_string();
        }
        if q.contains("alias") {
            return "git-aliases".to_string();
        }
        if q.contains("default") && q.contains("branch") {
            return "git-default-branch".to_string();
        }
    }

    // Shell recipes
    if q.contains("alias") {
        return "shell-alias".to_string();
    }
    if q.contains("path") && (q.contains("add") || q.contains("append")) {
        return "shell-path".to_string();
    }
    if q.contains("export") {
        return "shell-export".to_string();
    }

    // Service recipes
    if q.contains("restart") {
        if let Some(service) = extract_service_from_question(&q) {
            return format!("service-restart-{}", service);
        }
    }
    if q.contains("start") && !q.contains("restart") {
        if let Some(service) = extract_service_from_question(&q) {
            return format!("service-start-{}", service);
        }
    }
    if q.contains("stop") {
        if let Some(service) = extract_service_from_question(&q) {
            return format!("service-stop-{}", service);
        }
    }
    if q.contains("enable") {
        if let Some(service) = extract_service_from_question(&q) {
            return format!("service-enable-{}", service);
        }
    }
    if q.contains("disable") {
        if let Some(service) = extract_service_from_question(&q) {
            return format!("service-disable-{}", service);
        }
    }

    "unknown".to_string()
}

/// Extract service name from question
fn extract_service_from_question(q: &str) -> Option<String> {
    let services = [
        "nginx", "apache", "httpd", "mysql", "mariadb", "postgresql", "postgres",
        "docker", "containerd", "redis", "mongodb", "ssh", "sshd", "cups",
        "bluetooth", "networkmanager", "firewalld", "libvirtd", "pipewire",
        "pulseaudio", "avahi", "gdm", "sddm", "lightdm",
    ];

    for service in &services {
        if q.contains(service) {
            return Some(service.to_string());
        }
    }
    None
}

/// v0.0.998: Transform a dialogue step to use team-style messaging
/// This gives the "Hollywood IT teams" experience where users feel like
/// they're watching a team work on their problem.
fn team_style_content(step_type: &StepType, content: &str) -> String {
    match step_type {
        StepType::IntentClassifying => team_speak::phase_commentary("intent_classify", None),
        StepType::WikiSearch => team_speak::phase_commentary("wiki_search", None),
        StepType::CommandExec => {
            // Transform command into friendly description
            team_speak::describe_command(content)
        }
        StepType::FinalAnswer => content.to_string(), // Keep final answer as-is
        _ => content.to_string(),
    }
}
