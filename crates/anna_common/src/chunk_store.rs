//! Chunk Store v7.36.0 - Bounded Knowledge Storage
//!
//! Hard limits to prevent token overflow:
//! - MAX_CHUNK_BYTES = 16,384 (16 KiB) per chunk
//! - MAX_DOC_BYTES = 512,000 (500 KiB) total per document
//!
//! Storage model:
//! - kdb/index.json: document index with metadata
//! - kdb/chunks/<id>/<nnn>.txt: plain text chunks
//! - kdb/facts/<id>.json: extracted structured facts
//!
//! Invariants:
//! - No single stored field exceeds MAX_CHUNK_BYTES
//! - Documents exceeding MAX_DOC_BYTES are truncated at ingest
//! - All content sanitized to plain text before storage
//! - Truncation recorded with truncated_at_ingest flag

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Hard Limits - v7.36.0
// ============================================================================

/// Maximum bytes per chunk (16 KiB)
pub const MAX_CHUNK_BYTES: usize = 16_384;

/// Maximum total bytes per document (500 KiB)
pub const MAX_DOC_BYTES: usize = 512_000;

/// Maximum chunks per document (derived: 500KB / 16KB = ~31)
pub const MAX_CHUNKS_PER_DOC: usize = 32;

/// Chunk store base path
pub const CHUNK_STORE_PATH: &str = "/var/lib/anna/kdb";

// ============================================================================
// Rendering Budgets - v7.36.0
// ============================================================================

/// Page budget for annactl status (very small)
pub const BUDGET_STATUS: usize = 4_000;

/// Page budget for annactl sw / annactl hw (moderate)
pub const BUDGET_OVERVIEW: usize = 16_000;

/// Page budget for annactl sw <name> / annactl hw <name> (detailed)
pub const BUDGET_DETAIL: usize = 32_000;

// ============================================================================
// Document Types
// ============================================================================

/// Type of document stored
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocType {
    /// Arch Wiki article
    WikiArticle,
    /// Man page
    ManPage,
    /// Package documentation
    PackageDoc,
    /// Config file content
    ConfigContent,
    /// Command help output
    HelpOutput,
}

impl DocType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocType::WikiArticle => "wiki",
            DocType::ManPage => "man",
            DocType::PackageDoc => "doc",
            DocType::ConfigContent => "config",
            DocType::HelpOutput => "help",
        }
    }
}

// ============================================================================
// Document Index Entry
// ============================================================================

/// Metadata for a stored document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocEntry {
    /// Unique document ID (e.g., "wiki:vim", "man:grep")
    pub id: String,
    /// Document type
    pub doc_type: DocType,
    /// Human-readable title
    pub title: String,
    /// Source references (file paths, URLs, commands)
    pub sources: Vec<String>,
    /// When the document was last updated
    pub updated_at: u64,
    /// Total byte count (original, before chunking)
    pub original_bytes: usize,
    /// Stored byte count (may be less if truncated)
    pub stored_bytes: usize,
    /// Number of chunks stored
    pub chunk_count: usize,
    /// Number of extracted facts
    pub fact_count: usize,
    /// Whether document was truncated at ingest
    pub truncated_at_ingest: bool,
    /// Byte offset where truncation occurred (if truncated)
    pub truncated_at_byte: Option<usize>,
    /// Table of contents (first N headings)
    pub toc: Vec<String>,
}

impl DocEntry {
    pub fn new(id: &str, doc_type: DocType, title: &str) -> Self {
        Self {
            id: id.to_string(),
            doc_type,
            title: title.to_string(),
            sources: Vec::new(),
            updated_at: now_epoch(),
            original_bytes: 0,
            stored_bytes: 0,
            chunk_count: 0,
            fact_count: 0,
            truncated_at_ingest: false,
            truncated_at_byte: None,
            toc: Vec::new(),
        }
    }
}

// ============================================================================
// Extracted Facts
// ============================================================================

