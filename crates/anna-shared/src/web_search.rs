//! Web Search - Look up solutions from the web.
//!
//! Uses DuckDuckGo for searching without API keys.
//! Provides fallback when local knowledge is insufficient.

use anyhow::{anyhow, Result};
use serde::Deserialize;
use tracing::{debug, warn};

/// A web search result.
#[derive(Debug, Clone)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Search the web for solutions to a problem.
/// Focuses on Linux/Arch-related results.
pub async fn search_for_solution(query: &str, max_results: usize) -> Result<Vec<WebSearchResult>> {
    // Add Linux/Arch context to query
    let enhanced_query = if query.to_lowercase().contains("arch")
        || query.to_lowercase().contains("linux")
        || query.to_lowercase().contains("pacman") {
        query.to_string()
    } else {
        format!("{} arch linux", query)
    };

    search_duckduckgo(&enhanced_query, max_results).await
}

/// Search DuckDuckGo using their instant answer API.
async fn search_duckduckgo(query: &str, max_results: usize) -> Result<Vec<WebSearchResult>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Anna/1.0 (Linux assistant)")
        .build()?;

    // DuckDuckGo Instant Answer API
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding::encode(query)
    );

    debug!("Web search: {}", query);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("Search failed: {}", response.status()));
    }

    let data: DuckDuckGoResponse = response.json().await?;

    let mut results = Vec::new();

    // Add abstract if available
    if !data.abstract_text.is_empty() {
        results.push(WebSearchResult {
            title: data.heading.clone(),
            url: data.abstract_url.clone(),
            snippet: data.abstract_text.clone(),
        });
    }

    // Add related topics
    for topic in data.related_topics.into_iter().take(max_results.saturating_sub(results.len())) {
        if let Some(text) = topic.text {
            if let Some(first_url) = topic.first_url {
                results.push(WebSearchResult {
                    title: text.chars().take(80).collect::<String>(),
                    url: first_url,
                    snippet: text,
                });
            }
        }
    }

    // Add results section
    for result in data.results.into_iter().take(max_results.saturating_sub(results.len())) {
        results.push(WebSearchResult {
            title: result.text.chars().take(80).collect::<String>(),
            url: result.first_url,
            snippet: result.text,
        });
    }

    if results.is_empty() {
        debug!("No web results for: {}", query);
    } else {
        debug!("Found {} web results", results.len());
    }

    Ok(results)
}

/// Search for error message solutions.
pub async fn search_error_solution(error_msg: &str) -> Result<Vec<WebSearchResult>> {
    // Clean up error message for searching
    let clean_error = error_msg
        .lines()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(100)
        .collect::<String>();

    let query = format!("{} solution fix", clean_error);
    search_for_solution(&query, 3).await
}

/// Format search results for LLM context.
pub fn format_results_for_context(results: &[WebSearchResult]) -> String {
    if results.is_empty() {
        return String::new();
    }

    let mut lines = vec!["Web search results:".to_string()];

    for (i, result) in results.iter().enumerate().take(3) {
        lines.push(format!("{}. {} ({})", i + 1, result.title, result.url));
        // Truncate snippet
        let snippet: String = result.snippet.chars().take(200).collect();
        lines.push(format!("   {}", snippet));
    }

    lines.join("\n")
}

#[derive(Debug, Deserialize)]
struct DuckDuckGoResponse {
    #[serde(rename = "Abstract")]
    abstract_text: String,
    #[serde(rename = "AbstractURL")]
    abstract_url: String,
    #[serde(rename = "Heading")]
    heading: String,
    #[serde(rename = "RelatedTopics")]
    related_topics: Vec<RelatedTopic>,
    #[serde(rename = "Results")]
    results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct RelatedTopic {
    #[serde(rename = "Text")]
    text: Option<String>,
    #[serde(rename = "FirstURL")]
    first_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    #[serde(rename = "Text")]
    text: String,
    #[serde(rename = "FirstURL")]
    first_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_results() {
        let results = vec![
            WebSearchResult {
                title: "Fix pacman error".to_string(),
                url: "https://example.com".to_string(),
                snippet: "How to fix pacman database lock".to_string(),
            },
        ];

        let formatted = format_results_for_context(&results);
        assert!(formatted.contains("Fix pacman error"));
        assert!(formatted.contains("example.com"));
    }
}
