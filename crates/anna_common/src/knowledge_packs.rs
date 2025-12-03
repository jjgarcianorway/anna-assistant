//! Knowledge Packs v0.0.19 - Offline Documentation Engine
//!
//! Local-first knowledge management for answering general questions:
//! - Knowledge packs stored under /var/lib/anna/knowledge_packs/
//! - SQLite FTS5 index for fast full-text search
//! - Evidence-backed answers with citations
//! - No network access - pure local documentation
//!
//! ## Pack Sources
//! - manpages: System man pages (/usr/share/man)
//! - package_docs: Package documentation (/usr/share/doc)
//! - project_docs: Anna's own documentation
//! - user_notes: User-added documentation
//!
//! ## Security
//! - Secrets hygiene: redaction applied to excerpts
//! - Sensitive paths blocked from indexing
//! - Trust levels for provenance tracking

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::redaction::{is_path_restricted, redact_evidence};

// =============================================================================
// Constants
// =============================================================================

/// Knowledge packs storage directory
pub const KNOWLEDGE_PACKS_DIR: &str = "/var/lib/anna/knowledge_packs";

/// Knowledge index database path
pub const KNOWLEDGE_INDEX_PATH: &str = "/var/lib/anna/knowledge_packs/index.db";

/// Evidence ID prefix for knowledge citations
pub const KNOWLEDGE_EVIDENCE_PREFIX: &str = "K";

/// Maximum excerpt length for search results
pub const MAX_EXCERPT_LENGTH: usize = 500;

/// Maximum documents to return from search
pub const DEFAULT_TOP_K: usize = 5;

// =============================================================================
// Pack Types and Schema
// =============================================================================

/// Source type for a knowledge pack
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackSource {
    /// System man pages
    Manpages,
    /// Package documentation (/usr/share/doc)
    PackageDocs,
    /// Anna project documentation
    ProjectDocs,
    /// User-added local markdown/text
    UserNotes,
    /// Cached ArchWiki (future, opt-in)
    ArchwikiCache,
    /// Local markdown files
    LocalMarkdown,
}

impl PackSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackSource::Manpages => "manpages",
            PackSource::PackageDocs => "package_docs",
            PackSource::ProjectDocs => "project_docs",
            PackSource::UserNotes => "user_notes",
            PackSource::ArchwikiCache => "archwiki_cache",
            PackSource::LocalMarkdown => "local_markdown",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PackSource::Manpages => "Man Pages",
            PackSource::PackageDocs => "Package Docs",
            PackSource::ProjectDocs => "Project Docs",
            PackSource::UserNotes => "User Notes",
            PackSource::ArchwikiCache => "ArchWiki Cache",
            PackSource::LocalMarkdown => "Local Markdown",
        }
    }
}

/// Trust level for knowledge provenance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    /// Official system documentation (man pages, package docs)
    Official,
    /// Local project documentation
    Local,
    /// User-provided content
    User,
}

impl TrustLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrustLevel::Official => "official",
            TrustLevel::Local => "local",
            TrustLevel::User => "user",
        }
    }
}

/// Retention policy for a knowledge pack
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionPolicy {
    /// Keep forever
    Permanent,
    /// Re-index periodically (e.g., after package updates)
    RefreshOnUpdate,
    /// User-managed (explicit delete)
    Manual,
}

/// Metadata for a knowledge pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePack {
    /// Unique pack identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Source type
    pub source: PackSource,
    /// Trust level
    pub trust: TrustLevel,
    /// Retention policy
    pub retention: RetentionPolicy,
    /// Source paths (for local sources)
    pub source_paths: Vec<String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last index timestamp
    pub last_indexed_at: u64,
    /// Number of documents indexed
    pub document_count: usize,
    /// Index size in bytes (approximate)
    pub index_size_bytes: u64,
    /// Description
    pub description: String,
    /// Whether pack is enabled
    pub enabled: bool,
}

/// A document in a knowledge pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocument {
    /// Document ID
    pub id: i64,
    /// Pack ID this document belongs to
    pub pack_id: String,
    /// Document title
    pub title: String,
    /// Full text content
    pub content: String,
    /// Source path or URL
    pub source_path: String,
    /// Document type (man, md, txt, etc.)
    pub doc_type: String,
    /// Keywords/tags
    pub keywords: Vec<String>,
    /// Indexing timestamp
    pub indexed_at: u64,
}

