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
    _tx: mpsc::Sender<TuiMessage>,
    _rx: &mut mpsc::Receiver<TuiMessage>,
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
                            if handle_user_input(state).await {
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

    // Create main layout: [Header | Content | Status Bar | Input Bar]
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // Top header
            Constraint::Min(3),      // Main content
            Constraint::Length(1),   // Bottom status bar
            Constraint::Length(3),   // Input bar
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

    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .title("Conversation")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .wrap(Wrap { trim: true });  // Enable text wrapping!

    f.render_widget(paragraph, area);
}

/// Draw input bar (bottom)
fn draw_input_bar(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    let lang_indicator = format!("[{}]", state.language.as_str().to_uppercase());

    let input_text = Text::from(vec![
        Line::from(vec![
            Span::styled(lang_indicator, Style::default().fg(Color::Magenta)),
            Span::raw(" > "),
            Span::raw(&state.input),
            Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
        ]),
    ]);

    let input_block = Paragraph::new(input_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)));

    f.render_widget(input_block, area);
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

/// Handle user input
/// Returns true if should exit, false otherwise
async fn handle_user_input(state: &mut AnnaTuiState) -> bool {
    let input = state.input.clone();
    let input_lower = input.trim().to_lowercase();

    // Check for exit commands
    if input_lower == "bye" || input_lower == "exit" || input_lower == "quit" {
        state.add_system_message("Goodbye!".to_string());
        return true;
    }

    state.add_user_message(input.clone());
    state.input.clear();
    state.cursor_pos = 0;

    // Detect language change
    detect_language_change(&input, state);

    // Beta.91: Set thinking state before LLM processing
    state.is_thinking = true;
    state.thinking_frame = 0;

    // Generate reply (placeholder - will connect to recipe system)
    let reply = generate_reply(&input, state).await;

    // Beta.91: Clear thinking state after reply completes
    state.is_thinking = false;

    state.add_anna_reply(reply);

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

    // Call LLM (blocking call in spawn_blocking for async context)
    let llm_response = tokio::task::spawn_blocking(move || {
        llm_client.chat(&prompt)
    }).await;

    match llm_response {
        Ok(Ok(response)) => response.text,
        Ok(Err(e)) => format!("## LLM Error\n\nFailed to get response: {:?}", e),
        Err(e) => format!("## Internal Error\n\nTask failed: {:?}", e),
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

    // Pattern matching for template selection (Beta.93: expanded library)
    let (template_id, params) = if input_lower.contains("swap") {
        ("check_swap_status", HashMap::new())
    } else if input_lower.contains("gpu") || input_lower.contains("vram") {
        ("check_gpu_memory", HashMap::new())
    } else if input_lower.contains("kernel") {
        ("check_kernel_version", HashMap::new())
    } else if input_lower.contains("disk") || input_lower.contains("space") {
        ("check_disk_space", HashMap::new())
    } else if input_lower.contains("ram") || input_lower.contains("memory") || input_lower.contains("mem") {
        ("check_memory", HashMap::new())
    } else if input_lower.contains("uptime") {
        ("check_uptime", HashMap::new())
    } else if input_lower.contains("cpu model") || input_lower.contains("processor") {
        ("check_cpu_model", HashMap::new())
    } else if input_lower.contains("cpu load") || input_lower.contains("cpu usage") || input_lower.contains("load average") {
        ("check_cpu_load", HashMap::new())
    } else if input_lower.contains("distro") || input_lower.contains("distribution") || input_lower.contains("os-release") {
        ("check_distro", HashMap::new())
    } else if input_lower.contains("failed services") || (input_lower.contains("systemctl") && input_lower.contains("failed")) {
        ("check_failed_services", HashMap::new())
    } else if input_lower.contains("journal") || (input_lower.contains("system") && input_lower.contains("errors")) {
        ("check_journal_errors", HashMap::new())
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
    TelemetryUpdate,
}
