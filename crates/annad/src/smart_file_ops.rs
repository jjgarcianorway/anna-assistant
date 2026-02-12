//! Smart File Operations - Anna handles complex file tasks intelligently.
//! Uses better tools (fd, rg), progressive strategies, and doesn't give up.

use anyhow::{anyhow, Result};
use tracing::{debug, info};

/// Detect if question is about file operations.
pub fn is_file_operation(question: &str) -> bool {
    let q = question.to_lowercase();

    let file_patterns = [
        "find file", "find all", "search for", "locate",
        "largest", "biggest", "smallest",
        "duplicate", "duplicates",
        "log file", "logs",
        ">", "larger than", "bigger than",
        "modified", "created", "accessed",
    ];

    file_patterns.iter().any(|p| q.contains(p))
}

/// Execute file operation with smart strategy.
pub async fn execute_smart_file_operation(model: &str, question: &str) -> Result<String> {
    info!("Smart file operation handler activated");

    // Analyze what type of file operation this is
    let op_type = analyze_file_operation(question);

    match op_type {
        FileOperationType::FindLarge => find_large_files(question).await,
        FileOperationType::FindDuplicates => find_duplicates(question).await,
        FileOperationType::FindByName => find_by_name(question).await,
        FileOperationType::FindByContent => find_by_content(question).await,
        FileOperationType::Complex => {
            // Use adaptive intelligence for complex file ops
            crate::adaptive_intelligence::solve_adaptively(model, question).await
        }
    }
}

#[derive(Debug)]
enum FileOperationType {
    FindLarge,
    FindDuplicates,
    FindByName,
    FindByContent,
    Complex,
}

fn analyze_file_operation(question: &str) -> FileOperationType {
    let q = question.to_lowercase();

    if q.contains("duplicate") {
        FileOperationType::FindDuplicates
    } else if q.contains("largest") || q.contains("biggest") || q.contains(">") || q.contains("larger") {
        FileOperationType::FindLarge
    } else if q.contains("find") && (q.contains("name") || q.contains("called") || q.contains("*.")) {
        FileOperationType::FindByName
    } else if q.contains("contain") || q.contains("content") || q.contains("search for") {
        FileOperationType::FindByContent
    } else {
        FileOperationType::Complex
    }
}

/// Find large files using smart strategy.
async fn find_large_files(question: &str) -> Result<String> {
    info!("Finding large files");

    // Parse size threshold from question (default 100MB)
    let size_mb = extract_size_threshold(question).unwrap_or(100);
    let size_bytes = size_mb * 1024 * 1024;

    // Strategy 1: Try fd (much faster than find)
    if tool_available("fd") {
        info!("Using fd for fast file search");
        let cmd = format!(
            "fd --type f --size +{}m --exec-batch ls -lh {{}} | sort -rh -k5 | head -20",
            size_mb
        );

        match crate::core_loop::execute_command(&cmd) {
            Ok(output) if !output.trim().is_empty() => {
                return Ok(format!("Large files (>{}MB):\n{}", size_mb, output));
            }
            _ => {
                debug!("fd strategy failed, trying find");
            }
        }
    }

    // Strategy 2: Use find with optimizations
    info!("Using find with common paths only (faster)");
    let common_paths = vec!["/var/log", "/tmp", "/home", "/opt", "/usr/local"];

    for path in common_paths {
        let cmd = format!(
            "find {} -type f -size +{}M -ls 2>/dev/null | sort -rn -k7 | head -10",
            path, size_mb
        );

        match crate::core_loop::execute_command(&cmd) {
            Ok(output) if !output.trim().is_empty() => {
                return Ok(format!("Large files in {} (>{}MB):\n{}", path, size_mb, output));
            }
            _ => continue,
        }
    }

    // Strategy 3: Targeted log search
    info!("Trying targeted log search");
    let log_cmd = format!("du -h /var/log/*.log /var/log/*/*.log 2>/dev/null | sort -rh | head -10");
    match crate::core_loop::execute_command(&log_cmd) {
        Ok(output) if !output.trim().is_empty() => {
            Ok(format!("Large log files:\n{}", output))
        }
        _ => Err(anyhow!("Could not find large files with any strategy")),
    }
}