/// Search result with excerpt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub doc_id: i64,
    /// Evidence ID for citation (K1, K2, ...)
    pub evidence_id: String,
    /// Document title
    pub title: String,
    /// Pack ID
    pub pack_id: String,
    /// Pack name
    pub pack_name: String,
    /// Source path
    pub source_path: String,
    /// Trust level
    pub trust: TrustLevel,
    /// Matching excerpt (redacted)
    pub excerpt: String,
    /// Relevance score from FTS5
    pub score: f64,
    /// Keywords that matched
    pub matched_keywords: Vec<String>,
}

/// Knowledge pack statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStats {
    /// Total number of packs
    pub pack_count: usize,
    /// Total number of documents
    pub document_count: usize,
    /// Total index size in bytes
    pub total_size_bytes: u64,
    /// Last index time
    pub last_indexed_at: Option<u64>,
    /// Packs by source type
    pub packs_by_source: HashMap<String, usize>,
    /// Top packs by query count
    pub top_packs: Vec<(String, usize)>,
}

// =============================================================================
// Knowledge Index Manager
// =============================================================================

/// Manager for knowledge pack indexing and search
pub struct KnowledgeIndex {
    conn: Connection,
}

impl KnowledgeIndex {
    /// Open or create the knowledge index database
    pub fn open() -> Result<Self> {
        // Ensure directory exists
        let dir = Path::new(KNOWLEDGE_PACKS_DIR);
        if !dir.exists() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create {}", KNOWLEDGE_PACKS_DIR))?;
        }

        let conn = Connection::open(KNOWLEDGE_INDEX_PATH)
            .with_context(|| format!("Failed to open {}", KNOWLEDGE_INDEX_PATH))?;

