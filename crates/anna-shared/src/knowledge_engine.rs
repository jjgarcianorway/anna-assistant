//! Knowledge Engine (v0.0.416).
//!
//! Fetches structured knowledge from local sources:
//! - Man pages (man <command>)
//! - CLI help output (<command> --help)
//! - Local documentation (/usr/share/doc, /usr/share/help)
//! - Arch Wiki offline cache (optional)
//!
//! NO LLM calls. Just text retrieval, slicing, and caching.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Knowledge hit kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeKind {
    ManPage,
    CliHelp,
    LocalDoc,
    ArchWiki,
    BuiltIn,
}

impl std::fmt::Display for KnowledgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ManPage => write!(f, "man"),
            Self::CliHelp => write!(f, "help"),
            Self::LocalDoc => write!(f, "doc"),
            Self::ArchWiki => write!(f, "wiki"),
            Self::BuiltIn => write!(f, "built-in"),
        }
    }
}

/// A knowledge hit from the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEngineHit {
    /// Document ID (e.g., "man:systemctl", "help:pacman")
    pub doc_id: String,
    /// Kind of knowledge
    pub kind: KnowledgeKind,
    /// Title
    pub title: String,
    /// Command used to fetch (for reference)
    pub command: String,
    /// Relevant snippet (truncated)
    pub snippet: String,
    /// Source (local, cache)
    pub source: String,
    /// Relevance score (0-100)
    pub relevance: u8,
}

/// Knowledge query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRequest {
    /// Topic to search (e.g., "failed services", "boot time")
    pub topic: String,
    /// Context for relevance
    pub context: KnowledgeContext,
    /// Which sources to use
    pub sources: Vec<KnowledgeKind>,
    /// Maximum hits to return
    pub limit: usize,
}

/// Context for knowledge request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeContext {
    /// Intent (check_status, diagnose, etc.)
    pub intent: String,
    /// Domain (services, storage, etc.)
    pub domain: String,
    /// Related commands (for focused search)
    pub commands: Vec<String>,
}

/// Knowledge query result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeResponse {
    /// Hits found
    pub hits: Vec<KnowledgeEngineHit>,
    /// Query time (ms)
    pub query_time_ms: u64,
    /// Sources searched
    pub sources_searched: Vec<KnowledgeKind>,
    /// Errors (non-fatal)
    pub errors: Vec<String>,
}

/// Knowledge Engine - fetches and caches knowledge
pub struct KnowledgeEngine {
    /// Cache directory
    cache_dir: PathBuf,
    /// Max snippet length
    max_snippet_len: usize,
    /// Command timeout
    timeout: Duration,
    /// Enabled sources
    enabled_sources: Vec<KnowledgeKind>,
}

impl Default for KnowledgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeEngine {
    /// Create new knowledge engine
    pub fn new() -> Self {
        let cache_dir = std::env::var("ANNA_STATE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/var/lib/anna"))
            .join("knowledge_cache");

