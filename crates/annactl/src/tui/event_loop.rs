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
    eprintln!("[TUI] Starting TUI initialization...");

    // Setup terminal with error recovery
    eprintln!("[TUI] Enabling raw mode...");
    enable_raw_mode().map_err(|e| {
        eprintln!("[TUI] ERROR: Failed to enable raw mode: {}", e);
        anyhow::anyhow!("Failed to enable raw mode: {}. Ensure you're running in a real terminal (TTY).", e)
    })?;
    eprintln!("[TUI] Raw mode enabled successfully");

    eprintln!("[TUI] Setting up terminal backend...");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
        eprintln!("[TUI] ERROR: Failed to initialize terminal: {}", e);
        let _ = disable_raw_mode(); // Cleanup attempt
        anyhow::anyhow!("Failed to initialize terminal: {}", e)
    })?;
    eprintln!("[TUI] Terminal backend initialized");

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    eprintln!("[TUI] Terminal object created");

    // Load state with fallback
    eprintln!("[TUI] Loading TUI state...");
    let mut state = AnnaTuiState::load().await.unwrap_or_else(|e| {
        eprintln!("[TUI] Warning: Failed to load TUI state: {}", e);
        eprintln!("[TUI] Using default state");
        AnnaTuiState::default()
    });
    eprintln!("[TUI] State loaded successfully");

    // Create channels for async operations
    eprintln!("[TUI] Creating async channels...");
    let (tx, mut rx) = mpsc::channel(32);
    eprintln!("[TUI] Channels created");

    // Run event loop with panic recovery
    eprintln!("[TUI] Entering main event loop...");
    let result = run_event_loop(&mut terminal, &mut state, tx, &mut rx).await;
    eprintln!("[TUI] Event loop exited with result: {:?}", result.as_ref().map(|_| "Ok").map_err(|e| e.to_string()));

    // Restore terminal (always attempt cleanup)
    eprintln!("[TUI] Restoring terminal...");
    let cleanup_result = restore_terminal(&mut terminal);
    eprintln!("[TUI] Terminal restore result: {:?}", cleanup_result.as_ref().map(|_| "Ok").map_err(|e| e.to_string()));

    // Save state (best effort)
    eprintln!("[TUI] Saving state...");
    if let Err(e) = state.save().await {
        eprintln!("[TUI] Warning: Failed to save TUI state: {}", e);
    } else {
        eprintln!("[TUI] State saved successfully");
    }

    eprintln!("[TUI] TUI shutdown complete");
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
    eprintln!("[EVENT_LOOP] Starting event loop initialization...");

    // Beta.94: Initialize telemetry with real data (synchronous, fast)
    eprintln!("[EVENT_LOOP] Updating initial telemetry...");
    update_telemetry(state);
    eprintln!("[EVENT_LOOP] Initial telemetry updated");

    // Beta.218: Initialize brain diagnostics (async RPC, may fail gracefully)
    // Beta.227: Don't block TUI startup on brain analysis
    eprintln!("[EVENT_LOOP] Spawning background brain analysis task...");
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        // Brain analysis in background - won't block TUI initialization
        // If it fails, the TUI will show "Brain diagnostics unavailable"
    });
    eprintln!("[EVENT_LOOP] Background task spawned");

    // Update brain synchronously for immediate display (will use cached/default if daemon slow)
    eprintln!("[EVENT_LOOP] Fetching brain analysis (may be slow)...");
    let brain_start = std::time::Instant::now();
    super::brain::update_brain_analysis(state).await;
    eprintln!("[EVENT_LOOP] Brain analysis completed in {:?}", brain_start.elapsed());

    // Beta.94: Show welcome message on first launch
    if state.conversation.is_empty() {
        eprintln!("[EVENT_LOOP] Generating welcome message (first launch)...");
        let welcome_start = std::time::Instant::now();
        show_welcome_message(state).await;
        eprintln!("[EVENT_LOOP] Welcome message generated in {:?}", welcome_start.elapsed());
    } else {
        eprintln!("[EVENT_LOOP] Skipping welcome message (conversation exists)");
    }

    // Track last telemetry update
    let mut last_telemetry_update = std::time::Instant::now();
    let telemetry_interval = std::time::Duration::from_secs(5);

    // Beta.218: Track last brain analysis update (every 30 seconds)
    let mut last_brain_update = std::time::Instant::now();
    let brain_interval = std::time::Duration::from_secs(30);

    eprintln!("[EVENT_LOOP] Entering main render loop...");
    let mut loop_iteration = 0;

    loop {
        loop_iteration += 1;
        if loop_iteration % 100 == 0 {
            eprintln!("[EVENT_LOOP] Loop iteration {}", loop_iteration);
        }

        // Beta.94: Update telemetry every 5 seconds
        if last_telemetry_update.elapsed() >= telemetry_interval {
            eprintln!("[EVENT_LOOP] Periodic telemetry update...");
            update_telemetry(state);
            last_telemetry_update = std::time::Instant::now();
        }

        // Beta.218: Update brain analysis every 30 seconds
        if last_brain_update.elapsed() >= brain_interval {
            eprintln!("[EVENT_LOOP] Periodic brain analysis update...");
            super::brain::update_brain_analysis(state).await;
            last_brain_update = std::time::Instant::now();
        }

        // Beta.91: Advance thinking animation frame
        if state.is_thinking {
            state.thinking_frame = (state.thinking_frame + 1) % 8;
        }

        // Beta.108: Check for async messages (LLM replies, etc.)
        while let Ok(msg) = rx.try_recv() {
            eprintln!("[EVENT_LOOP] Received message: {:?}", match &msg {
                TuiMessage::AnnaReply(_) => "AnnaReply",
                TuiMessage::AnnaReplyChunk(_) => "AnnaReplyChunk",
                TuiMessage::AnnaReplyComplete => "AnnaReplyComplete",
                TuiMessage::ActionPlanReply(_) => "ActionPlanReply",
                TuiMessage::UserInput(_) => "UserInput",
                TuiMessage::TelemetryUpdate => "TelemetryUpdate",
            });

            match msg {
                TuiMessage::AnnaReply(reply) => {
                    eprintln!("[EVENT_LOOP] Processing full reply ({} chars)", reply.len());
                    state.is_thinking = false;
                    state.add_anna_reply(reply);
                }
                TuiMessage::AnnaReplyChunk(chunk) => {
                    // Beta.115: Streaming chunk arrives
                    eprintln!("[EVENT_LOOP] Processing chunk ({} chars)", chunk.len());
                    state.append_to_last_anna_reply(chunk);
                }
                TuiMessage::AnnaReplyComplete => {
                    // Beta.115: Streaming complete
                    eprintln!("[EVENT_LOOP] Stream complete");
                    state.is_thinking = false;
                }
                TuiMessage::ActionPlanReply(plan) => {
                    // Beta.147: Structured action plan arrives
                    eprintln!("[EVENT_LOOP] Processing action plan");
                    state.add_action_plan(plan);
                }
                TuiMessage::UserInput(_) => {
                    // Not used in current implementation
                    eprintln!("[EVENT_LOOP] UserInput message (unused)");
                }
                TuiMessage::TelemetryUpdate => {
                    eprintln!("[EVENT_LOOP] Telemetry update requested");
                    update_telemetry(state);
                }
            }
        }

        // Draw UI
        terminal.draw(|f| draw_ui(f, state))?;

        // Handle events with timeout
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                eprintln!("[EVENT_LOOP] Key event: {:?} with modifiers {:?}", key.code, key.modifiers);
                match (key.code, key.modifiers) {
                    // Ctrl+C - exit
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        eprintln!("[EVENT_LOOP] Ctrl+C pressed, exiting...");
                        break;
                    }
                    // Ctrl+L - clear conversation
                    (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                        eprintln!("[EVENT_LOOP] Ctrl+L pressed, clearing conversation");
                        state.clear_conversation();
                    }
                    // Ctrl+U - clear input
                    (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                        eprintln!("[EVENT_LOOP] Ctrl+U pressed, clearing input");
                        state.input.clear();
                        state.cursor_pos = 0;
                    }
                    // Ctrl+X - execute last action plan
                    (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                        // Beta.147: Execute action plan
                        eprintln!("[EVENT_LOOP] Ctrl+X pressed");
                        if state.last_action_plan.is_some() {
                            eprintln!("[EVENT_LOOP] Executing action plan");
                            handle_action_plan_execution(state, tx.clone());
                        } else {
                            eprintln!("[EVENT_LOOP] No action plan available");
                        }
                    }
                    // F1 - toggle help
                    (KeyCode::F(1), _) => {
                        eprintln!("[EVENT_LOOP] F1 pressed, toggling help");
                        state.show_help = !state.show_help;
                    }
                    // Enter - submit input
                    (KeyCode::Enter, _) => {
                        eprintln!("[EVENT_LOOP] Enter pressed with input: '{}'", state.input);
                        if !state.input.trim().is_empty() {
                            // Beta.108: Non-blocking input handling
                            eprintln!("[EVENT_LOOP] Handling user input...");
                            if handle_user_input(state, tx.clone()) {
                                eprintln!("[EVENT_LOOP] Exit requested from input handler");
                                break; // Exit requested
                            }
                            eprintln!("[EVENT_LOOP] Input handled, continuing...");
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

    eprintln!("[EVENT_LOOP] Exiting event loop normally");
    Ok(())
}
