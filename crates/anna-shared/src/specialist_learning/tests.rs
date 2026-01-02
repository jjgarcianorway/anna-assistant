//! Tests for specialist learning system

#[cfg(test)]
mod tests {
    use crate::specialist_learning::patterns::{detect_pattern_category, extract_target};
    use crate::specialist_learning::types::PatternCategory;
    use crate::specialist_learning::utils::extract_keywords;

    #[test]
    fn test_detect_pattern_category() {
        assert_eq!(
            detect_pattern_category("check hyprland config"),
            Some(PatternCategory::ConfigCheck)
        );
        assert_eq!(
            detect_pattern_category("enable syntax highlighting in vim"),
            Some(PatternCategory::ConfigEdit)
        );
        assert_eq!(
            detect_pattern_category("restart nginx service"),
            Some(PatternCategory::ServiceAction)
        );
        assert_eq!(
            detect_pattern_category("is docker installed"),
            Some(PatternCategory::PackageQuery)
        );
        assert_eq!(
            detect_pattern_category("what folders are taking space"),
            Some(PatternCategory::DiskAnalysis)
        );
    }

    #[test]
    fn test_extract_target() {
        assert_eq!(
            extract_target("check hyprland config"),
            Some("hyprland".to_string())
        );
        assert_eq!(
            extract_target("restart nginx service"),
            Some("nginx".to_string())
        );
        assert_eq!(extract_target("is vim installed"), Some("vim".to_string()));
    }

    #[test]
    fn test_extract_keywords() {
        let kw = extract_keywords("what folders are taking the most space");
        assert!(kw.contains(&"folders".to_string()) && kw.contains(&"space".to_string()));
        assert!(!kw.contains(&"the".to_string()) && !kw.contains(&"are".to_string()));
    }
}
