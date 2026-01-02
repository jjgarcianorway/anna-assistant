//! Tests for capabilities display module.

#[cfg(test)]
mod tests {
    use crate::capabilities_display::*;

    #[test]
    fn test_all_categories() {
        let categories = CapabilityCategory::all();
        assert_eq!(categories.len(), 9);
    }

    #[test]
    fn test_category_has_examples() {
        for cat in CapabilityCategory::all() {
            let examples = cat.examples();
            assert!(!examples.is_empty(), "{:?} has no examples", cat);
            assert!(examples.len() >= 2, "{:?} needs at least 2 examples", cat);
        }
    }

    #[test]
    fn test_category_has_description() {
        for cat in CapabilityCategory::all() {
            let desc = cat.description();
            assert!(!desc.is_empty(), "{:?} has no description", cat);
        }
    }

    #[test]
    fn test_format_capabilities() {
        let output = format_capabilities();

        assert!(output.contains("What Anna Can Do"));
        assert!(output.contains("System Information"));
        assert!(output.contains("Package Management"));
        assert!(output.contains("Just ask in natural language"));
    }

    #[test]
    fn test_format_capabilities_compact() {
        let output = format_capabilities_compact();

        assert!(output.contains("I can help with:"));
        assert!(output.contains("System Information"));
    }

    #[test]
    fn test_format_capability_category() {
        let output = format_capability_category(CapabilityCategory::Packages);

        assert!(output.contains("Package Management"));
        assert!(output.contains("Install htop"));
    }

    #[test]
    fn test_is_capabilities_query() {
        // Should match
        assert!(is_capabilities_query("what can you do?"));
        assert!(is_capabilities_query("What can Anna do?"));
        assert!(is_capabilities_query("help me"));
        assert!(is_capabilities_query("help"));
        assert!(is_capabilities_query("show capabilities"));
        assert!(is_capabilities_query("how can you help me?"));

        // Should not match
        assert!(!is_capabilities_query("check disk space"));
        assert!(!is_capabilities_query("help with vim")); // Has context
        assert!(!is_capabilities_query("restart nginx"));
    }

    #[test]
    fn test_parse_capability_category() {
        assert_eq!(
            parse_capability_category("help with packages"),
            Some(CapabilityCategory::Packages)
        );
        assert_eq!(
            parse_capability_category("network help"),
            Some(CapabilityCategory::Network)
        );
        assert_eq!(
            parse_capability_category("learning mode"),
            Some(CapabilityCategory::Learning)
        );
        assert_eq!(parse_capability_category("random text"), None);
    }

    #[test]
    fn test_capability_facts() {
        let facts = capability_facts();
        assert!(facts.len() >= 5);

        for fact in facts {
            assert!(!fact.is_empty());
            assert!(fact.len() > 20); // Should be meaningful sentences
        }
    }

    #[test]
    fn test_random_capability_fact() {
        let fact = random_capability_fact();
        assert!(!fact.is_empty());
    }
}
