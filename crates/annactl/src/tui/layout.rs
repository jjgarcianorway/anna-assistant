//! Layout - Canonical TUI layout grid computation
//!
//! Beta.262: Centralized layout grid with stable, predictable panel sizing.
//! This module defines the TUI layout structure and computes panel rectangles
//! for header, conversation, diagnostics, and status bar with graceful degradation.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// TUI layout structure containing all panel rectangles
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiLayout {
    /// Header bar (top)
    pub header: Rect,
    /// Main conversation panel
    pub conversation: Rect,
    /// Diagnostics / side panel (optional, may be zero height if terminal too small)
    pub diagnostics: Rect,
    /// Status bar (bottom)
    pub status_bar: Rect,
    /// Input area (bottom, above status bar)
    pub input: Rect,
}

/// Minimum heights for each panel
const MIN_HEADER_HEIGHT: u16 = 1;
const MIN_STATUS_BAR_HEIGHT: u16 = 1;
const MIN_INPUT_HEIGHT: u16 = 3;
const MIN_CONVERSATION_HEIGHT: u16 = 8;
const MIN_DIAGNOSTICS_HEIGHT: u16 = 5;

/// Compute canonical TUI layout grid
///
/// Takes the terminal area and returns a TuiLayout with non-overlapping rectangles.
/// Gracefully degrades on small terminals in this priority order:
/// 1. Header (always shown, 1 line)
/// 2. Status bar (always shown, 1 line)
/// 3. Input (always shown, minimum 3 lines)
/// 4. Conversation (priority, minimum 8 lines)
/// 5. Diagnostics (omitted first if space tight, minimum 5 lines)
///
/// # Examples
///
/// ```
/// use ratatui::layout::Rect;
/// use crate::tui::layout::compute_layout;
///
/// let area = Rect::new(0, 0, 80, 24);
/// let layout = compute_layout(area);
/// assert_eq!(layout.header.height, 1);
/// assert!(layout.conversation.height >= 8);
/// ```
pub fn compute_layout(frame_area: Rect) -> TuiLayout {
    let total_height = frame_area.height;

    // Calculate required height for fixed elements
    let fixed_height = MIN_HEADER_HEIGHT + MIN_STATUS_BAR_HEIGHT + MIN_INPUT_HEIGHT;

    // Calculate remaining height for conversation and diagnostics
    let remaining_height = total_height.saturating_sub(fixed_height);

    // Decide if we can show diagnostics panel
    let show_diagnostics = remaining_height >= (MIN_CONVERSATION_HEIGHT + MIN_DIAGNOSTICS_HEIGHT);

    // Split main area vertically: header, middle (conversation + diagnostics + input), status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(MIN_HEADER_HEIGHT),
            Constraint::Min(0), // Middle section (flexible)
            Constraint::Length(MIN_STATUS_BAR_HEIGHT),
        ])
        .split(frame_area);

    let header = main_chunks[0];
    let middle_area = main_chunks[1];
    let status_bar = main_chunks[2];

    // Split middle section: conversation, diagnostics (optional), input
    let middle_chunks = if show_diagnostics {
        // Calculate conversation and diagnostics heights
        let diagnostics_height = MIN_DIAGNOSTICS_HEIGHT;
        let conversation_height = middle_area
            .height
            .saturating_sub(diagnostics_height)
            .saturating_sub(MIN_INPUT_HEIGHT);

        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(conversation_height),
                Constraint::Length(diagnostics_height),
                Constraint::Length(MIN_INPUT_HEIGHT),
            ])
            .split(middle_area)
    } else {
        // No diagnostics panel, just conversation and input
        let conversation_height = middle_area.height.saturating_sub(MIN_INPUT_HEIGHT);

        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(conversation_height),
                Constraint::Length(MIN_INPUT_HEIGHT),
            ])
            .split(middle_area)
    };

    let conversation = middle_chunks[0];
    let (diagnostics, input) = if show_diagnostics {
        (middle_chunks[1], middle_chunks[2])
    } else {
        // Zero-height diagnostics panel
        (
            Rect::new(0, 0, 0, 0),
            middle_chunks[1],
        )
    };

    TuiLayout {
        header,
        conversation,
        diagnostics,
        status_bar,
        input,
    }
}

/// Check if a scroll indicator should be shown at the top of a panel
///
/// Returns true if the scroll offset is greater than 0 (content scrolled down)
pub fn should_show_scroll_up_indicator(scroll_offset: usize) -> bool {
    scroll_offset > 0
}

/// Check if a scroll indicator should be shown at the bottom of a panel
///
/// Returns true if there is more content below the visible area
///
/// # Arguments
///
/// * `total_content_lines` - Total number of lines in the content
/// * `visible_lines` - Number of lines visible in the panel
/// * `scroll_offset` - Current scroll offset (number of lines scrolled down)
pub fn should_show_scroll_down_indicator(
    total_content_lines: usize,
    visible_lines: usize,
    scroll_offset: usize,
) -> bool {
    let visible_end = scroll_offset + visible_lines;
    visible_end < total_content_lines
}

