//! Theatre-style REPL greeting for Service Desk experience (v0.0.82).
//!
//! Creates a personal, aware greeting that makes Anna feel like a real
//! IT support person who knows the user and their system.
//!
//! v0.0.106: Integrates user profile for personalized patterns.

use anna_shared::snapshot::{self, DeltaItem, SystemSnapshot};
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::telemetry::TelemetrySnapshot;
use anna_shared::ui::{colors, HR};
use anna_shared::user_profile::UserProfile;

/// Print the theatre-style REPL greeting
/// Shows: personalized greeting, time since last visit, health deltas, patterns
/// v0.0.106: Loads user profile for personalized patterns
pub fn print_theatre_greeting(status: Option<&DaemonStatus>) {
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

    // v0.0.106: Load user profile
    let mut profile = UserProfile::load();

    // Load last snapshot for comparison
    let last_snapshot = snapshot::load_last_snapshot();
    let interaction_info = calculate_interaction_info(&last_snapshot);

    // Collect current state
    let telemetry = TelemetrySnapshot::collect();
    let mut current_snapshot = SystemSnapshot::now();
    let failed_services = collect_failed_services(&mut current_snapshot);

    // Calculate health delta if we have a previous snapshot
    let health_deltas = if let Some(ref prev) = last_snapshot {
        snapshot::diff_snapshots(prev, &current_snapshot)
    } else {
        Vec::new()
    };

    // Print header
    println!();
    println!("{}Anna Service Desk{}", colors::HEADER, colors::RESET);
    println!("{}", HR);

    // Personalized greeting based on interaction history
    print_personalized_greeting(&username, &interaction_info);

    // v0.0.106: Show personalized patterns if we have history
    print_user_patterns(&profile);

    // "Since last time" section if we have history
    if last_snapshot.is_some() {
        print_since_last_time(&telemetry, &health_deltas, failed_services, &interaction_info);
    }

    // System readiness (LLM state)
    if let Some(st) = status {
        print_system_readiness(st);
    }

    // Closing
    println!();
    println!("{}But I believe you want to ask me something, don't you?{}", colors::DIM, colors::RESET);
    println!();

    // v0.0.106: Update profile and save
    profile.record_session();
    let _ = profile.save();

    // Save snapshot for next time
    let _ = snapshot::save_snapshot(&current_snapshot);
}

/// Information about user's interaction history
struct InteractionInfo {
    hours_since_last: Option<u64>,
    days_since_last: Option<u64>,
    is_first_time: bool,
}

fn calculate_interaction_info(last_snapshot: &Option<SystemSnapshot>) -> InteractionInfo {
    match last_snapshot {
        Some(s) if s.captured_at > 0 => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let hours = now.saturating_sub(s.captured_at) / 3600;
            let days = hours / 24;
            InteractionInfo {
                hours_since_last: Some(hours),
                days_since_last: if days > 0 { Some(days) } else { None },
                is_first_time: false,
            }
        }
        _ => InteractionInfo {
            hours_since_last: None,
            days_since_last: None,
            is_first_time: true,
        },
    }
}

fn print_personalized_greeting(username: &str, info: &InteractionInfo) {
    if info.is_first_time {
        println!("Hello {}, welcome to Anna!", username);
        println!();
        println!("I'm your local IT department. Ask me anything about your system -");
        println!("from disk space to service status, I'm here to help.");
    } else if let Some(days) = info.days_since_last {
        if days >= 1 {
            println!(
                "Hello {}!",
                username
            );
            println!();
            let day_word = if days == 1 { "day" } else { "days" };
            println!(
                "{}It's been about {} {} since you checked with me!{}",
                colors::DIM, days, day_word, colors::RESET
            );
        } else {
            println!("Hello {}, welcome back.", username);
        }
    } else if let Some(hours) = info.hours_since_last {
        if hours > 12 {
            println!("Hello {}!", username);
            println!();
            println!(
                "{}It's been about {} hours since we last spoke.{}",
                colors::DIM, hours, colors::RESET
            );
        } else if hours > 1 {
            println!("Hello {}, welcome back.", username);
        } else {
            println!("Hello again, {}!", username);
        }
    } else {
        println!("Hello {}, welcome back.", username);
    }
}