        Self {
            cache_dir,
            max_snippet_len: 500,
            timeout: Duration::from_secs(3),
            enabled_sources: vec![
                KnowledgeKind::ManPage,
                KnowledgeKind::CliHelp,
                KnowledgeKind::LocalDoc,
            ],
        }
    }

    /// Query knowledge sources
    pub fn query(&self, request: &KnowledgeRequest) -> KnowledgeResponse {
        let start = std::time::Instant::now();
        let mut response = KnowledgeResponse::default();
        let mut hits = Vec::new();

        // Determine commands to search
        let commands = if request.context.commands.is_empty() {
            self.commands_for_topic(&request.topic, &request.context.domain)
        } else {
            request.context.commands.clone()
        };

        // Search each requested source
        for source in &request.sources {
            if !self.enabled_sources.contains(source) {
                continue;
            }

            response.sources_searched.push(*source);

            match source {
                KnowledgeKind::ManPage => {
                    for cmd in &commands {
                        match self.fetch_man_page(cmd) {
                            Ok(hit) => hits.push(hit),
                            Err(e) => response.errors.push(e),
                        }
                    }
                }
                KnowledgeKind::CliHelp => {
                    for cmd in &commands {
                        match self.fetch_help(cmd) {
                            Ok(hit) => hits.push(hit),
                            Err(e) => response.errors.push(e),
                        }
                    }
                }
                KnowledgeKind::LocalDoc => {
                    if let Ok(doc_hits) = self.search_local_docs(&request.topic) {
                        hits.extend(doc_hits);
                    }
                }
                KnowledgeKind::ArchWiki => {
                    if let Ok(wiki_hits) = self.search_arch_wiki(&request.topic) {
                        hits.extend(wiki_hits);
                    }
                }
                KnowledgeKind::BuiltIn => {
                    // Built-in handled by doc_brain
                }
            }
        }

        // Score and sort by relevance
        for hit in &mut hits {
            hit.relevance = self.score_relevance(hit, &request.topic, &request.context);
        }
        hits.sort_by(|a, b| b.relevance.cmp(&a.relevance));
        hits.truncate(request.limit);

        response.hits = hits;
        response.query_time_ms = start.elapsed().as_millis() as u64;
        response
    }

    /// Fetch man page for command
    fn fetch_man_page(&self, cmd: &str) -> Result<KnowledgeEngineHit, String> {
        // Check cache first
        if let Some(cached) = self.get_cached(&format!("man:{}", cmd)) {
            return Ok(cached);
        }

        // Safe command whitelist check
        if !is_safe_command(cmd) {
            return Err(format!("Command '{}' not in safe list", cmd));
        }

        // Execute man
        let output = Command::new("man")
            .arg(cmd)
            .env("MANWIDTH", "80")
            .output()
            .map_err(|e| format!("man {}: {}", cmd, e))?;

        if !output.status.success() {
            return Err(format!("man {}: no manual entry", cmd));
        }

        let content = String::from_utf8_lossy(&output.stdout);
        let snippet = self.extract_snippet(&content, None);

        let hit = KnowledgeEngineHit {
            doc_id: format!("man:{}", cmd),
            kind: KnowledgeKind::ManPage,
            title: format!("{}(1)", cmd),
            command: format!("man {}", cmd),
            snippet,
            source: "local".to_string(),
            relevance: 80,
        };

        // Cache it
        self.cache_hit(&hit);
        Ok(hit)
    }

    /// Fetch --help output for command
    fn fetch_help(&self, cmd: &str) -> Result<KnowledgeEngineHit, String> {
        // Check cache first
        if let Some(cached) = self.get_cached(&format!("help:{}", cmd)) {
            return Ok(cached);
        }

        // Safe command whitelist check
        if !is_safe_command(cmd) {
            return Err(format!("Command '{}' not in safe list", cmd));
        }

        // Try --help, then -h
        let output = Command::new(cmd)
            .arg("--help")
            .output()
            .or_else(|_| Command::new(cmd).arg("-h").output())
            .map_err(|e| format!("{} --help: {}", cmd, e))?;

        // Some commands output help to stderr
        let content = if output.stdout.is_empty() {
            String::from_utf8_lossy(&output.stderr).to_string()
        } else {
            String::from_utf8_lossy(&output.stdout).to_string()
        };

        if content.is_empty() {
            return Err(format!("{} --help: no output", cmd));
        }

        let snippet = self.extract_snippet(&content, None);

        let hit = KnowledgeEngineHit {
            doc_id: format!("help:{}", cmd),
            kind: KnowledgeKind::CliHelp,
            title: format!("{} --help", cmd),
            command: format!("{} --help", cmd),
            snippet,
            source: "local".to_string(),
            relevance: 70,
        };

        self.cache_hit(&hit);
        Ok(hit)
    }

    /// Search local documentation
    fn search_local_docs(&self, topic: &str) -> Result<Vec<KnowledgeEngineHit>, String> {
        let mut hits = Vec::new();
        let doc_dirs = ["/usr/share/doc", "/usr/share/help"];

        for dir in &doc_dirs {
            let path = PathBuf::from(dir);
            if !path.exists() {
                continue;
            }

            // Simple search: look for files matching topic keywords
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.take(20).flatten() {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if topic_matches(&name, topic) {
                        if let Ok(hit) = self.read_local_doc(&entry.path(), topic) {
                            hits.push(hit);
                        }
                    }
                }
            }
        }

        Ok(hits)
    }

    /// Read a local doc file
    fn read_local_doc(&self, path: &PathBuf, topic: &str) -> Result<KnowledgeEngineHit, String> {
        // Find a README or main doc file
        let doc_file = find_doc_file(path)?;
        let content = std::fs::read_to_string(&doc_file)
            .map_err(|e| format!("Read {}: {}", doc_file.display(), e))?;

        let snippet = self.extract_snippet(&content, Some(topic));
        let title = path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "doc".to_string());

        Ok(KnowledgeEngineHit {
            doc_id: format!("doc:{}", title),
            kind: KnowledgeKind::LocalDoc,
            title,
            command: format!("cat {}", doc_file.display()),
            snippet,
            source: "local".to_string(),
            relevance: 60,
        })
    }

    /// Search Arch Wiki offline cache
    fn search_arch_wiki(&self, topic: &str) -> Result<Vec<KnowledgeEngineHit>, String> {
        // Check for offline wiki tool
        let wiki_path = PathBuf::from("/usr/share/doc/arch-wiki/html");
        if !wiki_path.exists() {
            return Ok(Vec::new()); // Wiki not available
        }

        // Search for topic-matching files
        let mut hits = Vec::new();
        let search_terms: Vec<&str> = topic.split_whitespace().collect();

        if let Ok(entries) = std::fs::read_dir(&wiki_path) {
            for entry in entries.take(100).flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if search_terms.iter().any(|t| name.contains(&t.to_lowercase())) {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        let snippet = self.extract_wiki_snippet(&content, topic);
                        hits.push(KnowledgeEngineHit {
                            doc_id: format!("wiki:{}", name.trim_end_matches(".html")),
                            kind: KnowledgeKind::ArchWiki,
                            title: format!("Arch Wiki: {}", name.trim_end_matches(".html")),
                            command: format!("wiki:{}", name),
                            snippet,
                            source: "offline".to_string(),
                            relevance: 75,
                        });
                    }
                }
            }
        }

        Ok(hits)
    }

    /// Extract relevant snippet from content
    fn extract_snippet(&self, content: &str, keyword: Option<&str>) -> String {
        let lines: Vec<&str> = content.lines().collect();

        // If keyword provided, try to find relevant section
        if let Some(kw) = keyword {
            let kw_lower = kw.to_lowercase();
            for (i, line) in lines.iter().enumerate() {
                if line.to_lowercase().contains(&kw_lower) {
                    // Return context around match
                    let start = i.saturating_sub(2);
                    let end = (i + 5).min(lines.len());
                    let snippet: String = lines[start..end].join("\n");
                    return truncate(&snippet, self.max_snippet_len);
                }
            }
        }

        // Otherwise, return beginning (skip empty lines)
        let meaningful: Vec<&str> = lines
            .into_iter()
            .filter(|l| !l.trim().is_empty())
            .take(10)
            .collect();
        truncate(&meaningful.join("\n"), self.max_snippet_len)
    }

    /// Extract snippet from HTML wiki content
    fn extract_wiki_snippet(&self, html: &str, topic: &str) -> String {
        // Basic HTML stripping (proper parsing would need html crate)
        let text = html
            .replace("<p>", "\n")
            .replace("</p>", "\n")
            .replace("<br>", "\n")
            .replace("<li>", "- ");

        // Strip remaining tags
        let re = regex::Regex::new(r"<[^>]+>").ok();
        let text = if let Some(r) = re {
            r.replace_all(&text, "").to_string()
        } else {
            text
        };

        self.extract_snippet(&text, Some(topic))
    }

    /// Score relevance of a hit
    fn score_relevance(&self, hit: &KnowledgeEngineHit, topic: &str, ctx: &KnowledgeContext) -> u8 {
        let mut score = hit.relevance;

        // Boost for domain match
        if hit.doc_id.to_lowercase().contains(&ctx.domain) {
            score = score.saturating_add(10);
        }

        // Boost for topic keywords in snippet
        let keywords: Vec<&str> = topic.split_whitespace().collect();
        for kw in keywords {
            if hit.snippet.to_lowercase().contains(&kw.to_lowercase()) {
                score = score.saturating_add(5);
            }
        }

        score.min(100)
    }

    /// Get commands relevant to a topic
    fn commands_for_topic(&self, topic: &str, domain: &str) -> Vec<String> {
        TOPIC_COMMANDS
            .get(domain)
            .map(|cmds| cmds.iter().map(|s| s.to_string()).collect())
            .unwrap_or_else(|| {
                // Extract command-like words from topic
                topic
                    .split_whitespace()
                    .filter(|w| is_safe_command(w))
                    .map(String::from)
                    .collect()
            })
    }

    /// Get cached hit
    fn get_cached(&self, doc_id: &str) -> Option<KnowledgeEngineHit> {
        let path = self.cache_path(doc_id);
        let content = std::fs::read_to_string(&path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;

        // Check expiry (1 week)
        let now = current_secs();
        if now > entry.cached_at + 7 * 24 * 3600 {
            return None;
        }

        Some(entry.hit)
    }

    /// Cache a hit
    fn cache_hit(&self, hit: &KnowledgeEngineHit) {
        let _ = std::fs::create_dir_all(&self.cache_dir);
        let entry = CacheEntry {
            hit: hit.clone(),
            cached_at: current_secs(),
        };
        let path = self.cache_path(&hit.doc_id);
        let _ = std::fs::write(path, serde_json::to_string(&entry).unwrap_or_default());
    }

    /// Get cache path for doc_id
    fn cache_path(&self, doc_id: &str) -> PathBuf {
        let safe_name = doc_id.replace([':', '/'], "_");
        self.cache_dir.join(format!("{}.json", safe_name))
    }
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    hit: KnowledgeEngineHit,
    cached_at: u64,
}