/// Compose header text with truncation for narrow terminals
///
/// Returns a string that fits in the given width without wrapping.
/// Priority order (rightmost truncates first):
/// 1. Left: "Anna v{version}"
/// 2. Middle: "{username}@{hostname}"
/// 3. Right: "{model_name}" (truncates first)
///
/// # Arguments
///
/// * `width` - Available width for header text
/// * `version` - Anna version string
/// * `username` - Current user
/// * `hostname` - System hostname
/// * `model_name` - LLM model name
pub fn compose_header_text(
    width: u16,
    version: &str,
    username: &str,
    hostname: &str,
    model_name: &str,
) -> String {
    let width = width as usize;

    // Minimum: "Anna v{version}" always shown
    let anna_part = format!("Anna v{}", version);

    // Check if we have space for anything beyond Anna version
    if width < anna_part.len() + 3 {
        // Too narrow, just show Anna part (truncated if necessary)
        return anna_part.chars().take(width).collect();
    }

    // Build user@host part
    let user_host = format!("{}@{}", username, hostname);

    // Build full text
    let separator = " │ ";
    let full_text = format!("{}{}{}{}{}",
        anna_part, separator, model_name, separator, user_host);

    if full_text.len() <= width {
        // Everything fits
        return full_text;
    }

    // Need to truncate - priority: truncate model name first
    let model_truncated = if model_name.len() > 15 {
        format!("{}...", &model_name[..12])
    } else {
        model_name.to_string()
    };

    let text_with_short_model = format!("{}{}{}{}{}",
        anna_part, separator, model_truncated, separator, user_host);

    if text_with_short_model.len() <= width {
        return text_with_short_model;
    }

    // Still too long - truncate hostname
    let host_truncated = if hostname.len() > 10 {
        format!("{}...", &hostname[..7])
    } else {
        hostname.to_string()
    };

    let text_with_short_host = format!("{}@{}", username, host_truncated);
    let final_text = format!("{}{}{}{}{}",
        anna_part, separator, model_truncated, separator, text_with_short_host);

    if final_text.len() <= width {
        return final_text;
    }

    // Last resort: just show Anna part and model (no user@host)
    let minimal = format!("{}{}{}", anna_part, separator, model_truncated);
    if minimal.len() <= width {
        return minimal;
    }

    // Ultra-minimal: just Anna
    anna_part.chars().take(width).collect()
}