/// v0.0.106: Print personalized patterns from user profile
/// v0.0.108: Enhanced with streak fire, tool counts, and more patterns
fn print_user_patterns(profile: &UserProfile) {
    // Only show if we have meaningful data
    if profile.tool_usage.is_empty() && profile.topic_interests.is_empty() && profile.streak_days <= 1 {
        return;
    }

    let mut patterns = Vec::new();

    // v0.0.108: Streak info with fire emoji for long streaks
    if profile.streak_days > 1 {
        let streak_msg = if profile.streak_days >= 7 {
            format!(
                "{} {}🔥 {} day streak!{} You're on fire!",
                bullet(), colors::OK, profile.streak_days, colors::RESET
            )
        } else {
            format!(
                "{} {} day streak! Keep it going.",
                bullet(), profile.streak_days
            )
        };
        patterns.push(streak_msg);
    }

    // Preferred editor
    if let Some(ref editor) = profile.preferred_editor {
        let count = profile.tool_usage.get(editor).copied().unwrap_or(0);
        if count > 2 {
            patterns.push(format!(
                "{} I've noticed you prefer {} ({} mentions).",
                bullet(), editor, count
            ));
        }
    }

    // Top topic
    if let Some(topic) = profile.top_topic() {
        let count = profile.topic_interests.get(topic).copied().unwrap_or(0);
        if count > 2 {
            patterns.push(format!(
                "{} You ask about {} a lot ({} times).",
                bullet(), topic, count
            ));
        }
    }

    // v0.0.108: Top tool if not an editor
    let editors = ["vim", "nvim", "nano", "emacs", "helix", "micro", "code"];
    if let Some((top_tool, count)) = profile.tool_usage.iter()
        .filter(|(k, _)| !editors.contains(&k.as_str()))
        .max_by_key(|(_, v)| *v)
    {
        if *count > 2 {
            patterns.push(format!(
                "{} You've been using {} ({} queries).",
                bullet(), top_tool, count
            ));
        }
    }

    // Show patterns if we have any (limit to 3)
    if !patterns.is_empty() {
        println!();
        for pattern in patterns.iter().take(3) {
            println!("{}{}{}", colors::DIM, pattern, colors::RESET);
        }
    }
}

fn print_since_last_time(
    telemetry: &TelemetrySnapshot,
    health_deltas: &[DeltaItem],
    failed_services: usize,
    info: &InteractionInfo,
) {
    // Only show "since last time" if it's been a while
    let show_section = info.hours_since_last.map(|h| h > 1).unwrap_or(false);
    if !show_section {
        return;
    }

    println!();
    println!("{}Since the last time, a few things happened:{}", colors::DIM, colors::RESET);
    println!();

    let mut items_shown = 0;
    let max_items = 4;

    // Boot time changes
    if let Some(boot_ms) = telemetry.boot_delta_ms {
        if boot_ms.abs() > 2000 && items_shown < max_items {
            let secs = boot_ms.unsigned_abs() / 1000;
            if boot_ms > 0 {
                println!(
                    "  {} Your boot time increased by {} seconds.",
                    bullet(), secs
                );
                // Add context if it's small
                if secs < 10 {
                    println!(
                        "    {}This is normal variation, nothing to worry about.{}",
                        colors::DIM, colors::RESET
                    );
                }
            } else {
                println!(
                    "  {} {}Your boot time improved by {} seconds!{}",
                    bullet(), colors::OK, secs, colors::RESET
                );
            }
            items_shown += 1;
        }
    }

    // Health deltas
    for delta in health_deltas.iter().take(max_items - items_shown) {
        match delta {
            DeltaItem::DiskWarning { mount, curr, .. } => {
                println!(
                    "  {} {}[warn]{} Disk {} is at {}% - getting full.",
                    bullet(), colors::WARN, colors::RESET, mount, curr
                );
            }
            DeltaItem::DiskCritical { mount, curr, .. } => {
                println!(
                    "  {} {}[critical]{} Disk {} is at {}% - needs attention!",
                    bullet(), colors::ERR, colors::RESET, mount, curr
                );
            }
            DeltaItem::DiskIncreased { mount, prev, curr } => {
                println!(
                    "  {} Disk {} increased from {}% to {}%.",
                    bullet(), mount, prev, curr
                );
            }
            DeltaItem::NewFailedService { unit } => {
                println!(
                    "  {} {}[fail]{} Service {} has failed.",
                    bullet(), colors::ERR, colors::RESET, unit
                );
            }
            DeltaItem::ServiceRecovered { unit } => {
                println!(
                    "  {} {}[recovered]{} Service {} is back up!",
                    bullet(), colors::OK, colors::RESET, unit
                );
            }
            DeltaItem::MemoryHigh { curr_percent, .. } => {
                println!(
                    "  {} {}[warn]{} Memory usage is high at {}%.",
                    bullet(), colors::WARN, colors::RESET, curr_percent
                );
            }
            DeltaItem::MemoryIncreased { prev_percent, curr_percent } => {
                println!(
                    "  {} Memory usage increased from {}% to {}%.",
                    bullet(), prev_percent, curr_percent
                );
            }
        }
        items_shown += 1;
    }

    // Service status summary
    if items_shown < max_items {
        if failed_services > 0 {
            println!(
                "  {} {} service{} currently in failed state.",
                bullet(), failed_services, if failed_services == 1 { "" } else { "s" }
            );
        } else if health_deltas.is_empty() {
            println!(
                "  {} {}No warnings or errors detected - looking good!{}",
                bullet(), colors::OK, colors::RESET
            );
        }
    }
}

