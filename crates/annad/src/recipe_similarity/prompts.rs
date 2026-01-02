//! Prompt construction and response parsing for similarity checking.

/// Build prompt to check semantic similarity between two queries
/// v0.0.391: Made MUCH stricter to prevent false positives within same domain
pub fn build_similarity_prompt(new_query: &str, recipe_query: &str) -> String {
    format!(
        r#"Are these two Linux queries asking for EXACTLY the same thing?

Query A: "{}"
Query B: "{}"

Reply with ONLY a JSON object: {{"similar": true/false, "score": 0-100, "reason": "brief explanation"}}

CRITICAL - BE EXTREMELY STRICT:
- "similar" = true ONLY if the EXACT SAME command would answer both
- score 100 = ONLY for identical or very trivially rephrased questions
- score 0 = different questions, even if in same domain

SAME DOMAIN ≠ SAME QUESTION:
- "how much disk space" vs "which folders use space" → NOT similar (df vs du)
- "memory usage" vs "memory info" → NOT similar (different detail level)
- "CPU usage" vs "CPU temperature" → NOT similar (different metrics)
- "list packages" vs "check if X installed" → NOT similar

SIMILAR examples (score 90+):
- "disk space" vs "how much storage" → similar (both want df)
- "free RAM" vs "available memory" → similar (both want free -h)

NOT similar (score < 50):
- "disk usage" vs "which folders" → NOT similar (df vs du)
- "overall space" vs "biggest directories" → NOT similar

When in doubt, return similar=false and LOW score.
Output raw JSON only."#,
        new_query, recipe_query
    )
}

/// Parse similarity response from LLM
pub fn parse_similarity_response(response: &str) -> Option<(bool, u8)> {
    // Try to extract JSON
    let json_str = if let (Some(s), Some(e)) = (response.find('{'), response.rfind('}')) {
        &response[s..=e]
    } else {
        return None;
    };

    // Parse JSON
    let json: serde_json::Value = serde_json::from_str(json_str).ok()?;

    let similar = json.get("similar")?.as_bool()?;
    let score = json.get("score")?.as_u64()? as u8;

    Some((similar, score.min(100)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_similarity_prompt() {
        let prompt = build_similarity_prompt("how much disk space", "what is storage usage");
        assert!(prompt.contains("how much disk space"));
        assert!(prompt.contains("what is storage usage"));
        assert!(prompt.contains("similar"));
    }

    #[test]
    fn test_parse_similarity_response_valid() {
        let response = r#"{"similar": true, "score": 85, "reason": "both ask about disk"}"#;
        let result = parse_similarity_response(response);
        assert!(result.is_some());
        let (similar, score) = result.unwrap();
        assert!(similar);
        assert_eq!(score, 85);
    }

    #[test]
    fn test_parse_similarity_response_not_similar() {
        let response = r#"{"similar": false, "score": 20, "reason": "different topics"}"#;
        let result = parse_similarity_response(response);
        assert!(result.is_some());
        let (similar, score) = result.unwrap();
        assert!(!similar);
        assert_eq!(score, 20);
    }

    #[test]
    fn test_parse_similarity_response_with_markdown() {
        let response = r#"Here's the result:
```json
{"similar": true, "score": 90, "reason": "same question"}
```"#;
        let result = parse_similarity_response(response);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_similarity_response_invalid() {
        let response = "I don't understand";
        let result = parse_similarity_response(response);
        assert!(result.is_none());
    }
}
