//! Recipe learning logic (v0.0.232).

/// Minimum reliability to learn a recipe
pub const MIN_LEARN_RELIABILITY: u8 = 85;

/// Should we learn a recipe from this outcome?
pub fn should_learn_recipe(reliability: u8, is_deterministic: bool, already_exists: bool) -> bool {
    if already_exists {
        return false;
    }
    if reliability < MIN_LEARN_RELIABILITY {
        return false;
    }
    // Only learn from deterministic paths (grounded answers)
    is_deterministic
}
