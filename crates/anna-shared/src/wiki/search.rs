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

/// Search using keywords only - improved to prioritize topic matches
pub fn keyword_search(index: &WikiIndex, query: &str, top_k: usize) -> Result<Vec<WikiSearchResult>> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    // First, try to find direct topic matches in the query
    // Look for article titles that appear in the query (e.g., "GDM" in "how to scale GDM")
    let mut direct_matches: Vec<WikiSearchResult> = Vec::new();

    for title in index.articles.keys() {
        let title_lower = title.to_lowercase();
        let title_words: Vec<&str> = title_lower.split_whitespace().collect();

        // Check if any significant title word appears in query
        for title_word in &title_words {
            if title_word.len() >= 3 && query_lower.contains(title_word) {
                if let Some(article) = index.get_article(title) {
                    // Give high score for direct title match
                    direct_matches.push(WikiSearchResult {
                        article,
                        score: 0.9,
                        relevant_section: None,
                    });
                    break;
                }
            }
        }
    }

    // If we found direct matches, prioritize those
    if !direct_matches.is_empty() {
        // Sort by relevance (titles that match more query words get higher score)
        direct_matches.sort_by(|a, b| {
            let a_matches = count_query_matches(&a.article.title, &query_words);
            let b_matches = count_query_matches(&b.article.title, &query_words);
            b_matches.cmp(&a_matches)
        });
        direct_matches.truncate(top_k);
        return Ok(direct_matches);
    }

    // Fallback to keyword index search
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

/// Count how many query words match a text
fn count_query_matches(text: &str, query_words: &[&str]) -> usize {
    let text_lower = text.to_lowercase();
    query_words.iter().filter(|w| text_lower.contains(*w)).count()
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
