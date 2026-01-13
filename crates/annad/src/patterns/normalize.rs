//! Query normalization for pattern matching.

/// Normalize query for better pattern matching.
/// Removes extra whitespace, punctuation, and common filler words.
pub fn normalize_query(q: &str) -> String {
    let mut result = q.to_lowercase();

    // Remove common punctuation
    result = result.replace(['?', '!', '.', ',', ':', ';', '"', '\''], " ");

    // Remove filler words that don't add meaning
    let fillers = [
        "please", "can you", "could you", "would you", "i want to",
        "i need to", "help me", "tell me", "show me how to",
        "how do i", "how can i", "what's the", "what is the",
    ];
    for filler in fillers {
        result = result.replace(filler, " ");
    }

    // Collapse multiple spaces into one
    let mut prev_space = false;
    result = result.chars().filter(|c| {
        if c.is_whitespace() {
            if prev_space { return false; }
            prev_space = true;
        } else {
            prev_space = false;
        }
        true
    }).collect();

    result.trim().to_string()
}