        let index = Self { conn };
        index.init_schema()?;
        Ok(index)
    }

    /// Open with a custom path (for testing)
    pub fn open_with_path(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open {:?}", path))?;

        let index = Self { conn };
        index.init_schema()?;
        Ok(index)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            -- Knowledge packs metadata table
            CREATE TABLE IF NOT EXISTS packs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                source TEXT NOT NULL,
                trust TEXT NOT NULL,
                retention TEXT NOT NULL,
                source_paths TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_indexed_at INTEGER NOT NULL,
                document_count INTEGER NOT NULL DEFAULT 0,
                index_size_bytes INTEGER NOT NULL DEFAULT 0,
                description TEXT NOT NULL DEFAULT '',
                enabled INTEGER NOT NULL DEFAULT 1
            );

            -- Documents table
            CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pack_id TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                source_path TEXT NOT NULL,
                doc_type TEXT NOT NULL,
                keywords TEXT NOT NULL DEFAULT '',
                indexed_at INTEGER NOT NULL,
                FOREIGN KEY (pack_id) REFERENCES packs(id) ON DELETE CASCADE
            );

            -- FTS5 virtual table for full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
                title,
                content,
                keywords,
                content=documents,
                content_rowid=id
            );

            -- Triggers to keep FTS index in sync
            CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
                INSERT INTO documents_fts(rowid, title, content, keywords)
                VALUES (new.id, new.title, new.content, new.keywords);
            END;

            CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, title, content, keywords)
                VALUES ('delete', old.id, old.title, old.content, old.keywords);
            END;

            CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, title, content, keywords)
                VALUES ('delete', old.id, old.title, old.content, old.keywords);
                INSERT INTO documents_fts(rowid, title, content, keywords)
                VALUES (new.id, new.title, new.content, new.keywords);
            END;

            -- Query statistics table
            CREATE TABLE IF NOT EXISTS query_stats (
                pack_id TEXT NOT NULL,
                query_count INTEGER NOT NULL DEFAULT 0,
                last_query_at INTEGER,
                PRIMARY KEY (pack_id),
                FOREIGN KEY (pack_id) REFERENCES packs(id) ON DELETE CASCADE
            );

            -- Index for pack lookups
            CREATE INDEX IF NOT EXISTS idx_documents_pack ON documents(pack_id);
            "#,
        )?;

        Ok(())
    }

    /// Register a new knowledge pack
    pub fn register_pack(&self, pack: &KnowledgePack) -> Result<()> {
        let source_paths_json = serde_json::to_string(&pack.source_paths)?;

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO packs
            (id, name, source, trust, retention, source_paths, created_at,
             last_indexed_at, document_count, index_size_bytes, description, enabled)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                pack.id,
                pack.name,
                pack.source.as_str(),
                pack.trust.as_str(),
                format!("{:?}", pack.retention).to_lowercase(),
                source_paths_json,
                pack.created_at,
                pack.last_indexed_at,
                pack.document_count,
                pack.index_size_bytes,
                pack.description,
                pack.enabled as i32,
            ],
        )?;

        // Initialize query stats
        self.conn.execute(
            "INSERT OR IGNORE INTO query_stats (pack_id, query_count) VALUES (?1, 0)",
            params![pack.id],
        )?;

        Ok(())
    }

    /// Get a pack by ID
    pub fn get_pack(&self, pack_id: &str) -> Result<Option<KnowledgePack>> {
        let result = self.conn.query_row(
            "SELECT * FROM packs WHERE id = ?1",
            params![pack_id],
            |row| {
                let source_str: String = row.get(2)?;
                let trust_str: String = row.get(3)?;
                let retention_str: String = row.get(4)?;
                let source_paths_json: String = row.get(5)?;

                Ok(KnowledgePack {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    source: match source_str.as_str() {
                        "manpages" => PackSource::Manpages,
                        "package_docs" => PackSource::PackageDocs,
                        "project_docs" => PackSource::ProjectDocs,
                        "user_notes" => PackSource::UserNotes,
                        "archwiki_cache" => PackSource::ArchwikiCache,
                        _ => PackSource::LocalMarkdown,
                    },
                    trust: match trust_str.as_str() {
                        "official" => TrustLevel::Official,
                        "local" => TrustLevel::Local,
                        _ => TrustLevel::User,
                    },
                    retention: match retention_str.as_str() {
                        "permanent" => RetentionPolicy::Permanent,
                        "refreshonupdate" => RetentionPolicy::RefreshOnUpdate,
                        _ => RetentionPolicy::Manual,
                    },
                    source_paths: serde_json::from_str(&source_paths_json).unwrap_or_default(),
                    created_at: row.get(6)?,
                    last_indexed_at: row.get(7)?,
                    document_count: row.get::<_, i64>(8)? as usize,
                    index_size_bytes: row.get::<_, i64>(9)? as u64,
                    description: row.get(10)?,
                    enabled: row.get::<_, i32>(11)? != 0,
                })
            },
        ).optional()?;

        Ok(result)
    }

    /// List all packs
    pub fn list_packs(&self) -> Result<Vec<KnowledgePack>> {
        let mut stmt = self.conn.prepare("SELECT * FROM packs ORDER BY name")?;
        let packs = stmt.query_map([], |row| {
            let source_str: String = row.get(2)?;
            let trust_str: String = row.get(3)?;
            let retention_str: String = row.get(4)?;
            let source_paths_json: String = row.get(5)?;

            Ok(KnowledgePack {
                id: row.get(0)?,
                name: row.get(1)?,
                source: match source_str.as_str() {
                    "manpages" => PackSource::Manpages,
                    "package_docs" => PackSource::PackageDocs,
                    "project_docs" => PackSource::ProjectDocs,
                    "user_notes" => PackSource::UserNotes,
                    "archwiki_cache" => PackSource::ArchwikiCache,
                    _ => PackSource::LocalMarkdown,
                },
                trust: match trust_str.as_str() {
                    "official" => TrustLevel::Official,
                    "local" => TrustLevel::Local,
                    _ => TrustLevel::User,
                },
                retention: match retention_str.as_str() {
                    "permanent" => RetentionPolicy::Permanent,
                    "refreshonupdate" => RetentionPolicy::RefreshOnUpdate,
                    _ => RetentionPolicy::Manual,
                },
                source_paths: serde_json::from_str(&source_paths_json).unwrap_or_default(),
                created_at: row.get(6)?,
                last_indexed_at: row.get(7)?,
                document_count: row.get::<_, i64>(8)? as usize,
                index_size_bytes: row.get::<_, i64>(9)? as u64,
                description: row.get(10)?,
                enabled: row.get::<_, i32>(11)? != 0,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(packs)
    }

    /// Add a document to a pack
    pub fn add_document(&self, doc: &KnowledgeDocument) -> Result<i64> {
        // Check if path is restricted
        if is_path_restricted(&doc.source_path) {
            return Err(anyhow::anyhow!(
                "Path is restricted by security policy: {}",
                doc.source_path
            ));
        }

        let keywords_str = doc.keywords.join(" ");

        self.conn.execute(
            r#"
            INSERT INTO documents (pack_id, title, content, source_path, doc_type, keywords, indexed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                doc.pack_id,
                doc.title,
                doc.content,
                doc.source_path,
                doc.doc_type,
                keywords_str,
                doc.indexed_at,
            ],
        )?;

        let doc_id = self.conn.last_insert_rowid();

        // Update pack stats
        self.conn.execute(
            r#"
            UPDATE packs SET
                document_count = (SELECT COUNT(*) FROM documents WHERE pack_id = ?1),
                last_indexed_at = ?2
            WHERE id = ?1
            "#,
            params![doc.pack_id, doc.indexed_at],
        )?;

        Ok(doc_id)
    }

    /// Search for documents matching a query
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        // Escape special FTS5 characters and build query
        let fts_query = sanitize_fts_query(query);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                d.id,
                d.title,
                d.pack_id,
                p.name as pack_name,
                d.source_path,
                p.trust,
                snippet(documents_fts, 1, '>>>', '<<<', '...', 64) as excerpt,
                bm25(documents_fts) as score,
                d.keywords
            FROM documents_fts
            JOIN documents d ON documents_fts.rowid = d.id
            JOIN packs p ON d.pack_id = p.id
            WHERE p.enabled = 1 AND documents_fts MATCH ?1
            ORDER BY score
            LIMIT ?2
            "#,
        )?;

        let mut results = Vec::new();
        let mut evidence_counter = 1;

        let rows = stmt.query_map(params![fts_query, top_k as i64], |row| {
            let trust_str: String = row.get(5)?;
            let keywords_str: String = row.get(8)?;

            Ok((
                row.get::<_, i64>(0)?,       // doc_id
                row.get::<_, String>(1)?,    // title
                row.get::<_, String>(2)?,    // pack_id
                row.get::<_, String>(3)?,    // pack_name
                row.get::<_, String>(4)?,    // source_path
                trust_str,
                row.get::<_, String>(6)?,    // excerpt
                row.get::<_, f64>(7)?,       // score
                keywords_str,
            ))
        })?;

        for row in rows {
            let (doc_id, title, pack_id, pack_name, source_path, trust_str, excerpt, score, keywords_str) = row?;

            // Apply redaction to excerpt
            let redacted_excerpt = redact_evidence(&excerpt, Some(&source_path))
                .unwrap_or_else(|e| e);

            // Update query stats for this pack
            self.conn.execute(
                r#"
                INSERT INTO query_stats (pack_id, query_count, last_query_at)
                VALUES (?1, 1, ?2)
                ON CONFLICT(pack_id) DO UPDATE SET
                    query_count = query_count + 1,
                    last_query_at = ?2
                "#,
                params![pack_id, current_timestamp()],
            )?;

            results.push(SearchResult {
                doc_id,
                evidence_id: format!("{}{}", KNOWLEDGE_EVIDENCE_PREFIX, evidence_counter),
                title,
                pack_id,
                pack_name,
                source_path,
                trust: match trust_str.as_str() {
                    "official" => TrustLevel::Official,
                    "local" => TrustLevel::Local,
                    _ => TrustLevel::User,
                },
                excerpt: redacted_excerpt,
                score: score.abs(), // BM25 returns negative scores
                matched_keywords: keywords_str.split_whitespace()
                    .filter(|kw| query.to_lowercase().contains(&kw.to_lowercase()))
                    .map(String::from)
                    .collect(),
            });

            evidence_counter += 1;
        }

        Ok(results)
    }

    /// Get knowledge statistics
    pub fn get_stats(&self) -> Result<KnowledgeStats> {
        let pack_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM packs",
            [],
            |row| row.get::<_, i64>(0),
        )? as usize;

        let document_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM documents",
            [],
            |row| row.get::<_, i64>(0),
        )? as usize;

        let total_size_bytes: u64 = self.conn.query_row(
            "SELECT COALESCE(SUM(index_size_bytes), 0) FROM packs",
            [],
            |row| row.get::<_, i64>(0),
        )? as u64;

        let last_indexed_at: Option<u64> = self.conn.query_row(
            "SELECT MAX(last_indexed_at) FROM packs",
            [],
            |row| row.get::<_, Option<i64>>(0),
        )?.map(|v| v as u64);

        // Packs by source
        let mut packs_by_source = HashMap::new();
        let mut stmt = self.conn.prepare(
            "SELECT source, COUNT(*) FROM packs GROUP BY source"
        )?;
        let source_counts = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;
        for result in source_counts {
            let (source, count) = result?;
            packs_by_source.insert(source, count);
        }

        // Top packs by query count
        let mut stmt = self.conn.prepare(
            r#"
            SELECT p.name, COALESCE(qs.query_count, 0) as queries
            FROM packs p
            LEFT JOIN query_stats qs ON p.id = qs.pack_id
            ORDER BY queries DESC
            LIMIT 5
            "#
        )?;
        let top_packs: Vec<(String, usize)> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?.filter_map(|r| r.ok()).collect();

        Ok(KnowledgeStats {
            pack_count,
            document_count,
            total_size_bytes,
            last_indexed_at,
            packs_by_source,
            top_packs,
        })
    }

    /// Delete a pack and all its documents
    pub fn delete_pack(&self, pack_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM documents WHERE pack_id = ?1", params![pack_id])?;
        self.conn.execute("DELETE FROM query_stats WHERE pack_id = ?1", params![pack_id])?;
        self.conn.execute("DELETE FROM packs WHERE id = ?1", params![pack_id])?;
        Ok(())
    }

    /// Clear all documents from a pack (for re-indexing)
    pub fn clear_pack_documents(&self, pack_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM documents WHERE pack_id = ?1", params![pack_id])?;
        self.conn.execute(
            "UPDATE packs SET document_count = 0 WHERE id = ?1",
            params![pack_id],
        )?;
        Ok(())
    }

    /// Update index size for a pack
    pub fn update_pack_size(&self, pack_id: &str) -> Result<()> {
        // Estimate size based on content length
        let size: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(LENGTH(content) + LENGTH(title)), 0) FROM documents WHERE pack_id = ?1",
            params![pack_id],
            |row| row.get(0),
        )?;

        self.conn.execute(
            "UPDATE packs SET index_size_bytes = ?1 WHERE id = ?2",
            params![size, pack_id],
        )?;

        Ok(())
    }
}

