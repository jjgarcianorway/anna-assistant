//! Anna Configurator TUI - Beautiful interactive configuration interface
//!
//! This module provides a ratatui-based TUI for configuring Anna with:
//! - 7 sections: Profile, Priorities, Desktop, Safety, Scheduler, Modules, Review
//! - Three-panel layout: Sidebar (navigation) + Center (content) + Help (context)
//! - Keyboard navigation with live preview
//! - Safe apply with snapshot creation

use anyhow::{Context, Result};
use anna_common::config_api::{self, Actor, EventSource};
use anna_common::configurator::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, BorderType, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

// ═══════════════════════════════════════════════════════════════════════════════
// Color Palette (Anna's Beautiful Theme)
// ═══════════════════════════════════════════════════════════════════════════════

const PASTEL_BLUE: Color = Color::Rgb(137, 180, 250);
const PASTEL_GREEN: Color = Color::Rgb(166, 227, 161);
const PASTEL_YELLOW: Color = Color::Rgb(249, 226, 175);
const PASTEL_PINK: Color = Color::Rgb(245, 194, 231);
const PASTEL_PEACH: Color = Color::Rgb(250, 179, 135);
const MUTED_TEXT: Color = Color::Rgb(166, 173, 200);
const DIM_TEXT: Color = Color::Rgb(108, 112, 134);
const BG_DARK: Color = Color::Rgb(30, 30, 46);

// ═══════════════════════════════════════════════════════════════════════════════
// Application State
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Profile,
    Priorities,
    Desktop,
    Safety,
    Scheduler,
    Modules,
    Review,
}

impl Section {
    fn all() -> Vec<Self> {
        vec![
            Self::Profile,
            Self::Priorities,
            Self::Desktop,
            Self::Safety,
            Self::Scheduler,
            Self::Modules,
            Self::Review,
        ]
    }

    fn title(&self) -> &str {
        match self {
            Self::Profile => "1. Profile",
            Self::Priorities => "2. Priorities",
            Self::Desktop => "3. Desktop",
            Self::Safety => "4. Safety",
            Self::Scheduler => "5. Scheduler",
            Self::Modules => "6. Modules",
            Self::Review => "7. Review",
        }
    }

    fn help_text(&self) -> Vec<&str> {
        match self {
            Self::Profile => vec![
                "💡 Profile Quick Reference",
                "",
                "minimal",
                "  Bare necessities",
                "",
                "beautiful (current)",
                "  Balanced aesthetics",
                "",
                "workstation",
                "  Dev tools, editors",
                "",
                "gaming",
                "  GPU, latency opts",
                "",
                "server",
                "  Stability, uptime",
                "",
                "📖 ArchWiki:",
                "   System_Profile",
            ],
            Self::Priorities => vec![
                "⚖️ Priority Balance",
                "",
                "These sliders affect",
                "how Anna ranks",
                "recommendations.",
                "",
                "\"balanced\" is safe",
                "for most systems.",
                "",
                "\"maximum\" may",
                "require trade-offs.",
                "",
                "\"conservative\"",
                "avoids all risk.",
                "",
                "📖 ArchWiki:",
                "   System_Tuning",
            ],
            Self::Desktop => vec![
                "🖥️ Desktop Policy",
                "",
                "Anna can beautify",
                "your environment",
                "with your consent.",
                "",
                "Supported:",
                " • Hyprland",
                " • Sway",
                " • i3",
                " • Foot",
                " • Kitty",
                " • Alacritty",
                "",
                "All changes:",
                " - Create backup",
                " - Show diff",
                " - Get confirmation",
            ],
            Self::Safety => vec![
                "🛡️ Safety First",
                "",
                "Anna creates a",
                "snapshot before",
                "every change.",
                "",
                "Snapshots include:",
                " • Config files",
                " • Package state",
                " • System facts",
                " • Audit log entry",
                "",
                "Rollback anytime:",
                "  annactl rollback",
                "",
                "📖 ArchWiki:",
                "   System_backup",
            ],
            Self::Scheduler => vec![
                "⏰ Scheduling Info",
                "",
                "Anna collects facts",
                "about your system",
                "periodically.",
                "",
                "Jitter prevents",
                "predictable timing.",
                "",
                "Facts collected:",
                " • CPU/RAM usage",
                " • Disk space",
                " • Package updates",
                " • System events",
                "",
                "📖 ArchWiki:",
                "   Systemd/Timers",
            ],
            Self::Modules => vec![
                "🧩 Module Guide",
                "",
                "Enable modules to",
                "get advice in those",
                "areas.",
                "",
                "Disable modules to",
                "skip categories you",
                "don't want changed.",
                "",
                "All modules respect",
                "your autonomy and",
                "confirmation",
                "settings.",
                "",
                "📖 ArchWiki:",
                "   System_Maintenance",
            ],
            Self::Review => vec![
                "✅ Review Checklist",
                "",
                "Before applying:",
                "",
                " • Snapshot created",
                " • Changes logged",
                " • Rollback token",
                "   generated",
                "",
                "After applying:",
                "",
                " • Daemon reloads",
                " • Verify changes",
                " • Test rollback",
                "",
                "⚠️ Risk Level: LOW",
            ],
        }
    }

