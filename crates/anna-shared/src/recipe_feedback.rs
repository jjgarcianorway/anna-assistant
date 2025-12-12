//! Recipe feedback system (v0.0.371).
//! Anna can ask for feedback when she's uncertain about a recipe answer.
//! Feedback adjusts recipe reliability scores for future matches.
//!
//! v0.0.295: "Not helpful" feedback now adds query to recipe's negative_match_patterns,
//! preventing the same query from matching this recipe via semantic similarity in the future.
//! v0.0.371: Adaptive feedback scoring - adjustments vary based on recipe maturity.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::recipe::Recipe;

/// v0.0.103: Request for user feedback on a recipe answer
/// Anna asks this when she's uncertain about her answer quality
/// v0.0.305: Added original_query for negative feedback learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRequest {
    /// The recipe ID that produced this answer
    pub recipe_id: String,
    /// Why Anna is asking for feedback (e.g., "borderline confidence")
    pub reason: String,
    /// The question to ask the user
    pub question: String,
    /// The original user query that triggered this recipe
    /// v0.0.305: Added for negative feedback learning
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_query: Option<String>,
}

impl FeedbackRequest {
    /// Create feedback request for borderline confidence
    /// v0.0.305: Now takes query for negative feedback learning
    pub fn borderline_confidence(recipe_id: &str, score: u8, query: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            reason: format!("confidence_score_{}", score),
            question: "Was this answer helpful? (y/n)".to_string(),
            original_query: Some(query.to_string()),
        }
    }

    /// Create feedback request when recipe is new/untested
    /// v0.0.305: Now takes query for negative feedback learning
    pub fn new_recipe(recipe_id: &str, query: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            reason: "new_recipe".to_string(),
            question: "This is from a newly learned pattern. Was it helpful? (y/n)".to_string(),
            original_query: Some(query.to_string()),
        }
    }

    /// Add method for setting query (builder pattern)
    pub fn with_query(mut self, query: &str) -> Self {
        self.original_query = Some(query.to_string());
        self
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

/// v0.0.371: Calculate adaptive score adjustment based on recipe maturity
/// New recipes get larger adjustments to learn faster
/// Mature recipes get smaller adjustments for stability
fn adaptive_adjustment(success_count: u32, base_amount: u8) -> u8 {
    match success_count {
        0..=2 => base_amount * 3,  // New: 3x adjustment (learn fast)
        3..=10 => base_amount * 2, // Young: 2x adjustment
        11..=30 => base_amount,    // Maturing: normal adjustment
        _ => base_amount.saturating_sub(1).max(1), // Mature: slightly reduced
    }
}

/// Apply feedback to a recipe, updating its scores
/// v0.0.371: Uses adaptive scoring based on recipe maturity
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
            // v0.0.371: Adaptive boost - new recipes learn faster
            let boost = adaptive_adjustment(previous_success_count, 2);
            if recipe.reliability_score < 99 {
                recipe.reliability_score = (recipe.reliability_score + boost).min(99);
            }
        }
        FeedbackRating::NotHelpful => {
            // v0.0.371: Adaptive penalty - new recipes get penalized more
            let penalty = adaptive_adjustment(previous_success_count, 5);
            if recipe.reliability_score > 50 {
                recipe.reliability_score = recipe.reliability_score.saturating_sub(penalty);
                // Floor at 50 to avoid complete discard
                recipe.reliability_score = recipe.reliability_score.max(50);
            }
            // v0.0.295: Add query to negative match patterns
            // This prevents this query from matching this recipe via semantic similarity
            if let Some(ref query) = feedback.query {
                recipe.add_negative_match(query);
            }
        }
        FeedbackRating::Partial => {
            // Slight increase in success count
            recipe.success_count = recipe.success_count.saturating_add(1);
            // v0.0.371: Small boost for partial feedback
            let boost = adaptive_adjustment(previous_success_count, 1);
            if recipe.reliability_score < 95 {
                recipe.reliability_score = (recipe.reliability_score + boost).min(95);
            }
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

    #[test]
    fn test_adaptive_adjustment() {
        // New recipes (0-2) get 3x adjustment
        assert_eq!(adaptive_adjustment(0, 2), 6);
        assert_eq!(adaptive_adjustment(2, 5), 15);

        // Young recipes (3-10) get 2x adjustment
        assert_eq!(adaptive_adjustment(5, 2), 4);
        assert_eq!(adaptive_adjustment(10, 5), 10);

        // Maturing recipes (11-30) get normal adjustment
        assert_eq!(adaptive_adjustment(15, 2), 2);
        assert_eq!(adaptive_adjustment(30, 5), 5);

        // Mature recipes (31+) get reduced adjustment
        assert_eq!(adaptive_adjustment(50, 2), 1);
        assert_eq!(adaptive_adjustment(100, 5), 4);
    }
}
