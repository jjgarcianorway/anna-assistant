//! Knowledge Fetchers
//!
//! Fetch knowledge from various sources (man pages, help output, docs, wiki).

use super::types::{KnowledgeEngineHit, KnowledgeKind};
use super::utils::{find_doc_file, is_safe_command, topic_matches};
use std::path::PathBuf;
use std::process::Command;

/// Fetcher trait - each source implements this
pub(super) trait KnowledgeFetcher {
    fn max_snippet_len(&self) -> usize;

    fn extract_snippet_with_lines(
        &self,
        content: &str,
        keyword: Option<&str>,
    ) -> (String, Option<(usize, usize)>);
}

/// Fetch man page for command
pub(super) fn fetch_man_page<F: KnowledgeFetcher>(
    fetcher: &F,
    cmd: &str,
) -> Result<KnowledgeEngineHit, String> {
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
    let (snippet, line_range) = fetcher.extract_snippet_with_lines(&content, None);
    let citation_id = if let Some((start, end)) = line_range {
        format!("man:{}:line{}-{}", cmd, start, end)
    } else {
        format!("man:{}", cmd)
    };

    Ok(KnowledgeEngineHit {
        doc_id: format!("man:{}", cmd),
        kind: KnowledgeKind::ManPage,
        title: format!("{}(1)", cmd),
        command: format!("man {}", cmd),
        snippet,
        source: "local".to_string(),
        relevance: 80,
        citation_id,
        line_range,
    })
}

/// Fetch --help output for command
pub(super) fn fetch_help<F: KnowledgeFetcher>(
    fetcher: &F,
    cmd: &str,
) -> Result<KnowledgeEngineHit, String> {
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

    let (snippet, line_range) = fetcher.extract_snippet_with_lines(&content, None);
    let citation_id = if let Some((start, end)) = line_range {
        format!("help:{}:line{}-{}", cmd, start, end)
    } else {
        format!("help:{}", cmd)
    };

    Ok(KnowledgeEngineHit {
        doc_id: format!("help:{}", cmd),
        kind: KnowledgeKind::CliHelp,
        title: format!("{} --help", cmd),
        command: format!("{} --help", cmd),
        snippet,
        source: "local".to_string(),
        relevance: 70,
        citation_id,
        line_range,
    })
}

/// Search local documentation
pub(super) fn search_local_docs<F: KnowledgeFetcher>(
    fetcher: &F,
    topic: &str,
) -> Result<Vec<KnowledgeEngineHit>, String> {
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
                    if let Ok(hit) = read_local_doc(fetcher, &entry.path(), topic) {
                        hits.push(hit);
                    }
                }
            }
        }
    }

    Ok(hits)
}

/// Read a local doc file
fn read_local_doc<F: KnowledgeFetcher>(
    fetcher: &F,
    path: &PathBuf,
    topic: &str,
) -> Result<KnowledgeEngineHit, String> {
    // Find a README or main doc file
    let doc_file = find_doc_file(path)?;
    let content = std::fs::read_to_string(&doc_file)
        .map_err(|e| format!("Read {}: {}", doc_file.display(), e))?;

    let (snippet, line_range) = fetcher.extract_snippet_with_lines(&content, Some(topic));
    let title = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "doc".to_string());

    let citation_id = if let Some((start, end)) = line_range {
        format!("doc:{}:line{}-{}", title, start, end)
    } else {
        format!("doc:{}", title)
    };

    Ok(KnowledgeEngineHit {
        doc_id: format!("doc:{}", title),
        kind: KnowledgeKind::LocalDoc,
        title,
        command: format!("cat {}", doc_file.display()),
        snippet,
        source: "local".to_string(),
        relevance: 60,
        citation_id,
        line_range,
    })
}

/// Search Arch Wiki offline cache
pub(super) fn search_arch_wiki<F: KnowledgeFetcher>(
    fetcher: &F,
    topic: &str,
) -> Result<Vec<KnowledgeEngineHit>, String> {
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
            if search_terms
                .iter()
                .any(|t| name.contains(&t.to_lowercase()))
            {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let (snippet, line_range) = extract_wiki_snippet_with_lines(fetcher, &content, topic);
                    let article = name.trim_end_matches(".html");
                    let citation_id = if let Some((start, end)) = line_range {
                        format!("wiki:{}:line{}-{}", article, start, end)
                    } else {
                        format!("wiki:{}", article)
                    };
                    hits.push(KnowledgeEngineHit {
                        doc_id: format!("wiki:{}", article),
                        kind: KnowledgeKind::ArchWiki,
                        title: format!("Arch Wiki: {}", article),
                        command: format!("wiki:{}", name),
                        snippet,
                        source: "offline".to_string(),
                        relevance: 75,
                        citation_id,
                        line_range,
                    });
                }
            }
        }
    }

    Ok(hits)
}

/// Extract snippet from HTML wiki content with line numbers
fn extract_wiki_snippet_with_lines<F: KnowledgeFetcher>(
    fetcher: &F,
    html: &str,
    topic: &str,
) -> (String, Option<(usize, usize)>) {
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

    fetcher.extract_snippet_with_lines(&text, Some(topic))
}
