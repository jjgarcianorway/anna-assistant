//! Formatting utilities for resolution time display.

use super::stats::ResolutionTimeTracker;

/// Format milliseconds as human-readable duration
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else if ms < 3600000 {
        let mins = ms / 60000;
        let secs = (ms % 60000) / 1000;
        format!("{}m {}s", mins, secs)
    } else {
        let hours = ms / 3600000;
        let mins = (ms % 3600000) / 60000;
        format!("{}h {}m", hours, mins)
    }
}

/// Format resolution time stats for display
pub fn format_resolution_times(tracker: &ResolutionTimeTracker) -> String {
    let mut output = String::new();

    output.push_str("Resolution Time Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    if tracker.total_resolutions == 0 {
        output.push_str("No resolutions recorded yet.\n");
        return output;
    }

    output.push_str(&format!(
        "Total Resolutions: {} ({:.1}% success)\n",
        tracker.total_resolutions,
        tracker.success_rate()
    ));
    output.push_str(&format!(
        "Average Time:      {}\n",
        format_duration_ms(tracker.average_ms() as u64)
    ));
    output.push_str(&format!(
        "Escalation Rate:   {:.1}%\n\n",
        tracker.escalation_rate()
    ));

    if let Some(fastest) = &tracker.fastest {
        output.push_str("Fastest Resolution:\n");
        output.push_str(&format!(
            "  {} - \"{}\"\n\n",
            fastest.duration_human(),
            if fastest.description.len() > 40 {
                format!("{}...", &fastest.description[..37])
            } else {
                fastest.description.clone()
            }
        ));
    }

    if let Some(slowest) = &tracker.slowest {
        output.push_str("Slowest Resolution:\n");
        output.push_str(&format!(
            "  {} - \"{}\"\n",
            slowest.duration_human(),
            if slowest.description.len() > 40 {
                format!("{}...", &slowest.description[..37])
            } else {
                slowest.description.clone()
            }
        ));
    }

    output
}

/// Format compact resolution time info
pub fn format_resolution_times_compact(tracker: &ResolutionTimeTracker) -> String {
    if tracker.total_resolutions == 0 {
        return "No resolutions yet".to_string();
    }

    let fastest = tracker.fastest.as_ref().map(|f| f.duration_ms).unwrap_or(0);
    let slowest = tracker.slowest.as_ref().map(|s| s.duration_ms).unwrap_or(0);

    format!(
        "{} resolutions, avg {}, range {}–{}",
        tracker.total_resolutions,
        format_duration_ms(tracker.average_ms() as u64),
        format_duration_ms(fastest),
        format_duration_ms(slowest)
    )
}
