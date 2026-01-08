//! Wiki index for fast search.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::{wiki_articles_dir, wiki_index_path, WikiArticle};

/// Wiki index for fast lookup
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikiIndex {
    /// Title -> file path mapping
    pub articles: HashMap<String, ArticleMetadata>,
    /// Keyword -> article titles mapping (inverted index)
    pub keywords: HashMap<String, Vec<String>>,
    /// Total article count
    pub total_articles: usize,
    /// Last index build time
    pub built_at: Option<String>,
}

/// Metadata about an article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMetadata {
    /// Title
    pub title: String,
    /// File path relative to wiki dir
    pub path: PathBuf,
    /// First 200 chars summary
    pub summary: String,
    /// Keywords extracted from title and content
    pub keywords: Vec<String>,
    /// File size
    pub size_bytes: u64,
}

impl WikiIndex {
    /// Load index from disk
    pub fn load() -> Result<Self> {
        let path = wiki_index_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let index: WikiIndex = serde_json::from_str(&content)?;
            Ok(index)
        } else {
            Ok(WikiIndex::default())
        }
    }

    /// Save index to disk
    pub fn save(&self) -> Result<()> {
        let path = wiki_index_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Search by keywords (simple)
    pub fn search_keywords(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scores: HashMap<String, usize> = HashMap::new();

        for word in &query_words {
            if let Some(articles) = self.keywords.get(*word) {
                for article in articles {
                    *scores.entry(article.clone()).or_insert(0) += 1;
                }
            }
        }

        // Sort by score (most keyword matches first)
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results.into_iter().map(|(title, _)| title).collect()
    }

    /// Get article by title
    pub fn get_article(&self, title: &str) -> Option<WikiArticle> {
        let metadata = self.articles.get(title)?;
        let articles_dir = wiki_articles_dir();
        let path = articles_dir.join(&metadata.path);

        let content = std::fs::read_to_string(&path).ok()?;

        Some(WikiArticle {
            title: title.to_string(),
            content,
            url: format!("https://wiki.archlinux.org/title/{}", title.replace(' ', "_")),
            categories: vec![],
        })
    }
}

/// Build index from downloaded wiki articles
pub async fn build_index() -> Result<WikiIndex> {
    let articles_dir = wiki_articles_dir();
    let mut index = WikiIndex::default();

    if !articles_dir.exists() {
        anyhow::bail!("Wiki articles not downloaded yet");
    }

    let entries = std::fs::read_dir(&articles_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "txt" || ext == "md" {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let title = path
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .replace('_', " ");

                        // Extract keywords from title and content
                        let keywords = extract_keywords(&title, &content);

                        // Create summary
                        let summary: String = content
                            .chars()
                            .take(200)
                            .collect::<String>()
                            .replace('\n', " ");

                        let metadata = ArticleMetadata {
                            title: title.clone(),
                            path: path.file_name().unwrap().into(),
                            summary,
                            keywords: keywords.clone(),
                            size_bytes: entry.metadata()?.len(),
                        };

                        // Add to articles map
                        index.articles.insert(title.clone(), metadata);

                        // Add to inverted index
                        for keyword in keywords {
                            index
                                .keywords
                                .entry(keyword)
                                .or_insert_with(Vec::new)
                                .push(title.clone());
                        }

                        index.total_articles += 1;
                    }
                }
            }
        }
    }

    index.built_at = Some(chrono::Utc::now().to_rfc3339());
    index.save()?;

    tracing::info!("Built index with {} articles", index.total_articles);
    Ok(index)
}

/// Extract keywords from title and content
fn extract_keywords(title: &str, content: &str) -> Vec<String> {
    let mut keywords = Vec::new();

    // Title words are important
    for word in title.split_whitespace() {
        let word = word.to_lowercase();
        if word.len() > 2 && !is_stop_word(&word) {
            keywords.push(word);
        }
    }

    // Extract important words from content (first 1000 chars)
    let sample: String = content.chars().take(1000).collect();
    for word in sample.split(|c: char| !c.is_alphanumeric()) {
        let word = word.to_lowercase();
        if word.len() > 3 && !is_stop_word(&word) && !keywords.contains(&word) {
            keywords.push(word);
        }
    }

    // Limit keywords per article
    keywords.truncate(50);
    keywords
}

/// Check if a word is a stop word
fn is_stop_word(word: &str) -> bool {
    const STOP_WORDS: &[&str] = &[
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "had",
        "her", "was", "one", "our", "out", "has", "have", "been", "were", "they",
        "this", "that", "with", "from", "your", "will", "more", "when", "which",
        "their", "said", "each", "about", "than", "into", "them", "these", "some",
        "would", "make", "like", "just", "over", "such", "also", "back", "after",
        "should", "because", "being", "where", "while", "there", "could", "other",
    ];
    STOP_WORDS.contains(&word)
}
