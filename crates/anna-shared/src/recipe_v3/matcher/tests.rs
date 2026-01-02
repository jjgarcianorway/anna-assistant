//! Tests for recipe matching (v0.0.423).

#[cfg(test)]
mod tests {
    use crate::recipe_v3::{RecipeDomain, RecipeMatcher as RM, RecipeV3};

    use super::super::{
        matcher_core::RecipeMatcher,
        matcher_helpers::{detect_domain, detect_intent, extract_entities, extract_keywords},
        matcher_types::MatchQuery,
    };

    #[test]
    fn test_extract_keywords() {
        let kw = extract_keywords("How do I restart the nginx service?");
        assert!(kw.contains(&"restart".to_string()));
        assert!(kw.contains(&"nginx".to_string()));
        assert!(kw.contains(&"service".to_string()));
        assert!(!kw.contains(&"how".to_string()));
    }

    #[test]
    fn test_extract_entities() {
        let entities = extract_entities("restart nginx.service");
        assert!(entities.contains(&"nginx.service".to_string()));

        // Path extraction only works for paths starting with /
        let entities2 = extract_entities("check /etc/nginx/nginx.conf file");
        assert!(entities2.iter().any(|e| e.starts_with("/etc")));
    }

    #[test]
    fn test_detect_domain() {
        assert_eq!(
            detect_domain("restart nginx service"),
            Some("systemd".to_string())
        );
        assert_eq!(detect_domain("install vim"), Some("package".to_string()));
        assert_eq!(
            detect_domain("check network connection"),
            Some("network".to_string())
        );
    }

    #[test]
    fn test_detect_intent() {
        assert_eq!(detect_intent("restart nginx"), Some("restart".to_string()));
        assert_eq!(
            detect_intent("how do I configure vim"),
            Some("howto".to_string())
        );
        assert_eq!(
            detect_intent("nginx is not working"),
            Some("fix".to_string())
        );
    }

    #[test]
    fn test_match_query() {
        let query = MatchQuery::from_question("How do I restart nginx?");
        assert!(query.keywords.contains(&"restart".to_string()));
        assert!(query.keywords.contains(&"nginx".to_string()));
        assert_eq!(query.intent, Some("restart".to_string()));
    }

    #[test]
    fn test_recipe_matching() {
        let recipe = RecipeV3::new("restart-service", "Restart a Service").with_matcher(
            RM::new(RecipeDomain::Systemd)
                .with_intents(&["restart"])
                .with_keywords(&["restart", "service", "systemctl"])
                .with_entities(&["*"]),
        );

        let query = MatchQuery::from_question("restart nginx service");
        let matcher = RecipeMatcher::new();
        let results = matcher.find_matches(&query, &[recipe]);

        assert!(!results.is_empty());
        assert!(results[0].score > 0.5);
    }
}
