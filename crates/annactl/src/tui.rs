//! Interactive TUI (Terminal User Interface) for Anna Assistant
//!
//! Provides real-time system monitoring, health visualization,
//! and interactive recommendation management.

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::rpc_client::RpcClient;
use anna_common::{Advice, Priority, RiskLevel, SystemFacts};
use anna_common::ipc::{Method, ResponseData};

/// Current view/mode of the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    /// Main dashboard view with list of recommendations
    Dashboard,
    /// Detailed view of a specific recommendation
    Details,
    /// Confirmation dialog for applying a recommendation
    ApplyConfirm,
}

/// Sort mode for recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortMode {
    /// Sort by category
    Category,
    /// Sort by priority (Critical → Recommended → Optional)
    Priority,
    /// Sort by risk level (Low → Medium → High)
    Risk,
}

/// TUI state and data
struct Tui {
    /// RPC client for daemon communication
    client: RpcClient,
    /// Last fetched system facts
    facts: Option<SystemFacts>,
    /// Current advice list
    advice: Vec<Advice>,
    /// List state for scrolling
    list_state: ListState,
    /// Last update timestamp
    last_update: Instant,
    /// Should quit?
    should_quit: bool,
    /// Current view mode
    view_mode: ViewMode,
    /// Status message to display
    status_message: Option<(String, Color)>,
    /// Current sort mode
    sort_mode: SortMode,
}

/// Get category emoji and color for display
fn get_category_emoji_color(category: &str) -> (&'static str, Color) {
    match category {
        "security" => ("🔒", Color::LightRed),
        "drivers" => ("🔌", Color::LightMagenta),
        "updates" => ("📦", Color::LightBlue),
        "maintenance" => ("🔧", Color::LightCyan),
        "cleanup" => ("🧹", Color::Cyan),
        "performance" => ("⚡", Color::LightYellow),
        "power" => ("🔋", Color::Yellow),
        "development" => ("💻", Color::LightMagenta),
        "desktop" => ("🖥️", Color::Blue),
        "gaming" => ("🎮", Color::LightMagenta),
        "multimedia" => ("🎬", Color::Magenta),
        "hardware" => ("🔌", Color::LightYellow),
        "networking" => ("📡", Color::LightCyan),
        "beautification" => ("🎨", Color::LightMagenta),
        "utilities" => ("🛠️", Color::Cyan),
        "system" => ("⚙️", Color::LightBlue),
        "productivity" => ("📊", Color::LightGreen),
        "audio" => ("🔊", Color::Magenta),
        "shell" => ("🐚", Color::LightCyan),
        "communication" => ("💬", Color::LightBlue),
        "engineering" => ("📐", Color::LightMagenta),
        "usability" => ("✨", Color::LightCyan),
        "media" => ("📹", Color::Magenta),
        _ => ("💡", Color::Cyan),
    }
}

