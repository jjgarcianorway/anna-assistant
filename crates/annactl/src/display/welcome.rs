//! REPL welcome flow - personalized greeting per VISION.md.
//!
//! The welcome displays:
//! - Personalized greeting with username
//! - Time context (how long since last interaction)
//! - Brief health summary if issues exist
//! - Deterministic, clean, ASCII-only output

use anna_shared::session::SessionStore;

use super::colors::*;
use super::formatting::format_duration;

/// Get hours since last user interaction
fn hours_since_last_interaction() -> Option<u64> {
    let store = SessionStore::load().ok()?;

    // Find the most recent session activity
    let mut most_recent: Option<chrono::DateTime<chrono::Utc>> = None;

    for session in store.sessions.values() {
        if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&session.last_activity) {
            let utc = ts.with_timezone(&chrono::Utc);
            if most_recent.is_none() || utc > most_recent.unwrap() {
                most_recent = Some(utc);
            }
        }
    }

    most_recent.map(|last| {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(last);
        duration.num_hours().max(0) as u64
    })
}

/// Generate personalized greeting based on time since last seen
fn generate_time_greeting(hours: Option<u64>) -> &'static str {
    match hours {
        None => "Nice to meet you!",
        Some(h) if h < 1 => "Welcome back!",
        Some(h) if h < 4 => "Hey there!",
        Some(h) if h < 24 => "Hello again!",
        Some(h) if h < 48 => "It's been a little while!",
        Some(h) if h < 168 => "Good to see you again!",
        Some(_) => "Long time no see!",
    }
}

/// Print REPL greeting with personalized context
pub fn print_greeting() {
    let username = std::env::var("USER").unwrap_or_else(|_| "there".to_string());
    let hours = hours_since_last_interaction();
    let time_greeting = generate_time_greeting(hours);

    println!();
    print_colored("Hello ", DIM);
    print_colored(&username, CYAN);
    print!(", ");
    println_colored(time_greeting, DIM);
    println!();

    // Show time since last interaction if relevant
    if let Some(h) = hours {
        if h >= 24 {
            let duration = format_duration(h * 3600);
            print_colored("  last session: ", DIM);
            println_colored(&format!("{} ago", duration), DIM);
            println!();
        }
    }

    println_colored("Ask questions about your system in plain English.", DIM);
    println_colored("Type 'quit' or Ctrl-D to exit, 'help' for commands.", DIM);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_greeting_first_time() {
        let greeting = generate_time_greeting(None);
        assert!(greeting.contains("meet"));
    }

    #[test]
    fn test_time_greeting_recent() {
        let greeting = generate_time_greeting(Some(0));
        assert!(greeting.contains("back"));
    }

    #[test]
    fn test_time_greeting_hours() {
        let greeting = generate_time_greeting(Some(2));
        assert!(greeting.contains("Hey"));
    }

    #[test]
    fn test_time_greeting_day() {
        let greeting = generate_time_greeting(Some(30));
        assert!(greeting.contains("while"));
    }

    #[test]
    fn test_time_greeting_week() {
        let greeting = generate_time_greeting(Some(100));
        assert!(greeting.contains("again"));
    }

    #[test]
    fn test_time_greeting_long() {
        let greeting = generate_time_greeting(Some(200));
        assert!(greeting.contains("Long time"));
    }
}