    fn next(&self) -> Self {
        match self {
            Self::Profile => Self::Priorities,
            Self::Priorities => Self::Desktop,
            Self::Desktop => Self::Safety,
            Self::Safety => Self::Scheduler,
            Self::Scheduler => Self::Modules,
            Self::Modules => Self::Review,
            Self::Review => Self::Review,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Self::Profile => Self::Profile,
            Self::Priorities => Self::Profile,
            Self::Desktop => Self::Priorities,
            Self::Safety => Self::Desktop,
            Self::Scheduler => Self::Safety,
            Self::Modules => Self::Scheduler,
            Self::Review => Self::Modules,
        }
    }
}

struct AppState {
    current_section: Section,
    master_config: MasterConfig,
    priorities_config: PrioritiesConfig,
    dirty: bool,
    selected_index: usize,
    quit: bool,
}

impl AppState {
    fn new() -> Result<Self> {
        let master_config = load_master_config()?;
        let priorities_config = load_priorities_config()?;

        Ok(Self {
            current_section: Section::Profile,
            master_config,
            priorities_config,
            dirty: false,
            selected_index: 0,
            quit: false,
        })
    }

    fn next_section(&mut self) {
        self.current_section = self.current_section.next();
        self.selected_index = 0;
    }

    fn prev_section(&mut self) {
        self.current_section = self.current_section.prev();
        self.selected_index = 0;
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn save_config(&mut self) -> Result<()> {
        // Use sync API for coordinated save with event emission
        let _result = config_api::save_with_sync(
            &self.master_config,
            &self.priorities_config,
            EventSource::Tui,
            Actor::User,
        )?;

        self.dirty = false;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main Entry Point
// ═══════════════════════════════════════════════════════════════════════════════

pub fn run_configurator() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = AppState::new()?;

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

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut AppState) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.quit {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_input(app, key.code, key.modifiers)?;
            }
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// UI Rendering
// ═══════════════════════════════════════════════════════════════════════════════

fn ui(f: &mut Frame, app: &AppState) {
    let size = f.size();

    // Main layout: header + body + footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Body
            Constraint::Length(2), // Footer
        ])
        .split(size);

    // Render header
    render_header(f, chunks[0], app);

    // Body layout: sidebar + center + help
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(16), // Sidebar
            Constraint::Min(40),    // Center content
            Constraint::Length(25), // Help panel
        ])
        .split(chunks[1]);

    // Render panels
    render_sidebar(f, body_chunks[0], app);
    render_content(f, body_chunks[1], app);
    render_help(f, body_chunks[2], app);

    // Render footer
    render_footer(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &AppState) {
    let profile_name = &app.master_config.profile;
    let dirty_marker = if app.dirty { " *" } else { "" };

    let title = format!(
        " 🌸 Anna Configurator v1.3{}                    [Profile: {}] ",
        dirty_marker, profile_name
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(PASTEL_BLUE));

    let paragraph = Paragraph::new(title)
        .block(block)
        .style(Style::default().fg(PASTEL_BLUE));

    f.render_widget(paragraph, area);
}

fn render_sidebar(f: &mut Frame, area: Rect, app: &AppState) {
    let sections: Vec<ListItem> = Section::all()
        .iter()
        .map(|section| {
            let style = if *section == app.current_section {
                Style::default()
                    .fg(PASTEL_GREEN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(MUTED_TEXT)
            };

            let prefix = if *section == app.current_section {
                "● "
            } else {
                "  "
            };

            ListItem::new(format!("{}{}", prefix, section.title())).style(style)
        })
        .collect();

    let list = List::new(sections)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(DIM_TEXT)),
        )
        .style(Style::default().fg(MUTED_TEXT));

    f.render_widget(list, area);
}