// =============================================================================
// Document Ingestion
// =============================================================================

/// Ingest man pages into a knowledge pack
pub fn ingest_manpages(index: &KnowledgeIndex, limit: Option<usize>) -> Result<usize> {
    let pack_id = "system-manpages";
    let timestamp = current_timestamp();

    // Check if pack exists, create if not
    if index.get_pack(pack_id)?.is_none() {
        let pack = KnowledgePack {
            id: pack_id.to_string(),
            name: "System Man Pages".to_string(),
            source: PackSource::Manpages,
            trust: TrustLevel::Official,
            retention: RetentionPolicy::RefreshOnUpdate,
            source_paths: vec!["/usr/share/man".to_string()],
            created_at: timestamp,
            last_indexed_at: timestamp,
            document_count: 0,
            index_size_bytes: 0,
            description: "Man pages for installed system commands".to_string(),
            enabled: true,
        };
        index.register_pack(&pack)?;
    }

    // Clear existing documents for fresh index
    index.clear_pack_documents(pack_id)?;

    // Get list of man pages using apropos
    let output = Command::new("apropos")
        .arg(".")
        .output()
        .context("Failed to run apropos")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("apropos command failed"));
    }

    let apropos_output = String::from_utf8_lossy(&output.stdout);
    let mut count = 0;
    let max_count = limit.unwrap_or(usize::MAX);

    for line in apropos_output.lines() {
        if count >= max_count {
            break;
        }

        // Parse apropos output: "command (section) - description"
        let parts: Vec<&str> = line.splitn(2, " - ").collect();
        if parts.len() != 2 {
            continue;
        }

        let cmd_section = parts[0].trim();
        let description = parts[1].trim();

        // Extract command name (before the section number)
        let cmd_name = cmd_section.split_whitespace().next().unwrap_or("");
        if cmd_name.is_empty() {
            continue;
        }

        // Get man page content
        let man_output = Command::new("man")
            .args(["--no-hyphenation", "--no-justification", cmd_name])
            .env("MANWIDTH", "80")
            .output();

        if let Ok(output) = man_output {
            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);

                // Skip if path would be restricted
                let source_path = format!("man:{}", cmd_name);

                let doc = KnowledgeDocument {
                    id: 0, // Will be set by DB
                    pack_id: pack_id.to_string(),
                    title: format!("{} - {}", cmd_name, description),
                    content: content.to_string(),
                    source_path,
                    doc_type: "man".to_string(),
                    keywords: vec![cmd_name.to_string()],
                    indexed_at: timestamp,
                };

                if index.add_document(&doc).is_ok() {
                    count += 1;
                }
            }
        }
    }

    index.update_pack_size(pack_id)?;
    Ok(count)
}

