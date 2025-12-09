//! Tests for greeting module (v0.0.186).

#[cfg(test)]
mod tests {
    use crate::greeting::{calculate_interaction_info, bullet};

    #[test]
    fn test_calculate_interaction_info_first_time() {
        let info = calculate_interaction_info(&None);
        assert!(info.is_first_time);
        assert!(info.hours_since_last.is_none());
    }

    #[test]
    fn test_bullet_char() {
        assert_eq!(bullet(), "›");
    }
}
