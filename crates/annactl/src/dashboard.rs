//! Interactive TUI Dashboard for Anna Assistant
//!
//! Provides real-time system monitoring, health visualization,
//! and interactive controls.

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
        Block, Borders, Gauge, List, ListItem, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::rpc_client::RpcClient;
use anna_common::{Advice, Priority, SystemFacts};
use anna_common::ipc::{Method, ResponseData};

/// Dashboard state and data
struct Dashboard {
    /// RPC client for daemon communication
    client: RpcClient,
    /// Last fetched system facts
    facts: Option<SystemFacts>,
    /// Current advice list
    advice: Vec<Advice>,
    /// Selected recommendation index
    selected_advice: usize,
    /// Last update timestamp
    last_update: Instant,
    /// Should quit?
    should_quit: bool,
}

impl Dashboard {
    fn new(client: RpcClient) -> Self {
        Self {
            client,
            facts: None,
            advice: Vec::new(),
            selected_advice: 0,
            last_update: Instant::now(),
            should_quit: false,
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
            }
            _ => anyhow::bail!("Unexpected response for GetAdvice"),
        }

        self.last_update = Instant::now();
        Ok(())
    }

    /// Handle keyboard input
    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !self.advice.is_empty() {
                            self.selected_advice = (self.selected_advice + 1).min(self.advice.len() - 1);
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if self.selected_advice > 0 {
                            self.selected_advice -= 1;
                        }
                    }
                    _ => {}
                }
            }
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

/// Draw the dashboard UI
fn draw(f: &mut Frame, dashboard: &Dashboard) {
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
    draw_health_score(f, chunks[1], dashboard);

    // Main content area - split horizontally
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Hardware monitoring
            Constraint::Percentage(60),  // Recommendations
        ])
        .split(chunks[2]);

    draw_hardware_monitoring(f, main_chunks[0], dashboard);
    draw_recommendations(f, main_chunks[1], dashboard);

    // Footer
    draw_footer(f, chunks[3]);
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
fn draw_health_score(f: &mut Frame, area: Rect, dashboard: &Dashboard) {
    let score = dashboard.calculate_health_score();
    let color = Dashboard::health_color(score);

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
fn draw_hardware_monitoring(f: &mut Frame, area: Rect, dashboard: &Dashboard) {
    let Some(facts) = &dashboard.facts else {
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

/// Draw recommendations panel
fn draw_recommendations(f: &mut Frame, area: Rect, dashboard: &Dashboard) {
    if dashboard.advice.is_empty() {
        let no_advice = Paragraph::new("✓ No recommendations - system looks great!")
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title(" Recommendations "))
            .alignment(Alignment::Center);
        f.render_widget(no_advice, area);
        return;
    }

    let items: Vec<ListItem> = dashboard.advice
        .iter()
        .enumerate()
        .map(|(i, advice)| {
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

            let style = if i == dashboard.selected_advice {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::raw(priority_icon),
                Span::raw(" "),
                Span::styled(&advice.title, Style::default().fg(priority_color)),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(" Recommendations ({}) ", dashboard.advice.len())))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

/// Draw footer with keyboard shortcuts
fn draw_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" q/Esc ", Style::default().fg(Color::Black).bg(Color::Gray)),
        Span::raw(" Quit  "),
        Span::styled(" ↑/k ", Style::default().fg(Color::Black).bg(Color::Gray)),
        Span::raw(" Up  "),
        Span::styled(" ↓/j ", Style::default().fg(Color::Black).bg(Color::Gray)),
        Span::raw(" Down  "),
        Span::raw("  Auto-refresh: 2s"),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Gray)))
    .alignment(Alignment::Left);

    f.render_widget(footer, area);
}

/// Run the dashboard TUI
pub async fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create dashboard state
    let client = RpcClient::connect().await?;
    let mut dashboard = Dashboard::new(client);

    // Initial data load
    if let Err(e) = dashboard.update().await {
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
        terminal.draw(|f| draw(f, &dashboard))?;

        // Handle events with timeout
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            let event = event::read()?;
            dashboard.handle_event(event);
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        // Auto-refresh data
        if dashboard.last_update.elapsed() >= refresh_rate {
            let _ = dashboard.update().await; // Ignore errors during refresh
        }

        // Check if should quit
        if dashboard.should_quit {
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
