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

/// Search using keywords only - prioritizes exact topic matches
pub fn keyword_search(index: &WikiIndex, query: &str, top_k: usize) -> Result<Vec<WikiSearchResult>> {
    let query_lower = query.to_lowercase();

    // Extract significant words from query (skip stop words)
    let query_words: Vec<&str> = query_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2 && !is_stop_word(w))
        .collect();

    let mut scored_results: Vec<(String, f32)> = Vec::new();

    for title in index.articles.keys() {
        let title_lower = title.to_lowercase();
        let mut score: f32 = 0.0;

        // Check for exact title match (highest priority)
        for word in &query_words {
            // Exact match with title or title word
            if title_lower == *word {
                score += 10.0; // Exact title match
            } else if title_lower.split(|c: char| !c.is_alphanumeric()).any(|tw| tw == *word) {
                score += 5.0; // Title contains this exact word
            } else if word.len() >= 4 && title_lower.contains(word) {
                score += 2.0; // Partial match (only for longer words)
            }
        }

        if score > 0.0 {
            scored_results.push((title.clone(), score));
        }
    }

    // Sort by score descending
    scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top results
    let mut results = Vec::new();
    for (title, score) in scored_results.into_iter().take(top_k) {
        if let Some(article) = index.get_article(&title) {
            results.push(WikiSearchResult {
                article,
                score: (score / 10.0).min(1.0), // Normalize score
                relevant_section: None,
            });
        }
    }

    // If no direct matches, fall back to keyword index
    if results.is_empty() {
        let titles = index.search_keywords(query);
        for title in titles.into_iter().take(top_k) {
            if let Some(article) = index.get_article(&title) {
                results.push(WikiSearchResult {
                    article,
                    score: 0.3,
                    relevant_section: None,
                });
            }
        }
    }

    Ok(results)
}

/// Check if word is a stop word (common words to ignore)
fn is_stop_word(word: &str) -> bool {
    const STOP_WORDS: &[&str] = &[
        "a", "an", "the", "is", "it", "to", "of", "in", "for", "on", "with",
        "at", "by", "from", "or", "and", "be", "was", "were", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "can", "this", "that", "these",
        "those", "i", "you", "he", "she", "we", "they", "my", "your", "his",
        "her", "its", "our", "their", "what", "which", "who", "whom", "how",
        "when", "where", "why", "if", "then", "else", "so", "but", "not",
        "no", "yes", "all", "any", "some", "every", "each", "much", "many",
        "more", "most", "other", "another", "such", "only", "just", "also",
        "very", "too", "really", "everything", "nothing", "something",
        "make", "made", "change", "tried", "works", "small",
    ];
    STOP_WORDS.contains(&word)
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
