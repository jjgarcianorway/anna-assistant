//! TUI (Terminal User Interface) REPL
//!
//! A modern, full-screen terminal interface for Anna using ratatui.
//! Inspired by Claude Code's clean, efficient design.
//!
//! Features:
//! - Full-screen terminal UI with clean layout
//! - Message history with scrollback
//! - Input field with multiline support
//! - Status bar with CPU/RAM/Model metrics
//! - LLM integration via daemon RPC
//! - Keyboard shortcuts
//! - Efficient rendering

use crate::llm_integration::query_llm_with_context;
use anna_common::context::db::{ContextDb, DbLocation};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use sysinfo::System;

/// System metrics for status bar
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub ram_usage: f32,
    pub ram_total: f32,
    pub model_name: String,
}

/// TUI application state
pub struct TuiApp {
    /// Message history
    messages: Vec<Message>,
    /// Current input buffer
    input: String,
    /// Input cursor position
    cursor_pos: usize,
    /// Should quit
    should_quit: bool,
    /// Scroll offset for message history
    scroll_offset: usize,
    /// Database for LLM config and context
    db: Option<Arc<ContextDb>>,
    /// System metrics
    sys: System,
    /// Last metrics update
    metrics: SystemMetrics,
    /// Processing response
    is_processing: bool,
    /// Channel for receiving LLM responses
    response_rx: Option<tokio::sync::mpsc::UnboundedReceiver<String>>,
}

/// A message in the conversation
#[derive(Clone, PartialEq)]
pub struct Message {
    /// Message role (user or assistant)
    pub role: MessageRole,
    /// Message content
    pub content: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
}

impl TuiApp {
    /// Create a new TUI app
    pub fn new(db: Option<Arc<ContextDb>>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Calculate average CPU usage
        let cpu_usage = if sys.cpus().is_empty() {
            0.0
        } else {
            sys.cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .sum::<f32>()
                / sys.cpus().len() as f32
        };

        // Beta.76: Load actual model name from database instead of "Loading..."
        let model_name = if let Some(ref database) = db {
            // Try to load LLM config synchronously (blocking is acceptable during TUI init)
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    database.load_llm_config().await
                        .map(|config| config.description)
                        .unwrap_or_else(|_| "Unknown".to_string())
                })
            })
        } else {
            "No database".to_string()
        };

        let metrics = SystemMetrics {
            cpu_usage,
            ram_usage: sys.used_memory() as f32 / (1024.0 * 1024.0 * 1024.0),
            ram_total: sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0),
            model_name,
        };

        Self {
            messages: Vec::new(),
            input: String::new(),
            cursor_pos: 0,
            should_quit: false,
            scroll_offset: 0,
            db,
            sys,
            metrics,
            is_processing: false,
            response_rx: None,
        }
    }

    /// Update system metrics
    pub fn update_metrics(&mut self) {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();

        // Calculate average CPU usage
        self.metrics.cpu_usage = if self.sys.cpus().is_empty() {
            0.0
        } else {
            self.sys
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .sum::<f32>()
                / self.sys.cpus().len() as f32
        };

        self.metrics.ram_usage = self.sys.used_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
        self.metrics.ram_total = self.sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
    }

    /// Send message to LLM and get response
    async fn send_to_llm(&self, message: String) -> Result<String> {
        match query_llm_with_context(&message, self.db.as_ref()).await {
            Ok(response) => Ok(response),
            Err(e) => Ok(format!("Error: {}", e)),
        }
    }

    /// Add a message to history
    pub fn add_message(&mut self, role: MessageRole, content: impl Into<String>) {
        self.messages.push(Message {
            role,
            content: content.into(),
        });
        // Auto-scroll to bottom
        self.scroll_offset = 0;
    }

    /// Check for and process LLM responses
    fn check_llm_response(&mut self) {
        if let Some(rx) = &mut self.response_rx {
            // Try to receive response (non-blocking)
            match rx.try_recv() {
                Ok(response) => {
                    // Remove "Thinking..." message
                    if let Some(last_msg) = self.messages.last() {
                        if last_msg.role == MessageRole::Assistant && last_msg.content == "Thinking..." {
                            self.messages.pop();
                        }
                    }

                    // Add actual response
                    self.add_message(MessageRole::Assistant, response);
                    self.is_processing = false;
                    self.response_rx = None; // Clear the channel
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    // No response yet, still processing
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    // Remove "Thinking..." message
                    if let Some(last_msg) = self.messages.last() {
                        if last_msg.role == MessageRole::Assistant && last_msg.content == "Thinking..." {
                            self.messages.pop();
                        }
                    }

                    // Channel closed, error occurred
                    self.add_message(MessageRole::Assistant, "Error: LLM request failed");
                    self.is_processing = false;
                    self.response_rx = None;
                }
            }
        }
    }

    /// Handle key press
    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match (key, modifiers) {
            // Quit
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
            | (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Submit message
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if !self.input.is_empty() && !self.is_processing {
                    let input = self.input.clone();
                    self.add_message(MessageRole::User, input.clone());
                    self.input.clear();
                    self.cursor_pos = 0;

                    // Show thinking indicator
                    self.add_message(MessageRole::Assistant, "Thinking...");
                    self.is_processing = true;

                    // Create channel for response
                    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                    self.response_rx = Some(rx);

                    // Clone db for async task
                    let db = self.db.clone();

                    // Spawn async task to query LLM
                    tokio::spawn(async move {
                        let response = match query_llm_with_context(&input, db.as_ref()).await {
                            Ok(r) => r,
                            Err(e) => format!("Error: {}", e),
                        };
                        let _ = tx.send(response);
                    });
                }
            }

            // Backspace
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                if self.cursor_pos > 0 {
                    self.input.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
            }

            // Delete
            (KeyCode::Delete, KeyModifiers::NONE) => {
                if self.cursor_pos < self.input.len() {
                    self.input.remove(self.cursor_pos);
                }
            }

            // Move cursor left
            (KeyCode::Left, KeyModifiers::NONE) => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }

            // Move cursor right
            (KeyCode::Right, KeyModifiers::NONE) => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
            }

            // Move cursor to start
            (KeyCode::Home, KeyModifiers::NONE) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                self.cursor_pos = 0;
            }

            // Move cursor to end
            (KeyCode::End, KeyModifiers::NONE) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.cursor_pos = self.input.len();
            }

            // Scroll up
            (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::PageUp, KeyModifiers::NONE) => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }

            // Scroll down
            (KeyCode::Down, KeyModifiers::NONE) | (KeyCode::PageDown, KeyModifiers::NONE) => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }

            // Regular character input
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }

            _ => {}
        }
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Run the TUI application
pub fn run() -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(_run_tui_async())
}

