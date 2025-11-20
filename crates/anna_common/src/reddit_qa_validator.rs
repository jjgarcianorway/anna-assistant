//! Reddit QA Validator - Beta.78
//!
//! Real-world validation using r/archlinux questions:
//! - Fetch 500-1000 actual user questions
//! - Run through Anna's LLM
//! - Compare against most-voted community answers
//! - Measure helpfulness and accuracy
//!
//! This validates Anna against REAL problems, not just synthetic tests.

use serde::{Deserialize, Serialize};

/// A Reddit post from r/archlinux
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditQuestion {
    /// Post ID
    pub id: String,
    /// Post title
    pub title: String,
    /// Post body (selftext)
    pub body: String,
    /// Number of upvotes
    pub score: i32,
    /// Number of comments
    pub num_comments: i32,
    /// Top-voted answer (if available)
    pub top_answer: Option<String>,
    /// Top answer score
    pub top_answer_score: Option<i32>,
    /// Post URL
    pub url: String,
}

/// Anna's response to a question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaResponse {
    /// Question ID
    pub question_id: String,
    /// Anna's answer
    pub answer: String,
    /// Response time (ms)
    pub response_time_ms: u64,
    /// Whether Anna provided actionable advice
    pub has_actionable_advice: bool,
    /// Commands suggested (if any)
    pub suggested_commands: Vec<String>,
}

/// Validation result for a single question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Question being validated
    pub question: RedditQuestion,
    /// Anna's response
    pub anna_response: AnnaResponse,
    /// Similarity score (0.0-1.0) between Anna and top answer
    pub similarity_score: f64,
    /// Manual validation (if performed)
    pub manual_validation: Option<ManualValidation>,
    /// Validation passed
    pub passed: bool,
}

/// Manual validation by reviewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualValidation {
    /// Is Anna's answer helpful? (1-5)
    pub helpfulness: u8,
    /// Is Anna's answer accurate? (1-5)
    pub accuracy: u8,
    /// Is Anna's answer complete? (1-5)
    pub completeness: u8,
    /// Would you follow Anna's advice? (yes/no)
    pub would_follow: bool,
    /// Notes
    pub notes: String,
}

/// Validation suite results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSuite {
    /// Total questions tested
    pub total_questions: usize,
    /// Questions where Anna provided helpful answers
    pub helpful_count: usize,
    /// Questions where Anna matched community answer
    pub matched_community: usize,
    /// Average similarity score
    pub avg_similarity: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Pass rate (0.0-1.0)
    pub pass_rate: f64,
    /// Individual results
    pub results: Vec<ValidationResult>,
}

impl ValidationSuite {
    /// Generate validation report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# Reddit QA Validation Report\n\n");
        report.push_str(&format!("**Total Questions:** {}\n", self.total_questions));
        report.push_str(&format!("**Helpful Answers:** {} ({:.1}%)\n",
            self.helpful_count,
            (self.helpful_count as f64 / self.total_questions as f64) * 100.0
        ));
        report.push_str(&format!("**Community Match:** {} ({:.1}%)\n",
            self.matched_community,
            (self.matched_community as f64 / self.total_questions as f64) * 100.0
        ));
        report.push_str(&format!("**Avg Similarity:** {:.2}\n", self.avg_similarity));
        report.push_str(&format!("**Avg Response Time:** {:.0}ms\n", self.avg_response_time_ms));
        report.push_str(&format!("**Pass Rate:** {:.1}%\n\n", self.pass_rate * 100.0));

        report.push_str("## Sample Comparisons\n\n");

        // Show top 5 best matches
        let mut sorted_results = self.results.clone();
        sorted_results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

        report.push_str("### ✅ Best Matches (Anna ≈ Community)\n\n");
        for result in sorted_results.iter().take(5) {
            report.push_str(&format!("**Q:** {}\n", result.question.title));
            report.push_str(&format!("**Similarity:** {:.0}%\n", result.similarity_score * 100.0));
            report.push_str(&format!("**Anna:** {}...\n\n",
                result.anna_response.answer.chars().take(200).collect::<String>()));
        }

        // Show 5 worst matches (areas for improvement)
        report.push_str("### ⚠ Areas for Improvement\n\n");
        sorted_results.reverse();
        for result in sorted_results.iter().take(5) {
            report.push_str(&format!("**Q:** {}\n", result.question.title));
            report.push_str(&format!("**Similarity:** {:.0}%\n", result.similarity_score * 100.0));
            report.push_str(&format!("**Issue:** Anna's answer diverged from community consensus\n\n"));
        }

        report
    }
}

/// Reddit API client (simplified)
pub struct RedditClient {
    /// User agent for API requests
    user_agent: String,
}

impl RedditClient {
    /// Create new Reddit client
    pub fn new() -> Self {
        Self {
            user_agent: "Anna Assistant QA Validator 1.0".to_string(),
        }
    }

    /// Fetch questions from r/archlinux
    ///
    /// Note: Reddit API requires OAuth for more than 100 requests/min
    /// For 500-1000 questions, we'd need to implement proper authentication
    /// or scrape gradually over time.
    pub async fn fetch_questions(&self, _limit: usize) -> anyhow::Result<Vec<RedditQuestion>> {
        // TODO: Implement actual Reddit API fetching
        // For now, return error indicating manual collection needed
        anyhow::bail!(
            "Reddit API fetching not yet implemented.\n\
             Please manually collect questions using:\n\
             - Reddit JSON API: https://www.reddit.com/r/archlinux/top.json?t=month&limit=100\n\
             - Or use a Reddit scraper tool\n\
             - Save to: reddit_questions.json"
        )
    }

    /// Load questions from JSON file (alternative to API)
    pub fn load_from_file(&self, path: &str) -> anyhow::Result<Vec<RedditQuestion>> {
        let contents = std::fs::read_to_string(path)?;
        let questions: Vec<RedditQuestion> = serde_json::from_str(&contents)?;
        Ok(questions)
    }
}

impl Default for RedditClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_suite_report() {
        let suite = ValidationSuite {
            total_questions: 100,
            helpful_count: 85,
            matched_community: 70,
            avg_similarity: 0.75,
            avg_response_time_ms: 1500.0,
            pass_rate: 0.85,
            results: vec![],
        };

        let report = suite.generate_report();
        assert!(report.contains("85"));
        assert!(report.contains("70"));
        assert!(report.contains("0.75"));
    }
}