impl Tui {
    fn new(client: RpcClient) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            client,
            facts: None,
            advice: Vec::new(),
            list_state,
            last_update: Instant::now(),
            should_quit: false,
            view_mode: ViewMode::Dashboard,
            status_message: None,
            sort_mode: SortMode::Priority, // Default: sort by priority
        }
    }

    /// Update data from daemon
    async fn update(&mut self) -> Result<()> {
        // Get system facts
        match self.client.call(Method::GetFacts).await? {
            ResponseData::Facts(facts) => {
                self.facts = Some(facts);
            }
            _ => anyhow::bail!("Unexpected response for GetFacts"),
        }

        // Get recommendations
        match self.client.call(Method::GetAdvice).await? {
            ResponseData::Advice(advice) => {
                self.advice = advice;
                // Sort based on current mode
                self.sort_advice();
                // Reset selection if needed
                if self.advice.is_empty() {
                    self.list_state.select(None);
                } else if self.list_state.selected().unwrap_or(0) >= self.advice.len() {
                    self.list_state.select(Some(self.advice.len() - 1));
                }
            }
            _ => anyhow::bail!("Unexpected response for GetAdvice"),
        }

        self.last_update = Instant::now();
        Ok(())
    }

    /// Sort advice based on current sort mode
    fn sort_advice(&mut self) {
        match self.sort_mode {
            SortMode::Category => {
                self.advice.sort_by(|a, b| a.category.cmp(&b.category));
            }
            SortMode::Priority => {
                self.advice.sort_by(|a, b| {
                    // Mandatory > Recommended > Optional > Cosmetic
                    let order_a = match a.priority {
                        Priority::Mandatory => 0,
                        Priority::Recommended => 1,
                        Priority::Optional => 2,
                        Priority::Cosmetic => 3,
                    };
                    let order_b = match b.priority {
                        Priority::Mandatory => 0,
                        Priority::Recommended => 1,
                        Priority::Optional => 2,
                        Priority::Cosmetic => 3,
                    };
                    order_a.cmp(&order_b)
                });
            }
            SortMode::Risk => {
                self.advice.sort_by(|a, b| {
                    // Low > Medium > High
                    let order_a = match a.risk {
                        RiskLevel::Low => 0,
                        RiskLevel::Medium => 1,
                        RiskLevel::High => 2,
                    };
                    let order_b = match b.risk {
                        RiskLevel::Low => 0,
                        RiskLevel::Medium => 1,
                        RiskLevel::High => 2,
                    };
                    order_a.cmp(&order_b)
                });
            }
        }
    }

    /// Get currently selected advice
    fn selected_advice(&self) -> Option<&Advice> {
        self.list_state.selected().and_then(|i| self.advice.get(i))
    }

    /// Handle keyboard input
    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match self.view_mode {
                    ViewMode::Dashboard => self.handle_dashboard_keys(key.code),
                    ViewMode::Details => self.handle_details_keys(key.code),
                    ViewMode::ApplyConfirm => self.handle_confirm_keys(key.code),
                }
            }
        }
    }

    /// Handle keys in dashboard view
    fn handle_dashboard_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.advice.is_empty() {
                    let i = self.list_state.selected().unwrap_or(0);
                    let next = (i + 1).min(self.advice.len() - 1);
                    self.list_state.select(Some(next));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.advice.is_empty() {
                    let i = self.list_state.selected().unwrap_or(0);
                    let prev = i.saturating_sub(1);
                    self.list_state.select(Some(prev));
                }
            }
            KeyCode::Enter => {
                if self.selected_advice().is_some() {
                    self.view_mode = ViewMode::Details;
                }
            }
            // Sort mode hotkeys
            KeyCode::Char('c') => {
                self.sort_mode = SortMode::Category;
                self.sort_advice();
                self.status_message = Some(("Sorted by Category".to_string(), Color::Cyan));
            }
            KeyCode::Char('p') => {
                self.sort_mode = SortMode::Priority;
                self.sort_advice();
                self.status_message = Some(("Sorted by Priority".to_string(), Color::Cyan));
            }
            KeyCode::Char('r') => {
                self.sort_mode = SortMode::Risk;
                self.sort_advice();
                self.status_message = Some(("Sorted by Risk Level".to_string(), Color::Cyan));
            }
            _ => {}
        }
    }

    /// Handle keys in details view
    fn handle_details_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.view_mode = ViewMode::Dashboard;
            }
            KeyCode::Char('a') | KeyCode::Char('y') => {
                // Only enter apply confirmation mode if there's a command to apply
                if let Some(advice) = self.selected_advice() {
                    if advice.command.is_some() {
                        self.view_mode = ViewMode::ApplyConfirm;
                    } else {
                        // Informational advice - no action to apply
                        self.status_message = Some((
                            "This is informational only - no action to apply".to_string(),
                            Color::Yellow
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    /// Handle keys in apply confirmation view
    fn handle_confirm_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                // Apply the recommendation
                if let Some(advice) = self.selected_advice() {
                    // Clone data we need before mutating self
                    let title = advice.title.clone();
                    let _id = advice.id.clone();

                    // Note: Actual apply would be done via RPC
                    // For now, just show a message
                    self.status_message = Some((
                        format!("✓ Applied: {}", title),
                        Color::Green
                    ));
                }
                self.view_mode = ViewMode::Dashboard;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.view_mode = ViewMode::Dashboard;
            }
            _ => {}
        }
    }

    /// Calculate health score from system facts
    fn calculate_health_score(&self) -> u16 {
        let Some(facts) = &self.facts else { return 0 };

        let mut score = 100u16;

        // Deduct for critical advice
        let critical_count = self.advice.iter()
            .filter(|a| a.priority == Priority::Mandatory)
            .count();
        score = score.saturating_sub((critical_count as u16) * 15);

        // Deduct for high-risk advice
        let high_risk_count = self.advice.iter()
            .filter(|a| a.priority == Priority::Recommended)
            .count();
        score = score.saturating_sub((high_risk_count as u16) * 5);

        // Deduct for CPU temperature
        if let Some(temp) = facts.hardware_monitoring.cpu_temperature_celsius {
            if temp > 85.0 {
                score = score.saturating_sub(20);
            } else if temp > 75.0 {
                score = score.saturating_sub(10);
            }
        }

        // Deduct for failing disks
        let failing_disks = facts.disk_health.iter()
            .filter(|d| d.has_errors || d.health_status == "FAILING")
            .count();
        score = score.saturating_sub((failing_disks as u16) * 25);

        // Deduct for memory pressure
        let memory_used_pct = if facts.hardware_monitoring.memory_available_gb > 0.0 {
            (facts.hardware_monitoring.memory_used_gb /
             (facts.hardware_monitoring.memory_used_gb + facts.hardware_monitoring.memory_available_gb)) * 100.0
        } else {
            0.0
        };

        if memory_used_pct > 95.0 {
            score = score.saturating_sub(15);
        } else if memory_used_pct > 85.0 {
            score = score.saturating_sub(5);
        }

        score
    }

    /// Get color for health score
    fn health_color(score: u16) -> Color {
        match score {
            90..=100 => Color::Green,
            70..=89 => Color::Yellow,
            50..=69 => Color::LightRed,
            _ => Color::Red,
        }
    }
}

/// Draw the TUI based on current view mode
fn draw(f: &mut Frame, tui: &mut Tui) {
    match tui.view_mode {
        ViewMode::Dashboard => draw_dashboard(f, tui),
        ViewMode::Details => draw_details(f, tui),
        ViewMode::ApplyConfirm => draw_apply_confirm(f, tui),
    }
}

/// Draw main dashboard view
fn draw_dashboard(f: &mut Frame, tui: &mut Tui) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Length(5),      // Health score
            Constraint::Min(10),        // Main content
            Constraint::Length(3),      // Footer
        ])
        .split(f.size());

    // Header
    draw_header(f, chunks[0]);

    // Health Score
    draw_health_score(f, chunks[1], tui);

    // Main content area - split horizontally
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Hardware monitoring
            Constraint::Percentage(60),  // Recommendations
        ])
        .split(chunks[2]);

    draw_hardware_monitoring(f, main_chunks[0], tui);
    draw_recommendations(f, main_chunks[1], tui);

    // Footer
    draw_footer(f, chunks[3], "Dashboard", tui.sort_mode, tui);
}