fn print_system_readiness(status: &DaemonStatus) {
    println!();

    match status.llm.state {
        LlmState::Ready => {
            // Show which models are ready
            if let (Some(trans), Some(spec)) = (&status.llm.translator_model, &status.llm.specialist_model) {
                println!(
                    "{}Systems ready. Translator: {}, Specialist: {}{}",
                    colors::DIM, trans, spec, colors::RESET
                );
            } else {
                println!("{}All systems ready.{}", colors::DIM, colors::RESET);
            }
        }
        LlmState::Bootstrapping => {
            if let Some(phase) = &status.llm.phase {
                println!(
                    "{}[starting]{} {}...",
                    colors::WARN, colors::RESET, phase
                );
            } else {
                println!(
                    "{}[starting]{} Preparing AI models...",
                    colors::WARN, colors::RESET
                );
            }
            // Show progress if available
            if let Some(progress) = &status.llm.progress {
                let bar = anna_shared::ui::progress_bar(progress.percent(), 30);
                println!("  {} {:.0}%", bar, progress.percent() * 100.0);
            }
        }
        LlmState::Error => {
            println!(
                "{}[error]{} AI models not available. Some features may be limited.",
                colors::ERR, colors::RESET
            );
            if let Some(err) = &status.last_error {
                println!("  {}{}{}", colors::DIM, err, colors::RESET);
            }
        }
    }

    // Update notification
    if status.update.update_available {
        if let Some(ver) = &status.update.latest_version {
            println!();
            println!(
                "{}[update]{} Version {} is available. I'll update automatically.",
                colors::CYAN, colors::RESET, ver
            );
        }
    }
}

fn collect_failed_services(snapshot: &mut SystemSnapshot) -> usize {
    let mut count = 0;

    if let Ok(output) = std::process::Command::new("systemctl")
        .args(["--failed", "--no-pager", "-q"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(".service") || line.contains(".mount") {
                    count += 1;
                    if let Some(unit) = line.split_whitespace().find(|p|
                        p.ends_with(".service") || p.ends_with(".mount")
                    ) {
                        snapshot.add_failed_service(unit);
                    }
                }
            }
        }
    }

    count
}

fn bullet() -> &'static str {
    "›"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_interaction_info_first_time() {
        let info = calculate_interaction_info(&None);
        assert!(info.is_first_time);
        assert!(info.hours_since_last.is_none());
    }

    #[test]
    fn test_bullet_char() {
        assert_eq!(bullet(), "›");
    }
}