/// Ingest package documentation from /usr/share/doc
pub fn ingest_package_docs(index: &KnowledgeIndex, limit: Option<usize>) -> Result<usize> {
    let pack_id = "package-docs";
    let timestamp = current_timestamp();

    // Check if pack exists, create if not
    if index.get_pack(pack_id)?.is_none() {
        let pack = KnowledgePack {
            id: pack_id.to_string(),
            name: "Package Documentation".to_string(),
            source: PackSource::PackageDocs,
            trust: TrustLevel::Official,
            retention: RetentionPolicy::RefreshOnUpdate,
            source_paths: vec!["/usr/share/doc".to_string()],
            created_at: timestamp,
            last_indexed_at: timestamp,
            document_count: 0,
            index_size_bytes: 0,
            description: "README and documentation files from installed packages".to_string(),
            enabled: true,
        };
        index.register_pack(&pack)?;
    }

    index.clear_pack_documents(pack_id)?;

    let doc_dir = Path::new("/usr/share/doc");
    if !doc_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let max_count = limit.unwrap_or(usize::MAX);

    // Iterate through package directories
    if let Ok(entries) = fs::read_dir(doc_dir) {
        for entry in entries.flatten() {
            if count >= max_count {
                break;
            }

            let pkg_path = entry.path();
            if !pkg_path.is_dir() {
                continue;
            }

            let pkg_name = entry.file_name().to_string_lossy().to_string();

            // Look for README files
            let readme_patterns = ["README", "README.md", "README.txt", "readme.md"];

            for pattern in readme_patterns {
                let readme_path = pkg_path.join(pattern);
                if readme_path.exists() && readme_path.is_file() {
                    // Skip restricted paths
                    let path_str = readme_path.to_string_lossy().to_string();
                    if is_path_restricted(&path_str) {
                        continue;
                    }

                    // Read content (limit to 50KB)
                    if let Ok(content) = fs::read_to_string(&readme_path) {
                        let content = if content.len() > 50_000 {
                            content[..50_000].to_string() + "\n[truncated]"
                        } else {
                            content
                        };

                        let doc = KnowledgeDocument {
                            id: 0,
                            pack_id: pack_id.to_string(),
                            title: format!("{} - {}", pkg_name, pattern),
                            content,
                            source_path: path_str,
                            doc_type: if pattern.ends_with(".md") { "markdown" } else { "text" }.to_string(),
                            keywords: vec![pkg_name.clone()],
                            indexed_at: timestamp,
                        };

                        if index.add_document(&doc).is_ok() {
                            count += 1;
                        }
                    }
                    break; // Only index first README found
                }
            }
        }
    }

    index.update_pack_size(pack_id)?;
    Ok(count)
}

