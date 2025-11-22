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
use super::state::update_telemetry;

/// TUI message types
#[derive(Debug)]
pub enum TuiMessage {
    UserInput(String),
    AnnaReply(String),
    AnnaReplyChunk(String), // Beta.115: Streaming chunk
    AnnaReplyComplete,      // Beta.115: Streaming complete
    ActionPlanReply(anna_common::action_plan_v3::ActionPlan), // Beta.147: Structured action plan
    TelemetryUpdate,
    BrainUpdate(anna_common::ipc::BrainAnalysisData), // Beta.234: Brain analysis result
}

/// Run the TUI
/// Beta.227: Enhanced error handling and graceful degradation
/// Beta.228: Comprehensive logging for debugging
/// Beta.235: Added fallback error screen for startup failures
pub async fn run() -> Result<()> {

    // Setup terminal with error recovery and fallback screen
    enable_raw_mode().map_err(|e| {
        display_fallback_error(&format!(
            "Failed to enable raw mode: {}.\nEnsure you're running in a real terminal (TTY).",
            e
        ));
        anyhow::anyhow!("Failed to enable raw mode: {}. Ensure you're running in a real terminal (TTY).", e)
    })?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
        let _ = disable_raw_mode(); // Cleanup attempt
        display_fallback_error(&format!("Failed to initialize terminal: {}", e));
        anyhow::anyhow!("Failed to initialize terminal: {}", e)
    })?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load state with timeout and fallback (Beta.235: prevent infinite hangs on I/O)
    let mut state = match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        AnnaTuiState::load()
    ).await {
        Ok(Ok(s)) => s,
        Ok(Err(_)) | Err(_) => AnnaTuiState::default(),
    };

    // Create channels for async operations
    let (tx, mut rx) = mpsc::channel(32);

    // Run event loop with panic recovery
    let result = run_event_loop(&mut terminal, &mut state, tx, &mut rx).await;

    // Beta.261: Show exit summary screen before cleanup
    if result.is_ok() {
        let _ = show_exit_summary(&mut terminal, &state);
    }

    // Restore terminal (always attempt cleanup)
    let cleanup_result = restore_terminal(&mut terminal);

    // Save state (best effort, with timeout - Beta.235)
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        state.save()
    ).await;

    // Return event loop result, or cleanup error if that failed
    result.and(cleanup_result)
}

/// Beta.261: Show exit summary screen
/// Displays brief goodbye message with health summary and hint
fn show_exit_summary(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &AnnaTuiState,
) -> Result<()> {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::style::{Color, Style};

    // Generate exit summary lines
    let exit_lines = super::flow::generate_exit_summary(state);

    // Clear screen and show exit summary
    terminal.draw(|f| {
        let area = f.size();

        let paragraph = Paragraph::new(exit_lines)
            .block(
                Block::default()
                    .title(" Anna Assistant ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(80, 180, 255))),
            )
            .style(Style::default().bg(Color::Rgb(0, 0, 0)));

        f.render_widget(paragraph, area);
    })?;

    // Pause for 1 second to let user see the message
    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
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

    // Beta.234: Initialize brain diagnostics in background (NON-BLOCKING)
    // Spawn async task to fetch brain analysis without blocking main loop
    {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            // Fetch brain analysis in background
            // Will send result via channel when ready
            if let Ok(analysis) = super::brain::fetch_brain_data().await {
                // Send brain update via channel (non-blocking for spawned task)
                let _ = tx_clone.send(TuiMessage::BrainUpdate(analysis)).await;
            }
        });
    }

    // Beta.261: Show startup welcome panel using flow module
    // This replaces the old background welcome message with a compact,
    // deterministic welcome strip that uses existing health data
    {
        let welcome_lines = super::flow::generate_welcome_lines(state);
        // Convert lines to conversation items (as System messages)
        for line in welcome_lines {
            let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            if !text.trim().is_empty() {
                state.add_system_message(text);
            }
        }
    }

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

        // Beta.234: Update brain analysis every 30 seconds (NON-BLOCKING)
        // Spawn background task instead of awaiting here
        if last_brain_update.elapsed() >= brain_interval {
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                if let Ok(analysis) = super::brain::fetch_brain_data().await {
                    let _ = tx_clone.send(TuiMessage::BrainUpdate(analysis)).await;
                }
            });
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
                TuiMessage::BrainUpdate(diagnostics) => {
                    // Beta.234: Brain analysis arrived from background task
                    state.update_brain_diagnostics(diagnostics);
                }
            }
        }

        // Draw UI
        terminal.draw(|f| draw_ui(f, state))?;

        // Handle events with timeout
        if event::poll(std::time::Duration::from_millis(100))? {
            let event = event::read()?;

            // Handle mouse events for scrolling
            if let Event::Mouse(mouse) = event {
                use crossterm::event::MouseEventKind;
                match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        if state.scroll_offset > 0 {
                            state.scroll_offset = state.scroll_offset.saturating_sub(3);
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        state.scroll_offset = state.scroll_offset.saturating_add(3);
                    }
                    _ => {}
                }
            }

            // Handle keyboard events
            if let Event::Key(key) = event {
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
                    // Beta.247: PageUp - scroll conversation up (half of visible conversation area)
                    (KeyCode::PageUp, _) => {
                        // Calculate actual conversation panel height
                        // Total height - header (1) - status bar (1) - input bar (variable, assume 3 minimum)
                        let input_height = super::utils::calculate_input_height(&state.input, terminal.size().ok().map(|s| s.width.saturating_sub(8)).unwrap_or(80));
                        let conversation_height = terminal.size()
                            .ok()
                            .map(|s| s.height.saturating_sub(1).saturating_sub(1).saturating_sub(input_height))
                            .unwrap_or(20);

                        // Scroll by half of conversation panel height (or 5 lines minimum)
                        let scroll_amount = std::cmp::max(5, (conversation_height / 2) as usize);
                        state.scroll_offset = state.scroll_offset.saturating_sub(scroll_amount);
                    }
                    // Beta.247: PageDown - scroll conversation down (half of visible conversation area)
                    (KeyCode::PageDown, _) => {
                        // Calculate actual conversation panel height
                        let input_height = super::utils::calculate_input_height(&state.input, terminal.size().ok().map(|s| s.width.saturating_sub(8)).unwrap_or(80));
                        let conversation_height = terminal.size()
                            .ok()
                            .map(|s| s.height.saturating_sub(1).saturating_sub(1).saturating_sub(input_height))
                            .unwrap_or(20);

                        // Scroll by half of conversation panel height (or 5 lines minimum)
                        let scroll_amount = std::cmp::max(5, (conversation_height / 2) as usize);
                        state.scroll_offset = state.scroll_offset.saturating_add(scroll_amount);
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

/// Beta.235: Display fallback error screen when TUI fails to start
/// Uses plain stderr output when terminal can't be initialized
fn display_fallback_error(message: &str) {
    eprintln!("\n╔══════════════════════════════════════════════════════════╗");
    eprintln!("║                   TUI STARTUP FAILED                     ║");
    eprintln!("╚══════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("Error: {}", message);
    eprintln!();
    eprintln!("Troubleshooting:");
    eprintln!("  • Ensure you're running in a real terminal (TTY), not redirected");
    eprintln!("  • Try resizing your terminal window");
    eprintln!("  • Check terminal permissions: ls -la $(tty)");
    eprintln!("  • Use 'annactl status' for non-interactive mode");
    eprintln!();
}
