//! XP Calculator v0.42.0
//!
//! Calculates experience points earned from answered questions based on:
//! - Reliability score (primary factor)
//! - Complexity indicators (skills used, iterations, decomposition)
//!
//! ## XP Earning Rules
//!
//! | Reliability (R) | XP Earned    | XP Penalty        |
//! |-----------------|--------------|-------------------|
//! | R < 0.40        | 0 XP         | (0.40-R) * 100 XP |
//! | 0.40 <= R < 0.70| Small (5-15) | 0                 |
//! | 0.70 <= R < 0.90| Medium (20-40)| 0                |
//! | R >= 0.90       | High (50-100)| 0                 |
//!
//! Complexity bonuses:
//! - +5 XP per skill used (max +20)
//! - +2 XP for multi-step decomposition
//! - -5 XP penalty for high iterations with low reliability
//!
//! v0.42.0: Added XP penalties for R < 0.40 (pain-driven learning)

use serde::{Deserialize, Serialize};

/// XP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpConfig {
    /// Minimum reliability for any XP gain
    pub min_reliability: f64,
    /// Reliability threshold for "medium" XP
    pub medium_threshold: f64,
    /// Reliability threshold for "high" XP
    pub high_threshold: f64,
    /// Base XP for low reliability answers
    pub base_xp_low: u64,
    /// Base XP for medium reliability answers
    pub base_xp_medium: u64,
    /// Base XP for high reliability answers
    pub base_xp_high: u64,
    /// Bonus XP per skill used
    pub skill_bonus: u64,
    /// Maximum skill bonus
    pub max_skill_bonus: u64,
    /// Bonus for multi-step decomposition
    pub decomposition_bonus: u64,
    /// Penalty for high iterations with low reliability
    pub iteration_penalty: u64,
    /// Iteration threshold for penalty
    pub iteration_penalty_threshold: u32,
    /// v0.42.0: Penalty multiplier for R < min_reliability
    /// Penalty = (min_reliability - R) * penalty_multiplier
    pub penalty_multiplier: f64,
}

impl Default for XpConfig {
    fn default() -> Self {
        Self {
            min_reliability: 0.40,
            medium_threshold: 0.70,
            high_threshold: 0.90,
            base_xp_low: 10,
            base_xp_medium: 30,
            base_xp_high: 75,
            skill_bonus: 5,
            max_skill_bonus: 20,
            decomposition_bonus: 10,
            iteration_penalty: 5,
            iteration_penalty_threshold: 3,
            penalty_multiplier: 100.0, // (0.40-R) * 100 = max 40 XP penalty
        }
    }
}

/// Default XP configuration
pub const XP_CONFIG: XpConfig = XpConfig {
    min_reliability: 0.40,
    medium_threshold: 0.70,
    high_threshold: 0.90,
    base_xp_low: 10,
    base_xp_medium: 30,
    base_xp_high: 75,
    skill_bonus: 5,
    max_skill_bonus: 20,
    decomposition_bonus: 10,
    iteration_penalty: 5,
    iteration_penalty_threshold: 3,
    penalty_multiplier: 100.0,
};

/// Input for XP calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpInput {
    /// Final reliability score (0.0 - 1.0)
    pub reliability: f64,
    /// Number of skills used
    pub skills_used: u32,
    /// Number of Junior/Senior iterations
    pub iterations: u32,
    /// Whether question was decomposed into sub-problems
    pub was_decomposed: bool,
    /// Whether the answer was successful
    pub answer_success: bool,
}

impl XpInput {
    /// Create input from answer metrics
    pub fn new(
        reliability: f64,
        skills_used: u32,
        iterations: u32,
        was_decomposed: bool,
        answer_success: bool,
    ) -> Self {
        Self {
            reliability: reliability.clamp(0.0, 1.0),
            skills_used,
            iterations,
            was_decomposed,
            answer_success,
        }
    }
}

/// Result of XP calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpGain {
    /// Total XP earned
    pub total: u64,
    /// Base XP from reliability
    pub base_xp: u64,
    /// Bonus from skills
    pub skill_bonus: u64,
    /// Bonus from decomposition
    pub decomposition_bonus: u64,
    /// Penalty from iterations
    pub iteration_penalty: u64,
    /// Reliability tier used
    pub reliability_tier: ReliabilityTier,
    /// v0.42.0: XP penalty for low reliability (R < 0.40)
    #[serde(default)]
    pub xp_penalty: Option<XpPenalty>,
}

impl XpGain {
    /// No XP earned
    pub fn zero(reason: &str) -> Self {
        let _ = reason; // Log this later
        Self {
            total: 0,
            base_xp: 0,
            skill_bonus: 0,
            decomposition_bonus: 0,
            iteration_penalty: 0,
            reliability_tier: ReliabilityTier::Failed,
            xp_penalty: None,
        }
    }

    /// No XP earned with penalty
    pub fn with_penalty(penalty: XpPenalty) -> Self {
        Self {
            total: 0,
            base_xp: 0,
            skill_bonus: 0,
            decomposition_bonus: 0,
            iteration_penalty: 0,
            reliability_tier: ReliabilityTier::Failed,
            xp_penalty: Some(penalty),
        }
    }
}

