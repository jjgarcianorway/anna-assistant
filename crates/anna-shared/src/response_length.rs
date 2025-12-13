//! Response Length Tracking (v0.0.486).
//!
//! Tracks response lengths for fun statistics.
//! Identifies longest and shortest replies.

use serde::{Deserialize, Serialize};

/// A recorded response with length metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedResponse {
    /// Unix timestamp
    pub timestamp: u64,
    /// Character count
    pub char_count: usize,
    /// Word count
    pub word_count: usize,
    /// Line count
    pub line_count: usize,
    /// Brief excerpt (first 50 chars)
    pub excerpt: String,
    /// Category/topic if known
    pub category: Option<String>,
    /// Was this a successful response
    pub successful: bool,
}

impl RecordedResponse {
    /// Create from response text
    pub fn from_text(text: &str, timestamp: u64) -> Self {
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let line_count = text.lines().count();
        let excerpt = if text.len() > 50 {
            format!("{}...", &text[..47])
        } else {
            text.to_string()
        };

        Self {
            timestamp,
            char_count,
            word_count,
            line_count,
            excerpt,
            category: None,
            successful: true,
        }
    }

    /// Set category
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Mark as unsuccessful
    pub fn mark_unsuccessful(mut self) -> Self {
        self.successful = false;
        self
    }
}

/// Response length statistics tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseLengthTracker {
    /// Total responses tracked
    pub total_responses: u64,
    /// Total characters across all responses
    pub total_chars: u64,
    /// Total words across all responses
    pub total_words: u64,
    /// Longest response
    pub longest: Option<RecordedResponse>,
    /// Shortest response (non-empty)
    pub shortest: Option<RecordedResponse>,
    /// Recent responses (last 10)
    pub recent: Vec<RecordedResponse>,
    /// Longest response by word count
    pub longest_words: Option<RecordedResponse>,
    /// Shortest response by word count
    pub shortest_words: Option<RecordedResponse>,
}

impl ResponseLengthTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a response
    pub fn record(&mut self, text: &str, timestamp: u64) {
        if text.is_empty() {
            return;
        }

        let response = RecordedResponse::from_text(text, timestamp);

        self.total_responses += 1;
        self.total_chars += response.char_count as u64;
        self.total_words += response.word_count as u64;

        // Update longest (by chars)
        if self
            .longest
            .as_ref()
            .map(|l| response.char_count > l.char_count)
            .unwrap_or(true)
        {
            self.longest = Some(response.clone());
        }

        // Update shortest (by chars)
        if self
            .shortest
            .as_ref()
            .map(|s| response.char_count < s.char_count)
            .unwrap_or(true)
        {
            self.shortest = Some(response.clone());
        }

        // Update longest by words
        if self
            .longest_words
            .as_ref()
            .map(|l| response.word_count > l.word_count)
            .unwrap_or(true)
        {
            self.longest_words = Some(response.clone());
        }

        // Update shortest by words
        if self
            .shortest_words
            .as_ref()
            .map(|s| response.word_count < s.word_count)
            .unwrap_or(true)
        {
            self.shortest_words = Some(response.clone());
        }

        // Add to recent, keep last 10
        self.recent.push(response);
        if self.recent.len() > 10 {
            self.recent.remove(0);
        }
    }

    /// Record with category
    pub fn record_with_category(&mut self, text: &str, timestamp: u64, category: &str) {
        if text.is_empty() {
            return;
        }

        let response = RecordedResponse::from_text(text, timestamp).with_category(category);

        self.total_responses += 1;
        self.total_chars += response.char_count as u64;
        self.total_words += response.word_count as u64;

        // Update records with category-aware response
        if self
            .longest
            .as_ref()
            .map(|l| response.char_count > l.char_count)
            .unwrap_or(true)
        {
            self.longest = Some(response.clone());
        }

        if self
            .shortest
            .as_ref()
            .map(|s| response.char_count < s.char_count)
            .unwrap_or(true)
        {
            self.shortest = Some(response.clone());
        }

        if self
            .longest_words
            .as_ref()
            .map(|l| response.word_count > l.word_count)
            .unwrap_or(true)
        {
            self.longest_words = Some(response.clone());
        }

        if self
            .shortest_words
            .as_ref()
            .map(|s| response.word_count < s.word_count)
            .unwrap_or(true)
        {
            self.shortest_words = Some(response.clone());
        }

        self.recent.push(response);
        if self.recent.len() > 10 {
            self.recent.remove(0);
        }
    }

    /// Get average response length (chars)
    pub fn average_chars(&self) -> f64 {
        if self.total_responses == 0 {
            0.0
        } else {
            self.total_chars as f64 / self.total_responses as f64
        }
    }

    /// Get average response length (words)
    pub fn average_words(&self) -> f64 {
        if self.total_responses == 0 {
            0.0
        } else {
            self.total_words as f64 / self.total_responses as f64
        }
    }

    /// Get response length range
    pub fn length_range(&self) -> Option<(usize, usize)> {
        match (&self.shortest, &self.longest) {
            (Some(s), Some(l)) => Some((s.char_count, l.char_count)),
            _ => None,
        }
    }
}

