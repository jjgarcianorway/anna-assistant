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
    // Beta.91: Initialize telemetry with real data
    update_telemetry(state);

    // Track last telemetry update
    let mut last_telemetry_update = std::time::Instant::now();
    let telemetry_interval = std::time::Duration::from_secs(2);

    loop {
        // Beta.91: Update telemetry every 2 seconds
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
                            handle_user_input(state).await;
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
/// Format: 15:42 Nov 19 | Health: ✓ | Model: llama3.2:3b | CPU: 8% | RAM: 4.2GB
/// With thinking indicator: 15:42 Nov 19 | ⣾ Thinking... | Model: llama3.2:3b | CPU: 8% | RAM: 4.2GB
fn draw_status_bar(f: &mut Frame, area: Rect, state: &AnnaTuiState) {
    use chrono::Local;

    let now = Local::now();
    let time_str = now.format("%H:%M %b %d").to_string();

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
        Span::raw("Model: "),
        Span::styled(&state.llm_panel.model_name, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
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
    let items: Vec<ListItem> = state
        .conversation
        .iter()
        .map(|item| {
            let content = match item {
                ChatItem::User(msg) => {
                    Line::from(vec![
                        Span::styled("You: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                        Span::raw(msg),
                    ])
                }
                ChatItem::Anna(msg) => {
                    Line::from(vec![
                        Span::styled("Anna: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::raw(msg),
                    ])
                }
                ChatItem::System(msg) => {
                    Line::from(vec![
                        Span::styled("System: ", Style::default().fg(Color::Yellow)),
                        Span::raw(msg),
                    ])
                }
            };
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .title("Conversation")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));

    f.render_widget(list, area);
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
async fn handle_user_input(state: &mut AnnaTuiState) {
    let input = state.input.clone();
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

/// Generate reply using template library and recipe formatter
async fn generate_reply(input: &str, state: &AnnaTuiState) -> String {
    use anna_common::command_recipe::Recipe;
    use anna_common::template_library::TemplateLibrary;
    use crate::recipe_formatter::format_recipe_answer;
    use std::collections::HashMap;

    let library = TemplateLibrary::default();
    let input_lower = input.to_lowercase();

    // Pattern matching for template selection
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
    } else {
        // No matching template - return helpful message
        return match state.language {
            LanguageCode::Spanish => format!(
                "## Entiendo\n\n'{}'\n\n## Estado Actual\n\n\
                 Estoy en modo de desarrollo. Por ahora, puedo ayudarte con:\n\n\
                 - swap (estado de swap)\n\
                 - GPU/VRAM (memoria de GPU)\n\
                 - kernel (versión del kernel)\n\
                 - disk/space (espacio en disco)\n\
                 - RAM/memory (memoria del sistema)\n\n\
                 ## Próximamente\n\n\
                 Planner/Critic LLM para generar recetas dinámicamente.",
                input
            ),
            LanguageCode::French => format!(
                "## Compris\n\n'{}'\n\n## État Actuel\n\n\
                 Je suis en mode développement. Pour l'instant, je peux vous aider avec:\n\n\
                 - swap (état du swap)\n\
                 - GPU/VRAM (mémoire GPU)\n\
                 - kernel (version du noyau)\n\
                 - disk/space (espace disque)\n\
                 - RAM/memory (mémoire système)\n\n\
                 ## Bientôt\n\n\
                 Planner/Critic LLM pour générer des recettes dynamiquement.",
                input
            ),
            _ => format!(
                "## Understood\n\n'{}'\n\n## Current Status\n\n\
                 I'm in development mode. For now, I can help with:\n\n\
                 - **swap** - Check swap status\n\
                 - **GPU/VRAM** - Check GPU memory\n\
                 - **kernel** - Check kernel version\n\
                 - **disk/space** - Check disk space\n\
                 - **RAM/memory** - Check system memory\n\n\
                 ## Coming Soon\n\n\
                 Planner/Critic LLM loop to generate dynamic recipes from Arch Wiki.\n\n\
                 ## Architecture\n\n\
                 Templates → Planner LLM → Critic LLM → Validated Recipe → Execution",
                input
            ),
        };
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

    // Update LLM panel - check if Ollama is available
    // For now, hardcode to llama3.2:3b (will fix model detection in next task)
    state.llm_panel.model_name = "llama3.2:3b".to_string();
    state.llm_panel.model_size = "3B".to_string();
    state.llm_panel.mode = "Local".to_string();
    state.llm_panel.available = std::process::Command::new("ollama")
        .arg("list")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
}

/// TUI message types
#[derive(Debug)]
pub enum TuiMessage {
    UserInput(String),
    AnnaReply(String),
    TelemetryUpdate,
}