/// Safe commands whitelist
fn is_safe_command(cmd: &str) -> bool {
    const SAFE_COMMANDS: &[&str] = &[
        "systemctl", "systemd-analyze", "journalctl",
        "df", "du", "lsblk", "mount", "findmnt", "blkid",
        "free", "top", "ps", "uptime", "uname",
        "ip", "ss", "networkctl", "nmcli", "resolvectl",
        "pacman", "paru", "yay", "makepkg",
        "pactl", "wpctl", "aplay", "arecord",
        "hyprctl", "swaymsg",
        "cat", "head", "tail", "grep", "ls", "stat", "file",
        "git", "ssh", "rsync",
        "fstrim", "lscpu", "lspci", "lsusb",
        "timedatectl", "localectl", "hostnamectl",
    ];
    SAFE_COMMANDS.contains(&cmd)
}

/// Check if topic matches name
fn topic_matches(name: &str, topic: &str) -> bool {
    topic
        .split_whitespace()
        .any(|t| name.contains(&t.to_lowercase()))
}

/// Find main doc file in directory
fn find_doc_file(dir: &PathBuf) -> Result<PathBuf, String> {
    let candidates = ["README.md", "README", "readme.md", "index.html", "doc.txt"];
    for candidate in candidates {
        let path = dir.join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try first .md or .txt file
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.ends_with(".md") || name.ends_with(".txt") {
                return Ok(entry.path());
            }
        }
    }

    Err(format!("No doc file found in {}", dir.display()))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

