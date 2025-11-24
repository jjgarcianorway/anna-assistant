//! Input handling - User input processing and language detection

use crate::tui_state::{AnnaTuiState, LanguageCode};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc;

use super::action_plan::{
    generate_action_plan_from_llm, send_demo_action_plan, should_generate_action_plan,
};
use super::event_loop::TuiMessage;

/// Draw input bar (bottom)
/// Beta.99: Added text wrapping for multi-line input
/// Beta.220: Enhanced input bar with truecolor and better cursor visibility
pub fn draw_input_bar(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    let lang_indicator = format!("[{}]", state.language.as_str().to_uppercase());
    let prompt = format!("{} > ", lang_indicator);

    // Beta.99: Build wrapped text with proper formatting
    let input_text = format!("{}{}_", prompt, &state.input);

    // Beta.220: Use truecolor for input box
    let paragraph = Paragraph::new(input_text)
        .block(
            Block::default()
                .title(" Input ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(100, 200, 100))), // Green
        )
        .style(Style::default().fg(Color::Rgb(220, 220, 220))) // Light gray text
        .wrap(Wrap { trim: false }); // Beta.99: Enable wrapping!

    f.render_widget(paragraph, area);
}

/// Beta.108: Handle user input (non-blocking)
/// Beta.228: Added comprehensive logging
/// Returns true if should exit, false otherwise
pub fn handle_user_input(state: &mut AnnaTuiState, tx: mpsc::Sender<TuiMessage>) -> bool {
    let input = state.input.clone();
    let input_lower = input.trim().to_lowercase();

    // Check for exit commands
    if input_lower == "bye" || input_lower == "exit" || input_lower == "quit" {
        state.add_system_message("Goodbye!".to_string());
        return true;
    }

    // Beta.147: Demo command for testing ActionPlan flow
    if input_lower == "demo" || input_lower == "demo plan" {
        state.add_user_message(input.clone());
        state.input.clear();
        state.cursor_pos = 0;

        // Generate sample ActionPlan
        send_demo_action_plan(tx, false);
        return false;
    }

    // Beta.147: Risky demo with rollback
    if input_lower == "demo risky" {
        state.add_user_message(input.clone());
        state.input.clear();
        state.cursor_pos = 0;

        // Generate risky ActionPlan with rollback
        send_demo_action_plan(tx, true);
        return false;
    }

    // Beta.108: Add user message immediately (visible in UI)
    state.add_user_message(input.clone());
    state.input.clear();
    state.cursor_pos = 0;

    // Detect language change
    detect_language_change(&input, state);

    // Beta.108: Set thinking state before LLM processing
    state.is_thinking = true;
    state.thinking_frame = 0;

    // Beta.148: Determine if query should use ActionPlan mode
    let should_use_action_plan = should_generate_action_plan(&input);

    // Clone state data needed for LLM query
    let state_clone = AnnaTuiState {
        system_panel: state.system_panel.clone(),
        llm_panel: state.llm_panel.clone(),
        language: state.language.clone(),
        conversation: Vec::new(), // Don't need full history for this query
        input: String::new(),
        cursor_pos: 0,
        input_history: Vec::new(),
        history_index: None,
        show_help: false,
        is_thinking: false,
        thinking_frame: 0,
        scroll_offset: 0,
        telemetry_ok: state.telemetry_ok,
        last_llm_reply: None,
        last_action_plan: None, // Beta.147
        brain_insights: Vec::new(), // Beta.218
        brain_timestamp: None,      // Beta.218
        brain_available: false,     // Beta.218
        proactive_issues: Vec::new(), // Beta.271
        daily_snapshot: None,       // Beta.259
    };

    // Beta.148: Route through appropriate handler
    if should_use_action_plan {
        // Use V3 JSON dialogue to generate ActionPlan
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            if let Err(e) = generate_action_plan_from_llm(&input, &state_clone, tx.clone()).await {
                // Fallback to simple error message on action plan failure
                let error_msg = format!(
                    "⚠️ Could not generate action plan: {}",
                    e
                );
                let _ = tx.send(TuiMessage::AnnaReply(error_msg)).await;
            } else {
            }
        });
    } else {
        // Beta.280: Handle informational queries with TUI streaming support
        tokio::spawn(async move {
            // Get telemetry for query
            let telemetry = match crate::system_query::query_system_telemetry() {
                Ok(t) => t,
                Err(e) => {
                    let _ = tx.send(TuiMessage::AnnaReply(format!("Error: {}", e))).await;
                    return;
                }
            };

            // Get LLM config
            let llm_config = crate::query_handler::get_llm_config();

            // Beta.280: Handle query with TUI streaming
            if let Err(e) = handle_tui_query_with_streaming(&input, &telemetry, &llm_config, tx.clone()).await {
                let _ = tx.send(TuiMessage::AnnaReply(format!("Error: {}", e))).await;
            }
        });
    }

    false
}

