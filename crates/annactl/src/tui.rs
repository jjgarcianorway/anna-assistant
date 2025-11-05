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
    /// Output display showing command execution in real-time
    OutputDisplay,
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
    /// Pending action to apply (advice_id)
    pending_apply: Option<String>,
    /// Mapping from list index to advice index (when headers are present)
    list_to_advice_map: Vec<Option<usize>>,
    /// Scroll offset for details view
    details_scroll: u16,
    /// Command output buffer for OutputDisplay view
    output_buffer: Vec<String>,
    /// Scroll offset for output view
    output_scroll: u16,
    /// Is command still executing?
    output_executing: bool,
    /// Title for output window
    output_title: String,
}

/// Get category emoji and color for display (using centralized categories)
fn get_category_emoji_color(category: &str) -> (&'static str, Color) {
    let emoji = anna_common::get_category_emoji(category);
    let color = match category {
        "Security & Privacy" => Color::LightRed,
        "Hardware Support" => Color::LightYellow,
        "Package Management" => Color::LightBlue,
        "System Maintenance" => Color::LightCyan,
        "Performance & Optimization" => Color::LightYellow,
        "Power Management" => Color::Yellow,
        "Development Tools" => Color::LightMagenta,
        "Desktop Environment" => Color::Blue,
        "Gaming & Entertainment" => Color::Magenta,
        "Multimedia & Graphics" => Color::Magenta,
        "Network Configuration" => Color::LightCyan,
        "System Utilities" | "Desktop Utilities" | "Utilities" => Color::Cyan,
        "Shell & Terminal" | "Terminal & CLI Tools" => Color::LightCyan,
        "Communication" => Color::LightBlue,
        "Desktop Customization" => Color::LightMagenta,
        "Productivity" => Color::LightGreen,
        "Engineering & CAD" => Color::LightBlue,
        "System Configuration" => Color::Cyan,
        _ => Color::Cyan,
    };
    (emoji, color)
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
            pending_apply: None,
            list_to_advice_map: Vec::new(),
            details_scroll: 0,
            output_buffer: Vec::new(),
            output_scroll: 0,
            output_executing: false,
            output_title: String::new(),
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
            ResponseData::Advice(mut advice) => {
                // Apply ignore filters
                if let Ok(filters) = anna_common::IgnoreFilters::load() {
                    advice.retain(|a| !filters.should_filter(a));
                }

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
        self.list_state.selected().and_then(|list_idx| {
            // If we have a mapping, use it; otherwise, use direct indexing
            if !self.list_to_advice_map.is_empty() {
                self.list_to_advice_map.get(list_idx)
                    .and_then(|opt| opt.as_ref())
                    .and_then(|&advice_idx| self.advice.get(advice_idx))
            } else {
                self.advice.get(list_idx)
            }
        })
    }

    /// Execute pending apply action if any
    async fn execute_pending_apply(&mut self) -> Result<()> {
        if let Some(advice_id) = self.pending_apply.take() {
            // Call RPC to apply the action
            use anna_common::ipc::{Method, ResponseData};

            match self.client.call(Method::ApplyAction {
                advice_id: advice_id.clone(),
                dry_run: false
            }).await? {
                ResponseData::ActionResult { success, message } => {
                    // Add output lines to buffer
                    self.output_buffer.push(String::new());
                    for line in message.lines() {
                        self.output_buffer.push(line.to_string());
                    }
                    self.output_buffer.push(String::new());

                    if success {
                        self.output_buffer.push("✓ Action completed successfully!".to_string());
                        self.status_message = Some((
                            "✓ Action completed successfully".to_string(),
                            Color::Green
                        ));
                        // Mark as finished
                        self.output_executing = false;
                        // Don't refresh immediately - let auto-refresh handle it after a few seconds
                        // This gives the user time to see the success message
                    } else {
                        self.output_buffer.push("✗ Action failed!".to_string());
                        self.status_message = Some((
                            "✗ Action failed".to_string(),
                            Color::Red
                        ));
                        // Mark as finished (even on failure)
                        self.output_executing = false;
                        // Still don't refresh on failure - the advice remains to retry if needed
                    }
                }
                _ => {
                    self.output_buffer.push("✗ Unexpected response from daemon".to_string());
                    self.status_message = Some((
                        "Unexpected response from daemon - please check logs".to_string(),
                        Color::Red
                    ));
                    self.output_executing = false;
                }
            }
        }
        Ok(())
    }

    /// Handle keyboard input
    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match self.view_mode {
                    ViewMode::Dashboard => self.handle_dashboard_keys(key.code),
                    ViewMode::Details => self.handle_details_keys(key.code),
                    ViewMode::ApplyConfirm => self.handle_confirm_keys(key.code),
                    ViewMode::OutputDisplay => self.handle_output_keys(key.code),
                }
            }
        }
    }

    /// Handle keys in dashboard view
    fn handle_dashboard_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection_down();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection_up();
            }
            KeyCode::Enter => {
                if self.selected_advice().is_some() {
                    self.details_scroll = 0; // Reset scroll when entering details
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
            KeyCode::Char('d') => {
                // Dismiss/ignore current advice
                if let Some(advice) = self.selected_advice() {
                    // Add to ignore filters by category
                    if let Ok(mut filters) = anna_common::IgnoreFilters::load() {
                        filters.ignore_category(&advice.category);
                        if let Err(e) = filters.save() {
                            self.status_message = Some((
                                format!("Failed to save ignore filter: {}", e),
                                Color::Red
                            ));
                        } else {
                            self.status_message = Some((
                                format!("Ignored category: {}", advice.category),
                                Color::Yellow
                            ));
                            // Mark for refresh on next cycle
                            self.last_update = std::time::Instant::now() - std::time::Duration::from_secs(5);
                        }
                    }
                }
            }
            KeyCode::Char('i') => {
                // Ignore by priority
                if let Some(advice) = self.selected_advice() {
                    if let Ok(mut filters) = anna_common::IgnoreFilters::load() {
                        filters.ignore_priority(advice.priority);
                        if let Err(e) = filters.save() {
                            self.status_message = Some((
                                format!("Failed to save ignore filter: {}", e),
                                Color::Red
                            ));
                        } else {
                            self.status_message = Some((
                                format!("Ignored priority: {:?}", advice.priority),
                                Color::Yellow
                            ));
                            // Mark for refresh on next cycle
                            self.last_update = std::time::Instant::now() - std::time::Duration::from_secs(5);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Move selection down, skipping headers
    fn move_selection_down(&mut self) {
        if self.list_to_advice_map.is_empty() {
            return;
        }

        let current = self.list_state.selected().unwrap_or(0);
        let max = self.list_to_advice_map.len() - 1;

        // Find next selectable item (not a header)
        for next in (current + 1)..=max {
            if self.list_to_advice_map[next].is_some() {
                self.list_state.select(Some(next));
                return;
            }
        }
        // If no next item found, stay at current
    }

    /// Move selection up, skipping headers
    fn move_selection_up(&mut self) {
        if self.list_to_advice_map.is_empty() {
            return;
        }

        let current = self.list_state.selected().unwrap_or(0);

        // Find previous selectable item (not a header)
        for prev in (0..current).rev() {
            if self.list_to_advice_map[prev].is_some() {
                self.list_state.select(Some(prev));
                return;
            }
        }
        // If no previous item found, stay at current
    }

    /// Handle keys in details view
    fn handle_details_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.view_mode = ViewMode::Dashboard;
                self.details_scroll = 0; // Reset scroll when leaving details
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Scroll down in details
                self.details_scroll = self.details_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Scroll up in details
                self.details_scroll = self.details_scroll.saturating_sub(1);
            }
            KeyCode::Char('a') => {
                // Only enter apply confirmation mode if there's a command to apply
                if let Some(advice) = self.selected_advice() {
                    if advice.command.is_some() {
                        self.view_mode = ViewMode::ApplyConfirm;
                    } else {
                        // Informational advice - no action to apply
                        self.status_message = Some((
                            "This recommendation is informational only - no action to apply".to_string(),
                            Color::Yellow
                        ));
                    }
                }
            }
            KeyCode::Char('d') => {
                // Dismiss/ignore current advice by category
                if let Some(advice) = self.selected_advice() {
                    // Add to ignore filters by category
                    if let Ok(mut filters) = anna_common::IgnoreFilters::load() {
                        filters.ignore_category(&advice.category);
                        if let Err(e) = filters.save() {
                            self.status_message = Some((
                                format!("Failed to save ignore filter: {}", e),
                                Color::Red
                            ));
                        } else {
                            self.status_message = Some((
                                format!("Ignored category: {}", advice.category),
                                Color::Yellow
                            ));
                            // Mark for refresh on next cycle
                            self.last_update = std::time::Instant::now() - std::time::Duration::from_secs(5);
                            // Return to dashboard
                            self.view_mode = ViewMode::Dashboard;
                        }
                    }
                }
            }
            KeyCode::Char('i') => {
                // Ignore by priority
                if let Some(advice) = self.selected_advice() {
                    if let Ok(mut filters) = anna_common::IgnoreFilters::load() {
                        filters.ignore_priority(advice.priority);
                        if let Err(e) = filters.save() {
                            self.status_message = Some((
                                format!("Failed to save ignore filter: {}", e),
                                Color::Red
                            ));
                        } else {
                            self.status_message = Some((
                                format!("Ignored priority: {:?}", advice.priority),
                                Color::Yellow
                            ));
                            // Mark for refresh on next cycle
                            self.last_update = std::time::Instant::now() - std::time::Duration::from_secs(5);
                            // Return to dashboard
                            self.view_mode = ViewMode::Dashboard;
                        }
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
                // Clone data first to avoid borrow issues
                if let Some(advice) = self.selected_advice().cloned() {
                    // Set pending action to be executed in async context
                    self.pending_apply = Some(advice.id.clone());
                    self.output_title = format!("Applying: {}", advice.title);
                    self.output_buffer.clear();
                    self.output_buffer.push(format!("→ Executing: {}", advice.command.as_ref().unwrap_or(&"N/A".to_string())));
                    self.output_buffer.push(String::new());
                    self.output_executing = true;
                    self.output_scroll = 0;
                    // Switch to output display view to show progress
                    self.view_mode = ViewMode::OutputDisplay;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.view_mode = ViewMode::Dashboard;
            }
            _ => {}
        }
    }

    /// Handle keys in output display view
    fn handle_output_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Only allow closing if command finished executing
                if !self.output_executing {
                    self.view_mode = ViewMode::Dashboard;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Scroll down in output
                if self.output_scroll < self.output_buffer.len().saturating_sub(1) as u16 {
                    self.output_scroll += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Scroll up in output
                self.output_scroll = self.output_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                // Scroll down by page
                self.output_scroll = self.output_scroll.saturating_add(10);
                if self.output_scroll >= self.output_buffer.len() as u16 {
                    self.output_scroll = self.output_buffer.len().saturating_sub(1) as u16;
                }
            }
            KeyCode::PageUp => {
                // Scroll up by page
                self.output_scroll = self.output_scroll.saturating_sub(10);
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

        // Deduct for recommended advice
        let recommended_count = self.advice.iter()
            .filter(|a| a.priority == Priority::Recommended)
            .count();
        score = score.saturating_sub((recommended_count as u16) * 5);

        // Deduct for optional advice (small deduction)
        let optional_count = self.advice.iter()
            .filter(|a| a.priority == Priority::Optional)
            .count();
        score = score.saturating_sub((optional_count as u16) * 2);

        // Deduct for cosmetic advice (tiny deduction)
        let cosmetic_count = self.advice.iter()
            .filter(|a| a.priority == Priority::Cosmetic)
            .count();
        score = score.saturating_sub((cosmetic_count as u16) * 1);

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
        ViewMode::OutputDisplay => draw_output_display(f, tui),
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
fn draw_details(f: &mut Frame, tui: &mut Tui) {
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

        let is_informational = advice.command.is_none();

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
        ];

        // Add special banner for informational items
        if is_informational {
            lines.push(Line::from(vec![
                Span::styled("ℹ  INFORMATIONAL NOTICE  ", Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)),
                Span::styled(" - No action required, this is for your awareness", Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(Span::styled(
            if is_informational { "Details:" } else { "Why this matters:" },
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        )));
        lines.push(Line::from(""));

        // Word wrap the reason with better formatting for informational items
        let wrap_width = if chunks[1].width > 10 { chunks[1].width - 8 } else { 60 };
        for chunk in textwrap::wrap(&advice.reason, wrap_width as usize) {
            lines.push(Line::from(format!("  {}", chunk)));
        }

        // Action to take
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            if is_informational { "What you should know:" } else { "Recommended action:" },
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        )));
        lines.push(Line::from(""));
        for chunk in textwrap::wrap(&advice.action, wrap_width as usize) {
            lines.push(Line::from(format!("  {}", chunk)));
        }

        // Command if present
        if let Some(ref cmd) = advice.command {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Command to execute:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(format!("  $ {}", cmd), Style::default().fg(Color::Cyan))));
        }

        // Alternatives if present
        if !advice.alternatives.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Alternative approaches:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(""));
            for alt in &advice.alternatives {
                lines.push(Line::from(format!("  • {}: {}", alt.name, alt.description)));
            }
        }

        // Wiki references
        if !advice.wiki_refs.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Documentation (Arch Wiki):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(""));
            for wiki in &advice.wiki_refs {
                lines.push(Line::from(Span::styled(format!("  🔗 {}", wiki), Style::default().fg(Color::Blue))));
            }
        }

        // Add scrolling hint if content is long
        if lines.len() > 20 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  ↑/↓ or j/k to scroll",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
            )));
        }

        let details = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(if is_informational {
                    " Information Notice "
                } else {
                    " Recommendation Details "
                }))
            .scroll((tui.details_scroll, 0))
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