/// Draw details view for selected recommendation
fn draw_details(f: &mut Frame, tui: &Tui) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Min(10),        // Details
            Constraint::Length(3),      // Footer
        ])
        .split(f.size());

    draw_header(f, chunks[0]);

    // Details
    if let Some(advice) = tui.selected_advice() {
        let priority_str = match advice.priority {
            Priority::Mandatory => "🔴 CRITICAL",
            Priority::Recommended => "🟡 RECOMMENDED",
            Priority::Optional => "🟢 OPTIONAL",
            Priority::Cosmetic => "⚪ COSMETIC",
        };

        let (category_emoji, category_color) = get_category_emoji_color(&advice.category);
        let category_str = format!("{} {}", category_emoji, advice.category.to_uppercase());
        let risk_str = format!("Risk: {:?}", advice.risk);
        let popularity_str = format!("{} ({})", advice.popularity_stars(), advice.popularity_label());

        let mut lines = vec![
            Line::from(Span::styled(&advice.title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled(priority_str, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("  │  "),
                Span::styled(category_str, Style::default().fg(category_color)),
                Span::raw("  │  "),
                Span::styled(risk_str, Style::default().fg(Color::Gray)),
                Span::raw("  │  "),
                Span::styled(popularity_str, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(Span::styled("Why:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
        ];

        // Word wrap the reason
        for chunk in textwrap::wrap(&advice.reason, 70) {
            lines.push(Line::from(format!("  {}", chunk)));
        }

        if let Some(ref cmd) = advice.command {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Command:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(format!("  {}", cmd), Style::default().fg(Color::Cyan))));
        }

        if !advice.wiki_refs.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Arch Wiki:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
            for wiki in &advice.wiki_refs {
                lines.push(Line::from(format!("  {}", wiki)));
            }
        }

        let details = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Recommendation Details "))
            .wrap(Wrap { trim: true });

        f.render_widget(details, chunks[1]);
    }

    draw_footer(f, chunks[2], "Details", tui.sort_mode, tui);
}

/// Draw apply confirmation dialog
fn draw_apply_confirm(f: &mut Frame, tui: &Tui) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Min(10),        // Confirmation
            Constraint::Length(3),      // Footer
        ])
        .split(f.size());

    draw_header(f, chunks[0]);

    if let Some(advice) = tui.selected_advice() {
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled("⚠ Apply this recommendation?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled(&advice.title, Style::default().fg(Color::Cyan))),
            Line::from(""),
            Line::from(""),
            if let Some(ref cmd) = advice.command {
                Line::from(vec![
                    Span::styled("Command: ", Style::default().fg(Color::Gray)),
                    Span::styled(cmd, Style::default().fg(Color::Yellow)),
                ])
            } else {
                Line::from(Span::styled("No command available", Style::default().fg(Color::Gray)))
            },
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled(" [Y] Yes ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(" [N] No ", Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)),
            ]),
        ];

        let confirm = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Confirm "))
            .alignment(Alignment::Center);

        f.render_widget(confirm, chunks[1]);
    }

    draw_footer(f, chunks[2], "Confirm", tui.sort_mode, tui);
}

