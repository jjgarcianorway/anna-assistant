//! Real TUI Implementation - Claude-CLI / Codex style interface
//!
//! Three-pane layout:
//! - Left: System panel (telemetry, health, LLM status)
//! - Middle: Conversation scrollback
//! - Bottom: Input bar with language indicator
//!
//! NO println! after init. All rendering via ratatui frames.

use crate::tui_state::{AnnaTuiState, ChatItem, LanguageCode};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use tokio::sync::mpsc;

/// Run the TUI
pub async fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load state
    let mut state = AnnaTuiState::load().await.unwrap_or_default();

    // Create channels for async operations
    let (tx, mut rx) = mpsc::channel(32);

    // Run event loop
    let result = run_event_loop(&mut terminal, &mut state, tx, &mut rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Save state
    let _ = state.save().await;

    result
}

/// Main event loop
async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AnnaTuiState,
    tx: mpsc::Sender<TuiMessage>,
    rx: &mut mpsc::Receiver<TuiMessage>,
) -> Result<()> {
    // Beta.94: Initialize telemetry with real data
    update_telemetry(state);

    // Beta.94: Show welcome message on first launch
    if state.conversation.is_empty() {
        show_welcome_message(state);
    }

    // Track last telemetry update
    let mut last_telemetry_update = std::time::Instant::now();
    let telemetry_interval = std::time::Duration::from_secs(5);

    loop {
        // Beta.94: Update telemetry every 5 seconds
        if last_telemetry_update.elapsed() >= telemetry_interval {
            update_telemetry(state);
            last_telemetry_update = std::time::Instant::now();
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
                    // F1 - toggle help
                    (KeyCode::F(1), _) => {
                        state.show_help = !state.show_help;
                    }
                    // Enter - submit input
                    (KeyCode::Enter, _) => {
                        if !state.input.trim().is_empty() {
                            // Beta.108: Non-blocking input handling
                            if handle_user_input(state, tx.clone()) {
                                break;  // Exit requested
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
                    (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
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

/// Draw the UI - Claude CLI style with header and status bar
fn draw_ui(f: &mut Frame, state: &AnnaTuiState) {
    let size = f.size();

    // Beta.99: Calculate dynamic input bar height (3 to 10 lines max)
    let input_height = calculate_input_height(&state.input, size.width.saturating_sub(8));

    // Create main layout: [Header | Content | Status Bar | Input Bar]
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),       // Top header
            Constraint::Min(3),          // Main content
            Constraint::Length(1),       // Bottom status bar
            Constraint::Length(input_height),  // Beta.99: Dynamic input bar
        ])
        .split(size);

    // Draw top header
    draw_header(f, main_chunks[0], state);

    // Draw conversation panel (full width now)
    draw_conversation_panel(f, main_chunks[1], state);

    // Draw bottom status bar
    draw_status_bar(f, main_chunks[2], state);

    // Draw input bar
    draw_input_bar(f, main_chunks[3], state);

    // Draw help overlay if active
    if state.show_help {
        draw_help_overlay(f, size);
    }
}

/// Draw professional header (Claude CLI style)
/// Format: Anna v5.7.0 | llama3.2:3b | user@hostname | ● LIVE
fn draw_header(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    use std::env;

    let hostname = env::var("HOSTNAME")
        .or_else(|_| env::var("NAME"))
        .unwrap_or_else(|_| "localhost".to_string());
    let username = env::var("USER").unwrap_or_else(|_| "user".to_string());

    let header_text = Line::from(vec![
        Span::styled("Anna ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("v{}", state.system_panel.anna_version),
            Style::default().fg(Color::Gray)
        ),
        Span::raw(" | "),
        Span::styled(&state.llm_panel.model_name, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(format!("{}@{}", username, hostname), Style::default().fg(Color::Blue)),
        Span::raw(" | "),
        Span::styled(
            if state.llm_panel.available { "● LIVE" } else { "○ OFFLINE" },
            Style::default().fg(if state.llm_panel.available { Color::Green } else { Color::Red })
        ),
    ]);

    let header = Paragraph::new(header_text)
        .style(Style::default().bg(Color::Black));

    f.render_widget(header, area);
}

/// Draw professional status bar (bottom)
/// Format: 15:42:08 Nov 19 | Health: ✓ | CPU: 8% | RAM: 4.2GB
/// With thinking indicator: 15:42:08 Nov 19 | ⣾ Thinking... | CPU: 8% | RAM: 4.2GB
fn draw_status_bar(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    use chrono::Local;

    let now = Local::now();
    let time_str = now.format("%H:%M:%S %b %d").to_string();

    // Beta.91: Thinking indicator with animation
    let thinking_spinner = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    let thinking_indicator = if state.is_thinking {
        let frame = thinking_spinner[state.thinking_frame % thinking_spinner.len()];
        Some((frame, "Thinking..."))
    } else {
        None
    };

    let health_icon = if state.system_panel.cpu_load_1min < 80.0 && state.system_panel.ram_used_gb < 14.0 {
        ("✓", Color::Green)
    } else if state.system_panel.cpu_load_1min < 95.0 {
        ("⚠", Color::Yellow)
    } else {
        ("✗", Color::Red)
    };

    let mut spans = vec![
        Span::styled(time_str, Style::default().fg(Color::Gray)),
        Span::raw(" | "),
    ];

    // Add thinking indicator if active, otherwise show health
    if let Some((spinner, text)) = thinking_indicator {
        spans.push(Span::styled(spinner, Style::default().fg(Color::Cyan)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(text, Style::default().fg(Color::Cyan)));
        spans.push(Span::raw(" | "));
    } else {
        spans.push(Span::raw("Health: "));
        spans.push(Span::styled(health_icon.0, Style::default().fg(health_icon.1)));
        spans.push(Span::raw(" | "));
    }

    spans.extend_from_slice(&[
        Span::raw(format!("CPU: {:.0}%", state.system_panel.cpu_load_1min)),
        Span::raw(" | "),
        Span::raw(format!("RAM: {:.1}GB", state.system_panel.ram_used_gb)),
    ]);

    let status_text = Line::from(spans);
    let status_bar = Paragraph::new(status_text)
        .style(Style::default().bg(Color::Black));

    f.render_widget(status_bar, area);
}

/// Draw conversation panel (middle)
/// Beta.99: Added scrolling support with PageUp/PageDown
fn draw_conversation_panel(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    // Beta.93: Use wrapped text for conversation history (no more cut-off messages)
    let mut lines: Vec<Line> = Vec::new();

    for item in &state.conversation {
        match item {
            ChatItem::User(msg) => {
                lines.push(Line::from(vec![
                    Span::styled("You: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                    Span::raw(msg.clone()),
                ]));
                lines.push(Line::from("")); // Add spacing between messages
            }
            ChatItem::Anna(msg) => {
                lines.push(Line::from(vec![
                    Span::styled("Anna: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
                // Split long Anna replies into separate lines for better readability
                for line in msg.lines() {
                    lines.push(Line::from(Span::raw(line.to_string())));
                }
                lines.push(Line::from("")); // Add spacing between messages
            }
            ChatItem::System(msg) => {
                lines.push(Line::from(vec![
                    Span::styled("System: ", Style::default().fg(Color::Yellow)),
                    Span::raw(msg.clone()),
                ]));
                lines.push(Line::from("")); // Add spacing between messages
            }
        }
    }

    // Beta.99: Calculate scroll indicator and clamp scroll_offset
    let total_lines = lines.len();
    let visible_lines = area.height.saturating_sub(2) as usize; // Subtract 2 for borders
    let max_scroll = total_lines.saturating_sub(visible_lines);

    // Beta.115: Auto-scroll to bottom if scroll_offset is at max
    let actual_scroll = if state.scroll_offset == usize::MAX || state.scroll_offset >= max_scroll {
        max_scroll
    } else {
        state.scroll_offset
    };

    let scroll_indicator = if total_lines > visible_lines {
        format!(" [↑↓ {}/{}]", actual_scroll, max_scroll)
    } else {
        String::new()
    };

    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .title(format!("Conversation{}", scroll_indicator))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .wrap(Wrap { trim: true })  // Enable text wrapping!
        .scroll((actual_scroll as u16, 0));  // Beta.99: Enable scrolling!

    f.render_widget(paragraph, area);
}

/// Draw input bar (bottom)
/// Beta.99: Added text wrapping for multi-line input
fn draw_input_bar(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    let lang_indicator = format!("[{}]", state.language.as_str().to_uppercase());
    let prompt = format!("{} > ", lang_indicator);

    // Beta.99: Build wrapped text with proper formatting
    let input_text = format!("{}{}_", prompt, &state.input);

    let paragraph = Paragraph::new(input_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });  // Beta.99: Enable wrapping!

    f.render_widget(paragraph, area);
}

/// Beta.99: Calculate dynamic input bar height
/// Returns height in range [3, 10] based on input content
fn calculate_input_height(input: &str, available_width: u16) -> u16 {
    if input.is_empty() {
        return 3;  // Minimum height for empty input
    }

    // Account for prompt "[EN] > " (approximately 7 chars) + cursor "_"
    let prompt_width = 8;
    let usable_width = available_width.saturating_sub(prompt_width).max(20) as usize;

    // Calculate how many lines the input will wrap to
    let input_len = input.len();
    let wrapped_lines = (input_len + usable_width - 1) / usable_width;

    // Add 2 for borders, then cap between 3 and 10
    let total_height = (wrapped_lines + 2).max(3).min(10) as u16;

    total_height
}

/// Draw help overlay
fn draw_help_overlay(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Keyboard Shortcuts", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Ctrl+C", Style::default().fg(Color::Cyan)),
            Span::raw(" - Exit"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+L", Style::default().fg(Color::Cyan)),
            Span::raw(" - Clear conversation"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+U", Style::default().fg(Color::Cyan)),
            Span::raw(" - Clear input"),
        ]),
        Line::from(vec![
            Span::styled("F1", Style::default().fg(Color::Cyan)),
            Span::raw(" - Toggle help"),
        ]),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw(" - Navigate history"),
        ]),
        Line::from(vec![
            Span::styled("PgUp/PgDn", Style::default().fg(Color::Cyan)),
            Span::raw(" - Scroll conversation"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Press F1 to close", Style::default().fg(Color::Gray))),
    ];

    // Center the help box
    let help_area = centered_rect(50, 50, area);

    let help_block = Paragraph::new(help_text)
        .block(Block::default()
            .title("Help")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .style(Style::default().bg(Color::Black));

    f.render_widget(help_block, help_area);
}

/// Create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Beta.108: Handle user input (non-blocking)
/// Returns true if should exit, false otherwise
fn handle_user_input(state: &mut AnnaTuiState, tx: mpsc::Sender<TuiMessage>) -> bool {
    let input = state.input.clone();
    let input_lower = input.trim().to_lowercase();

    // Check for exit commands
    if input_lower == "bye" || input_lower == "exit" || input_lower == "quit" {
        state.add_system_message("Goodbye!".to_string());
        return true;
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

    // Beta.108: Spawn LLM query in background (non-blocking)
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
    };

    tokio::spawn(async move {
        let reply = generate_reply_streaming(&input, &state_clone, tx.clone()).await;
        // If template-based (non-streaming), send as complete reply
        if !reply.is_empty() {
            let _ = tx.send(TuiMessage::AnnaReply(reply)).await;
        }
    });

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

/// Generate reply using LLM for questions without template match
async fn generate_llm_reply(input: &str, state: &AnnaTuiState) -> String {
    use anna_common::llm::{LlmClient, LlmConfig, LlmPrompt};

    // Build system context from telemetry
    let system_context = format!(
        "System Information:\n\
         - CPU: {}\n\
         - CPU Load: {:.2}, {:.2}, {:.2} (1/5/15 min)\n\
         - RAM: {:.1} GB used / {:.1} GB total\n\
         - GPU: {}\n\
         - Disk: {:.1} GB free\n\
         - OS: Arch Linux\n\
         - Anna Version: {}",
        state.system_panel.cpu_model,
        state.system_panel.cpu_load_1min,
        state.system_panel.cpu_load_5min,
        state.system_panel.cpu_load_15min,
        state.system_panel.ram_used_gb,
        state.system_panel.ram_total_gb,
        state.system_panel.gpu_name.as_ref().map(|s| s.as_str()).unwrap_or("None"),
        state.system_panel.disk_free_gb,
        state.system_panel.anna_version,
    );

    // Use detected model from state, fallback to llama3.1:8b if none detected
    let model_name = if state.llm_panel.model_name == "None" || state.llm_panel.model_name == "Unknown" || state.llm_panel.model_name == "Ollama N/A" {
        "llama3.1:8b"
    } else {
        &state.llm_panel.model_name
    };

    let llm_config = LlmConfig::local("http://127.0.0.1:11434/v1", model_name);

    let llm_client = match LlmClient::from_config(&llm_config) {
        Ok(client) => client,
        Err(_) => {
            // LLM not available - return helpful fallback
            return format!(
                "## ⚠ LLM Unavailable\n\n\
                 I couldn't connect to the local LLM server (Ollama).\n\n\
                 **Your question:** {}\n\n\
                 **What I can help with (using templates):**\n\
                 - swap - Check swap status\n\
                 - GPU/VRAM - Check GPU memory\n\
                 - kernel - Check kernel version\n\
                 - disk/space - Check disk space\n\
                 - RAM/memory - Check system memory\n\n\
                 **To enable full LLM responses:**\n\
                 1. Install Ollama: `curl -fsSL https://ollama.com/install.sh | sh`\n\
                 2. Pull a model: `ollama pull llama3.1:8b`\n\
                 3. Ensure Ollama is running: `ollama list`",
                input
            );
        }
    };

    // Build prompt with system context
    let system_prompt = format!(
        "{}\n\n{}",
        LlmClient::anna_system_prompt(),
        system_context
    );

    let prompt = LlmPrompt {
        system: system_prompt,
        user: input.to_string(),
        conversation_history: None,
    };

    // Beta.111: Word-by-word streaming for consistency with one-shot and REPL modes
    // TUI accumulates chunks into a string buffer (can't print to stdout like REPL)
    let mut full_response = String::new();
    let mut callback = |chunk: &str| {
        full_response.push_str(chunk);
    };

    match llm_client.chat_stream(&prompt, &mut callback) {
        Ok(_) => full_response,
        Err(e) => format!("## LLM Error\n\nFailed to get response: {:?}", e),
    }
}

/// Generate reply using template library and recipe formatter
async fn generate_reply(input: &str, state: &AnnaTuiState) -> String {
    use anna_common::command_recipe::Recipe;
    use anna_common::template_library::TemplateLibrary;
    use crate::recipe_formatter::format_recipe_answer;
    use std::collections::HashMap;

    let library = TemplateLibrary::default();
    let input_lower = input.to_lowercase();

    // Beta.108: Helper function for word-boundary keyword matching
    // Prevents false positives like "programming" matching "ram"
    let contains_word = |text: &str, keyword: &str| {
        text.split(|c: char| !c.is_alphanumeric())
            .any(|word| word == keyword)
    };

    // Pattern matching for template selection (Beta.112: MASSIVELY expanded - 68 templates)
    let (template_id, params) = if contains_word(&input_lower, "swap") {
        ("check_swap_status", HashMap::new())
    } else if contains_word(&input_lower, "gpu") || contains_word(&input_lower, "vram") {
        ("check_gpu_memory", HashMap::new())
    } else if input_lower.contains("wifi") || input_lower.contains("wireless") ||
              (input_lower.contains("network") && (input_lower.contains("slow") || input_lower.contains("issue") || input_lower.contains("problem"))) {
        ("wifi_diagnostics", HashMap::new())
    } else if contains_word(&input_lower, "kernel") && (input_lower.contains("version") || input_lower.contains("what") || input_lower.contains("running")) {
        ("check_kernel_version", HashMap::new())
    } else if input_lower.contains("disk") && input_lower.contains("space") {
        ("check_disk_space", HashMap::new())
    } else if (contains_word(&input_lower, "memory") || contains_word(&input_lower, "ram")) && !input_lower.contains("gpu") {
        ("check_memory", HashMap::new())
    } else if contains_word(&input_lower, "uptime") {
        ("check_uptime", HashMap::new())
    } else if contains_word(&input_lower, "cpu") && (input_lower.contains("model") || input_lower.contains("what") || input_lower.contains("processor")) {
        ("check_cpu_model", HashMap::new())
    } else if contains_word(&input_lower, "cpu") && (input_lower.contains("load") || input_lower.contains("usage")) {
        ("check_cpu_load", HashMap::new())
    } else if contains_word(&input_lower, "distro") || (input_lower.contains("arch") && input_lower.contains("version")) {
        ("check_distro", HashMap::new())
    } else if input_lower.contains("failed") && input_lower.contains("service") {
        ("check_failed_services", HashMap::new())
    } else if input_lower.contains("journal") && input_lower.contains("error") {
        ("check_journal_errors", HashMap::new())
    } else if input_lower.contains("weak") || (input_lower.contains("diagnostic") && input_lower.contains("system")) {
        ("system_weak_points_diagnostic", HashMap::new())

    // Beta.112: PACKAGE MANAGEMENT (13 new templates)
    } else if input_lower.contains("orphan") || (input_lower.contains("unused") && input_lower.contains("package")) {
        ("list_orphaned_packages", HashMap::new())
    } else if input_lower.contains("aur") {
        ("list_aur_packages", HashMap::new())
    } else if input_lower.contains("pacman") && (input_lower.contains("cache") || input_lower.contains("size")) {
        ("check_pacman_cache_size", HashMap::new())
    } else if input_lower.contains("mirror") {
        ("check_pacman_mirrors", HashMap::new())
    } else if (input_lower.contains("update") || input_lower.contains("upgrade")) && !input_lower.contains("brain") {
        ("check_package_updates", HashMap::new())
    } else if input_lower.contains("keyring") {
        ("check_archlinux_keyring", HashMap::new())
    } else if input_lower.contains("package") && input_lower.contains("integrity") {
        ("check_package_integrity", HashMap::new())
    } else if input_lower.contains("clean") && input_lower.contains("cache") {
        ("clean_package_cache", HashMap::new())
    } else if input_lower.contains("explicit") && input_lower.contains("package") {
        ("list_explicit_packages", HashMap::new())
    } else if input_lower.contains("pacman") && (input_lower.contains("status") || input_lower.contains("lock")) {
        ("check_pacman_status", HashMap::new())
    } else if input_lower.contains("dependency") && input_lower.contains("conflict") {
        ("check_dependency_conflicts", HashMap::new())
    } else if input_lower.contains("pending") && input_lower.contains("update") {
        ("check_pending_updates", HashMap::new())
    } else if input_lower.contains("recent") && input_lower.contains("pacman") {
        ("show_recent_pacman_operations", HashMap::new())

    // Beta.112: BOOT & SYSTEMD (8 new templates)
    } else if input_lower.contains("boot") && (input_lower.contains("time") || input_lower.contains("slow")) {
        ("analyze_boot_time", HashMap::new())
    } else if input_lower.contains("boot") && input_lower.contains("error") {
        ("check_boot_errors", HashMap::new())
    } else if input_lower.contains("boot") && input_lower.contains("log") {
        ("show_boot_log", HashMap::new())
    } else if input_lower.contains("systemd") && input_lower.contains("timer") {
        ("check_systemd_timers", HashMap::new())
    } else if input_lower.contains("journal") && input_lower.contains("size") {
        ("analyze_journal_size", HashMap::new())
    } else if input_lower.contains("recent") && input_lower.contains("error") {
        ("show_recent_journal_errors", HashMap::new())
    } else if input_lower.contains("kernel") && input_lower.contains("update") {
        ("check_recent_kernel_updates", HashMap::new())
    } else if input_lower.contains("systemd") && input_lower.contains("version") {
        ("check_systemd_version", HashMap::new())

    // Beta.112: CPU & PERFORMANCE (8 new templates)
    } else if input_lower.contains("cpu") && (input_lower.contains("freq") || input_lower.contains("speed")) {
        ("check_cpu_frequency", HashMap::new())
    } else if input_lower.contains("cpu") && input_lower.contains("governor") {
        ("check_cpu_governor", HashMap::new())
    } else if input_lower.contains("cpu") && input_lower.contains("usage") {
        ("analyze_cpu_usage", HashMap::new())
    } else if (input_lower.contains("cpu") || input_lower.contains("temperature")) && input_lower.contains("temp") {
        ("check_cpu_temperature", HashMap::new())
    } else if input_lower.contains("throttl") {
        ("detect_cpu_throttling", HashMap::new())
    } else if input_lower.contains("top") && input_lower.contains("cpu") {
        ("show_top_cpu_processes", HashMap::new())
    } else if input_lower.contains("load") && input_lower.contains("average") {
        ("check_load_average", HashMap::new())
    } else if input_lower.contains("context") && input_lower.contains("switch") {
        ("analyze_context_switches", HashMap::new())

    // Beta.112: MEMORY (6 new templates)
    } else if input_lower.contains("memory") && input_lower.contains("usage") {
        ("check_memory_usage", HashMap::new())
    } else if input_lower.contains("swap") && input_lower.contains("usage") {
        ("check_swap_usage", HashMap::new())
    } else if input_lower.contains("memory") && input_lower.contains("pressure") {
        ("analyze_memory_pressure", HashMap::new())
    } else if input_lower.contains("top") && input_lower.contains("memory") {
        ("show_top_memory_processes", HashMap::new())
    } else if input_lower.contains("oom") {
        ("check_oom_killer", HashMap::new())
    } else if input_lower.contains("huge") && input_lower.contains("page") {
        ("check_huge_pages", HashMap::new())

    // Beta.112: NETWORK (7 new templates)
    } else if input_lower.contains("dns") {
        ("check_dns_resolution", HashMap::new())
    } else if input_lower.contains("network") && input_lower.contains("interface") {
        ("check_network_interfaces", HashMap::new())
    } else if input_lower.contains("routing") || input_lower.contains("route") {
        ("check_routing_table", HashMap::new())
    } else if input_lower.contains("firewall") {
        ("check_firewall_rules", HashMap::new())
    } else if input_lower.contains("port") && input_lower.contains("listen") {
        ("check_listening_ports", HashMap::new())
    } else if input_lower.contains("latency") || input_lower.contains("ping") {
        ("check_network_latency", HashMap::new())
    } else if input_lower.contains("networkmanager") {
        ("check_networkmanager_status", HashMap::new())

    // Beta.112: GPU & DISPLAY (9 new templates)
    } else if input_lower.contains("nvidia") && !input_lower.contains("install") {
        ("check_nvidia_status", HashMap::new())
    } else if input_lower.contains("amd") && (input_lower.contains("gpu") || input_lower.contains("radeon")) {
        ("check_amd_gpu", HashMap::new())
    } else if input_lower.contains("gpu") && input_lower.contains("driver") {
        ("check_gpu_drivers", HashMap::new())
    } else if input_lower.contains("gpu") && input_lower.contains("process") {
        ("check_gpu_processes", HashMap::new())
    } else if input_lower.contains("gpu") && input_lower.contains("temp") {
        ("check_gpu_temperature", HashMap::new())
    } else if input_lower.contains("display") || input_lower.contains("xorg") || input_lower.contains("wayland") {
        ("check_display_server", HashMap::new())
    } else if input_lower.contains("desktop") && input_lower.contains("environment") {
        ("check_desktop_environment", HashMap::new())
    } else if input_lower.contains("xorg") && input_lower.contains("error") {
        ("analyze_xorg_errors", HashMap::new())
    } else if input_lower.contains("wayland") && input_lower.contains("compositor") {
        ("check_wayland_compositor", HashMap::new())

    // Beta.112: HARDWARE (4 new templates)
    } else if input_lower.contains("temperature") || input_lower.contains("temp") || input_lower.contains("heat") {
        ("check_temperature", HashMap::new())
    } else if input_lower.contains("usb") {
        ("check_usb_devices", HashMap::new())
    } else if input_lower.contains("pci") {
        ("check_pci_devices", HashMap::new())
    } else if input_lower.contains("hostname") {
        ("check_hostname", HashMap::new())
    } else {
        // No matching template - use LLM to generate response
        return generate_llm_reply(input, state).await;
    };

    // Instantiate template
    match library.get(template_id) {
        Some(template) => {
            match template.instantiate(&params) {
                Ok(recipe_step) => {
                    // Wrap in full recipe structure
                    let recipe = Recipe {
                        question: input.to_string(),
                        steps: vec![recipe_step.clone()],
                        overall_safety: recipe_step.safety_level,
                        all_read_only: true,
                        wiki_sources: recipe_step.doc_sources.clone(),
                        summary: recipe_step.explanation.clone(),
                        generated_by: Some("template_library".to_string()),
                        critic_approval: None,
                    };

                    // Format with recipe formatter
                    format_recipe_answer(&recipe, input)
                }
                Err(e) => format!("## Error\n\nFailed to instantiate template: {}", e),
            }
        }
        None => format!("## Error\n\nTemplate '{}' not found", template_id),
    }
}

/// Beta.115: Generate reply with streaming support
async fn generate_reply_streaming(input: &str, state: &AnnaTuiState, tx: mpsc::Sender<TuiMessage>) -> String {
    use anna_common::command_recipe::Recipe;
    use anna_common::template_library::TemplateLibrary;
    use crate::recipe_formatter::format_recipe_answer;
    use std::collections::HashMap;

    let library = TemplateLibrary::default();
    let input_lower = input.to_lowercase();

    // Helper function for word-boundary keyword matching
    let contains_word = |text: &str, keyword: &str| {
        text.split(|c: char| !c.is_alphanumeric())
            .any(|word| word == keyword)
    };

    // Check if input matches a template (same logic as generate_reply)
    // If it does, return template response immediately (non-streaming)
    // If not, use streaming LLM
    let has_template = contains_word(&input_lower, "swap") ||
                      contains_word(&input_lower, "gpu") ||
                      contains_word(&input_lower, "vram") ||
                      input_lower.contains("wifi") ||
                      input_lower.contains("wireless") ||
                      input_lower.contains("kernel") ||
                      input_lower.contains("disk") ||
                      contains_word(&input_lower, "memory") ||
                      contains_word(&input_lower, "ram") ||
                      contains_word(&input_lower, "uptime") ||
                      contains_word(&input_lower, "cpu") ||
                      contains_word(&input_lower, "distro") ||
                      input_lower.contains("failed") ||
                      input_lower.contains("service") ||
                      input_lower.contains("journal") ||
                      input_lower.contains("error") ||
                      input_lower.contains("orphan") ||
                      input_lower.contains("package") ||
                      input_lower.contains("aur") ||
                      input_lower.contains("pacman") ||
                      input_lower.contains("mirror") ||
                      input_lower.contains("update") ||
                      input_lower.contains("upgrade") ||
                      input_lower.contains("boot") ||
                      input_lower.contains("systemd") ||
                      input_lower.contains("load") ||
                      input_lower.contains("temperature") ||
                      input_lower.contains("throttl") ||
                      input_lower.contains("network") ||
                      input_lower.contains("dns") ||
                      input_lower.contains("firewall") ||
                      input_lower.contains("port") ||
                      input_lower.contains("storage") ||
                      input_lower.contains("battery") ||
                      input_lower.contains("bluetooth") ||
                      input_lower.contains("audio") ||
                      input_lower.contains("usb") ||
                      input_lower.contains("pci") ||
                      input_lower.contains("hostname");

    if has_template {
        // Use non-streaming template-based reply
        return generate_reply(input, state).await;
    }

    // TODO Beta.118: Add RecipePlanner tier here (like one-off mode has)
    // Current: Templates → LLM (2 tiers)
    // Needed:  Templates → RecipePlanner → LLM (3 tiers, consistent with one-off)
    //
    // Challenges:
    // 1. RecipePlanner needs interactive confirmation (can't do in streaming TUI)
    // 2. Recipe execution needs step-by-step display (requires TUI redesign)
    // 3. Need to integrate with channel-based messaging
    //
    // For now, TUI only uses generic LLM (inconsistent with one-off mode)
    // This is why same question gets different answers in TUI vs one-off

    // No template match - use streaming LLM
    generate_llm_reply_streaming(input, state, tx).await;
    String::new() // Return empty - streaming handled via channel
}

/// Beta.115: Generate LLM reply with streaming chunks sent via channel
async fn generate_llm_reply_streaming(input: &str, state: &AnnaTuiState, tx: mpsc::Sender<TuiMessage>) {
    use anna_common::llm::{LlmClient, LlmConfig, LlmPrompt};

    // Build system context (same as generate_llm_reply)
    let system_context = format!(
        "System Information:\n\
         - CPU: {}\n\
         - CPU Load: {:.2}, {:.2}, {:.2} (1/5/15 min)\n\
         - RAM: {:.1} GB used / {:.1} GB total\n\
         - GPU: {}\n\
         - Disk: {:.1} GB free\n\
         - OS: Arch Linux\n\
         - Anna Version: {}",
        state.system_panel.cpu_model,
        state.system_panel.cpu_load_1min,
        state.system_panel.cpu_load_5min,
        state.system_panel.cpu_load_15min,
        state.system_panel.ram_used_gb,
        state.system_panel.ram_total_gb,
        state.system_panel.gpu_name.as_ref().unwrap_or(&"None".to_string()),
        state.system_panel.disk_free_gb,
        state.system_panel.anna_version
    );

    // Detect model
    let model_name = if state.llm_panel.model_name == "None" || state.llm_panel.model_name == "Unknown" || state.llm_panel.model_name == "Ollama N/A" {
        "llama3.1:8b"
    } else {
        &state.llm_panel.model_name
    };

    let llm_config = LlmConfig::local("http://127.0.0.1:11434/v1", model_name);

    let llm_client = match LlmClient::from_config(&llm_config) {
        Ok(client) => client,
        Err(_) => {
            // LLM not available
            let _ = tx.send(TuiMessage::AnnaReply(
                "## ⚠ LLM Unavailable\n\nI couldn't connect to the local LLM server (Ollama).".to_string()
            )).await;
            return;
        }
    };

    // Build prompt
    let system_prompt = format!(
        "{}\n\n{}",
        LlmClient::anna_system_prompt(),
        system_context
    );

    let prompt = LlmPrompt {
        system: system_prompt,
        user: input.to_string(),
        conversation_history: None,
    };

    // Beta.115: Streaming callback - send each chunk via channel
    let tx_clone = tx.clone();
    let mut callback = move |chunk: &str| {
        let chunk_string = chunk.to_string();
        let tx_inner = tx_clone.clone();
        // Send chunk asynchronously
        tokio::spawn(async move {
            let _ = tx_inner.send(TuiMessage::AnnaReplyChunk(chunk_string)).await;
        });
    };

    match llm_client.chat_stream(&prompt, &mut callback) {
        Ok(_) => {
            // Streaming complete
            let _ = tx.send(TuiMessage::AnnaReplyComplete).await;
        }
        Err(_) => {
            let _ = tx.send(TuiMessage::AnnaReply("## LLM Error\n\nFailed to get response.".to_string())).await;
        }
    }
}

/// Beta.94: Show proactive welcome message with system info
fn show_welcome_message(state: &mut AnnaTuiState) {
    use std::env;

    let username = env::var("USER").unwrap_or_else(|_| "friend".to_string());

    // Gather system highlights
    let cpu_status = if state.system_panel.cpu_load_1min < 50.0 {
        "✅ running smoothly"
    } else if state.system_panel.cpu_load_1min < 80.0 {
        "⚠️ moderate load"
    } else {
        "🔥 high load"
    };

    let ram_status = if state.system_panel.ram_used_gb < state.system_panel.ram_total_gb * 0.7 {
        "✅ plenty available"
    } else if state.system_panel.ram_used_gb < state.system_panel.ram_total_gb * 0.9 {
        "⚠️ getting full"
    } else {
        "🔴 critically low"
    };

    let llm_status = if state.llm_panel.available {
        format!("✅ {} ready", state.llm_panel.model_name)
    } else {
        "⚠️ LLM not available (install Ollama)".to_string()
    };

    // Build beautiful welcome message
    let welcome = format!(
        "👋 **Hello {}!** Welcome to Anna v{}\n\n\
         Here's what I can tell you right now:\n\n\
         🖥️  **System Status:**\n\
         • CPU: {} ({:.0}% load)\n\
         • RAM: {:.1}GB / {:.1}GB used ({})\n\
         • Disk: {:.1}GB free\n\
         {}\n\n\
         🤖 **AI Assistant:**\n\
         • {}\n\n\
         💡 **Quick Actions:**\n\
         • Ask about system health: \"how is my system?\"\n\
         • Check resources: \"how much RAM do I have?\"\n\
         • Monitor services: \"show failed services\"\n\
         • Get help: Press F1\n\n\
         **What would you like to know or do?**",
        username,
        state.system_panel.anna_version,
        state.system_panel.cpu_model,
        state.system_panel.cpu_load_1min,
        state.system_panel.ram_used_gb,
        state.system_panel.ram_total_gb,
        ram_status,
        if state.system_panel.gpu_name.is_some() {
            format!("• GPU: {}\n", state.system_panel.gpu_name.as_ref().unwrap())
        } else {
            String::new()
        },
        llm_status,
        cpu_status
    );

    state.add_anna_reply(welcome);
}

/// Update telemetry data in state
fn update_telemetry(state: &mut AnnaTuiState) {
    use crate::system_query::query_system_telemetry;

    // Beta.91: Collect real system telemetry
    if let Ok(telemetry) = query_system_telemetry() {
        // Update system panel
        state.system_panel.cpu_model = telemetry.hardware.cpu_model.clone();
        state.system_panel.cpu_load_1min = telemetry.cpu.load_avg_1min;
        state.system_panel.cpu_load_5min = telemetry.cpu.load_avg_5min;
        state.system_panel.cpu_load_15min = 0.0; // Not collected by query_cpu yet
        state.system_panel.ram_total_gb = telemetry.memory.total_mb as f64 / 1024.0;
        state.system_panel.ram_used_gb = telemetry.memory.used_mb as f64 / 1024.0;
        state.system_panel.anna_version = env!("CARGO_PKG_VERSION").to_string();

        // Update GPU info if available
        state.system_panel.gpu_name = telemetry.hardware.gpu_info.clone();
        // GPU VRAM would need nvidia-smi or similar

        state.telemetry_ok = true;
    } else {
        state.telemetry_ok = false;
    }

    // Update LLM panel - detect actual Ollama model
    state.llm_panel.mode = "Local".to_string();

    // Run `ollama list` and parse output to detect installed models
    match std::process::Command::new("ollama").arg("list").output() {
        Ok(output) if output.status.success() => {
            state.llm_panel.available = true;

            // Parse ollama list output (format: NAME ID SIZE MODIFIED)
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Get first non-header line (most recently used model)
            if let Some(first_line) = stdout.lines().skip(1).next() {
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if let Some(model_name) = parts.first() {
                    state.llm_panel.model_name = model_name.to_string();

                    // Extract size from model name (e.g., "llama3.1:8b" -> "8B")
                    if let Some(size_part) = model_name.split(':').nth(1) {
                        state.llm_panel.model_size = size_part.to_uppercase();
                    } else {
                        state.llm_panel.model_size = "Unknown".to_string();
                    }
                } else {
                    // Fallback if parsing fails
                    state.llm_panel.model_name = "Unknown".to_string();
                    state.llm_panel.model_size = "?".to_string();
                }
            } else {
                // No models installed
                state.llm_panel.model_name = "None".to_string();
                state.llm_panel.model_size = "-".to_string();
                state.llm_panel.available = false;
            }
        }
        _ => {
            // Ollama not available or command failed
            state.llm_panel.available = false;
            state.llm_panel.model_name = "Ollama N/A".to_string();
            state.llm_panel.model_size = "-".to_string();
        }
    }
}

/// TUI message types
#[derive(Debug)]
pub enum TuiMessage {
    UserInput(String),
    AnnaReply(String),
    AnnaReplyChunk(String),  // Beta.115: Streaming chunk
    AnnaReplyComplete,       // Beta.115: Streaming complete
    TelemetryUpdate,
}
