//! Parsing utilities for email and time extraction.

/// Extract email from query
pub(super) fn extract_email(lower: &str, original: &str) -> Option<String> {
    // Look for "email to X" or "email is X" or "my email X"
    let patterns = ["email to ", "email is ", "my email ", "email: "];

    for pattern in patterns {
        if let Some(pos) = lower.find(pattern) {
            let start = pos + pattern.len();
            let rest = &original[start..];
            // Extract email-like string
            if let Some(email) = extract_email_address(rest) {
                return Some(email);
            }
        }
    }

    // Also check for standalone email in query
    for word in original.split_whitespace() {
        if word.contains('@') && word.contains('.') {
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '@' && c != '.' && c != '_' && c != '-');
            if is_valid_email(cleaned) {
                return Some(cleaned.to_string());
            }
        }
    }

    None
}

/// Extract email address from text
fn extract_email_address(text: &str) -> Option<String> {
    let email: String = text
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '@' || *c == '.' || *c == '_' || *c == '-' || *c == '+')
        .collect();

    if is_valid_email(&email) {
        Some(email)
    } else {
        None
    }
}

/// Basic email validation
pub(super) fn is_valid_email(s: &str) -> bool {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];

    !local.is_empty() && !domain.is_empty() && domain.contains('.')
}

/// Extract quiet hours from query
pub(super) fn extract_quiet_hours(lower: &str) -> Option<(String, String)> {
    // Look for patterns like "quiet hours 22:00 to 08:00"
    if !lower.contains("quiet") {
        return None;
    }

    // Extract times using regex-like patterns
    let times: Vec<String> = extract_times(lower);
    if times.len() >= 2 {
        return Some((times[0].clone(), times[1].clone()));
    }

    None
}

/// Extract time patterns (HH:MM) from text
fn extract_times(text: &str) -> Vec<String> {
    let mut times = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Look for digit patterns
        if chars[i].is_ascii_digit() {
            let start = i;
            // Collect digits
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            // Check for colon
            if i < chars.len() && chars[i] == ':' {
                i += 1;
                // Collect more digits
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let time_str: String = chars[start..i].iter().collect();
                if is_valid_time(&time_str) {
                    times.push(time_str);
                }
            }
        } else {
            i += 1;
        }
    }

    times
}

/// Check if string is valid time format
pub(super) fn is_valid_time(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    let hour: u8 = parts[0].parse().unwrap_or(99);
    let minute: u8 = parts[1].parse().unwrap_or(99);
    hour < 24 && minute < 60
}

/// Helper to check if text matches any pattern
pub(super) fn matches_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("test.user@domain.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("no@domain"));
    }

    #[test]
    fn test_valid_time() {
        assert!(is_valid_time("22:00"));
        assert!(is_valid_time("08:30"));
        assert!(is_valid_time("00:00"));
        assert!(!is_valid_time("25:00"));
        assert!(!is_valid_time("12:60"));
    }
}