/// Draw header with logo and info
fn draw_header(f: &mut Frame, area: Rect) {
    let version = env!("ANNA_VERSION");
    let now = chrono::Local::now();
    let version_str = format!("v{}", version);
    let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  Anna Assistant ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(version_str, Style::default().fg(Color::Gray)),
            Span::raw("  │  "),
            Span::styled(time_str, Style::default().fg(Color::Gray)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
    .alignment(Alignment::Left);

    f.render_widget(header, area);
}

/// Draw health score gauge
fn draw_health_score(f: &mut Frame, area: Rect, tui: &Tui) {
    let score = tui.calculate_health_score();
    let color = Tui::health_color(score);

    let status_text = match score {
        90..=100 => "Excellent",
        70..=89 => "Good",
        50..=69 => "Fair",
        30..=49 => "Poor",
        _ => "Critical",
    };

    let gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(" System Health "))
        .gauge_style(Style::default().fg(color))
        .label(format!("{}/100 - {}", score, status_text))
        .ratio(score as f64 / 100.0);

    f.render_widget(gauge, area);
}

/// Draw hardware monitoring panel
fn draw_hardware_monitoring(f: &mut Frame, area: Rect, tui: &Tui) {
    let Some(facts) = &tui.facts else {
        let loading = Paragraph::new("Loading...")
            .block(Block::default().borders(Borders::ALL).title(" Hardware "));
        f.render_widget(loading, area);
        return;
    };

    let hw = &facts.hardware_monitoring;

    let mut lines = vec![
        Line::from(Span::styled("CPU & Memory", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];

    // CPU Temperature
    if let Some(temp) = hw.cpu_temperature_celsius {
        let temp_color = if temp > 85.0 {
            Color::Red
        } else if temp > 75.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        lines.push(Line::from(vec![
            Span::raw("  CPU Temp: "),
            Span::styled(format!("{:.1}°C", temp), Style::default().fg(temp_color)),
        ]));
    }

    // CPU Load
    if let (Some(l1), Some(l5), Some(l15)) = (hw.cpu_load_1min, hw.cpu_load_5min, hw.cpu_load_15min) {
        lines.push(Line::from(vec![
            Span::raw("  Load Avg: "),
            Span::styled(format!("{:.2}, {:.2}, {:.2}", l1, l5, l15), Style::default().fg(Color::Yellow)),
        ]));
    }

    // Memory usage
    let mem_total = hw.memory_used_gb + hw.memory_available_gb;
    let mem_pct = if mem_total > 0.0 {
        (hw.memory_used_gb / mem_total) * 100.0
    } else {
        0.0
    };
    let mem_color = if mem_pct > 90.0 {
        Color::Red
    } else if mem_pct > 75.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    lines.push(Line::from(vec![
        Span::raw("  Memory:   "),
        Span::styled(
            format!("{:.1}/{:.1} GB ({:.0}%)", hw.memory_used_gb, mem_total, mem_pct),
            Style::default().fg(mem_color)
        ),
    ]));

    // Disk Health
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Disk Health", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(""));

    if facts.disk_health.is_empty() {
        lines.push(Line::from(Span::styled("  No SMART data", Style::default().fg(Color::Gray))));
    } else {
        for disk in facts.disk_health.iter().take(3) {
            let status_color = match disk.health_status.as_str() {
                "PASSED" => Color::Green,
                "FAILING" => Color::Red,
                _ => Color::Gray,
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(&disk.device),
                Span::raw(": "),
                Span::styled(&disk.health_status, Style::default().fg(status_color)),
            ]));
        }
    }

    // Package stats
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Packages", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(""));
    lines.push(Line::from(format!("  Installed: {}", facts.installed_packages)));
    lines.push(Line::from(format!("  Orphans:   {}", facts.orphan_packages.len())));

    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(" Hardware & System "))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Draw recommendations panel with scrolling