async fn _run_tui_async() -> Result<()> {
    // Try to connect to database
    let db_location = DbLocation::auto_detect();
    let db = match ContextDb::open(db_location).await {
        Ok(db) => {
            eprintln!("Connected to database");
            Some(Arc::new(db))
        }
        Err(e) => {
            eprintln!("Warning: Could not open database: {}", e);
            eprintln!("TUI will run without LLM support");
            None
        }
    };

    _run_tui(db)
}

fn _run_tui(db: Option<Arc<ContextDb>>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with database connection
    let mut app = TuiApp::new(db);

    // Add welcome message
    let welcome_msg = if app.db.is_some() {
        "Welcome to Anna's TUI REPL!\n\n\
         Controls:\n\
         - Type and press Enter to send messages\n\
         - Ctrl+C or Ctrl+Q to quit\n\
         - Arrow keys or Page Up/Down to scroll\n\
         - Ctrl+A/E or Home/End to move cursor\n\n\
         LLM integration is active. Ask me anything!"
    } else {
        "Welcome to Anna's TUI REPL (Demo Mode)\n\n\
         Database not connected. LLM functionality unavailable.\n\
         Please ensure annad is running and database is accessible."
    };

    app.add_message(MessageRole::Assistant, welcome_msg);

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

/// Main app loop
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut TuiApp) -> Result<()> {
    loop {
        // Check for LLM responses
        app.check_llm_response();

        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key.code, key.modifiers);
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Draw the UI
fn ui(f: &mut Frame, app: &mut TuiApp) {
    // Update metrics every frame
    app.update_metrics();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Messages area
            Constraint::Length(3), // Input area
            Constraint::Length(1), // Status bar
        ])
        .split(f.size());

    // Render messages
    render_messages(f, app, chunks[0]);

    // Render input
    render_input(f, app, chunks[1]);

    // Render status bar with live metrics
    render_status(f, app, chunks[2]);
}

/// Render message history
fn render_messages(f: &mut Frame, app: &TuiApp, area: Rect) {
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .rev() // Reverse to show newest at bottom
        .skip(app.scroll_offset)
        .take(area.height as usize - 2) // Account for borders
        .map(|msg| {
            let (prefix, style) = match msg.role {
                MessageRole::User => ("You: ", Style::default().fg(Color::Cyan)),
                MessageRole::Assistant => ("Anna: ", Style::default().fg(Color::Green)),
            };

            let content = format!("{}{}", prefix, msg.content);
            ListItem::new(Text::from(content)).style(style)
        })
        .collect();

    let messages_list = List::new(messages).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Conversation")
            .style(Style::default()),
    );

    f.render_widget(messages_list, area);
}

/// Render input field
fn render_input(f: &mut Frame, app: &TuiApp, area: Rect) {
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input (Enter to send)")
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(input, area);

    // Show cursor
    f.set_cursor(
        area.x + app.cursor_pos as u16 + 1, // +1 for border
        area.y + 1,                         // +1 for border
    );
}

/// Render status bar with live metrics
fn render_status(f: &mut Frame, app: &TuiApp, area: Rect) {
    let status_text = format!(
        "Anna TUI | CPU: {:.1}% | RAM: {:.1}/{:.1} GB | Model: {} | Ctrl+C=Quit",
        app.metrics.cpu_usage,
        app.metrics.ram_usage,
        app.metrics.ram_total,
        app.metrics.model_name
    );

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(status, area);
}