/// v0.42.0: XP Penalty for low reliability answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpPenalty {
    /// Amount of XP to subtract from total
    pub amount: u64,
    /// The reliability that caused the penalty
    pub reliability: f64,
    /// Description of penalty reason
    pub reason: String,
}

impl XpPenalty {
    /// Calculate penalty for reliability below threshold
    /// Penalty = (threshold - reliability) * penalty_multiplier
    pub fn from_reliability(reliability: f64, threshold: f64, multiplier: f64) -> Self {
        let gap = (threshold - reliability).max(0.0);
        let amount = (gap * multiplier) as u64;
        Self {
            amount,
            reliability,
            reason: format!("R={:.2} below {:.2} threshold", reliability, threshold),
        }
    }
}

/// Reliability tier for XP calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReliabilityTier {
    /// R < 0.40 - No XP
    Failed,
    /// 0.40 <= R < 0.70 - Small XP
    Low,
    /// 0.70 <= R < 0.90 - Medium XP
    Medium,
    /// R >= 0.90 - High XP
    High,
}

impl ReliabilityTier {
    /// Determine tier from reliability score
    pub fn from_reliability(r: f64, config: &XpConfig) -> Self {
        if r < config.min_reliability {
            Self::Failed
        } else if r < config.medium_threshold {
            Self::Low
        } else if r < config.high_threshold {
            Self::Medium
        } else {
            Self::High
        }
    }

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Failed => "Failed",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }
}

/// XP Calculator
#[derive(Debug, Clone)]
pub struct XpCalculator {
    config: XpConfig,
}

impl XpCalculator {
    /// Create calculator with default config
    pub fn new() -> Self {
        Self {
            config: XpConfig::default(),
        }
    }

    /// Create calculator with custom config
    pub fn with_config(config: XpConfig) -> Self {
        Self { config }
    }

    /// Calculate XP gain from answer metrics
    pub fn calculate(&self, input: &XpInput) -> XpGain {
        // No XP for failed answers
        if !input.answer_success {
            return XpGain::zero("answer_failed");
        }

        // Determine reliability tier
        let tier = ReliabilityTier::from_reliability(input.reliability, &self.config);

        // v0.42.0: Apply XP penalty for very low reliability (R < 0.40)
        if tier == ReliabilityTier::Failed {
            let penalty = XpPenalty::from_reliability(
                input.reliability,
                self.config.min_reliability,
                self.config.penalty_multiplier,
            );
            return XpGain::with_penalty(penalty);
        }

        // Calculate base XP from reliability tier
        let base_xp = match tier {
            ReliabilityTier::Failed => 0,
            ReliabilityTier::Low => self.config.base_xp_low,
            ReliabilityTier::Medium => self.config.base_xp_medium,
            ReliabilityTier::High => self.config.base_xp_high,
        };

        // Scale base XP by actual reliability within tier
        // Higher reliability within tier = more XP
        let reliability_scale = self.reliability_scale(input.reliability, tier);
        let scaled_base = (base_xp as f64 * reliability_scale) as u64;

        // Calculate skill bonus
        let skill_bonus = (input.skills_used as u64 * self.config.skill_bonus)
            .min(self.config.max_skill_bonus);

        // Calculate decomposition bonus
        let decomposition_bonus = if input.was_decomposed {
            self.config.decomposition_bonus
        } else {
            0
        };

        // Calculate iteration penalty (only if low reliability + many iterations)
        let iteration_penalty = if tier == ReliabilityTier::Low
            && input.iterations > self.config.iteration_penalty_threshold
        {
            self.config.iteration_penalty
                * (input.iterations - self.config.iteration_penalty_threshold) as u64
        } else {
            0
        };

        // Calculate total
        let total = scaled_base
            .saturating_add(skill_bonus)
            .saturating_add(decomposition_bonus)
            .saturating_sub(iteration_penalty);

        XpGain {
            total,
            base_xp: scaled_base,
            skill_bonus,
            decomposition_bonus,
            iteration_penalty,
            reliability_tier: tier,
            xp_penalty: None,
        }
    }

    /// Calculate reliability scale factor within tier (1.0 - 1.5)
    fn reliability_scale(&self, reliability: f64, tier: ReliabilityTier) -> f64 {
        let (min, max) = match tier {
            ReliabilityTier::Failed => return 0.0,
            ReliabilityTier::Low => (self.config.min_reliability, self.config.medium_threshold),
            ReliabilityTier::Medium => (self.config.medium_threshold, self.config.high_threshold),
            ReliabilityTier::High => (self.config.high_threshold, 1.0),
        };

        let range = max - min;
        if range <= 0.0 {
            return 1.0;
        }

        let position = (reliability - min) / range;
        1.0 + (position * 0.5) // 1.0 to 1.5 scale
    }
}

