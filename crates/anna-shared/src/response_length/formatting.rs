//! Formatting functions for response length statistics.

use super::tracker::ResponseLengthTracker;

/// Format response length stats for display
pub fn format_response_lengths(tracker: &ResponseLengthTracker) -> String {
    let mut output = String::new();

    output.push_str("Response Length Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    if tracker.total_responses == 0 {
        output.push_str("No responses recorded yet.\n");
        return output;
    }

    output.push_str(&format!(
        "Total Responses: {}\n",
        tracker.total_responses
    ));
    output.push_str(&format!(
        "Average Length:  {:.0} chars, {:.0} words\n\n",
        tracker.average_chars(),
        tracker.average_words()
    ));

    if let Some(longest) = &tracker.longest {
        output.push_str("Longest Response:\n");
        output.push_str(&format!(
            "  {} chars, {} words, {} lines\n",
            longest.char_count, longest.word_count, longest.line_count
        ));
        output.push_str(&format!("  \"{}\"\n\n", longest.excerpt));
    }

    if let Some(shortest) = &tracker.shortest {
        output.push_str("Shortest Response:\n");
        output.push_str(&format!(
            "  {} chars, {} words, {} lines\n",
            shortest.char_count, shortest.word_count, shortest.line_count
        ));
        output.push_str(&format!("  \"{}\"\n", shortest.excerpt));
    }

    output
}

/// Format compact response length info
pub fn format_response_lengths_compact(tracker: &ResponseLengthTracker) -> String {
    if tracker.total_responses == 0 {
        return "No responses yet".to_string();
    }

    let shortest = tracker.shortest.as_ref().map(|s| s.char_count).unwrap_or(0);
    let longest = tracker.longest.as_ref().map(|l| l.char_count).unwrap_or(0);

    format!(
        "{} responses, avg {:.0} chars ({}–{} range)",
        tracker.total_responses,
        tracker.average_chars(),
        shortest,
        longest
    )
}

/// Generate fun fact about response lengths
pub fn response_length_fun_fact(tracker: &ResponseLengthTracker) -> Option<String> {
    if tracker.total_responses < 5 {
        return None;
    }

    let facts = vec![
        format!(
            "Average response is {:.0} words - {} a tweet!",
            tracker.average_words(),
            if tracker.average_words() <= 50.0 {
                "shorter than"
            } else {
                "longer than"
            }
        ),
        format!(
            "Longest reply was {} characters - that's {} pages!",
            tracker.longest.as_ref().map(|l| l.char_count).unwrap_or(0),
            tracker.longest.as_ref().map(|l| l.char_count).unwrap_or(0) / 2000 + 1
        ),
        format!(
            "Shortest answer was just {} words - straight to the point!",
            tracker
                .shortest_words
                .as_ref()
                .map(|s| s.word_count)
                .unwrap_or(0)
        ),
        format!(
            "Total words written: {} - that's like {} short stories!",
            tracker.total_words,
            tracker.total_words / 7500 + 1
        ),
    ];

    // Pick based on some variety
    let index = (tracker.total_responses as usize) % facts.len();
    Some(facts[index].clone())
}
