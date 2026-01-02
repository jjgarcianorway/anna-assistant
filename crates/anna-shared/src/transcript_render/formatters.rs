//! High-level formatters for answers and errors with context.

use crate::ui::colors;

/// Format an answer with evidence footer
pub fn format_answer_with_evidence(
    headline: &str,
    body: &str,
    evidence: &[&str],
    quick_actions: Option<&[&str]>,
) -> String {
    let mut answer = String::new();

    // Headline
    answer.push_str(headline);
    answer.push_str("\n\n");

    // Body
    answer.push_str(body);

    // Quick actions
    if let Some(actions) = quick_actions {
        if !actions.is_empty() {
            answer.push_str("\n\nQuick actions:\n");
            for (i, action) in actions.iter().enumerate() {
                answer.push_str(&format!("  {}) {}\n", i + 1, action));
            }
        }
    }

    // Evidence footer
    if !evidence.is_empty() {
        answer.push_str(&format!(
            "\n{}Evidence: {}{}",
            colors::DIM,
            evidence.join(", "),
            colors::RESET
        ));
    }

    answer
}

/// Format error with collected evidence
pub fn format_error_with_context(
    error_headline: &str,
    collected_data: &[&str],
    ticket_info: Option<(&str, &str)>,
    fallback_message: Option<&str>,
) -> String {
    let mut error = String::new();

    error.push_str(error_headline);
    error.push('\n');

    if !collected_data.is_empty() {
        error.push_str("\nWhat I collected:\n");
        for data in collected_data {
            error.push_str(&format!("  - {}\n", data));
        }
    }

    if let Some((ticket_id, domain)) = ticket_info {
        error.push_str(&format!("\nTicket: {} ({})\n", ticket_id, domain));
    }

    if let Some(fallback) = fallback_message {
        error.push_str(&format!("\n{}\n", fallback));
    }

    error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_answer_with_evidence() {
        let answer = format_answer_with_evidence(
            "Memory available: 17.0 GiB",
            "54% of 31.0 GiB total",
            &["/proc/meminfo"],
            None,
        );

        assert!(answer.contains("17.0 GiB"));
        assert!(answer.contains("Evidence:"));
        assert!(answer.contains("/proc/meminfo"));
    }
}