impl Default for XpCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failed_answer_no_xp() {
        let calc = XpCalculator::new();
        let input = XpInput::new(0.95, 2, 1, false, false);
        let gain = calc.calculate(&input);
        assert_eq!(gain.total, 0);
    }

    #[test]
    fn test_low_reliability_no_xp() {
        let calc = XpCalculator::new();
        let input = XpInput::new(0.30, 2, 1, false, true);
        let gain = calc.calculate(&input);
        assert_eq!(gain.total, 0);
        assert_eq!(gain.reliability_tier, ReliabilityTier::Failed);
    }

    #[test]
    fn test_low_tier_xp() {
        let calc = XpCalculator::new();
        let input = XpInput::new(0.50, 0, 1, false, true);
        let gain = calc.calculate(&input);

        assert!(gain.total > 0);
        assert!(gain.total <= 20); // Low tier base is 10, scaled up to 15
        assert_eq!(gain.reliability_tier, ReliabilityTier::Low);
    }

    #[test]
    fn test_medium_tier_xp() {
        let calc = XpCalculator::new();
        let input = XpInput::new(0.80, 0, 1, false, true);
        let gain = calc.calculate(&input);

        assert!(gain.total >= 30);
        assert!(gain.total <= 50);
        assert_eq!(gain.reliability_tier, ReliabilityTier::Medium);
    }

    #[test]
    fn test_high_tier_xp() {
        let calc = XpCalculator::new();
        let input = XpInput::new(0.95, 0, 1, false, true);
        let gain = calc.calculate(&input);

        assert!(gain.total >= 75);
        assert!(gain.total <= 120);
        assert_eq!(gain.reliability_tier, ReliabilityTier::High);
    }

    #[test]
    fn test_skill_bonus() {
        let calc = XpCalculator::new();

        let input_no_skills = XpInput::new(0.80, 0, 1, false, true);
        let input_with_skills = XpInput::new(0.80, 3, 1, false, true);

        let gain_no = calc.calculate(&input_no_skills);
        let gain_with = calc.calculate(&input_with_skills);

        assert!(gain_with.total > gain_no.total);
        assert_eq!(gain_with.skill_bonus, 15); // 3 skills * 5 XP
    }

    #[test]
    fn test_skill_bonus_capped() {
        let calc = XpCalculator::new();
        let input = XpInput::new(0.80, 10, 1, false, true);
        let gain = calc.calculate(&input);

        assert_eq!(gain.skill_bonus, 20); // Capped at max_skill_bonus
    }

    #[test]
    fn test_decomposition_bonus() {
        let calc = XpCalculator::new();

        let input_simple = XpInput::new(0.80, 0, 1, false, true);
        let input_decomposed = XpInput::new(0.80, 0, 1, true, true);

        let gain_simple = calc.calculate(&input_simple);
        let gain_decomposed = calc.calculate(&input_decomposed);

        assert!(gain_decomposed.total > gain_simple.total);
        assert_eq!(gain_decomposed.decomposition_bonus, 10);
    }

    #[test]
    fn test_iteration_penalty() {
        let calc = XpCalculator::new();

        let input_few = XpInput::new(0.50, 0, 2, false, true);
        let input_many = XpInput::new(0.50, 0, 6, false, true);

        let gain_few = calc.calculate(&input_few);
        let gain_many = calc.calculate(&input_many);

        // Only applies to low tier
        assert!(gain_many.total < gain_few.total);
        assert!(gain_many.iteration_penalty > 0);
    }

    #[test]
    fn test_iteration_penalty_not_applied_to_high_tier() {
        let calc = XpCalculator::new();
        let input = XpInput::new(0.95, 0, 10, false, true);
        let gain = calc.calculate(&input);

        // No penalty for high reliability even with many iterations
        assert_eq!(gain.iteration_penalty, 0);
    }

    #[test]
    fn test_reliability_scale_within_tier() {
        let calc = XpCalculator::new();

        // Two inputs in same tier but different reliability
        let input_low_r = XpInput::new(0.70, 0, 1, false, true);
        let input_high_r = XpInput::new(0.89, 0, 1, false, true);

        let gain_low = calc.calculate(&input_low_r);
        let gain_high = calc.calculate(&input_high_r);

        // Higher reliability within tier should give more XP
        assert!(gain_high.base_xp > gain_low.base_xp);
    }

    #[test]
    fn test_tier_detection() {
        let config = XpConfig::default();

        assert_eq!(
            ReliabilityTier::from_reliability(0.30, &config),
            ReliabilityTier::Failed
        );
        assert_eq!(
            ReliabilityTier::from_reliability(0.40, &config),
            ReliabilityTier::Low
        );
        assert_eq!(
            ReliabilityTier::from_reliability(0.69, &config),
            ReliabilityTier::Low
        );
        assert_eq!(
            ReliabilityTier::from_reliability(0.70, &config),
            ReliabilityTier::Medium
        );
        assert_eq!(
            ReliabilityTier::from_reliability(0.89, &config),
            ReliabilityTier::Medium
        );
        assert_eq!(
            ReliabilityTier::from_reliability(0.90, &config),
            ReliabilityTier::High
        );
        assert_eq!(
            ReliabilityTier::from_reliability(1.0, &config),
            ReliabilityTier::High
        );
    }
}
