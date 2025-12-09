//! Personalized greeting sections (v0.0.186).

use anna_shared::ticket_tracker::TicketTracker;
use anna_shared::ui::colors;
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
        println!(
            "{}Try asking: \"is my system healthy?\" or \"show disk usage\"{}",
            colors::DIM,
            colors::RESET
        );
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

/// Print personalized patterns from user profile
/// v0.0.142: More conversational pattern observations
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

    // Preferred editor - conversational observation
    if let Some(ref editor) = profile.preferred_editor {
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
                patterns.push(format!(
                    "    {}If you want, I can suggest some {} tips!{}",
                    colors::DIM,
                    editor,
                    colors::RESET
                ));
            }
        }
    }

    // Top topic - conversational
    if let Some(topic) = profile.top_topic() {
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
        println!("{}On your patterns:{}", colors::DIM, colors::RESET);
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
    println!("{}Open Tickets:{}", colors::WARN, colors::RESET);
    for ticket in open_tickets.iter().take(3) {
        // Show full query, wrapped naturally by terminal
        println!("  {} {} ({})", bullet(), ticket.case_number, ticket.status);
        println!("    {}", ticket.query);
    }
    if open_tickets.len() > 3 {
        println!("  {} and {} more", bullet(), open_tickets.len() - 3);
    }
    println!();
    println!(
        "{}To reply: annactl reply CN-XXXX \"message\"{}",
        colors::DIM,
        colors::RESET
    );
}