/// Structured facts extracted from a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractedFacts {
    /// Config file paths mentioned
    pub config_paths: Vec<FactWithSource>,
    /// Service unit names mentioned
    pub service_units: Vec<FactWithSource>,
    /// Kernel modules mentioned
    pub kernel_modules: Vec<FactWithSource>,
    /// Package names mentioned
    pub packages: Vec<FactWithSource>,
    /// Environment variables mentioned
    pub env_vars: Vec<FactWithSource>,
    /// Default vs per-user locations
    pub location_hints: Vec<LocationHint>,
}

/// A fact with its source reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactWithSource {
    pub value: String,
    pub source_chunk: usize,
    pub line_hint: Option<usize>,
}

/// Location hint (default vs user config)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationHint {
    pub path: String,
    pub scope: LocationScope,
    pub is_default: bool,
    pub source_chunk: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocationScope {
    System,
    User,
}

// ============================================================================
// Document Index
// ============================================================================

/// The main document index
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocIndex {
    pub entries: HashMap<String, DocEntry>,
    pub created_at: u64,
    pub last_updated: u64,
}

impl DocIndex {
    pub fn new() -> Self {
        let now = now_epoch();
        Self {
            entries: HashMap::new(),
            created_at: now,
            last_updated: now,
        }
    }

