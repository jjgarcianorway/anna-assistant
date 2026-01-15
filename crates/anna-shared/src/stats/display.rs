//! v0.3.21: XP formula documentation and verification functions
//!
//! The XP formula is transparent and verifiable. All values are derived
//! from real events tracked in the audit trail.
//!
//! ## Formula Components
//!
//! 1. **Questions XP** (0-50): `log2(total_questions) * 10`
//!    - Each doubling of questions adds ~10 XP
//!    - 1 question = 0 XP, 2 = 10, 4 = 20, 8 = 30, 16 = 40, 32 = 50
//!
//! 2. **Efficiency Bonus** (0-20): `(instant + memory) / total * 20`
//!    - Rewards answering without LLM
//!    - 100% efficiency = 20 XP bonus
//!
//! 3. **Recipe Bonus** (0-20): `min(recipes_learned, 20)`
//!    - 1 XP per recipe, capped at 20
//!
//! 4. **Reliability Multiplier** (0.5-1.0): `0.5 + reliability * 0.5`
//!    - Low reliability (0%) = 0.5x multiplier
//!    - High reliability (100%) = 1.0x multiplier
//!
//! ## Final Formula
//!
//! ```text
//! XP = min(100, (questions_xp + efficiency_bonus + recipe_bonus) * reliability_mult)
//! ```
//!
//! ## Titles by XP
//!
//! | XP Range | Title |
//! |----------|-------|
//! | 0-4 | Novice Apprentice |
//! | 5-9 | Eager Learner |
//! | 10-19 | Junior Technician |
//! | 20-29 | Curious Explorer |
//! | 30-39 | Competent Assistant |
//! | 40-49 | Skilled Operator |
//! | 50-59 | Senior Specialist |
//! | 60-69 | Expert Analyst |
//! | 70-79 | Master Troubleshooter |
//! | 80-89 | IT Sage |
//! | 90-94 | System Whisperer |
//! | 95-99 | Arch Wizard |
//! | 100 | Omniscient Oracle |

/// Calculate XP from components (for verification)
pub fn calculate_xp(
    total_questions: u64,
    instant_answers: u64,
    memory_answers: u64,
    recipes_learned: u32,
    reliability: f32,
) -> u32 {
    // Questions XP: logarithmic scaling
    let questions_xp = if total_questions > 0 {
        (total_questions as f64).log2() * 10.0
    } else {
        0.0
    };

    // Efficiency bonus
    let efficiency = if total_questions > 0 {
        (instant_answers + memory_answers) as f64 / total_questions as f64
    } else {
        0.0
    };
    let efficiency_bonus = efficiency * 20.0;

    // Recipe bonus (capped at 20)
    let recipe_bonus = (recipes_learned as f64).min(20.0);

    // Reliability multiplier
    let reliability_mult = 0.5 + (reliability as f64 * 0.5);

    // Final calculation
    let raw_xp = (questions_xp + efficiency_bonus + recipe_bonus) * reliability_mult;
    (raw_xp as u32).min(100)
}

/// Explain XP calculation for transparency
pub fn explain_xp(
    total_questions: u64,
    instant_answers: u64,
    memory_answers: u64,
    recipes_learned: u32,
    reliability: f32,
) -> String {
    let questions_xp = if total_questions > 0 {
        (total_questions as f64).log2() * 10.0
    } else {
        0.0
    };

    let efficiency = if total_questions > 0 {
        (instant_answers + memory_answers) as f64 / total_questions as f64
    } else {
        0.0
    };
    let efficiency_bonus = efficiency * 20.0;

    let recipe_bonus = (recipes_learned as f64).min(20.0);
    let reliability_mult = 0.5 + (reliability as f64 * 0.5);

    let raw_xp = (questions_xp + efficiency_bonus + recipe_bonus) * reliability_mult;
    let final_xp = (raw_xp as u32).min(100);

    format!(
        "XP Breakdown:\n\
         - Questions ({} total): {:.1} XP (log2 * 10)\n\
         - Efficiency ({:.0}% solved alone): {:.1} XP\n\
         - Recipes ({} learned): {:.1} XP\n\
         - Reliability ({:.0}%): {:.2}x multiplier\n\
         - Raw: ({:.1} + {:.1} + {:.1}) * {:.2} = {:.1}\n\
         - Final: {} XP",
        total_questions, questions_xp,
        efficiency * 100.0, efficiency_bonus,
        recipes_learned, recipe_bonus,
        reliability * 100.0, reliability_mult,
        questions_xp, efficiency_bonus, recipe_bonus, reliability_mult, raw_xp,
        final_xp
    )
}
