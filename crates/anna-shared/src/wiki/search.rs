//! Wiki search combining keyword and semantic search.

use anyhow::Result;

use super::embeddings;
use super::index::WikiIndex;
use super::{WikiArticle, WikiSearchResult};

/// Search wiki using combined keyword + semantic search
pub async fn search(
    ollama_url: &str,
    query: &str,
    top_k: usize,
    use_embeddings: bool,
) -> Result<Vec<WikiSearchResult>> {
    let index = WikiIndex::load()?;

    if use_embeddings {
        // Try semantic search first
        match embeddings::semantic_search(ollama_url, query, top_k).await {
            Ok(results) if !results.is_empty() => return Ok(results),
            Ok(_) => tracing::debug!("Semantic search returned no results, falling back to keywords"),
            Err(e) => tracing::warn!("Semantic search failed: {}, falling back to keywords", e),
        }
    }

    // Fallback to keyword search
    keyword_search(&index, query, top_k)
}

/// Search using keywords only
pub fn keyword_search(index: &WikiIndex, query: &str, top_k: usize) -> Result<Vec<WikiSearchResult>> {
    let titles = index.search_keywords(query);

    let mut results = Vec::new();
    for title in titles.into_iter().take(top_k) {
        if let Some(article) = index.get_article(&title) {
            results.push(WikiSearchResult {
                article,
                score: 0.5, // Default score for keyword matches
                relevant_section: None,
            });
        }
    }

    Ok(results)
}

/// Find relevant section within an article
pub fn find_relevant_section(article: &WikiArticle, query: &str) -> Option<String> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    // Split article into sections (by headers or double newlines)
    let sections: Vec<&str> = article.content.split("\n\n").collect();

    let mut best_section = None;
    let mut best_score = 0;

    for section in sections {
        let section_lower = section.to_lowercase();
        let mut score = 0;

        for word in &query_words {
            if section_lower.contains(word) {
                score += 1;
            }
        }

        // Bonus for sections with commands
        if section.contains("# ") || section.contains("$ ") || section.contains("```") {
            score += 2;
        }

        if score > best_score {
            best_score = score;
            best_section = Some(section.to_string());
        }
    }

    best_section
}

/// Quick search for specific topics (exact title match)
pub fn quick_lookup(index: &WikiIndex, topic: &str) -> Option<WikiArticle> {
    // Try exact match first
    if let Some(article) = index.get_article(topic) {
        return Some(article);
    }

    // Try case-insensitive match
    let topic_lower = topic.to_lowercase();
    for title in index.articles.keys() {
        if title.to_lowercase() == topic_lower {
            return index.get_article(title);
        }
    }

    // Try partial match
    for title in index.articles.keys() {
        if title.to_lowercase().contains(&topic_lower) {
            return index.get_article(title);
        }
    }

    None
}