/// Detect language change from natural language
fn detect_language_change(input: &str, state: &mut AnnaTuiState) {
    let input_lower = input.to_lowercase();

    if input_lower.contains("habla español") || input_lower.contains("en español") {
        state.language = LanguageCode::Spanish;
    } else if input_lower.contains("speak english") || input_lower.contains("in english") {
        state.language = LanguageCode::English;
    } else if input_lower.contains("parle français") || input_lower.contains("en français") {
        state.language = LanguageCode::French;
    } else if input_lower.contains("sprich deutsch") || input_lower.contains("auf deutsch") {
        state.language = LanguageCode::German;
    } else if input_lower.contains("parla italiano") || input_lower.contains("in italiano") {
        state.language = LanguageCode::Italian;
    }
}

/// Beta.280: Handle TUI query with streaming support
///
/// This function intercepts LLM streaming and sends chunks to the TUI channel
/// instead of printing to stdout (which would appear outside the TUI).
async fn handle_tui_query_with_streaming(
    user_text: &str,
    telemetry: &anna_common::telemetry::SystemTelemetry,
    llm_config: &anna_common::llm::LlmConfig,
    tx: tokio::sync::mpsc::Sender<TuiMessage>,
) -> anyhow::Result<()> {
    use anna_common::llm::{LlmClient, LlmPrompt};
    use std::sync::Arc;

    // Build conversational prompt
    // 6.12.2: TUI doesn't pass knowledge (legacy code, not a priority)
    let prompt = crate::unified_query_handler::build_conversational_prompt_for_tui(user_text, telemetry, None);

    // Create LLM client
    let client = Arc::new(LlmClient::from_config(llm_config)
        .map_err(|e| anyhow::anyhow!("LLM not available: {}", e))?);

    let llm_prompt = Arc::new(LlmPrompt {
        system: LlmClient::anna_system_prompt().to_string(),
        user: prompt,
        conversation_history: None,
    });

    // Send start streaming message
    let _ = tx.send(TuiMessage::AnnaReplyStart).await;

    // Clone for spawn_blocking
    let client_clone = Arc::clone(&client);
    let prompt_clone = Arc::clone(&llm_prompt);
    let tx_clone = tx.clone();

    // Stream the response with TUI chunks
    let response_text = tokio::task::spawn_blocking(move || {
        let mut accumulated = String::new();

        // Beta.280: Send chunks to TUI instead of printing to stdout
        let result = client_clone.chat_stream(&prompt_clone, &mut |chunk: &str| {
            accumulated.push_str(chunk);

            // Send chunk to TUI (best effort, don't fail streaming on send error)
            let tx_chunk = tx_clone.clone();
            let chunk_owned = chunk.to_string();
            tokio::spawn(async move {
                let _ = tx_chunk.send(TuiMessage::AnnaReplyChunk(chunk_owned)).await;
            });
        });

        match result {
            Ok(()) => Ok(accumulated),
            Err(e) => Err(e),
        }
    })
    .await
    .map_err(|e| anyhow::anyhow!("LLM task panicked: {}", e))?
    .map_err(|e| anyhow::anyhow!("LLM query failed: {}", e))?;

    // Send completion message
    let _ = tx.send(TuiMessage::AnnaReplyComplete).await;

    Ok(())
}
