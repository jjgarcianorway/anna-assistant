//! Helper functions for REPL greetings.

/// Get user's name from environment
pub(super) fn get_user_name() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "there".to_string())
}

/// Capitalize domain name for display
pub(super) fn capitalize_domain(domain: &str) -> String {
    let mut chars = domain.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
