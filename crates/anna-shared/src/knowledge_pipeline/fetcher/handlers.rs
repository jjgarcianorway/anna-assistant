//! Knowledge source handlers (v0.0.432).

use super::super::sources::{KnowledgeSource, SourceResult};
use super::types::FetchConfig;
use super::utils::{compute_relevance, run_command, sanitize_filename};

/// Fetch from a probe.
pub fn fetch_probe(
    name: &str,
    cmd: Option<&str>,
    query: &str,
) -> Result<SourceResult, String> {
    let content = if let Some(cmd) = cmd {
        run_command(cmd)?
    } else {
        // Common probe mappings
        match name {
            "meminfo" | "memory" => {
                std::fs::read_to_string("/proc/meminfo").map_err(|e| e.to_string())?
            }
            "cpuinfo" | "cpu" => {
                std::fs::read_to_string("/proc/cpuinfo").map_err(|e| e.to_string())?
            }
            "uptime" => std::fs::read_to_string("/proc/uptime").map_err(|e| e.to_string())?,
            "loadavg" => std::fs::read_to_string("/proc/loadavg").map_err(|e| e.to_string())?,
            _ => return Err(format!("Unknown probe: {}", name)),
        }
    };

    let relevance = compute_relevance(&content, query);
    Ok(SourceResult::new(
        KnowledgeSource::probe(name),
        content,
        relevance,
    ))
}

/// Fetch from a man page.
pub fn fetch_man(name: &str, section: Option<u8>, query: &str) -> Result<SourceResult, String> {
    let cmd = match section {
        Some(s) => format!("man {} {} 2>/dev/null | col -b", s, name),
        None => format!("man {} 2>/dev/null | col -b", name),
    };

    let content = run_command(&cmd)?;
    if content.trim().is_empty() {
        return Err(format!("Man page not found: {}", name));
    }

    let relevance = compute_relevance(&content, query);
    Ok(SourceResult::new(
        KnowledgeSource::man(name),
        content,
        relevance,
    ))
}

/// Fetch from command help.
pub fn fetch_help(command: &str, query: &str) -> Result<SourceResult, String> {
    let content = run_command(&format!("{} --help 2>&1", command))?;
    if content.trim().is_empty() {
        return Err(format!("No help output: {}", command));
    }

    let relevance = compute_relevance(&content, query);
    Ok(SourceResult::new(
        KnowledgeSource::help(command),
        content,
        relevance,
    ))
}

/// Fetch from a doc file.
pub fn fetch_doc_file(path: &str, query: &str) -> Result<SourceResult, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let relevance = compute_relevance(&content, query);
    Ok(SourceResult::new(
        KnowledgeSource::DocFile {
            path: path.to_string(),
        },
        content,
        relevance,
    ))
}

/// Fetch from cached wiki.
pub fn fetch_cached_wiki(
    config: &FetchConfig,
    article: &str,
    query: &str,
) -> Result<SourceResult, String> {
    let cache_path = config
        .wiki_cache_path
        .as_ref()
        .ok_or("Wiki cache not configured")?;

    let article_path = cache_path.join(format!("{}.txt", sanitize_filename(article)));
    if !article_path.exists() {
        return Err(format!("Wiki article not cached: {}", article));
    }

    let content = std::fs::read_to_string(&article_path).map_err(|e| e.to_string())?;
    let relevance = compute_relevance(&content, query);
    Ok(SourceResult::new(
        KnowledgeSource::arch_wiki(article),
        content,
        relevance,
    ))
}