/// Ingest Anna project documentation
pub fn ingest_project_docs(index: &KnowledgeIndex, project_root: &Path) -> Result<usize> {
    let pack_id = "anna-project";
    let timestamp = current_timestamp();

    // Check if pack exists, create if not
    if index.get_pack(pack_id)?.is_none() {
        let pack = KnowledgePack {
            id: pack_id.to_string(),
            name: "Anna Project Docs".to_string(),
            source: PackSource::ProjectDocs,
            trust: TrustLevel::Local,
            retention: RetentionPolicy::Manual,
            source_paths: vec![project_root.to_string_lossy().to_string()],
            created_at: timestamp,
            last_indexed_at: timestamp,
            document_count: 0,
            index_size_bytes: 0,
            description: "Anna Assistant project documentation".to_string(),
            enabled: true,
        };
        index.register_pack(&pack)?;
    }

    index.clear_pack_documents(pack_id)?;

    let mut count = 0;

    // Index specific project docs
    let doc_files = ["README.md", "CLAUDE.md", "TODO.md", "RELEASE_NOTES.md"];

    for doc_file in doc_files {
        let doc_path = project_root.join(doc_file);
        if doc_path.exists() {
            let path_str = doc_path.to_string_lossy().to_string();

            if let Ok(content) = fs::read_to_string(&doc_path) {
                let doc = KnowledgeDocument {
                    id: 0,
                    pack_id: pack_id.to_string(),
                    title: doc_file.to_string(),
                    content,
                    source_path: path_str,
                    doc_type: "markdown".to_string(),
                    keywords: vec!["anna".to_string(), "assistant".to_string()],
                    indexed_at: timestamp,
                };

                if index.add_document(&doc).is_ok() {
                    count += 1;
                }
            }
        }
    }

    index.update_pack_size(pack_id)?;
    Ok(count)
}

/// Ingest a single markdown file as user notes
pub fn ingest_user_note(
    index: &KnowledgeIndex,
    path: &Path,
    title: Option<&str>,
    keywords: Vec<String>,
) -> Result<i64> {
    let pack_id = "user-notes";
    let timestamp = current_timestamp();

    // Check if pack exists, create if not
    if index.get_pack(pack_id)?.is_none() {
        let pack = KnowledgePack {
            id: pack_id.to_string(),
            name: "User Notes".to_string(),
            source: PackSource::UserNotes,
            trust: TrustLevel::User,
            retention: RetentionPolicy::Manual,
            source_paths: vec![],
            created_at: timestamp,
            last_indexed_at: timestamp,
            document_count: 0,
            index_size_bytes: 0,
            description: "User-added documentation and notes".to_string(),
            enabled: true,
        };
        index.register_pack(&pack)?;
    }

    let path_str = path.to_string_lossy().to_string();

    // Check if path is restricted
    if is_path_restricted(&path_str) {
        return Err(anyhow::anyhow!(
            "Path is restricted by security policy: {}",
            path_str
        ));
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path_str))?;

    let doc_title = title.map(String::from).unwrap_or_else(|| {
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    });

    let doc = KnowledgeDocument {
        id: 0,
        pack_id: pack_id.to_string(),
        title: doc_title,
        content,
        source_path: path_str,
        doc_type: "markdown".to_string(),
        keywords,
        indexed_at: timestamp,
    };

    let doc_id = index.add_document(&doc)?;
    index.update_pack_size(pack_id)?;

    Ok(doc_id)
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Sanitize a query for FTS5
fn sanitize_fts_query(query: &str) -> String {
    // Remove special FTS5 operators and escape quotes
    let mut sanitized = query
        .replace('"', "")
        .replace('*', "")
        .replace('(', "")
        .replace(')', "")
        .replace(':', " ");

    // If query has multiple words, join with OR for broader matching
    let words: Vec<&str> = sanitized.split_whitespace().collect();
    if words.len() > 1 {
        sanitized = words.join(" OR ");
    }

    sanitized
}