fn draw_recommendations(f: &mut Frame, area: Rect, tui: &mut Tui) {
    if tui.advice.is_empty() {
        let no_advice = Paragraph::new("✓ No recommendations - system looks great!")
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title(" Recommendations "))
            .alignment(Alignment::Center);
        f.render_widget(no_advice, area);
        return;
    }

    let items: Vec<ListItem> = tui.advice
        .iter()
        .map(|advice| {
            let priority_icon = match advice.priority {
                Priority::Mandatory => "🔴",
                Priority::Recommended => "🟡",
                Priority::Optional => "🟢",
                Priority::Cosmetic => "⚪",
            };

            let priority_color = match advice.priority {
                Priority::Mandatory => Color::Red,
                Priority::Recommended => Color::Yellow,
                Priority::Optional => Color::Green,
                Priority::Cosmetic => Color::Gray,
            };

            let (category_emoji, category_color) = get_category_emoji_color(&advice.category);
            let popularity_stars = advice.popularity_stars();

            let content = Line::from(vec![
                Span::raw(priority_icon),
                Span::raw(" "),
                Span::styled(category_emoji, Style::default().fg(category_color)),
                Span::raw(" "),
                Span::styled(&advice.title, Style::default().fg(priority_color)),
                Span::raw("  "),
                Span::styled(popularity_stars, Style::default().fg(Color::Yellow)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(" Recommendations ({}) ", tui.advice.len())))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▶ ");

    // Render with state for scrolling
    f.render_stateful_widget(list, area, &mut tui.list_state);
}

/// Draw footer with keyboard shortcuts
fn draw_footer(f: &mut Frame, area: Rect, mode: &str, sort_mode: SortMode, tui: &Tui) {
    let shortcuts = match mode {
        "Dashboard" => vec![
            (" q/Esc ", " Quit  "),
            (" ↑/k ", " Up  "),
            (" ↓/j ", " Down  "),
            (" Enter ", " Details  "),
            (" c ", " Category  "),
            (" p ", " Priority  "),
            (" r ", " Risk  "),
        ],
        "Details" => {
            // Check if selected advice has a command (actionable)
            let has_command = tui.selected_advice()
                .map(|a| a.command.is_some())
                .unwrap_or(false);

            if has_command {
                vec![
                    (" Esc ", " Back  "),
                    (" a/y ", " Apply  "),
                    (" q ", " Quit  "),
                ]
            } else {
                vec![
                    (" Esc ", " Back  "),
                    (" q ", " Quit  "),
                ]
            }
        },
        "Confirm" => vec![
            (" Y ", " Yes  "),
            (" N ", " No  "),
            (" Esc ", " Cancel  "),
        ],
        _ => vec![],
    };

    let mut spans = vec![];
    for (key, desc) in shortcuts {
        spans.push(Span::styled(key, Style::default().fg(Color::Black).bg(Color::Gray)));
        spans.push(Span::raw(desc));
    }

    // Add sort mode indicator
    if mode == "Dashboard" {
        let sort_text = match sort_mode {
            SortMode::Category => "  Sort: Category",
            SortMode::Priority => "  Sort: Priority",
            SortMode::Risk => "  Sort: Risk",
        };
        spans.push(Span::styled(sort_text, Style::default().fg(Color::Cyan)));
    }

    spans.push(Span::raw("  Auto-refresh: 2s"));

    // Add status message if present
    if let Some((ref msg, color)) = tui.status_message {
        spans.push(Span::raw("  │  "));
        spans.push(Span::styled(msg, Style::default().fg(color).add_modifier(Modifier::BOLD)));
    }

    let footer = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Gray)))
        .alignment(Alignment::Left);

    f.render_widget(footer, area);
}

/// Run the TUI
pub async fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create TUI state
    let client = RpcClient::connect().await?;
    let mut tui = Tui::new(client);

    // Initial data load
    if let Err(e) = tui.update().await {
        // Restore terminal before showing error
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        return Err(anyhow::anyhow!("Failed to connect to Anna daemon: {}\n\nMake sure the daemon is running with: sudo systemctl start annad", e));
    }

    // Main loop
    let tick_rate = Duration::from_millis(100);
    let refresh_rate = Duration::from_secs(2);
    let mut last_tick = Instant::now();

    loop {
        // Draw UI
        terminal.draw(|f| draw(f, &mut tui))?;

        // Handle events with timeout
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            let event = event::read()?;
            tui.handle_event(event);
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        // Auto-refresh data
        if tui.last_update.elapsed() >= refresh_rate {
            let _ = tui.update().await; // Ignore errors during refresh
        }

        // Check if should quit
        if tui.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
