//! Embedding-based semantic search for wiki.
//!
//! Uses Ollama's embedding API with nomic-embed-text model.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};

use super::{wiki_articles_dir, wiki_embeddings_path, WikiSearchResult, WikiArticle};
use super::index::WikiIndex;

/// Embedding model to use
const EMBEDDING_MODEL: &str = "nomic-embed-text";

/// Embedding dimension (nomic-embed-text uses 768)
const EMBEDDING_DIM: usize = 768;

/// Wiki embeddings store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiEmbeddings {
    /// Article title -> embedding vector
    pub embeddings: HashMap<String, Vec<f32>>,
    /// Model used
    pub model: String,
    /// Built timestamp
    pub built_at: Option<String>,
}

impl Default for WikiEmbeddings {
    fn default() -> Self {
        Self {
            embeddings: HashMap::new(),
            model: EMBEDDING_MODEL.to_string(),
            built_at: None,
        }
    }
}

impl WikiEmbeddings {
    /// Load embeddings from disk (binary format for efficiency)
    pub fn load() -> Result<Self> {
        let path = wiki_embeddings_path();
        if !path.exists() {
            return Ok(WikiEmbeddings::default());
        }

        let file = std::fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let embeddings: WikiEmbeddings = bincode::deserialize_from(reader)?;
        Ok(embeddings)
    }

    /// Save embeddings to disk
    pub fn save(&self) -> Result<()> {
        let path = wiki_embeddings_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(&path)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self)?;
        Ok(())
    }

    /// Search by semantic similarity
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = self
            .embeddings
            .iter()
            .map(|(title, emb)| {
                let score = cosine_similarity(query_embedding, emb);
                (title.clone(), score)
            })
            .collect();

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores.truncate(top_k);
        scores
    }
}

/// Build embeddings for all wiki articles
pub async fn build_embeddings(ollama_url: &str) -> Result<WikiEmbeddings> {
    // First ensure the embedding model is available
    ensure_embedding_model(ollama_url).await?;

    let index = WikiIndex::load()?;
    let mut embeddings = WikiEmbeddings::default();

    let total = index.articles.len();
    let mut processed = 0;

    for (title, metadata) in &index.articles {
        // Create text to embed (title + summary)
        let text = format!("{}\n{}", title, metadata.summary);

        match get_embedding(ollama_url, &text).await {
            Ok(embedding) => {
                embeddings.embeddings.insert(title.clone(), embedding);
                processed += 1;

                if processed % 100 == 0 {
                    tracing::info!("Embedded {}/{} articles", processed, total);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to embed '{}': {}", title, e);
            }
        }
    }

    embeddings.built_at = Some(chrono::Utc::now().to_rfc3339());
    embeddings.save()?;

    tracing::info!("Built embeddings for {} articles", embeddings.embeddings.len());
    Ok(embeddings)
}

/// Ensure the embedding model is pulled
async fn ensure_embedding_model(ollama_url: &str) -> Result<()> {
    let client = reqwest::Client::new();

    // Check if model exists
    let response = client
        .post(format!("{}/api/show", ollama_url))
        .json(&serde_json::json!({ "name": EMBEDDING_MODEL }))
        .send()
        .await;

    if response.is_ok() && response.unwrap().status().is_success() {
        return Ok(());
    }

    // Pull the model
    tracing::info!("Pulling embedding model: {}", EMBEDDING_MODEL);

    let response = client
        .post(format!("{}/api/pull", ollama_url))
        .json(&serde_json::json!({ "name": EMBEDDING_MODEL }))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to pull embedding model");
    }

    Ok(())
}

/// Get embedding for text using Ollama
pub async fn get_embedding(ollama_url: &str, text: &str) -> Result<Vec<f32>> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/api/embeddings", ollama_url))
        .json(&serde_json::json!({
            "model": EMBEDDING_MODEL,
            "prompt": text
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Embedding request failed: {} - {}", status, body);
    }

    #[derive(Deserialize)]
    struct EmbeddingResponse {
        embedding: Vec<f32>,
    }

    let result: EmbeddingResponse = response.json().await?;
    Ok(result.embedding)
}

/// Compute cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

/// Search wiki using embeddings
pub async fn semantic_search(
    ollama_url: &str,
    query: &str,
    top_k: usize,
) -> Result<Vec<WikiSearchResult>> {
    // Get query embedding
    let query_embedding = get_embedding(ollama_url, query).await?;

    // Load embeddings
    let embeddings = WikiEmbeddings::load()?;

    // Search
    let matches = embeddings.search(&query_embedding, top_k);

    // Load index for article content
    let index = WikiIndex::load()?;

    // Build results
    let mut results = Vec::new();
    for (title, score) in matches {
        if let Some(article) = index.get_article(&title) {
            results.push(WikiSearchResult {
                article,
                score,
                relevant_section: None,
            });
        }
    }

    Ok(results)
}
