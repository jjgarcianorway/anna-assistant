//! Tests for knowledge fetcher.

#[cfg(test)]
mod tests {
    use crate::knowledge_v2::fetcher::helpers::{extract_keywords, extract_summary, is_command_like};
    use crate::knowledge_v2::fetcher::types::FetchResult;
    use crate::knowledge_v2::fetcher::KnowledgeFetcher;
    use crate::knowledge_v2::MAX_SNIPPETS_PER_TICKET;

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("How do I enable vim syntax highlighting?");
        assert!(keywords.contains(&"enable".to_string()));
        assert!(keywords.contains(&"vim".to_string()));
        assert!(keywords.contains(&"syntax".to_string()));
        assert!(!keywords.contains(&"how".to_string()));
        assert!(!keywords.contains(&"do".to_string()));
    }

    #[test]
    fn test_extract_summary() {
        let content = "First sentence here.\nSecond sentence.\nThird sentence.";
        let summary = extract_summary(content, 2);
        assert!(summary.contains("First"));
        assert!(summary.contains("Second"));
    }

    #[test]
    fn test_is_command_like() {
        assert!(is_command_like("systemctl"));
        assert!(is_command_like("vim"));
        assert!(is_command_like("python3.11"));
        assert!(!is_command_like("how to"));
        assert!(!is_command_like("some long topic name"));
    }

    #[test]
    fn test_fetch_result_empty() {
        let result = FetchResult::empty();
        assert!(!result.has_knowledge);
        assert!(result.snippets.is_empty());
    }

    #[test]
    fn test_fetcher_new() {
        let fetcher = KnowledgeFetcher::new();
        assert_eq!(fetcher.max_snippets(), MAX_SNIPPETS_PER_TICKET);
    }
}