/// Draw command output display view
fn draw_output_display(f: &mut Frame, tui: &Tui) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Min(10),        // Output
            Constraint::Length(3),      // Footer
        ])
        .split(f.size());

    draw_header(f, chunks[0]);

    // Build output lines with scrolling
    let output_lines: Vec<Line> = tui.output_buffer
        .iter()
        .skip(tui.output_scroll as usize)
        .map(|line| Line::from(line.as_str()))
        .collect();

    // Add status indicator at the end
    let mut final_lines = output_lines;
    if tui.output_executing {
        final_lines.push(Line::from(""));
        final_lines.push(Line::from(Span::styled("⏳ Executing...", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
    } else {
        final_lines.push(Line::from(""));
        final_lines.push(Line::from(Span::styled("✓ Complete", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
        final_lines.push(Line::from(""));
        final_lines.push(Line::from(Span::styled("Press 'q' or ESC to close", Style::default().fg(Color::Gray))));
    }

    let title = format!(" {} ", tui.output_title);
    let border_color = if tui.output_executing { Color::Yellow } else { Color::Green };

    let output = Paragraph::new(final_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title))
        .wrap(Wrap { trim: false });

    f.render_widget(output, chunks[1]);

    // Custom footer for output view
    let footer_text = if tui.output_executing {
        vec![
            Span::styled(" ⏳ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Command is executing... "),
            Span::styled("↑↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Scroll"),
        ]
    } else {
        vec![
            Span::styled(" [q] ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Close  "),
            Span::styled("↑↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Scroll"),
        ]
    };

    let footer = Paragraph::new(Line::from(footer_text))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Left);

    f.render_widget(footer, chunks[2]);
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

    // Count items by current sort mode for contextual information
    let (status_text, detail_text) = match tui.sort_mode {
        SortMode::Priority => {
            let critical = tui.advice.iter().filter(|a| matches!(a.priority, Priority::Mandatory)).count();
            let recommended = tui.advice.iter().filter(|a| matches!(a.priority, Priority::Recommended)).count();
            let optional = tui.advice.iter().filter(|a| matches!(a.priority, Priority::Optional)).count();
            let cosmetic = tui.advice.iter().filter(|a| matches!(a.priority, Priority::Cosmetic)).count();

            let status = if critical > 0 {
                format!("Critical: {} issue{}", critical, if critical == 1 { "" } else { "s" })
            } else if score >= 90 {
                "Excellent".to_string()
            } else if score >= 70 {
                "Good".to_string()
            } else if score >= 50 {
                "Fair".to_string()
            } else {
                "Needs Attention".to_string()
            };

            let detail = format!("  🔴{} 🟡{} 🟢{} ⚪{}", critical, recommended, optional, cosmetic);
            (status, detail)
        },
        SortMode::Risk => {
            let high = tui.advice.iter().filter(|a| matches!(a.risk, RiskLevel::High)).count();
            let medium = tui.advice.iter().filter(|a| matches!(a.risk, RiskLevel::Medium)).count();
            let low = tui.advice.iter().filter(|a| matches!(a.risk, RiskLevel::Low)).count();

            let status = if score >= 90 {
                "Excellent".to_string()
            } else if score >= 70 {
                "Good".to_string()
            } else if score >= 50 {
                "Fair".to_string()
            } else {
                "Needs Attention".to_string()
            };

            let detail = format!("  High:{} Med:{} Low:{}", high, medium, low);
            (status, detail)
        },
        SortMode::Category => {
            let categories: std::collections::HashSet<_> = tui.advice.iter()
                .map(|a| &a.category)
                .collect();

            let status = if score >= 90 {
                "Excellent".to_string()
            } else if score >= 70 {
                "Good".to_string()
            } else if score >= 50 {
                "Fair".to_string()
            } else {
                "Needs Attention".to_string()
            };

            let detail = format!("  {} categories affected", categories.len());
            (status, detail)
        },
    };

    let gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(" System Health Score "))
        .gauge_style(Style::default().fg(color))
        .label(format!("{}/100 - {}{}", score, status_text, detail_text))
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
        tui.list_to_advice_map.clear();
        return;
    }

    let mut items: Vec<ListItem> = Vec::new();
    let mut mapping: Vec<Option<usize>> = Vec::new();

    // Add category headers when sorted by category
    if tui.sort_mode == SortMode::Category {
        let mut last_category = "";
        for (advice_idx, advice) in tui.advice.iter().enumerate() {
            // Add category header if changed
            if advice.category != last_category {
                last_category = &advice.category;
                let (category_emoji, category_color) = get_category_emoji_color(&advice.category);

                let header = Line::from(vec![
                    Span::styled(
                        format!("━━ {} {} ", category_emoji, advice.category),
                        Style::default()
                            .fg(category_color)
                            .add_modifier(Modifier::BOLD)
                    ),
                ]);
                items.push(ListItem::new(header));
                mapping.push(None); // Header is not selectable
            }

            // Add the recommendation item
            items.push(create_recommendation_item(advice, tui.sort_mode));
            mapping.push(Some(advice_idx)); // Map this list item to advice index
        }
    } else {
        // No headers for other sort modes - direct 1:1 mapping
        for (advice_idx, advice) in tui.advice.iter().enumerate() {
            items.push(create_recommendation_item(advice, tui.sort_mode));
            mapping.push(Some(advice_idx));
        }
    }

    // Update the mapping
    tui.list_to_advice_map = mapping;

    // Ensure selection is valid and points to a selectable item
    if let Some(selected) = tui.list_state.selected() {
        if selected >= tui.list_to_advice_map.len() || tui.list_to_advice_map[selected].is_none() {
            // Find first selectable item
            if let Some(first_selectable) = tui.list_to_advice_map.iter().position(|opt| opt.is_some()) {
                tui.list_state.select(Some(first_selectable));
            }
        }
    } else if !tui.list_to_advice_map.is_empty() {
        // No selection, select first selectable item
        if let Some(first_selectable) = tui.list_to_advice_map.iter().position(|opt| opt.is_some()) {
            tui.list_state.select(Some(first_selectable));
        }
    }

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

/// Create a single recommendation list item with appropriate styling
fn create_recommendation_item(advice: &Advice, sort_mode: SortMode) -> ListItem<'_> {
    let priority_icon = match advice.priority {
        Priority::Mandatory => "🔴",
        Priority::Recommended => "🟡",
        Priority::Optional => "🟢",
        Priority::Cosmetic => "⚪",
    };

    let risk_badge = match advice.risk {
        RiskLevel::Low => ("✓", Color::Green),
        RiskLevel::Medium => ("⚠", Color::Yellow),
        RiskLevel::High => ("⚠", Color::Red),
    };

    let (category_emoji, category_color) = get_category_emoji_color(&advice.category);
    let popularity_stars = advice.popularity_stars();

    // Choose text color based on sort mode
    let (text_color, show_category_name) = match sort_mode {
        SortMode::Category => (category_color, false), // Category already shown in header
        SortMode::Priority => (match advice.priority {
            Priority::Mandatory => Color::Red,
            Priority::Recommended => Color::Yellow,
            Priority::Optional => Color::Green,
            Priority::Cosmetic => Color::Gray,
        }, false),
        SortMode::Risk => (Color::White, true), // Show category name for clarity
    };

    let mut spans = vec![
        Span::raw(priority_icon),
        Span::raw(" "),
        Span::styled(risk_badge.0, Style::default().fg(risk_badge.1)),
        Span::raw(" "),
        Span::styled(category_emoji, Style::default().fg(category_color)),
        Span::raw(" "),
    ];

    // When sorted by risk, show category name in brackets for clarity
    if show_category_name {
        spans.push(Span::styled(
            format!("[{}] ", advice.category),
            Style::default().fg(category_color)
        ));
    }

    spans.push(Span::styled(&advice.title, Style::default().fg(text_color)));
    spans.push(Span::raw("  "));
    spans.push(Span::styled(popularity_stars, Style::default().fg(Color::Yellow)));

    ListItem::new(Line::from(spans))
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
            (" d ", " Ignore Cat  "),
            (" i ", " Ignore Pri  "),
        ],
        "Details" => {
            // Check if selected advice has a command (actionable)
            let has_command = tui.selected_advice()
                .map(|a| a.command.is_some())
                .unwrap_or(false);

            if has_command {
                vec![
                    (" Esc ", " Back  "),
                    (" a ", " Apply  "),
                    (" d ", " Ignore Cat  "),
                    (" i ", " Ignore Pri  "),
                    (" q ", " Quit  "),
                ]
            } else {
                vec![
                    (" Esc ", " Back  "),
                    (" d ", " Ignore Cat  "),
                    (" i ", " Ignore Pri  "),
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

        // Add filter indicator if filters are active
        if let Ok(filters) = anna_common::IgnoreFilters::load() {
            let total_filters = filters.ignored_categories.len() + filters.ignored_priorities.len();
            if total_filters > 0 {
                spans.push(Span::raw("  │  "));
                spans.push(Span::styled(
                    format!("🔍 {} filter{}", total_filters, if total_filters == 1 { "" } else { "s" }),
                    Style::default().fg(Color::Yellow)
                ));
            }
        }
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

        // Execute pending apply action if any
        if tui.pending_apply.is_some() {
            let _ = tui.execute_pending_apply().await; // Ignore errors, status shown to user
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
