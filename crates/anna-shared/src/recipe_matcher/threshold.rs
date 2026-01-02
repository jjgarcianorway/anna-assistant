//! Dynamic threshold calculation based on recipe maturity and reliability.

use crate::recipe::Recipe;

use super::types::BASE_MATCH_THRESHOLD;

/// v0.0.373: Calculate dynamic match threshold based on recipe maturity
/// Immature recipes need higher scores to match (prevents wrong answers)
/// Mature, high-reliability recipes can match with lower scores
pub fn dynamic_threshold(recipe: &Recipe) -> u32 {
    let maturity_factor = match recipe.success_count {
        0 => 25,     // Untested: need very high score
        1..=2 => 15, // New: need higher score
        3..=5 => 10, // Young: slightly elevated
        6..=10 => 5, // Maturing: slight boost
        _ => 0,      // Mature: base threshold
    };

    let reliability_factor = match recipe.reliability_score {
        90..=100 => 0, // Excellent: no penalty
        80..=89 => 5,  // Good: small boost needed
        70..=79 => 10, // Okay: moderate boost
        _ => 15,       // Low: need much higher match
    };

    // Higher threshold = harder to match = fewer wrong answers
    (BASE_MATCH_THRESHOLD + maturity_factor + reliability_factor).min(95)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::RecipeSignature;
    use crate::teams::Team;
    use crate::ticket::RiskLevel;

    #[test]
    fn test_dynamic_threshold_maturity() {
        let sig = RecipeSignature {
            domain: "test".to_string(),
            intent: "test".to_string(),
            route_class: "test".to_string(),
            query_pattern: "test query".to_string(),
        };

        // Create test recipes with different maturity levels
        let mut new_recipe = Recipe::new(
            sig.clone(),
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test".to_string(),
            85,
        );
        new_recipe.success_count = 0;

        let mut young_recipe = Recipe::new(
            sig.clone(),
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test".to_string(),
            85,
        );
        young_recipe.success_count = 2;

        let mut mature_recipe = Recipe::new(
            sig.clone(),
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test".to_string(),
            95,
        );
        mature_recipe.success_count = 20;

        // New recipes should require higher match scores
        let new_threshold = dynamic_threshold(&new_recipe);
        let young_threshold = dynamic_threshold(&young_recipe);
        let mature_threshold = dynamic_threshold(&mature_recipe);

        assert!(
            new_threshold > young_threshold,
            "new={} should > young={}",
            new_threshold,
            young_threshold
        );
        assert!(
            young_threshold > mature_threshold,
            "young={} should > mature={}",
            young_threshold,
            mature_threshold
        );
    }

    #[test]
    fn test_dynamic_threshold_reliability() {
        let sig = RecipeSignature {
            domain: "test".to_string(),
            intent: "test".to_string(),
            route_class: "test".to_string(),
            query_pattern: "test query".to_string(),
        };

        // Same maturity, different reliability
        let mut high_reliability = Recipe::new(
            sig.clone(),
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test".to_string(),
            95,
        );
        high_reliability.success_count = 10;

        let mut low_reliability = Recipe::new(
            sig.clone(),
            Team::General,
            RiskLevel::ReadOnly,
            vec![],
            vec![],
            "test".to_string(),
            65,
        );
        low_reliability.success_count = 10;

        let high_threshold = dynamic_threshold(&high_reliability);
        let low_threshold = dynamic_threshold(&low_reliability);

        assert!(
            low_threshold > high_threshold,
            "low reliability should need higher match score"
        );
    }
}
