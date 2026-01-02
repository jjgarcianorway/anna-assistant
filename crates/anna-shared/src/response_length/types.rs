//! Response types and metadata structures.

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