/// Find duplicate files using smart strategy.
async fn find_duplicates(question: &str) -> Result<String> {
    info!("Finding duplicate files");

    // Extract target directory (default ~/home)
    let target_dir = extract_target_directory(question)
        .unwrap_or_else(|| std::env::var("HOME").unwrap_or_else(|_| "/home".to_string()));

    // Strategy 1: Try fdupes if available (fastest)
    if tool_available("fdupes") {
        info!("Using fdupes for duplicate detection");
        let cmd = format!("fdupes -r -S -n {} 2>/dev/null | head -50", target_dir);

        match crate::core_loop::execute_command(&cmd) {
            Ok(output) if !output.trim().is_empty() => {
                return Ok(format!("Duplicate files in {}:\n{}", target_dir, output));
            }
            _ => debug!("fdupes failed, trying rdfind"),
        }
    }

    // Strategy 2: Try rdfind
    if tool_available("rdfind") {
        info!("Using rdfind for duplicate detection");
        let cmd = format!("rdfind -dryrun true {} 2>/dev/null", target_dir);

        match crate::core_loop::execute_command(&cmd) {
            Ok(output) if !output.trim().is_empty() => {
                return Ok(format!("Duplicate analysis of {}:\n{}", target_dir, output));
            }
            _ => debug!("rdfind failed, trying manual approach"),
        }
    }

    // Strategy 3: Manual duplicate detection with find + md5sum (slower but works)
    info!("Using manual md5sum approach (this may take a minute)");
    let cmd = format!(
        "find {} -type f -exec md5sum {{}} \\; 2>/dev/null | sort | uniq -w32 -d --all-repeated=separate | head -50",
        target_dir
    );

    match crate::core_loop::execute_command(&cmd) {
        Ok(output) if !output.trim().is_empty() => {
            Ok(format!("Duplicate files (by hash) in {}:\n{}", target_dir, output))
        }
        _ => {
            // Strategy 4: At least find files with same name
            info!("Fallback: finding files with same names");
            let name_cmd = format!(
                "find {} -type f -printf '%f\\n' 2>/dev/null | sort | uniq -d | head -20",
                target_dir
            );
            match crate::core_loop::execute_command(&name_cmd) {
                Ok(output) if !output.trim().is_empty() => {
                    Ok(format!("Files with duplicate names in {}:\n{}", target_dir, output))
                }
                _ => Err(anyhow!("Could not find duplicates with any strategy")),
            }
        }
    }
}

/// Find files by name pattern.
async fn find_by_name(question: &str) -> Result<String> {
    info!("Finding files by name");

    let pattern = extract_name_pattern(question)?;
    let target_dir = extract_target_directory(question).unwrap_or_else(|| ".".to_string());

    // Strategy 1: Try fd (blazing fast)
    if tool_available("fd") {
        info!("Using fd for name search");
        let cmd = format!("fd --type f '{}' {} 2>/dev/null | head -50", pattern, target_dir);

        match crate::core_loop::execute_command(&cmd) {
            Ok(output) if !output.trim().is_empty() => {
                return Ok(format!("Files matching '{}':\n{}", pattern, output));
            }
            _ => debug!("fd failed, trying find"),
        }
    }

    // Strategy 2: Use find
    let cmd = format!("find {} -type f -name '{}' 2>/dev/null | head -50", target_dir, pattern);

    match crate::core_loop::execute_command(&cmd) {
        Ok(output) if !output.trim().is_empty() => {
            Ok(format!("Files matching '{}':\n{}", pattern, output))
        }
        _ => Err(anyhow!("No files found matching '{}'", pattern)),
    }
}

