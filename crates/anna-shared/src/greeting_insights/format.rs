//! Formatting utilities for greeting insights.

use crate::snapshot::SystemSnapshot;

use super::types::GreetingInsight;

/// Format insights for display in greeting
pub fn format_insights_for_greeting(insights: &[GreetingInsight]) -> Option<String> {
    if insights.is_empty() {
        return None;
    }

    let mut output = String::new();

    if insights.len() == 1 {
        let insight = &insights[0];
        if insight.positive {
            output.push_str(&format!(
                "Heads up from {}: {}",
                insight.staff_name, insight.message
            ));
        } else {
            output.push_str(&format!(
                "Quick note from {}: {}",
                insight.staff_name, insight.message
            ));
        }
    } else {
        output.push_str("A few things to note:\n");
        for insight in insights {
            let icon = if insight.positive { "✓" } else { "•" };
            output.push_str(&format!(
                "  {} {} says: {}\n",
                icon, insight.staff_name, insight.message
            ));
        }
    }

    Some(output)
}

/// Get a one-liner status summary
pub fn quick_status_line(snapshot: &SystemSnapshot) -> String {
    let disk_status = snapshot
        .disk
        .values()
        .max()
        .map(|p| {
            if *p >= 90 {
                "disks critical"
            } else if *p >= 80 {
                "disks busy"
            } else {
                "disks ok"
            }
        })
        .unwrap_or("disks ok");

    let mem_status = if snapshot.memory_total_bytes > 0 {
        let pct =
            (snapshot.memory_used_bytes as f64 / snapshot.memory_total_bytes as f64 * 100.0) as u8;
        if pct >= 90 {
            "memory high"
        } else if pct >= 80 {
            "memory busy"
        } else {
            "memory ok"
        }
    } else {
        "memory ok"
    };

    let svc_status = if snapshot.failed_services.is_empty() {
        "services ok"
    } else {
        "services need attention"
    };

    format!("{} • {} • {}", disk_status, mem_status, svc_status)
}