/// Generate a unique evidence ID for knowledge citations
pub fn generate_knowledge_evidence_id() -> String {
    let timestamp = current_timestamp();
    format!("{}{}", KNOWLEDGE_EVIDENCE_PREFIX, timestamp % 100000)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_index() -> (KnowledgeIndex, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_index.db");
        let index = KnowledgeIndex::open_with_path(&db_path).unwrap();
        (index, temp_dir)  // Return both to keep temp_dir alive
    }

    #[test]
    fn test_register_and_get_pack() {
        let (index, _temp_dir) = create_test_index();

        let pack = KnowledgePack {
            id: "test-pack".to_string(),
            name: "Test Pack".to_string(),
            source: PackSource::UserNotes,
            trust: TrustLevel::User,
            retention: RetentionPolicy::Manual,
            source_paths: vec!["/tmp/test".to_string()],
            created_at: 1000,
            last_indexed_at: 1000,
            document_count: 0,
            index_size_bytes: 0,
            description: "Test description".to_string(),
            enabled: true,
        };

        index.register_pack(&pack).unwrap();

        let retrieved = index.get_pack("test-pack").unwrap().unwrap();
        assert_eq!(retrieved.name, "Test Pack");
        assert_eq!(retrieved.source, PackSource::UserNotes);
        assert_eq!(retrieved.trust, TrustLevel::User);
    }

    #[test]
    fn test_add_and_search_document() {
        let (index, _temp_dir) = create_test_index();

        // Create pack
        let pack = KnowledgePack {
            id: "test-pack".to_string(),
            name: "Test Pack".to_string(),
            source: PackSource::UserNotes,
            trust: TrustLevel::User,
            retention: RetentionPolicy::Manual,
            source_paths: vec![],
            created_at: 1000,
            last_indexed_at: 1000,
            document_count: 0,
            index_size_bytes: 0,
            description: "Test".to_string(),
            enabled: true,
        };
        index.register_pack(&pack).unwrap();

        // Add document
        let doc = KnowledgeDocument {
            id: 0,
            pack_id: "test-pack".to_string(),
            title: "Vim Configuration Guide".to_string(),
            content: "To enable syntax highlighting in Vim, add 'syntax on' to your .vimrc file.".to_string(),
            source_path: "/tmp/vim-guide.md".to_string(),
            doc_type: "markdown".to_string(),
            keywords: vec!["vim".to_string(), "editor".to_string()],
            indexed_at: 1000,
        };
        index.add_document(&doc).unwrap();

        // Search
        let results = index.search("syntax highlighting vim", 5).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].title.contains("Vim"));
    }

    #[test]
    fn test_search_returns_evidence_ids() {
        let (index, _temp_dir) = create_test_index();

        let pack = KnowledgePack {
            id: "test-pack".to_string(),
            name: "Test".to_string(),
            source: PackSource::UserNotes,
            trust: TrustLevel::User,
            retention: RetentionPolicy::Manual,
            source_paths: vec![],
            created_at: 1000,
            last_indexed_at: 1000,
            document_count: 0,
            index_size_bytes: 0,
            description: "Test".to_string(),
            enabled: true,
        };
        index.register_pack(&pack).unwrap();

        let doc = KnowledgeDocument {
            id: 0,
            pack_id: "test-pack".to_string(),
            title: "Test Doc".to_string(),
            content: "This is test content about configuration.".to_string(),
            source_path: "/tmp/test.md".to_string(),
            doc_type: "markdown".to_string(),
            keywords: vec!["test".to_string()],
            indexed_at: 1000,
        };
        index.add_document(&doc).unwrap();

        let results = index.search("configuration", 5).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].evidence_id.starts_with("K"));
    }

    #[test]
    fn test_get_stats() {
        let (index, _temp_dir) = create_test_index();

        let pack = KnowledgePack {
            id: "test-pack".to_string(),
            name: "Test".to_string(),
            source: PackSource::Manpages,
            trust: TrustLevel::Official,
            retention: RetentionPolicy::Permanent,
            source_paths: vec![],
            created_at: 1000,
            last_indexed_at: 1000,
            document_count: 0,
            index_size_bytes: 0,
            description: "Test".to_string(),
            enabled: true,
        };
        index.register_pack(&pack).unwrap();

        let stats = index.get_stats().unwrap();
        assert_eq!(stats.pack_count, 1);
        assert_eq!(stats.packs_by_source.get("manpages"), Some(&1));
    }

    #[test]
    fn test_delete_pack() {
        let (index, _temp_dir) = create_test_index();

        let pack = KnowledgePack {
            id: "to-delete".to_string(),
            name: "Delete Me".to_string(),
            source: PackSource::UserNotes,
            trust: TrustLevel::User,
            retention: RetentionPolicy::Manual,
            source_paths: vec![],
            created_at: 1000,
            last_indexed_at: 1000,
            document_count: 0,
            index_size_bytes: 0,
            description: "Test".to_string(),
            enabled: true,
        };
        index.register_pack(&pack).unwrap();

        assert!(index.get_pack("to-delete").unwrap().is_some());

        index.delete_pack("to-delete").unwrap();

        assert!(index.get_pack("to-delete").unwrap().is_none());
    }

    #[test]
    fn test_sanitize_fts_query() {
        assert_eq!(sanitize_fts_query("simple"), "simple");
        assert_eq!(sanitize_fts_query("vim syntax"), "vim OR syntax");
        assert_eq!(sanitize_fts_query("test \"quoted\""), "test OR quoted");  // quotes removed, words joined with OR
        assert_eq!(sanitize_fts_query("test*"), "test");
    }

    #[test]
    fn test_pack_source_display() {
        assert_eq!(PackSource::Manpages.display_name(), "Man Pages");
        assert_eq!(PackSource::PackageDocs.display_name(), "Package Docs");
        assert_eq!(PackSource::UserNotes.as_str(), "user_notes");
    }

    #[test]
    fn test_trust_level() {
        assert_eq!(TrustLevel::Official.as_str(), "official");
        assert_eq!(TrustLevel::Local.as_str(), "local");
        assert_eq!(TrustLevel::User.as_str(), "user");
    }

    #[test]
    fn test_list_packs() {
        let (index, _temp_dir) = create_test_index();

        for i in 0..3 {
            let pack = KnowledgePack {
                id: format!("pack-{}", i),
                name: format!("Pack {}", i),
                source: PackSource::UserNotes,
                trust: TrustLevel::User,
                retention: RetentionPolicy::Manual,
                source_paths: vec![],
                created_at: 1000,
                last_indexed_at: 1000,
                document_count: 0,
                index_size_bytes: 0,
                description: "Test".to_string(),
                enabled: true,
            };
            index.register_pack(&pack).unwrap();
        }

        let packs = index.list_packs().unwrap();
        assert_eq!(packs.len(), 3);
    }

    #[test]
    fn test_multiple_search_results() {
        let (index, _temp_dir) = create_test_index();

        let pack = KnowledgePack {
            id: "test-pack".to_string(),
            name: "Test".to_string(),
            source: PackSource::UserNotes,
            trust: TrustLevel::User,
            retention: RetentionPolicy::Manual,
            source_paths: vec![],
            created_at: 1000,
            last_indexed_at: 1000,
            document_count: 0,
            index_size_bytes: 0,
            description: "Test".to_string(),
            enabled: true,
        };
        index.register_pack(&pack).unwrap();

        // Add multiple docs about vim
        for i in 0..5 {
            let doc = KnowledgeDocument {
                id: 0,
                pack_id: "test-pack".to_string(),
                title: format!("Vim Guide {}", i),
                content: format!("Vim configuration tip number {}. Learn vim editing.", i),
                source_path: format!("/tmp/vim{}.md", i),
                doc_type: "markdown".to_string(),
                keywords: vec!["vim".to_string()],
                indexed_at: 1000,
            };
            index.add_document(&doc).unwrap();
        }

        let results = index.search("vim", 3).unwrap();
        assert_eq!(results.len(), 3);

        // Check evidence IDs are sequential
        assert_eq!(results[0].evidence_id, "K1");
        assert_eq!(results[1].evidence_id, "K2");
        assert_eq!(results[2].evidence_id, "K3");
    }
}
