//! Input handling - User input processing and language detection

use crate::tui_state::{AnnaTuiState, LanguageCode};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc;

use super::action_plan::{generate_action_plan_from_llm, send_demo_action_plan, should_generate_action_plan};
use super::event_loop::TuiMessage;
use super::llm::generate_reply_streaming;

/// Draw input bar (bottom)
/// Beta.99: Added text wrapping for multi-line input
pub fn draw_input_bar(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    let lang_indicator = format!("[{}]", state.language.as_str().to_uppercase());
    let prompt = format!("{} > ", lang_indicator);

    // Beta.99: Build wrapped text with proper formatting
    let input_text = format!("{}{}_", prompt, &state.input);

    let paragraph = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false }); // Beta.99: Enable wrapping!

    f.render_widget(paragraph, area);
}

/// Beta.108: Handle user input (non-blocking)
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
    };

    // Beta.148: Route through appropriate handler
    if should_use_action_plan {
        // Use V3 JSON dialogue to generate ActionPlan
        tokio::spawn(async move {
            if let Err(e) = generate_action_plan_from_llm(&input, &state_clone, tx.clone()).await {
                // Fallback to regular reply on error
                let error_msg = format!(
                    "⚠️ Could not generate action plan: {}\n\nFalling back to standard reply...",
                    e
                );
                let _ = tx.send(TuiMessage::AnnaReply(error_msg)).await;

                let reply = generate_reply_streaming(&input, &state_clone, tx.clone()).await;
                if !reply.is_empty() {
                    let _ = tx.send(TuiMessage::AnnaReply(reply)).await;
                }
            }
        });
    } else {
        // Use standard reply generation
        tokio::spawn(async move {
            let reply = generate_reply_streaming(&input, &state_clone, tx.clone()).await;
            // If template-based (non-streaming), send as complete reply
            if !reply.is_empty() {
                let _ = tx.send(TuiMessage::AnnaReply(reply)).await;
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
