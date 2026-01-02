//! Response length statistics tracker.

use serde::{Deserialize, Serialize};

use super::types::{RecordedResponse, ResponseLengthSummary};

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