fn render_content(f: &mut Frame, area: Rect, app: &AppState) {
    match app.current_section {
        Section::Profile => render_profile_section(f, area, app),
        Section::Priorities => render_priorities_section(f, area, app),
        Section::Desktop => render_desktop_section(f, area, app),
        Section::Safety => render_safety_section(f, area, app),
        Section::Scheduler => render_scheduler_section(f, area, app),
        Section::Modules => render_modules_section(f, area, app),
        Section::Review => render_review_section(f, area, app),
    }
}

fn render_help(f: &mut Frame, area: Rect, app: &AppState) {
    let help_lines: Vec<Line> = app
        .current_section
        .help_text()
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(MUTED_TEXT))))
        .collect();

    let paragraph = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(DIM_TEXT)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &AppState) {
    let footer_text = match app.current_section {
        Section::Review => " ↑↓ scroll • Enter apply • Esc cancel • b back • q quit ",
        _ => " ↑↓ navigate • ←→ change • Tab next • Enter apply • q quit ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(DIM_TEXT));

    let paragraph = Paragraph::new(footer_text)
        .block(block)
        .style(Style::default().fg(MUTED_TEXT))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section Renderers
// ═══════════════════════════════════════════════════════════════════════════════

fn render_profile_section(f: &mut Frame, area: Rect, app: &AppState) {
    let profiles = bundled_profiles();
    let profile_names: Vec<&String> = profiles.keys().collect();

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "╭─ System Profile ─────────────────────╮",
            Style::default().fg(PASTEL_BLUE),
        )),
        Line::from("│                                      │"),
        Line::from("│ Choose your system's role:           │"),
        Line::from("│                                      │"),
    ];

    for (i, name) in profile_names.iter().enumerate() {
        if let Some(template) = profiles.get(*name) {
            let is_selected = app.master_config.profile == **name;
            let marker = if is_selected { "●" } else { "○" };
            let style = if is_selected {
                Style::default().fg(PASTEL_GREEN)
            } else {
                Style::default().fg(MUTED_TEXT)
            };

            let line_text = format!("│  {} {:<14} {}│", marker, name, template.description);
            lines.push(Line::from(Span::styled(line_text, style)));
        }
    }

    lines.push(Line::from("│                                      │"));
    lines.push(Line::from(Span::styled(
        "╰──────────────────────────────────────╯",
        Style::default().fg(PASTEL_BLUE),
    )));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "╭─ Autonomy Level ─────────────────────╮",
        Style::default().fg(PASTEL_BLUE),
    )));
    lines.push(Line::from("│                                      │"));

    let autonomy_options = [
        ("advice_only", "Ask before all"),
        ("auto_low_risk", "Apply safe fixes"),
    ];

    for (value, desc) in autonomy_options {
        let is_selected = app.master_config.autonomy.to_string() == value;
        let marker = if is_selected { "●" } else { "○" };
        let style = if is_selected {
            Style::default().fg(PASTEL_GREEN)
        } else {
            Style::default().fg(MUTED_TEXT)
        };

        let line_text = format!("│  {} {:<14} {}│", marker, value, desc);
        lines.push(Line::from(Span::styled(line_text, style)));
    }

    lines.push(Line::from("│                                      │"));
    lines.push(Line::from(Span::styled(
        "╰──────────────────────────────────────╯",
        Style::default().fg(PASTEL_BLUE),
    )));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(DIM_TEXT)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_priorities_section(f: &mut Frame, area: Rect, app: &AppState) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "╭─ System Priorities ──────────────────╮",
            Style::default().fg(PASTEL_BLUE),
        )),
        Line::from("│                                      │"),
    ];

    // Performance
    lines.push(Line::from(vec![
        Span::styled("│  Performance      ", Style::default().fg(MUTED_TEXT)),
        render_priority_slider(&app.priorities_config.performance),
        Span::raw("│"),
    ]));
    lines.push(Line::from(Span::styled(
        "│  conservative  balanced  maximum     │",
        Style::default().fg(DIM_TEXT),
    )));
    lines.push(Line::from("│                                      │"));

    // Responsiveness
    lines.push(Line::from(vec![
        Span::styled("│  Responsiveness   ", Style::default().fg(MUTED_TEXT)),
        render_priority_slider(&app.priorities_config.responsiveness),
        Span::raw("│"),
    ]));
    lines.push(Line::from(Span::styled(
        "│  relaxed       balanced  instant     │",
        Style::default().fg(DIM_TEXT),
    )));
    lines.push(Line::from("│                                      │"));

    // Battery Life
    lines.push(Line::from(vec![
        Span::styled("│  Battery Life     ", Style::default().fg(MUTED_TEXT)),
        render_priority_slider(&app.priorities_config.battery_life),
        Span::raw("│"),
    ]));
    lines.push(Line::from(Span::styled(
        "│  performance   balanced  maxsave     │",
        Style::default().fg(DIM_TEXT),
    )));
    lines.push(Line::from("│                                      │"));

    // Aesthetics
    lines.push(Line::from(vec![
        Span::styled("│  Aesthetics       ", Style::default().fg(MUTED_TEXT)),
        render_priority_slider(&app.priorities_config.aesthetics),
        Span::raw("│"),
    ]));
    lines.push(Line::from(Span::styled(
        "│  minimal       pleasant  beautiful   │",
        Style::default().fg(DIM_TEXT),
    )));
    lines.push(Line::from("│                                      │"));

    lines.push(Line::from(Span::styled(
        "╰──────────────────────────────────────╯",
        Style::default().fg(PASTEL_BLUE),
    )));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "ℹ️  Changes take effect immediately",
        Style::default().fg(PASTEL_YELLOW),
    )));
    lines.push(Line::from(Span::styled(
        "    Snapshot created before apply",
        Style::default().fg(DIM_TEXT),
    )));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(DIM_TEXT)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_desktop_section(f: &mut Frame, area: Rect, app: &AppState) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "╭─ Desktop Environment ────────────────╮",
            Style::default().fg(PASTEL_BLUE),
        )),
        Line::from("│                                      │"),
        Line::from("│  Detected: (auto-detect DE)          │"),
        Line::from("│                                      │"),
    ];

    let beautify_checkbox = if app.master_config.desktop.beautify_desktop {
        "[✓]"
    } else {
        "[ ]"
    };
    lines.push(Line::from(Span::styled(
        format!("│  {} Offer aesthetic improvements    │", beautify_checkbox),
        Style::default().fg(MUTED_TEXT),
    )));

    let auto_checkbox = if app.master_config.desktop.auto_apply_theme {
        "[✓]"
    } else {
        "[ ]"
    };
    lines.push(Line::from(Span::styled(
        format!("│  {} Apply theme automatically        │", auto_checkbox),
        Style::default().fg(MUTED_TEXT),
    )));

    let preview_checkbox = if app.master_config.desktop.preview_diffs {
        "[✓]"
    } else {
        "[ ]"
    };
    lines.push(Line::from(Span::styled(
        format!("│  {} Preview diffs before changes    │", preview_checkbox),
        Style::default().fg(MUTED_TEXT),
    )));

    lines.push(Line::from("│                                      │"));
    lines.push(Line::from(Span::styled(
        "╰──────────────────────────────────────╯",
        Style::default().fg(PASTEL_BLUE),
    )));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(DIM_TEXT)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_safety_section(f: &mut Frame, area: Rect, app: &AppState) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "╭─ Snapshot Retention ─────────────────╮",
            Style::default().fg(PASTEL_BLUE),
        )),
        Line::from("│                                      │"),
        Line::from(Span::styled(
            format!(
                "│  Keep snapshots for: [{}] days       │",
                app.master_config.safety.snapshot_retention_days
            ),
            Style::default().fg(MUTED_TEXT),
        )),
        Line::from(Span::styled(
            format!(
                "│  Maximum snapshots:  [{}] total      │",
                app.master_config.safety.max_snapshots
            ),
            Style::default().fg(MUTED_TEXT),
        )),
        Line::from("│                                      │"),
        Line::from(Span::styled(
            format!(
                "│  Auto-prune: [{}] Enabled            │",
                if app.master_config.safety.auto_prune {
                    "✓"
                } else {
                    " "
                }
            ),
            Style::default().fg(MUTED_TEXT),
        )),
        Line::from("│                                      │"),
        Line::from(Span::styled(
            "╰──────────────────────────────────────╯",
            Style::default().fg(PASTEL_BLUE),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "╭─ Confirmation Policy ────────────────╮",
            Style::default().fg(PASTEL_BLUE),
        )),
        Line::from("│                                      │"),
    ];

    let policies = [
        ("always_ask", "Always ask before changes"),
        ("auto_low_risk", "Auto-apply low-risk only"),
        ("manual", "Manual mode (never auto)"),
    ];

    for (value, desc) in policies {
        let is_selected = format!("{:?}", app.master_config.safety.confirmation)
            .to_lowercase()
            .contains(value);
        let marker = if is_selected { "●" } else { "○" };
        let style = if is_selected {
            Style::default().fg(PASTEL_GREEN)
        } else {
            Style::default().fg(MUTED_TEXT)
        };

        lines.push(Line::from(Span::styled(
            format!("│  {} {}│", marker, desc),
            style,
        )));
    }

    lines.push(Line::from("│                                      │"));
    lines.push(Line::from(Span::styled(
        "╰──────────────────────────────────────╯",
        Style::default().fg(PASTEL_BLUE),
    )));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(DIM_TEXT)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_scheduler_section(f: &mut Frame, area: Rect, app: &AppState) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "╭─ Fact Collection ────────────────────╮",
            Style::default().fg(PASTEL_BLUE),
        )),
        Line::from("│                                      │"),
        Line::from(Span::styled(
            format!(
                "│  Interval:  [{}] hours ± jitter      │",
                app.master_config.scheduler.fact_interval_hours
            ),
            Style::default().fg(MUTED_TEXT),
        )),
        Line::from(Span::styled(
            format!(
                "│  Jitter:    [±{}] minutes            │",
                app.master_config.scheduler.jitter_minutes
            ),
            Style::default().fg(MUTED_TEXT),
        )),
        Line::from("│                                      │"),
        Line::from(Span::styled(
            format!(
                "│  [{}] Enabled                         │",
                if app.master_config.scheduler.enabled {
                    "✓"
                } else {
                    " "
                }
            ),
            Style::default().fg(MUTED_TEXT),
        )),
        Line::from("│                                      │"),
    ];
    lines.push(Line::from(Span::styled(
        "╰──────────────────────────────────────╯",
        Style::default().fg(PASTEL_BLUE),
    )));

    if let Some(quiet) = &app.master_config.scheduler.quiet_hours {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "╭─ Quiet Hours ────────────────────────╮",
            Style::default().fg(PASTEL_BLUE),
        )));
        lines.push(Line::from("│                                      │"));
        lines.push(Line::from(Span::styled(
            format!(
                "│  [{}] Respect quiet hours             │",
                if quiet.enabled { "✓" } else { " " }
            ),
            Style::default().fg(MUTED_TEXT),
        )));
        lines.push(Line::from(Span::styled(
            format!("│  Start:  [{}]                      │", quiet.start),
            Style::default().fg(MUTED_TEXT),
        )));
        lines.push(Line::from(Span::styled(
            format!("│  End:    [{}]                      │", quiet.end),
            Style::default().fg(MUTED_TEXT),
        )));
        lines.push(Line::from("│                                      │"));
        lines.push(Line::from(Span::styled(
            "╰──────────────────────────────────────╯",
            Style::default().fg(PASTEL_BLUE),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(DIM_TEXT)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_modules_section(f: &mut Frame, area: Rect, app: &AppState) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "╭─ Advisor Modules ────────────────────╮",
            Style::default().fg(PASTEL_BLUE),
        )),
        Line::from("│                                      │"),
    ];

    let modules = [
        ("editor-ux", app.master_config.modules.editor_ux, "Editor configs, LSP, fonts"),
        ("desktop-env", app.master_config.modules.desktop_env, "Wayland, compositor, themes"),
        ("graphics", app.master_config.modules.graphics, "GPU drivers, acceleration"),
        ("network", app.master_config.modules.network, "WiFi, VPN, firewall"),
        ("package-hygiene", app.master_config.modules.package_hygiene, "Orphans, cache, mirrors"),
        ("security", app.master_config.modules.security, "Firewall, updates, hardening"),
        ("power", app.master_config.modules.power, "TLP, laptop mode, suspend"),
        ("performance", app.master_config.modules.performance, "CPU governor, I/O scheduler"),
        ("storage", app.master_config.modules.storage, "BTRFS, mount options, TRIM"),
        ("gaming", app.master_config.modules.gaming, "Wine, Proton, GPU tuning"),
    ];

    for (name, enabled, desc) in modules {
        let checkbox = if enabled { "[✓]" } else { "[ ]" };
        let style = if enabled {
            Style::default().fg(PASTEL_GREEN)
        } else {
            Style::default().fg(MUTED_TEXT)
        };

        lines.push(Line::from(Span::styled(
            format!("│  {} {:<18}           │", checkbox, name),
            style,
        )));
        lines.push(Line::from(Span::styled(
            format!("│      {}│", desc),
            Style::default().fg(DIM_TEXT),
        )));
        lines.push(Line::from("│                                      │"));
    }

    lines.push(Line::from(Span::styled(
        "╰──────────────────────────────────────╯",
        Style::default().fg(PASTEL_BLUE),
    )));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(DIM_TEXT)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_review_section(f: &mut Frame, area: Rect, app: &AppState) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "╭─ Pending Configuration Changes ─────╮",
            Style::default().fg(PASTEL_BLUE),
        )),
        Line::from("│                                      │"),
        Line::from(Span::styled(
            format!("│  Profile: {}                       │", app.master_config.profile),
            Style::default().fg(MUTED_TEXT),
        )),
        Line::from(Span::styled(
            format!("│  ├─ autonomy: {}              │", app.master_config.autonomy),
            Style::default().fg(DIM_TEXT),
        )),
        Line::from(Span::styled(
            format!("│  ├─ stability: {}             │", app.master_config.stability),
            Style::default().fg(DIM_TEXT),
        )),
        Line::from(Span::styled(
            format!("│  └─ privacy: {}                │", app.master_config.privacy),
            Style::default().fg(DIM_TEXT),
        )),
        Line::from("│                                      │"),
        Line::from(Span::styled(
            format!(
                "│  Modules: {} enabled, {} disabled     │",
                app.master_config.modules.enabled_count(),
                app.master_config.modules.disabled_count()
            ),
            Style::default().fg(MUTED_TEXT),
        )),
        Line::from("│                                      │"),
        Line::from(Span::styled(
            "╰──────────────────────────────────────╯",
            Style::default().fg(PASTEL_BLUE),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "📝 These changes will be written to:",
            Style::default().fg(PASTEL_YELLOW),
        )),
        Line::from(Span::styled(
            "   ~/.config/anna/anna.yaml",
            Style::default().fg(DIM_TEXT),
        )),
        Line::from(Span::styled(
            "   ~/.config/anna/priorities.yaml",
            Style::default().fg(DIM_TEXT),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "          [Enter to Apply Changes]",
            Style::default()
                .fg(PASTEL_GREEN)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(DIM_TEXT)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

fn render_priority_slider(priority: &Priority) -> Span {
    // Simple visualization: ●───○───○ for 3-level priorities
    let marker_pos = match priority {
        Priority::Conservative | Priority::Minimal | Priority::AdviceOnly
        | Priority::Strict | Priority::Relaxed | Priority::Performance => 0,
        Priority::Balanced | Priority::Pleasant | Priority::MinimalPrivacy => 1,
        Priority::Maximum | Priority::Beautiful | Priority::AutoLowRisk
        | Priority::Instant | Priority::Maxsave | Priority::Cutting => 2,
    };

    let visual = match marker_pos {
        0 => "●───○───○ ",
        1 => "○───●───○ ",
        2 => "○───○───● ",
        _ => "○───○───○ ",
    };

    Span::styled(visual, Style::default().fg(PASTEL_PINK))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Input Handling
// ═══════════════════════════════════════════════════════════════════════════════

fn handle_input(app: &mut AppState, key: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
    match key {
        KeyCode::Char('q') => {
            app.quit = true;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_section();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_section();
        }
        KeyCode::Tab => {
            app.next_section();
        }
        KeyCode::Enter => {
            if app.current_section == Section::Review {
                // Apply changes
                if let Err(e) = app.save_config() {
                    eprintln!("Failed to save config: {}", e);
                } else {
                    app.quit = true;
                }
            }
        }
        KeyCode::Char('1') => app.current_section = Section::Profile,
        KeyCode::Char('2') => app.current_section = Section::Priorities,
        KeyCode::Char('3') => app.current_section = Section::Desktop,
        KeyCode::Char('4') => app.current_section = Section::Safety,
        KeyCode::Char('5') => app.current_section = Section::Scheduler,
        KeyCode::Char('6') => app.current_section = Section::Modules,
        KeyCode::Char('7') => app.current_section = Section::Review,
        KeyCode::Char(' ') => {
            // Toggle boolean values in relevant sections
            match app.current_section {
                Section::Desktop => {
                    app.master_config.desktop.beautify_desktop =
                        !app.master_config.desktop.beautify_desktop;
                    app.mark_dirty();
                }
                _ => {}
            }
        }
        _ => {}
    }

    Ok(())
}
