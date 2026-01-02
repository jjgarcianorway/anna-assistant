//! Question normalization and similarity calculation.

/// Normalize a question for comparison
pub fn normalize_question(question: &str) -> String {
    let lower = question.to_lowercase();

    // Remove common filler phrases (must be whole words/phrases)
    let fillers = [
        "please ",
        "can you ",
        "could you ",
        "would you ",
        "i want to ",
        "i need to ",
        "i'd like to ",
        "help me ",
        "show me ",
        "tell me ",
        "how do i ",
        "how can i ",
        "what is ",
        "what's ",
    ];

    let mut result = format!(" {} ", lower);
    for filler in fillers {
        result = result.replace(filler, " ");
    }

    // Collapse whitespace and trim
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Calculate similarity between two normalized questions (0.0 to 1.0)
pub fn calculate_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }

    let words_a: Vec<&str> = a.split_whitespace().collect();
    let words_b: Vec<&str> = b.split_whitespace().collect();

    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    // Count common words
    let common = words_a.iter().filter(|w| words_b.contains(w)).count();

    // Jaccard-like similarity
    let union = words_a.len() + words_b.len() - common;
    if union == 0 {
        return 0.0;
    }

    common as f32 / union as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_question() {
        assert_eq!(normalize_question("How do I install vim?"), "install vim?");
        assert_eq!(
            normalize_question("Please help me restart nginx"),
            "restart nginx"
        );
        assert_eq!(
            normalize_question("Can you show me disk usage"),
            "disk usage"
        );
    }

    #[test]
    fn test_calculate_similarity() {
        assert_eq!(calculate_similarity("install vim", "install vim"), 1.0);
        assert!(calculate_similarity("install vim", "install nano") > 0.3);
        assert!(calculate_similarity("install vim", "restart nginx") < 0.3);
    }
}