/// Find files by content.
async fn find_by_content(question: &str) -> Result<String> {
    info!("Finding files by content");

    let search_term = extract_search_term(question)?;
    let target_dir = extract_target_directory(question).unwrap_or_else(|| ".".to_string());

    // Strategy 1: Try ripgrep (fastest)
    if tool_available("rg") {
        info!("Using ripgrep for content search");
        let cmd = format!("rg -l '{}' {} 2>/dev/null | head -50", search_term, target_dir);

        match crate::core_loop::execute_command(&cmd) {
            Ok(output) if !output.trim().is_empty() => {
                return Ok(format!("Files containing '{}':\n{}", search_term, output));
            }
            _ => debug!("ripgrep failed, trying grep"),
        }
    }

    // Strategy 2: Use grep
    let cmd = format!(
        "grep -rl '{}' {} 2>/dev/null | head -50",
        search_term, target_dir
    );

    match crate::core_loop::execute_command(&cmd) {
        Ok(output) if !output.trim().is_empty() => {
            Ok(format!("Files containing '{}':\n{}", search_term, output))
        }
        _ => Err(anyhow!("No files found containing '{}'", search_term)),
    }
}

/// Check if a tool is available.
fn tool_available(tool: &str) -> bool {
    let cmd = format!("command -v {} 2>/dev/null", tool);
    crate::core_loop::execute_command(&cmd).is_ok()
}

/// Extract size threshold from question (in MB).
fn extract_size_threshold(question: &str) -> Option<u64> {
    let q = question.to_lowercase();

    // Look for patterns like "100MB", "1GB", "500M"
    if let Some(caps) = regex::Regex::new(r"(\d+)\s*(mb|m|gb|g)")
        .ok()?
        .captures(&q)
    {
        let num: u64 = caps.get(1)?.as_str().parse().ok()?;
        let unit = caps.get(2)?.as_str();

        return Some(match unit {
            "gb" | "g" => num * 1024,
            _ => num,
        });
    }

    None
}

/// Extract target directory from question.
fn extract_target_directory(question: &str) -> Option<String> {
    let q = question.to_lowercase();

    // Common patterns
    if q.contains("home directory") || q.contains("my home") {
        return std::env::var("HOME").ok();
    }
    if q.contains("/var/log") {
        return Some("/var/log".to_string());
    }
    if q.contains("/tmp") {
        return Some("/tmp".to_string());
    }

    // Look for quoted paths or paths starting with /
    if let Some(caps) = regex::Regex::new(r#"['"](/[^'"]+)['"]"#)
        .ok()?
        .captures(question)
    {
        return Some(caps.get(1)?.as_str().to_string());
    }

    None
}

/// Extract name pattern from question.
fn extract_name_pattern(question: &str) -> Result<String> {
    // Look for quoted strings
    if let Some(caps) = regex::Regex::new(r#"['"]([^'"]+)['"]"#)
        .ok()
        .and_then(|re| re.captures(question))
    {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    // Look for *.extension patterns
    if let Some(caps) = regex::Regex::new(r"\*\.(\w+)")
        .ok()
        .and_then(|re| re.captures(question))
    {
        return Ok(format!("*.{}", caps.get(1).unwrap().as_str()));
    }

    Err(anyhow!("Could not extract file name pattern"))
}

/// Extract search term from question.
fn extract_search_term(question: &str) -> Result<String> {
    // Look for quoted strings
    if let Some(caps) = regex::Regex::new(r#"['"]([^'"]+)['"]"#)
        .ok()
        .and_then(|re| re.captures(question))
    {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    // Look for "containing X" or "with X"
    if let Some(caps) = regex::Regex::new(r"(?:containing|with)\s+(\S+)")
        .ok()
        .and_then(|re| re.captures(question))
    {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    Err(anyhow!("Could not extract search term"))
}