/// Compose status bar text with truncation for narrow terminals
///
/// Returns a string that fits in the given width without wrapping.
/// Sections (rightmost truncates first):
/// - Mode, Time, Health/Thinking always shown
/// - CPU/RAM shown if space
/// - Daemon status shown if space
/// - Brain diagnostics shown if space
///
/// # Arguments
///
/// * `width` - Available width for status bar text
/// * `time_str` - Current time (HH:MM:SS format)
/// * `thinking` - Whether LLM is thinking
/// * `health_ok` - Whether system is healthy
/// * `cpu_pct` - CPU usage percentage
/// * `ram_gb` - RAM usage in GB
/// * `daemon_ok` - Whether daemon is available
/// * `brain_critical` - Number of critical brain issues
/// * `brain_warning` - Number of warning brain issues
pub fn compose_status_bar_text(
    width: u16,
    time_str: &str,
    thinking: bool,
    health_ok: bool,
    cpu_pct: f64,
    ram_gb: f64,
    daemon_ok: bool,
    brain_critical: usize,
    brain_warning: usize,
) -> String {
    let width = width as usize;
    let sep = " │ ";

    // Core parts (always shown)
    let mode = "Mode: TUI";
    let time_part = time_str;
    let health_part = if thinking {
        "⣾ Thinking..."
    } else if health_ok {
        "Health: ✓"
    } else {
        "Health: ✗"
    };

    let core = format!("{}{}{}{}{}", mode, sep, time_part, sep, health_part);

    if core.len() >= width {
        // Too narrow for even core - truncate
        return core.chars().take(width).collect();
    }

    // Add CPU/RAM if space
    let cpu_ram = format!("CPU: {:.0}%{}RAM: {:.1}G", cpu_pct, sep, ram_gb);
    let with_resources = format!("{}{}{}", core, sep, cpu_ram);

    if with_resources.len() >= width {
        return core;
    }

    // Add daemon status if space
    let daemon_part = format!("Daemon: {}", if daemon_ok { "✓" } else { "✗" });
    let with_daemon = format!("{}{}{}", with_resources, sep, daemon_part);

    if with_daemon.len() >= width {
        return with_resources;
    }

    // Add brain diagnostics if space and there are issues
    if brain_critical > 0 {
        let brain_part = format!("Brain: {}✗", brain_critical);
        let final_text = format!("{}{}{}", with_daemon, sep, brain_part);
        if final_text.len() <= width {
            return final_text;
        }
    } else if brain_warning > 0 {
        let brain_part = format!("Brain: {}⚠", brain_warning);
        let final_text = format!("{}{}{}", with_daemon, sep, brain_part);
        if final_text.len() <= width {
            return final_text;
        }
    }

    // Return what fits
    with_daemon
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_80x24() {
        // Classic terminal size
        let area = Rect::new(0, 0, 80, 24);
        let layout = compute_layout(area);

        // Header should be 1 line
        assert_eq!(layout.header.height, 1);

        // Status bar should be 1 line
        assert_eq!(layout.status_bar.height, 1);

        // Input should be at least minimum
        assert!(layout.input.height >= MIN_INPUT_HEIGHT);

        // Conversation should be at least minimum
        assert!(layout.conversation.height >= MIN_CONVERSATION_HEIGHT);

        // Diagnostics should be shown (enough space)
        assert!(layout.diagnostics.height >= MIN_DIAGNOSTICS_HEIGHT);

        // All panels should be non-overlapping (sum of heights should equal total)
        let total = layout.header.height
            + layout.conversation.height
            + layout.diagnostics.height
            + layout.input.height
            + layout.status_bar.height;
        assert_eq!(total, 24);
    }

    #[test]
    fn test_layout_100x40() {
        // Larger terminal
        let area = Rect::new(0, 0, 100, 40);
        let layout = compute_layout(area);

        assert_eq!(layout.header.height, 1);
        assert_eq!(layout.status_bar.height, 1);
        assert!(layout.input.height >= MIN_INPUT_HEIGHT);
        assert!(layout.conversation.height >= MIN_CONVERSATION_HEIGHT);
        assert!(layout.diagnostics.height >= MIN_DIAGNOSTICS_HEIGHT);

        // Should have more space for conversation
        assert!(layout.conversation.height > MIN_CONVERSATION_HEIGHT);

        let total = layout.header.height
            + layout.conversation.height
            + layout.diagnostics.height
            + layout.input.height
            + layout.status_bar.height;
        assert_eq!(total, 40);
    }

    #[test]
    fn test_layout_120x30() {
        // Wide but moderate height
        let area = Rect::new(0, 0, 120, 30);
        let layout = compute_layout(area);

        assert_eq!(layout.header.height, 1);
        assert_eq!(layout.status_bar.height, 1);
        assert!(layout.input.height >= MIN_INPUT_HEIGHT);
        assert!(layout.conversation.height >= MIN_CONVERSATION_HEIGHT);
        assert!(layout.diagnostics.height >= MIN_DIAGNOSTICS_HEIGHT);

        let total = layout.header.height
            + layout.conversation.height
            + layout.diagnostics.height
            + layout.input.height
            + layout.status_bar.height;
        assert_eq!(total, 30);
    }

    #[test]
    fn test_layout_small_terminal_no_diagnostics() {
        // Small terminal where diagnostics panel should be omitted
        let area = Rect::new(0, 0, 80, 15);
        let layout = compute_layout(area);

        assert_eq!(layout.header.height, 1);
        assert_eq!(layout.status_bar.height, 1);
        assert!(layout.input.height >= MIN_INPUT_HEIGHT);
        assert!(layout.conversation.height >= MIN_CONVERSATION_HEIGHT);

        // Diagnostics should be zero height (not shown)
        assert_eq!(layout.diagnostics.height, 0);

        // Total should still match (without diagnostics)
        let total = layout.header.height
            + layout.conversation.height
            + layout.input.height
            + layout.status_bar.height;
        assert_eq!(total, 15);
    }

    #[test]
    fn test_layout_minimal_viable() {
        // Minimal terminal size (header + status + input + conversation minimum)
        let min_height = MIN_HEADER_HEIGHT + MIN_STATUS_BAR_HEIGHT + MIN_INPUT_HEIGHT + MIN_CONVERSATION_HEIGHT;
        let area = Rect::new(0, 0, 80, min_height);
        let layout = compute_layout(area);

        assert_eq!(layout.header.height, 1);
        assert_eq!(layout.status_bar.height, 1);
        assert_eq!(layout.input.height, MIN_INPUT_HEIGHT);
        assert_eq!(layout.conversation.height, MIN_CONVERSATION_HEIGHT);
        assert_eq!(layout.diagnostics.height, 0); // No space for diagnostics
    }

    #[test]
    fn test_scroll_up_indicator() {
        // No scroll offset - no indicator
        assert!(!should_show_scroll_up_indicator(0));

        // Scrolled down - show indicator
        assert!(should_show_scroll_up_indicator(5));
        assert!(should_show_scroll_up_indicator(100));
    }

    #[test]
    fn test_scroll_down_indicator() {
        // Total content: 20 lines, visible: 10 lines
        // Scroll offset: 0 - can scroll down
        assert!(should_show_scroll_down_indicator(20, 10, 0));

        // Scroll offset: 5 - still can scroll down (5 + 10 = 15 < 20)
        assert!(should_show_scroll_down_indicator(20, 10, 5));

        // Scroll offset: 10 - at bottom (10 + 10 = 20)
        assert!(!should_show_scroll_down_indicator(20, 10, 10));

        // Scroll offset: 15 - past bottom
        assert!(!should_show_scroll_down_indicator(20, 10, 15));

        // Content fits exactly - no indicator
        assert!(!should_show_scroll_down_indicator(10, 10, 0));

        // Content smaller than visible - no indicator
        assert!(!should_show_scroll_down_indicator(5, 10, 0));
    }

    #[test]
    fn test_layout_width_propagation() {
        // Ensure all panels have the same width as the terminal
        let area = Rect::new(0, 0, 100, 30);
        let layout = compute_layout(area);

        assert_eq!(layout.header.width, 100);
        assert_eq!(layout.conversation.width, 100);
        assert_eq!(layout.status_bar.width, 100);
        assert_eq!(layout.input.width, 100);
        // Diagnostics width should match (even if height is 0)
        if layout.diagnostics.height > 0 {
            assert_eq!(layout.diagnostics.width, 100);
        }
    }

    #[test]
    fn test_compose_header_full_width() {
        // Header with plenty of space
        let result = compose_header_text(80, "5.7.0-beta.262", "user", "testhost", "qwen2.5:3b");
        assert!(result.contains("Anna v5.7.0-beta.262"));
        assert!(result.contains("qwen2.5:3b"));
        assert!(result.contains("user@testhost"));
        assert!(result.len() <= 80);
    }

    #[test]
    fn test_compose_header_truncate_model() {
        // Narrow width - should truncate model name first
        let result = compose_header_text(50, "5.7.0-beta.262", "user", "testhost", "very-long-model-name-here");
        assert!(result.contains("Anna v5.7.0-beta.262"));
        // Model should be truncated
        assert!(result.contains("...") || !result.contains("very-long-model-name-here"));
        assert!(result.len() <= 50);
    }

    #[test]
    fn test_compose_header_truncate_hostname() {
        // Very narrow - should truncate hostname
        let result = compose_header_text(40, "5.7.0-beta.262", "user", "very-long-hostname", "qwen2.5:3b");
        assert!(result.contains("Anna"));
        assert!(result.len() <= 40);
    }

    #[test]
    fn test_compose_header_minimal() {
        // Ultra narrow - just Anna version
        let result = compose_header_text(20, "5.7.0-beta.262", "user", "testhost", "qwen2.5:3b");
        assert!(result.contains("Anna"));
        assert!(result.len() <= 20);
    }

    #[test]
    fn test_compose_status_bar_full_width() {
        // Status bar with plenty of space
        let result = compose_status_bar_text(100, "15:42:08", false, true, 8.5, 4.2, true, 0, 0);
        assert!(result.contains("Mode: TUI"));
        assert!(result.contains("15:42:08"));
        assert!(result.contains("Health: ✓"));
        assert!(result.contains("CPU:"));
        assert!(result.contains("RAM:"));
        assert!(result.contains("Daemon:"));
        assert!(result.len() <= 100);
    }

    #[test]
    fn test_compose_status_bar_narrow() {
        // Narrow width - should omit daemon and brain
        let result = compose_status_bar_text(40, "15:42:08", false, true, 8.5, 4.2, true, 0, 0);
        assert!(result.contains("Mode: TUI"));
        assert!(result.contains("15:42:08"));
        assert!(result.len() <= 40);
    }

    #[test]
    fn test_compose_status_bar_thinking() {
        // Status bar with thinking indicator
        let result = compose_status_bar_text(100, "15:42:08", true, true, 8.5, 4.2, true, 0, 0);
        assert!(result.contains("Thinking"));
    }

    #[test]
    fn test_compose_status_bar_brain_critical() {
        // Status bar with brain critical issues
        let result = compose_status_bar_text(100, "15:42:08", false, true, 8.5, 4.2, true, 2, 1);
        assert!(result.contains("Brain:"));
        assert!(result.contains("2✗"));
    }

    #[test]
    fn test_compose_status_bar_brain_warning() {
        // Status bar with brain warnings (no critical)
        let result = compose_status_bar_text(100, "15:42:08", false, true, 8.5, 4.2, true, 0, 3);
        assert!(result.contains("Brain:"));
        assert!(result.contains("3⚠"));
    }
}
