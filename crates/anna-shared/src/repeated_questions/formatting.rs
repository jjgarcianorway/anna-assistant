//! Formatting and display functions for repeated questions.

use crate::repeated_questions::types::QuestionHistory;

/// Format repeated questions for display
pub fn format_repeated_questions(history: &QuestionHistory) -> String {
    let mut output = String::new();

    output.push_str("Repeated Questions\n");
    output.push_str("══════════════════════════════════════\n\n");

    let repeated = history.top_repeated(10);

    if repeated.is_empty() {
        output.push_str("No repeated questions detected yet.\n");
        return output;
    }

    for (i, q) in repeated.iter().enumerate() {
        let status = if q.resolved { "[OK]" } else { "[?]" };
        let category = q.category.as_deref().unwrap_or("general");

        output.push_str(&format!(
            "{}. {} ({}) - {} times\n",
            i + 1,
            q.variants.first().unwrap_or(&q.normalized),
            category,
            q.count
        ));
        output.push_str(&format!("   {} Status: {}\n", status, if q.resolved { "Resolved" } else { "Pending" }));

        if q.variants.len() > 1 {
            output.push_str("   Variants:\n");
            for variant in q.variants.iter().skip(1).take(3) {
                output.push_str(&format!("   - {}\n", variant));
            }
        }
        output.push('\n');
    }

    let summary = history.summary();
    output.push_str(&format!(
        "Summary: {} unique questions, {} repeated ({} unresolved)\n",
        summary.total_unique, summary.repeated_count, summary.unresolved_count
    ));

    output
}

/// Format compact repeated questions
pub fn format_repeated_compact(history: &QuestionHistory) -> String {
    let repeated = history.top_repeated(5);

    if repeated.is_empty() {
        return "No repeated questions".to_string();
    }

    let items: Vec<String> = repeated
        .iter()
        .map(|q| {
            let short = q
                .variants
                .first()
                .map(|s| if s.len() > 30 { format!("{}...", &s[..27]) } else { s.clone() })
                .unwrap_or_default();
            format!("\"{}\" ({}x)", short, q.count)
        })
        .collect();

    items.join(", ")
}

/// Check if query is asking about repeated questions
pub fn is_repeated_questions_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "repeated questions",
        "repeat questions",
        "common questions",
        "frequent questions",
        "asked questions",
        "what do i ask",
        "same questions",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_repeated_compact() {
        let mut history = QuestionHistory::new();

        history.record("install vim", 1000);
        history.record("install vim", 2000);
        history.record("install vim", 3000);

        let output = format_repeated_compact(&history);
        assert!(output.contains("3x"));
    }

    #[test]
    fn test_is_repeated_questions_query() {
        assert!(is_repeated_questions_query("show repeated questions"));
        assert!(is_repeated_questions_query("what are my common questions"));
        assert!(is_repeated_questions_query("frequent questions"));

        assert!(!is_repeated_questions_query("install vim"));
        assert!(!is_repeated_questions_query("status"));
    }
}