    /// Load index from disk
    pub fn load() -> Self {
        let path = PathBuf::from(CHUNK_STORE_PATH).join("index.json");
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    /// Save index to disk
    pub fn save(&self) -> std::io::Result<()> {
        ensure_dirs()?;
        let path = PathBuf::from(CHUNK_STORE_PATH).join("index.json");
        let json = serde_json::to_string_pretty(self)?;
        crate::atomic_write(&path.to_string_lossy(), &json)
    }

    /// Get document by ID
    pub fn get(&self, id: &str) -> Option<&DocEntry> {
        self.entries.get(id)
    }

    /// Check if document exists
    pub fn has(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    /// Total document count
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Total chunks across all documents
    pub fn total_chunks(&self) -> usize {
        self.entries.values().map(|e| e.chunk_count).sum()
    }
}

// ============================================================================
// Chunk Store Operations
// ============================================================================

/// Store a document with chunking and fact extraction
pub fn store_document(
    id: &str,
    doc_type: DocType,
    title: &str,
    content: &str,
    sources: &[String],
) -> std::io::Result<DocEntry> {
    ensure_dirs()?;

    // Sanitize content to plain text
    let sanitized = sanitize_to_plain_text(content);
    let original_bytes = sanitized.len();

    // Determine if truncation needed
    let (final_content, truncated, truncated_at) = if original_bytes > MAX_DOC_BYTES {
        let truncated_content = &sanitized[..MAX_DOC_BYTES];
        // Find last complete line
        let last_newline = truncated_content.rfind('\n').unwrap_or(MAX_DOC_BYTES);
        (sanitized[..last_newline].to_string(), true, Some(last_newline))
    } else {
        (sanitized, false, None)
    };

    // Extract table of contents (first 20 headings)
    let toc = extract_toc(&final_content, 20);

    // Chunk the content
    let chunks = chunk_content(&final_content);
    let chunk_count = chunks.len();
    let stored_bytes: usize = chunks.iter().map(|c| c.len()).sum();

    // Write chunks to disk
    let chunk_dir = PathBuf::from(CHUNK_STORE_PATH).join("chunks").join(id);
    fs::create_dir_all(&chunk_dir)?;

    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_path = chunk_dir.join(format!("{:03}.txt", i));
        fs::write(&chunk_path, chunk)?;
    }

    // Extract facts
    let facts = extract_facts(&final_content);
    let fact_count = facts.config_paths.len()
        + facts.service_units.len()
        + facts.kernel_modules.len()
        + facts.packages.len()
        + facts.env_vars.len();

    // Write facts to disk
    let facts_path = PathBuf::from(CHUNK_STORE_PATH).join("facts").join(format!("{}.json", id));
    if let Some(parent) = facts_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let facts_json = serde_json::to_string_pretty(&facts)?;
    fs::write(&facts_path, facts_json)?;

    // Create entry
    let mut entry = DocEntry::new(id, doc_type, title);
    entry.sources = sources.to_vec();
    entry.original_bytes = original_bytes;
    entry.stored_bytes = stored_bytes;
    entry.chunk_count = chunk_count;
    entry.fact_count = fact_count;
    entry.truncated_at_ingest = truncated;
    entry.truncated_at_byte = truncated_at;
    entry.toc = toc;

    // Update index
    let mut index = DocIndex::load();
    index.entries.insert(id.to_string(), entry.clone());
    index.last_updated = now_epoch();
    index.save()?;

    Ok(entry)
}

/// Read chunks for a document
pub fn read_chunks(id: &str) -> Vec<String> {
    let chunk_dir = PathBuf::from(CHUNK_STORE_PATH).join("chunks").join(id);
    let mut chunks = Vec::new();

    if !chunk_dir.exists() {
        return chunks;
    }

    for i in 0..MAX_CHUNKS_PER_DOC {
        let chunk_path = chunk_dir.join(format!("{:03}.txt", i));
        if let Ok(content) = fs::read_to_string(&chunk_path) {
            chunks.push(content);
        } else {
            break;
        }
    }

    chunks
}

/// Read facts for a document
pub fn read_facts(id: &str) -> Option<ExtractedFacts> {
    let facts_path = PathBuf::from(CHUNK_STORE_PATH).join("facts").join(format!("{}.json", id));
    if let Ok(content) = fs::read_to_string(&facts_path) {
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

/// Delete a document and its chunks/facts
pub fn delete_document(id: &str) -> std::io::Result<()> {
    // Remove chunks directory
    let chunk_dir = PathBuf::from(CHUNK_STORE_PATH).join("chunks").join(id);
    if chunk_dir.exists() {
        fs::remove_dir_all(&chunk_dir)?;
    }

    // Remove facts file
    let facts_path = PathBuf::from(CHUNK_STORE_PATH).join("facts").join(format!("{}.json", id));
    if facts_path.exists() {
        fs::remove_file(&facts_path)?;
    }

    // Update index
    let mut index = DocIndex::load();
    index.entries.remove(id);
    index.last_updated = now_epoch();
    index.save()?;

    Ok(())
}

// ============================================================================
// Bounded Rendering
// ============================================================================

/// Render content with a budget, returning (rendered, overflow_info)
pub fn render_bounded(content: &str, budget: usize) -> (String, Option<OverflowInfo>) {
    if content.len() <= budget {
        return (content.to_string(), None);
    }

    // Find a good break point (line boundary)
    let truncate_at = content[..budget].rfind('\n').unwrap_or(budget);
    let rendered = content[..truncate_at].to_string();

    // Count remaining lines
    let remaining = &content[truncate_at..];
    let remaining_lines = remaining.lines().count();
    let remaining_bytes = remaining.len();

    let overflow = OverflowInfo {
        remaining_bytes,
        remaining_lines,
    };

    (rendered, Some(overflow))
}

/// Information about content that didn't fit in budget
#[derive(Debug, Clone)]
pub struct OverflowInfo {
    pub remaining_bytes: usize,
    pub remaining_lines: usize,
}

impl OverflowInfo {
    pub fn format_hint(&self) -> String {
        format!(
            "More available in knowledge store ({} lines, {} bytes)",
            self.remaining_lines, self.remaining_bytes
        )
    }
}

// ============================================================================
// Content Processing
// ============================================================================

/// Sanitize content to plain text (strip HTML, wiki markup, etc.)
pub fn sanitize_to_plain_text(content: &str) -> String {
    let mut result = content.to_string();

    // Remove HTML tags
    let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    result = tag_re.replace_all(&result, "").to_string();

    // Remove HTML entities
    result = result.replace("&nbsp;", " ");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&amp;", "&");
    result = result.replace("&quot;", "\"");
    result = result.replace("&#39;", "'");

    // Remove wiki markup [[link|text]] -> text
    let wiki_link_re = regex::Regex::new(r"\[\[([^|\]]+\|)?([^\]]+)\]\]").unwrap();
    result = wiki_link_re.replace_all(&result, "$2").to_string();

    // Remove wiki templates {{...}}
    let template_re = regex::Regex::new(r"\{\{[^}]+\}\}").unwrap();
    result = template_re.replace_all(&result, "").to_string();

    // Normalize whitespace
    let multi_space_re = regex::Regex::new(r"[ \t]+").unwrap();
    result = multi_space_re.replace_all(&result, " ").to_string();

    // Remove excessive blank lines
    let multi_newline_re = regex::Regex::new(r"\n{3,}").unwrap();
    result = multi_newline_re.replace_all(&result, "\n\n").to_string();

    result.trim().to_string()
}

/// Chunk content into pieces of MAX_CHUNK_BYTES or less
fn chunk_content(content: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for line in content.lines() {
        let line_with_newline = format!("{}\n", line);

        // If adding this line would exceed chunk size, start new chunk
        if current_chunk.len() + line_with_newline.len() > MAX_CHUNK_BYTES {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk);
                current_chunk = String::new();
            }

            // If single line exceeds chunk size, split it
            if line_with_newline.len() > MAX_CHUNK_BYTES {
                let mut remaining = line.to_string();
                while remaining.len() > MAX_CHUNK_BYTES - 1 {
                    chunks.push(remaining[..MAX_CHUNK_BYTES - 1].to_string());
                    remaining = remaining[MAX_CHUNK_BYTES - 1..].to_string();
                }
                current_chunk = format!("{}\n", remaining);
            } else {
                current_chunk.clone_from(&line_with_newline);
            }
        } else {
            current_chunk.push_str(&line_with_newline);
        }

        // Limit total chunks
        if chunks.len() >= MAX_CHUNKS_PER_DOC {
            break;
        }
    }

    if !current_chunk.is_empty() && chunks.len() < MAX_CHUNKS_PER_DOC {
        chunks.push(current_chunk);
    }

    chunks
}

/// Extract table of contents (headings) from content
fn extract_toc(content: &str, max_headings: usize) -> Vec<String> {
    let mut toc = Vec::new();

    // Look for markdown-style headings (# Heading) or wiki-style (== Heading ==)
    for line in content.lines() {
        let trimmed = line.trim();

        // Markdown heading
        if trimmed.starts_with('#') {
            let heading = trimmed.trim_start_matches('#').trim();
            if !heading.is_empty() {
                toc.push(heading.to_string());
            }
        }
        // Wiki heading
        else if trimmed.starts_with("==") && trimmed.ends_with("==") {
            let heading = trimmed.trim_matches('=').trim();
            if !heading.is_empty() {
                toc.push(heading.to_string());
            }
        }

        if toc.len() >= max_headings {
            break;
        }
    }

    toc
}

// ============================================================================
// Fact Extraction (Deterministic, No LLM)
// ============================================================================

/// Extract structured facts from plain text content
fn extract_facts(content: &str) -> ExtractedFacts {
    let mut facts = ExtractedFacts::default();

    // Config path patterns
    let config_re = regex::Regex::new(
        r"(?:^|\s|`)(/(?:etc|home/[^/]+|~)/[a-zA-Z0-9._/-]+(?:\.conf|\.cfg|\.ini|\.toml|\.yaml|\.yml|\.json|rc)?)"
    ).unwrap();

    // Service unit patterns
    let service_re = regex::Regex::new(
        r"\b([a-z][a-z0-9_-]*\.(?:service|socket|timer|mount|target))\b"
    ).unwrap();

    // Kernel module patterns
    let module_re = regex::Regex::new(
        r"(?:modprobe|insmod|rmmod|lsmod.*)\s+([a-z][a-z0-9_]+)"
    ).unwrap();

    // Package patterns (pacman -S, apt install, etc.)
    let package_re = regex::Regex::new(
        r"(?:pacman\s+-S[yu]*|apt\s+install|dnf\s+install|yay\s+-S)\s+([a-z][a-z0-9._+-]+)"
    ).unwrap();

    // Environment variable patterns
    let env_re = regex::Regex::new(
        r"\$\{?([A-Z][A-Z0-9_]+)\}?"
    ).unwrap();

    for (chunk_idx, chunk) in content.split('\n').collect::<Vec<_>>().chunks(100).enumerate() {
        let chunk_text = chunk.join("\n");

        for cap in config_re.captures_iter(&chunk_text) {
            if let Some(m) = cap.get(1) {
                facts.config_paths.push(FactWithSource {
                    value: m.as_str().to_string(),
                    source_chunk: chunk_idx,
                    line_hint: None,
                });
            }
        }

        for cap in service_re.captures_iter(&chunk_text) {
            if let Some(m) = cap.get(1) {
                facts.service_units.push(FactWithSource {
                    value: m.as_str().to_string(),
                    source_chunk: chunk_idx,
                    line_hint: None,
                });
            }
        }

        for cap in module_re.captures_iter(&chunk_text) {
            if let Some(m) = cap.get(1) {
                facts.kernel_modules.push(FactWithSource {
                    value: m.as_str().to_string(),
                    source_chunk: chunk_idx,
                    line_hint: None,
                });
            }
        }

        for cap in package_re.captures_iter(&chunk_text) {
            if let Some(m) = cap.get(1) {
                facts.packages.push(FactWithSource {
                    value: m.as_str().to_string(),
                    source_chunk: chunk_idx,
                    line_hint: None,
                });
            }
        }

        for cap in env_re.captures_iter(&chunk_text) {
            if let Some(m) = cap.get(1) {
                let var = m.as_str();
                // Filter common env vars
                if ["HOME", "XDG_CONFIG_HOME", "XDG_DATA_HOME", "PATH", "USER", "SHELL"].contains(&var) {
                    facts.env_vars.push(FactWithSource {
                        value: var.to_string(),
                        source_chunk: chunk_idx,
                        line_hint: None,
                    });
                }
            }
        }
    }

    // Deduplicate
    facts.config_paths.dedup_by(|a, b| a.value == b.value);
    facts.service_units.dedup_by(|a, b| a.value == b.value);
    facts.kernel_modules.dedup_by(|a, b| a.value == b.value);
    facts.packages.dedup_by(|a, b| a.value == b.value);
    facts.env_vars.dedup_by(|a, b| a.value == b.value);

    facts
}

// ============================================================================
// Helpers
// ============================================================================

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn ensure_dirs() -> std::io::Result<()> {
    let base = PathBuf::from(CHUNK_STORE_PATH);
    fs::create_dir_all(base.join("chunks"))?;
    fs::create_dir_all(base.join("facts"))?;
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html() {
        let html = "<p>Hello <b>world</b></p>";
        let plain = sanitize_to_plain_text(html);
        assert_eq!(plain, "Hello world");
    }

    #[test]
    fn test_sanitize_wiki_links() {
        let wiki = "See [[Vim|the editor]] for details";
        let plain = sanitize_to_plain_text(wiki);
        assert_eq!(plain, "See the editor for details");
    }

    #[test]
    fn test_chunk_small_content() {
        let content = "Line 1\nLine 2\nLine 3";
        let chunks = chunk_content(content);
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_chunk_respects_limit() {
        // Create content larger than one chunk
        let line = "x".repeat(1000);
        let content = format!("{}\n{}\n{}\n", line, line, line);
        let chunks = chunk_content(&content.repeat(10));
        for chunk in &chunks {
            assert!(chunk.len() <= MAX_CHUNK_BYTES, "Chunk exceeds max: {} > {}", chunk.len(), MAX_CHUNK_BYTES);
        }
    }

    #[test]
    fn test_extract_toc() {
        let content = "# Introduction\n\nSome text\n\n## Installation\n\nMore text\n\n### Dependencies";
        let toc = extract_toc(content, 10);
        assert_eq!(toc, vec!["Introduction", "Installation", "Dependencies"]);
    }

    #[test]
    fn test_render_bounded() {
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let (rendered, overflow) = render_bounded(content, 20);
        assert!(rendered.len() <= 20);
        assert!(overflow.is_some());
    }

    #[test]
    fn test_fact_extraction_config_paths() {
        let content = "Edit /etc/pacman.conf to configure";
        let facts = extract_facts(content);
        assert!(facts.config_paths.iter().any(|f| f.value.contains("pacman.conf")));
    }

    #[test]
    fn test_fact_extraction_services() {
        let content = "Enable systemd-networkd.service";
        let facts = extract_facts(content);
        assert!(facts.service_units.iter().any(|f| f.value == "systemd-networkd.service"));
    }
}