/// Response length summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseLengthSummary {
    /// Total responses
    pub total_responses: u64,
    /// Average chars per response
    pub avg_chars: f64,
    /// Average words per response
    pub avg_words: f64,
    /// Longest response chars
    pub longest_chars: usize,
    /// Shortest response chars
    pub shortest_chars: usize,
    /// Longest response excerpt
    pub longest_excerpt: String,
    /// Shortest response excerpt
    pub shortest_excerpt: String,
}

impl ResponseLengthTracker {
    /// Generate summary
    pub fn summary(&self) -> ResponseLengthSummary {
        ResponseLengthSummary {
            total_responses: self.total_responses,
            avg_chars: self.average_chars(),
            avg_words: self.average_words(),
            longest_chars: self.longest.as_ref().map(|l| l.char_count).unwrap_or(0),
            shortest_chars: self.shortest.as_ref().map(|s| s.char_count).unwrap_or(0),
            longest_excerpt: self
                .longest
                .as_ref()
                .map(|l| l.excerpt.clone())
                .unwrap_or_default(),
            shortest_excerpt: self
                .shortest
                .as_ref()
                .map(|s| s.excerpt.clone())
                .unwrap_or_default(),
        }
    }
}

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

/// Check if query is asking about response lengths
pub fn is_response_length_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "response length",
        "reply length",
        "longest reply",
        "shortest reply",
        "longest response",
        "shortest response",
        "average response",
        "how long",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recorded_response_from_text() {
        let response = RecordedResponse::from_text("Hello world", 1000);
        assert_eq!(response.char_count, 11);
        assert_eq!(response.word_count, 2);
        assert_eq!(response.line_count, 1);
    }

    #[test]
    fn test_excerpt_truncation() {
        let long_text = "This is a very long response that should be truncated to fifty characters plus ellipsis";
        let response = RecordedResponse::from_text(long_text, 1000);
        assert!(response.excerpt.len() <= 50);
        assert!(response.excerpt.ends_with("..."));
    }

    #[test]
    fn test_tracker_record() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("Short", 1000);
        tracker.record("This is a longer response with more words", 2000);
        tracker.record("Medium length", 3000);

        assert_eq!(tracker.total_responses, 3);
        assert!(tracker.longest.is_some());
        assert!(tracker.shortest.is_some());
    }

    #[test]
    fn test_longest_shortest() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("Short", 1000);
        tracker.record("This is significantly longer than the other one", 2000);

        assert_eq!(tracker.shortest.as_ref().unwrap().char_count, 5);
        assert_eq!(tracker.longest.as_ref().unwrap().char_count, 47);
    }

    #[test]
    fn test_averages() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("One two three", 1000); // 13 chars, 3 words
        tracker.record("Four five six seven", 2000); // 19 chars, 4 words

        assert_eq!(tracker.average_chars(), 16.0);
        assert_eq!(tracker.average_words(), 3.5);
    }

    #[test]
    fn test_empty_response_ignored() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("", 1000);
        tracker.record("Valid", 2000);

        assert_eq!(tracker.total_responses, 1);
    }

    #[test]
    fn test_recent_limit() {
        let mut tracker = ResponseLengthTracker::new();

        for i in 0..15 {
            tracker.record(&format!("Response {}", i), i as u64 * 1000);
        }

        assert_eq!(tracker.recent.len(), 10);
    }

    #[test]
    fn test_summary() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("Short", 1000);
        tracker.record("Much longer response here", 2000);

        let summary = tracker.summary();
        assert_eq!(summary.total_responses, 2);
        assert!(summary.longest_chars > summary.shortest_chars);
    }

    #[test]
    fn test_format_compact() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("Hello", 1000);
        tracker.record("World", 2000);

        let output = format_response_lengths_compact(&tracker);
        assert!(output.contains("2 responses"));
    }

    #[test]
    fn test_response_length_fun_fact() {
        let mut tracker = ResponseLengthTracker::new();

        for i in 0..10 {
            tracker.record(&format!("Response number {} with some content", i), i as u64 * 1000);
        }

        let fact = response_length_fun_fact(&tracker);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_response_length_query() {
        assert!(is_response_length_query("what was the longest reply"));
        assert!(is_response_length_query("show response length stats"));
        assert!(is_response_length_query("shortest response"));

        assert!(!is_response_length_query("install vim"));
        assert!(!is_response_length_query("status"));
    }

    #[test]
    fn test_with_category() {
        let response = RecordedResponse::from_text("Test", 1000).with_category("system");
        assert_eq!(response.category, Some("system".to_string()));
    }

    #[test]
    fn test_record_with_category() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record_with_category("System response", 1000, "system");

        assert!(tracker.longest.as_ref().unwrap().category.is_some());
    }

    #[test]
    fn test_length_range() {
        let mut tracker = ResponseLengthTracker::new();

        tracker.record("Short", 1000);
        tracker.record("Much much longer", 2000);

        let range = tracker.length_range();
        assert!(range.is_some());
        let (min, max) = range.unwrap();
        assert!(min < max);
    }
}
