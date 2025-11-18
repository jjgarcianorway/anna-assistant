//! TUI (Terminal User Interface) REPL
//!
//! A modern, full-screen terminal interface for Anna using ratatui.
//! Inspired by Claude Code's clean, efficient design.
//!
//! Features:
//! - Full-screen terminal UI with clean layout
//! - Message history with scrollback
//! - Input field with multiline support
//! - Status bar with system info
//! - Keyboard shortcuts
//! - Efficient rendering

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
}

/// A message in the conversation
#[derive(Clone)]
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
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            cursor_pos: 0,
            should_quit: false,
            scroll_offset: 0,
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
                if !self.input.is_empty() {
                    let input = self.input.clone();
                    self.add_message(MessageRole::User, input);
                    self.input.clear();
                    self.cursor_pos = 0;

                    // TODO: Send to LLM and get response
                    // For now, just echo back
                    self.add_message(
                        MessageRole::Assistant,
                        "TUI REPL is under construction. Full implementation coming soon!",
                    );
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
        Self::new()
    }
}

/// Run the TUI application
pub fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = TuiApp::new();

    // Add welcome message
    app.add_message(
        MessageRole::Assistant,
        "Welcome to Anna's TUI REPL! This is a prototype interface.\n\n\
         Controls:\n\
         - Type and press Enter to send messages\n\
         - Ctrl+C or Ctrl+Q to quit\n\
         - Arrow keys or Page Up/Down to scroll\n\
         - Ctrl+A/E or Home/End to move cursor\n\n\
         Full TUI implementation coming soon with:\n\
         - LLM integration\n\
         - Message streaming\n\
         - Syntax highlighting\n\
         - Command history\n\
         - And more!",
    );

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
fn ui(f: &mut Frame, app: &TuiApp) {
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

    // Render status bar
    render_status(f, chunks[2]);
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

/// Render status bar
fn render_status(f: &mut Frame, area: Rect) {
    let status = Paragraph::new("Anna TUI REPL (Prototype) | Ctrl+C to quit | Arrows to scroll")
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(status, area);
}