fn current_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Topic to commands mapping
static TOPIC_COMMANDS: once_cell::sync::Lazy<HashMap<&str, Vec<&str>>> =
    once_cell::sync::Lazy::new(|| {
        let mut m = HashMap::new();
        m.insert("system", vec!["free", "uptime", "uname", "lscpu"]);
        m.insert("boot", vec!["systemd-analyze", "journalctl"]);
        m.insert("services", vec!["systemctl", "journalctl"]);
        m.insert("network", vec!["ip", "ss", "networkctl", "nmcli"]);
        m.insert("storage", vec!["df", "lsblk", "du", "mount", "fstrim"]);
        m.insert("packages", vec!["pacman", "paru", "yay"]);
        m.insert("audio", vec!["pactl", "wpctl", "aplay"]);
        m.insert("display", vec!["lspci"]);
        m.insert("desktop", vec!["hyprctl", "swaymsg"]);
        m.insert("security", vec!["ss", "journalctl"]);
        m
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_command() {
        assert!(is_safe_command("systemctl"));
        assert!(is_safe_command("df"));
        assert!(!is_safe_command("rm"));
        assert!(!is_safe_command("curl"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hello...");
    }

    #[test]
    fn test_topic_matches() {
        assert!(topic_matches("systemd-boot", "boot"));
        assert!(topic_matches("networkmanager", "network"));
        assert!(!topic_matches("pacman", "boot"));
    }

    #[test]
    fn test_knowledge_request() {
        let req = KnowledgeRequest {
            topic: "failed services".to_string(),
            context: KnowledgeContext {
                intent: "check_status".to_string(),
                domain: "services".to_string(),
                commands: vec!["systemctl".to_string()],
            },
            sources: vec![KnowledgeKind::ManPage, KnowledgeKind::CliHelp],
            limit: 3,
        };

        let engine = KnowledgeEngine::new();
        let response = engine.query(&req);

        // Should have searched the sources
        assert!(response.sources_searched.contains(&KnowledgeKind::ManPage));
    }
}
