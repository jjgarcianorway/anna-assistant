//! Recipe feedback system (v0.0.103).
//! Anna can ask for feedback when she's uncertain about a recipe answer.
//! Feedback adjusts recipe reliability scores for future matches.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::recipe::Recipe;

/// v0.0.103: Request for user feedback on a recipe answer
/// Anna asks this when she's uncertain about her answer quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRequest {
    /// The recipe ID that produced this answer
    pub recipe_id: String,
    /// Why Anna is asking for feedback (e.g., "borderline confidence")
    pub reason: String,
    /// The question to ask the user
    pub question: String,
}

impl FeedbackRequest {
    /// Create feedback request for borderline confidence
    pub fn borderline_confidence(recipe_id: &str, score: u8) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            reason: format!("confidence_score_{}", score),
            question: "Was this answer helpful? (y/n)".to_string(),
        }
    }

    /// Create feedback request when recipe is new/untested
    pub fn new_recipe(recipe_id: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            reason: "new_recipe".to_string(),
            question: "This is from a newly learned pattern. Was it helpful? (y/n)".to_string(),
        }
    }
}

/// Feedback rating for a recipe answer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackRating {
    /// Answer was helpful and correct
    Helpful,
    /// Answer was not helpful or incorrect
    NotHelpful,
    /// Answer was partially helpful
    Partial,
}

/// Feedback submission for a recipe answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeFeedback {
    /// The recipe ID that was used
    pub recipe_id: String,
    /// User's rating
    pub rating: FeedbackRating,
    /// Optional comment from user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Timestamp of feedback
    pub timestamp: u64,
    /// The original query that triggered the recipe
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

impl RecipeFeedback {
    pub fn new(recipe_id: impl Into<String>, rating: FeedbackRating) -> Self {
        Self {
            recipe_id: recipe_id.into(),
            rating,
            comment: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            query: None,
        }
    }

    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }
}

/// Result of applying feedback to a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackResult {
    pub recipe_id: String,
    pub previous_score: u8,
    pub new_score: u8,
    pub previous_success_count: u32,
    pub new_success_count: u32,
    pub applied: bool,
    pub message: String,
}

/// Apply feedback to a recipe, updating its scores
pub fn apply_feedback(feedback: &RecipeFeedback) -> Option<FeedbackResult> {
    let recipe_path = recipe_path(&feedback.recipe_id);

    // Load existing recipe
    let content = fs::read_to_string(&recipe_path).ok()?;
    let mut recipe: Recipe = serde_json::from_str(&content).ok()?;

    let previous_score = recipe.reliability_score;
    let previous_success_count = recipe.success_count;

    // Apply feedback based on rating
    match feedback.rating {
        FeedbackRating::Helpful => {
            // Increase success count
            recipe.success_count = recipe.success_count.saturating_add(1);
            // Boost reliability score slightly (max 99)
            if recipe.reliability_score < 99 {
                recipe.reliability_score = (recipe.reliability_score + 1).min(99);
            }
        }
        FeedbackRating::NotHelpful => {
            // Decrease reliability score (min 50 to avoid complete discard)
            if recipe.reliability_score > 50 {
                recipe.reliability_score = recipe.reliability_score.saturating_sub(5);
            }
        }
        FeedbackRating::Partial => {
            // Slight increase in success count, no score change
            recipe.success_count = recipe.success_count.saturating_add(1);
        }
    }

    // Save updated recipe
    let updated_content = serde_json::to_string_pretty(&recipe).ok()?;
    fs::write(&recipe_path, updated_content).ok()?;

    Some(FeedbackResult {
        recipe_id: feedback.recipe_id.clone(),
        previous_score,
        new_score: recipe.reliability_score,
        previous_success_count,
        new_success_count: recipe.success_count,
        applied: true,
        message: format!(
            "Feedback applied: {} (score {} → {})",
            match feedback.rating {
                FeedbackRating::Helpful => "helpful",
                FeedbackRating::NotHelpful => "not helpful",
                FeedbackRating::Partial => "partial",
            },
            previous_score,
            recipe.reliability_score
        ),
    })
}

/// Get path to recipe file
fn recipe_path(recipe_id: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".anna")
        .join("recipes")
        .join(format!("{}.json", recipe_id))
}

/// Log feedback to feedback history (append-only)
pub fn log_feedback(feedback: &RecipeFeedback) {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let feedback_log = PathBuf::from(home)
        .join(".anna")
        .join("feedback_history.jsonl");

    if let Ok(line) = serde_json::to_string(feedback) {
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&feedback_log)
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "{}", line)
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_creation() {
        let feedback = RecipeFeedback::new("test-recipe", FeedbackRating::Helpful)
            .with_comment("Great answer!")
            .with_query("how do I enable syntax highlighting");

        assert_eq!(feedback.recipe_id, "test-recipe");
        assert_eq!(feedback.rating, FeedbackRating::Helpful);
        assert!(feedback.comment.is_some());
        assert!(feedback.query.is_some());
    }

    #[test]
    fn test_feedback_rating_serde() {
        let helpful = serde_json::to_string(&FeedbackRating::Helpful).unwrap();
        assert_eq!(helpful, "\"helpful\"");

        let not_helpful = serde_json::to_string(&FeedbackRating::NotHelpful).unwrap();
        assert_eq!(not_helpful, "\"not_helpful\"");
    }
}
