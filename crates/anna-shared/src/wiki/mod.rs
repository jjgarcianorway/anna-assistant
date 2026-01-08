//! Arch Wiki RAG (Retrieval Augmented Generation) module.
//!
//! Provides offline access to Arch Wiki for grounded answers.
//!
//! # Architecture
//!
//! ```text
//! User Question
//!     ↓
//! Embed question (Ollama nomic-embed-text)
//!     ↓
//! Search wiki embeddings (cosine similarity)
//!     ↓
//! Get top-k relevant articles
//!     ↓
//! Extract commands from articles
//!     ↓
//! Execute commands + LLM formats answer
//! ```

pub mod download;
pub mod embeddings;
pub mod extract;
pub mod index;
pub mod search;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::anna_data_dir;

/// A wiki article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiArticle {
    /// Article title (e.g., "Pacman", "Vim")
    pub title: String,
    /// Article content (markdown)
    pub content: String,
    /// Source URL
    pub url: String,
    /// Categories/tags
    pub categories: Vec<String>,
}

/// A search result from the wiki
#[derive(Debug, Clone)]
pub struct WikiSearchResult {
    /// The article
    pub article: WikiArticle,
    /// Relevance score (0.0 - 1.0)
    pub score: f32,
    /// Relevant section of the article
    pub relevant_section: Option<String>,
}

/// A command extracted from wiki text
#[derive(Debug, Clone)]
pub struct ExtractedCommand {
    /// The command itself
    pub command: String,
    /// Description from context
    pub description: Option<String>,
    /// Whether it requires root
    pub requires_root: bool,
    /// Source article
    pub source_article: String,
}

/// Get wiki data directory
pub fn wiki_data_dir() -> PathBuf {
    anna_data_dir().join("wiki")
}

/// Get wiki articles directory
pub fn wiki_articles_dir() -> PathBuf {
    wiki_data_dir().join("articles")
}

/// Get wiki index path
pub fn wiki_index_path() -> PathBuf {
    wiki_data_dir().join("index.json")
}

/// Get wiki embeddings path
pub fn wiki_embeddings_path() -> PathBuf {
    wiki_data_dir().join("embeddings.bin")
}

/// Check if wiki is downloaded and indexed
pub fn wiki_available() -> bool {
    wiki_index_path().exists() && wiki_articles_dir().exists()
}

/// Initialize wiki (download if needed, build index)
pub async fn init_wiki(ollama_url: &str) -> Result<()> {
    // Ensure directories exist
    std::fs::create_dir_all(wiki_articles_dir())?;

    // Check if we need to download/process wiki
    let need_download = !wiki_available();
    let need_reindex = if let Ok(idx) = index::WikiIndex::load() {
        idx.total_articles == 0
    } else {
        true
    };

    // Download if not present
    if need_download || need_reindex {
        tracing::info!("Downloading Arch Wiki...");
        download::download_wiki().await?;
    }

    // Build index if not present or empty
    if !wiki_index_path().exists() || need_reindex {
        tracing::info!("Building wiki index...");
        index::build_index().await?;
    }

    // Build embeddings if not present or index was rebuilt
    if !wiki_embeddings_path().exists() || need_reindex {
        tracing::info!("Building wiki embeddings (this may take a while)...");
        embeddings::build_embeddings(ollama_url).await?;
    }

    Ok(())
}
