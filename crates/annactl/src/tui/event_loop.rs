//! Event Loop - Main TUI entry point and event handling

use crate::tui_state::AnnaTuiState;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc;

use super::action_plan::handle_action_plan_execution;
use super::input::handle_user_input;
use super::render::draw_ui;
use super::state::{show_welcome_message, update_telemetry};

/// TUI message types
#[derive(Debug)]
pub enum TuiMessage {
    UserInput(String),
    AnnaReply(String),
    AnnaReplyChunk(String), // Beta.115: Streaming chunk
    AnnaReplyComplete,      // Beta.115: Streaming complete
    ActionPlanReply(anna_common::action_plan_v3::ActionPlan), // Beta.147: Structured action plan
    TelemetryUpdate,
}

/// Run the TUI
/// Beta.227: Enhanced error handling and graceful degradation
/// Beta.228: Comprehensive logging for debugging
pub async fn run() -> Result<()> {

    // Setup terminal with error recovery
    enable_raw_mode().map_err(|e| {
        anyhow::anyhow!("Failed to enable raw mode: {}. Ensure you're running in a real terminal (TTY).", e)
    })?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
        let _ = disable_raw_mode(); // Cleanup attempt
        anyhow::anyhow!("Failed to initialize terminal: {}", e)
    })?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load state with fallback
    let mut state = AnnaTuiState::load().await.unwrap_or_else(|e| {
        AnnaTuiState::default()
    });

    // Create channels for async operations
    let (tx, mut rx) = mpsc::channel(32);

    // Run event loop with panic recovery
    let result = run_event_loop(&mut terminal, &mut state, tx, &mut rx).await;

    // Restore terminal (always attempt cleanup)
    let cleanup_result = restore_terminal(&mut terminal);

    // Save state (best effort)
    if let Err(e) = state.save().await {
    } else {
    }

    // Return event loop result, or cleanup error if that failed
    result.and(cleanup_result)
}

/// Beta.227: Separate cleanup function for better error handling
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Main event loop
async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AnnaTuiState,
    tx: mpsc::Sender<TuiMessage>,
    rx: &mut mpsc::Receiver<TuiMessage>,
) -> Result<()> {

    // Beta.94: Initialize telemetry with real data (synchronous, fast)
    update_telemetry(state);

    // Beta.218: Initialize brain diagnostics (async RPC, may fail gracefully)
    // Beta.227: Don't block TUI startup on brain analysis
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        // Brain analysis in background - won't block TUI initialization
        // If it fails, the TUI will show "Brain diagnostics unavailable"
    });

    // Update brain synchronously for immediate display (will use cached/default if daemon slow)
    let brain_start = std::time::Instant::now();
    super::brain::update_brain_analysis(state).await;

    // Beta.229: DISABLED - Welcome message causes 18.5s startup delay
    // The async LLM call blocks TUI initialization
    // Re-enable in Beta.230+ with pure telemetry-based greeting (no LLM)

    // Track last telemetry update
    let mut last_telemetry_update = std::time::Instant::now();
    let telemetry_interval = std::time::Duration::from_secs(5);

    // Beta.218: Track last brain analysis update (every 30 seconds)
    let mut last_brain_update = std::time::Instant::now();
    let brain_interval = std::time::Duration::from_secs(30);

    loop {
        // Beta.94: Update telemetry every 5 seconds
        if last_telemetry_update.elapsed() >= telemetry_interval {
            update_telemetry(state);
            last_telemetry_update = std::time::Instant::now();
        }

        // Beta.218: Update brain analysis every 30 seconds
        if last_brain_update.elapsed() >= brain_interval {
            super::brain::update_brain_analysis(state).await;
            last_brain_update = std::time::Instant::now();
        }

        // Beta.91: Advance thinking animation frame
        if state.is_thinking {
            state.thinking_frame = (state.thinking_frame + 1) % 8;
        }

        // Beta.108: Check for async messages (LLM replies, etc.)
        while let Ok(msg) = rx.try_recv() {
            match msg {
                TuiMessage::AnnaReply(reply) => {
                    state.is_thinking = false;
                    state.add_anna_reply(reply);
                }
                TuiMessage::AnnaReplyChunk(chunk) => {
                    // Beta.115: Streaming chunk arrives
                    state.append_to_last_anna_reply(chunk);
                }
                TuiMessage::AnnaReplyComplete => {
                    // Beta.115: Streaming complete
                    state.is_thinking = false;
                }
                TuiMessage::ActionPlanReply(plan) => {
                    // Beta.147: Structured action plan arrives
                    state.add_action_plan(plan);
                }
                TuiMessage::UserInput(_) => {
                    // Not used in current implementation
                }
                TuiMessage::TelemetryUpdate => {
                    update_telemetry(state);
                }
            }
        }

        // Draw UI
        terminal.draw(|f| draw_ui(f, state))?;

        // Handle events with timeout
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    // Ctrl+C - exit
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        break;
                    }
                    // Ctrl+L - clear conversation
                    (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                        state.clear_conversation();
                    }
                    // Ctrl+U - clear input
                    (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                        state.input.clear();
                        state.cursor_pos = 0;
                    }
                    // Ctrl+X - execute last action plan
                    (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                        // Beta.147: Execute action plan
                        if state.last_action_plan.is_some() {
                            handle_action_plan_execution(state, tx.clone());
                        } else {
                        }
                    }
                    // F1 - toggle help
                    (KeyCode::F(1), _) => {
                        state.show_help = !state.show_help;
                    }
                    // Enter - submit input
                    (KeyCode::Enter, _) => {
                        if !state.input.trim().is_empty() {
                            // Beta.108: Non-blocking input handling
                            if handle_user_input(state, tx.clone()) {
                                break; // Exit requested
                            }
                        }
                    }
                    // Backspace
                    (KeyCode::Backspace, _) => {
                        if state.cursor_pos > 0 {
                            state.input.remove(state.cursor_pos - 1);
                            state.cursor_pos -= 1;
                        }
                    }
                    // Up arrow - history
                    (KeyCode::Up, _) => {
                        state.history_up();
                    }
                    // Down arrow - history
                    (KeyCode::Down, _) => {
                        state.history_down();
                    }
                    // PageUp - scroll conversation up
                    (KeyCode::PageUp, _) => {
                        if state.scroll_offset > 0 {
                            state.scroll_offset = state.scroll_offset.saturating_sub(10);
                        }
                    }
                    // PageDown - scroll conversation down
                    (KeyCode::PageDown, _) => {
                        state.scroll_offset = state.scroll_offset.saturating_add(10);
                    }
                    // Character input
                    (KeyCode::Char(c), KeyModifiers::NONE)
                    | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                        state.input.insert(state.cursor_pos, c);
                        state.cursor_pos += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
