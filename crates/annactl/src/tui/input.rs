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
    eprintln!("[INPUT] Processing input: '{}'", input);

    // Check for exit commands
    if input_lower == "bye" || input_lower == "exit" || input_lower == "quit" {
        eprintln!("[INPUT] Exit command detected");
        state.add_system_message("Goodbye!".to_string());
        return true;
    }

    // Beta.147: Demo command for testing ActionPlan flow
    if input_lower == "demo" || input_lower == "demo plan" {
        eprintln!("[INPUT] Demo command detected");
        state.add_user_message(input.clone());
        state.input.clear();
        state.cursor_pos = 0;

        // Generate sample ActionPlan
        send_demo_action_plan(tx, false);
        return false;
    }

    // Beta.147: Risky demo with rollback
    if input_lower == "demo risky" {
        eprintln!("[INPUT] Risky demo command detected");
        state.add_user_message(input.clone());
        state.input.clear();
        state.cursor_pos = 0;

        // Generate risky ActionPlan with rollback
        send_demo_action_plan(tx, true);
        return false;
    }

    // Beta.108: Add user message immediately (visible in UI)
    eprintln!("[INPUT] Adding user message to conversation");
    state.add_user_message(input.clone());
    state.input.clear();
    state.cursor_pos = 0;

    // Detect language change
    eprintln!("[INPUT] Detecting language change...");
    detect_language_change(&input, state);
    eprintln!("[INPUT] Current language: {:?}", state.language);

    // Beta.108: Set thinking state before LLM processing
    eprintln!("[INPUT] Setting thinking state");
    state.is_thinking = true;
    state.thinking_frame = 0;

    // Beta.148: Determine if query should use ActionPlan mode
    let should_use_action_plan = should_generate_action_plan(&input);
    eprintln!("[INPUT] Should use action plan: {}", should_use_action_plan);

    // Clone state data needed for LLM query
    eprintln!("[INPUT] Cloning state for async task");
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
    };

    // Beta.148: Route through appropriate handler
    if should_use_action_plan {
        // Use V3 JSON dialogue to generate ActionPlan
        eprintln!("[INPUT] Spawning action plan generation task");
        tokio::spawn(async move {
            eprintln!("[INPUT_TASK] Starting action plan generation...");
            let start = std::time::Instant::now();
            if let Err(e) = generate_action_plan_from_llm(&input, &state_clone, tx.clone()).await {
                eprintln!("[INPUT_TASK] Action plan generation failed after {:?}: {}", start.elapsed(), e);
                // Fallback to simple error message on action plan failure
                let error_msg = format!(
                    "⚠️ Could not generate action plan: {}",
                    e
                );
                let _ = tx.send(TuiMessage::AnnaReply(error_msg)).await;
            } else {
                eprintln!("[INPUT_TASK] Action plan generated in {:?}", start.elapsed());
            }
        });
    } else {
        // Beta.229: Handle informational queries in TUI using unified query handler
        eprintln!("[INPUT] Spawning informational query task");
        tokio::spawn(async move {
            eprintln!("[INPUT_TASK] Starting informational query...");
            let start = std::time::Instant::now();

            // Get telemetry for query
            let telemetry = match crate::system_query::query_system_telemetry() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[INPUT_TASK] Failed to get telemetry: {}", e);
                    let _ = tx.send(TuiMessage::AnnaReply(format!("Error: {}", e))).await;
                    return;
                }
            };

            // Get LLM config
            let llm_config = crate::query_handler::get_llm_config();

            // Use unified query handler
            match crate::unified_query_handler::handle_unified_query(&input, &telemetry, &llm_config).await {
                Ok(result) => {
                    use crate::unified_query_handler::UnifiedQueryResult;
                    let reply = match result {
                        UnifiedQueryResult::ConversationalAnswer { answer, .. } => answer,
                        UnifiedQueryResult::Template { output, .. } => output,
                        _ => "Query processed successfully".to_string(),
                    };
                    eprintln!("[INPUT_TASK] Informational query completed in {:?}", start.elapsed());
                    let _ = tx.send(TuiMessage::AnnaReply(reply)).await;
                }
                Err(e) => {
                    eprintln!("[INPUT_TASK] Query failed after {:?}: {}", start.elapsed(), e);
                    let _ = tx.send(TuiMessage::AnnaReply(format!("Error: {}", e))).await;
                }
            }
        });
    }

    eprintln!("[INPUT] Input handling complete, returning to event loop");
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
