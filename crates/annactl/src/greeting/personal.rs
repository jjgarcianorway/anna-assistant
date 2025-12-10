//! Personalized greeting sections (v0.0.347).
//!
//! v0.0.236: Added editor trend insights to pattern display.
//! v0.0.238: Added "since last time" summary display.
//! v0.0.275: Most functions now unused (LLM generates greetings), kept for fallback.
//! v0.0.347: Use print_hint(), print_label(), print_section_header() for consistency.

#![allow(dead_code)]

use anna_shared::ticket_tracker::TicketTracker;
use anna_shared::ui::{colors, print_hint, print_label, print_section_header};
use anna_shared::user_profile::UserProfile;

use super::types::{bullet, InteractionInfo};

pub fn print_personalized_greeting(username: &str, info: &InteractionInfo) {
    println!();

    if info.is_first_time {
        println!("Hello {},", username);
        println!();
        println!("Welcome! I'm Anna, your local IT department.");
        println!("Just ask me anything about your system - I'm here to help.");
        println!();
        print_hint("Try asking: \"is my system healthy?\" or \"show disk usage\"");
    } else if let Some(days) = info.days_since_last {
        if days >= 1 {
            println!("Hello {},", username);
            println!();
            let day_word = if days == 1 { "day" } else { "days" };
            println!(
                "It's been a while since you checked with me! (Almost {} {}).",
                days, day_word
            );
        } else {
            println!("Hello {}, welcome back.", username);
        }
    } else if let Some(hours) = info.hours_since_last {
        if hours > 12 {
            println!("Hello {},", username);
            println!();
            println!("It's been about {} hours since we last spoke.", hours);
        } else if hours > 1 {
            println!("Hello {}, welcome back.", username);
        } else {
            println!("Hello again, {}!", username);
        }
    } else {
        println!("Hello {}, welcome back.", username);
    }
}

/// v0.0.238: Print "since last time" summary if available
pub fn print_since_last_time(profile: &UserProfile) {
    if let Some(summary) = profile.since_last_time() {
        println!();
        print_hint(&format!("📋 {}", summary));
    }
}

/// Print personalized patterns from user profile
/// v0.0.142: More conversational pattern observations
/// v0.0.236: Added editor trend insights
pub fn print_user_patterns(profile: &UserProfile) {
    // Only show if we have meaningful data
    if profile.tool_usage.is_empty()
        && profile.topic_interests.is_empty()
        && profile.streak_days <= 1
    {
        return;
    }

    let mut patterns = Vec::new();

    // Streak info - conversational
    if profile.streak_days > 1 {
        let streak_msg = if profile.streak_days >= 7 {
            format!(
                "{} You've been checking in for {} days straight - nice streak!",
                bullet(),
                profile.streak_days
            )
        } else {
            format!("{} {} day streak so far.", bullet(), profile.streak_days)
        };
        patterns.push(streak_msg);
    }

    // v0.0.236: Editor trend insight (learning new editor, switching)
    if let Some(trend) = profile.editor_trend() {
        patterns.push(format!("{} {}", bullet(), trend.to_message()));
    } else if let Some(ref editor) = profile.preferred_editor {
        // Fall back to simple preferred editor observation
        let count = profile.tool_usage.get(editor).copied().unwrap_or(0);
        if count > 2 {
            patterns.push(format!(
                "{} I've noticed you prefer {} (mentioned {} times).",
                bullet(),
                editor,
                count
            ));
            // Offer help if it's an editor
            if count > 5 {
                // Note: Can't use print_hint here as we're building a string vec
            patterns.push(format!(
                "    {}If you want, I can suggest some {} tips!{}",
                colors::DIM,
                editor,
                colors::RESET
            ));
            }
        }
    }

    // v0.0.236: Topic trend insight
    if let Some(trend) = profile.topic_trend() {
        patterns.push(format!("{} {}", bullet(), trend.to_message()));
    } else if let Some(topic) = profile.top_topic() {
        // Fall back to simple top topic observation
        let count = profile.topic_interests.get(topic).copied().unwrap_or(0);
        if count > 2 {
            patterns.push(format!(
                "{} You ask about {} frequently ({} times).",
                bullet(),
                topic,
                count
            ));
        }
    }

    // v0.0.108: Top tool if not an editor
    let editors = ["vim", "nvim", "nano", "emacs", "helix", "micro", "code"];
    if let Some((top_tool, count)) = profile
        .tool_usage
        .iter()
        .filter(|(k, _)| !editors.contains(&k.as_str()))
        .max_by_key(|(_, v)| *v)
    {
        if *count > 2 {
            patterns.push(format!(
                "{} You've been using {} quite a bit ({} queries).",
                bullet(),
                top_tool,
                count
            ));
        }
    }

    // Show patterns if we have any (limit to 3)
    if !patterns.is_empty() {
        println!();
        print_section_header("on your patterns");
        for pattern in patterns.iter().take(3) {
            println!("{}", pattern);
        }
    }
}

/// v0.0.116: Show open tickets if any
pub fn print_open_tickets() {
    let tracker = TicketTracker::for_user();

    // Get open tickets
    let open_tickets = match tracker.open_tickets() {
        Ok(tickets) if !tickets.is_empty() => tickets,
        _ => return, // No tickets, nothing to show
    };

    println!();
    print_label("open tickets", "", colors::WARN);
    for ticket in open_tickets.iter().take(3) {
        // Show full query, wrapped naturally by terminal
        println!("  {} {} ({})", bullet(), ticket.case_number, ticket.status);
        println!("    {}", ticket.query);
    }
    if open_tickets.len() > 3 {
        println!("  {} and {} more", bullet(), open_tickets.len() - 3);
    }
    println!();
    print_hint("Ask me about any ticket to continue the conversation.");
}
