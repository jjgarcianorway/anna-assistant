//! Staff XP and level system
//!
//! Defines progression mechanics for staff members, including
//! XP-to-level conversion and title assignments.

/// v0.0.301: Convert XP to level (1-6)
/// Progression is slower - meaningful growth over time
pub fn xp_to_level(xp: u64) -> u8 {
    match xp {
        0..=99 => 1,      // Novice
        100..=299 => 2,   // Apprentice
        300..=699 => 3,   // Competent
        700..=1499 => 4,  // Expert
        1500..=2999 => 5, // Master
        _ => 6,           // Principal
    }
}

/// v0.0.301: Get title for level (for juniors and seniors)
pub fn level_title(level: u8, is_senior: bool) -> &'static str {
    if is_senior {
        match level {
            1..=3 => "Expert",
            4..=5 => "Master",
            _ => "Principal",
        }
    } else {
        match level {
            1 => "Novice",
            2 => "Apprentice",
            3 => "Competent",
            4 => "Skilled",
            5 => "Proficient",
            _ => "Expert",
        }
    }
}
