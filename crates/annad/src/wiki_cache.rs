//! Arch Wiki offline cache system
//!
//! Downloads and caches Arch Wiki pages for offline access

use anna_common::{WikiCache, WikiCacheEntry};
use anyhow::Result;
use std::process::Command;
use tracing::{info, warn};

/// Fetch and cache a wiki page
pub async fn fetch_and_cache_page(url: &str, title: &str) -> Result<WikiCacheEntry> {
    info!("Fetching wiki page: {}", url);

    // Use curl to fetch the page
    let output = Command::new("curl")
        .args(&[
            "-s", // Silent
            "-L", // Follow redirects
            url,
        ])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to fetch wiki page: {}", url);
    }

    let html_content = String::from_utf8_lossy(&output.stdout).to_string();

    // Extract the main content from HTML (simplified - just get text)
    let content = extract_wiki_content(&html_content);

    // Calculate checksum
    let checksum = calculate_checksum(&content);

    Ok(WikiCacheEntry {
        page_title: title.to_string(),
        url: url.to_string(),
        content,
        cached_at: chrono::Utc::now(),
        checksum,
    })
}

/// Extract readable content from wiki HTML
fn extract_wiki_content(html: &str) -> String {
    // Simple extraction: get content between <div id="content"> and </div>
    // In a real implementation, we'd use an HTML parser like scraper

    // For now, just strip HTML tags and get plain text
    let mut content = html.to_string();

    // Remove script tags
    content = remove_between(&content, "<script", "</script>");
    content = remove_between(&content, "<style", "</style>");

    // Remove HTML tags (simplified)
    content = content
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("</p>", "\n\n")
        .replace("</div>", "\n")
        .replace("</li>", "\n");

    // Remove all remaining tags
    let mut result = String::new();
    let mut in_tag = false;
    for c in content.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    // Clean up excessive whitespace
    let lines: Vec<&str> = result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    lines.join("\n")
}

/// Remove content between two markers
fn remove_between(content: &str, start: &str, end: &str) -> String {
    let mut result = content.to_string();
    while let Some(start_pos) = result.find(start) {
        if let Some(end_pos) = result[start_pos..].find(end) {
            result.replace_range(start_pos..start_pos + end_pos + end.len(), "");
        } else {
            break;
        }
    }
    result
}

/// Calculate checksum of content
fn calculate_checksum(content: &str) -> String {
    // Simple checksum using length and first/last chars
    // In production, use SHA256 or similar
    format!("{:x}", content.len())
}

/// Update wiki cache for commonly referenced pages
pub async fn update_common_pages() -> Result<()> {
    info!("Updating wiki cache for common pages");

    let mut cache = WikiCache::load().unwrap_or_default();

    // List of commonly referenced pages
    let common_pages = vec![
        ("https://wiki.archlinux.org/title/Security", "Security"),
        ("https://wiki.archlinux.org/title/Improving_performance", "Improving performance"),
        ("https://wiki.archlinux.org/title/System_maintenance", "System maintenance"),
        ("https://wiki.archlinux.org/title/Power_management", "Power management"),
        ("https://wiki.archlinux.org/title/Pacman", "Pacman"),
        ("https://wiki.archlinux.org/title/Systemd", "Systemd"),
        ("https://wiki.archlinux.org/title/Kernel_parameters", "Kernel parameters"),
        ("https://wiki.archlinux.org/title/Docker", "Docker"),
        ("https://wiki.archlinux.org/title/Python", "Python"),
        ("https://wiki.archlinux.org/title/Rust", "Rust"),
        ("https://wiki.archlinux.org/title/Gaming", "Gaming"),
        ("https://wiki.archlinux.org/title/Firewall", "Firewall"),
        ("https://wiki.archlinux.org/title/SSH", "SSH"),
        ("https://wiki.archlinux.org/title/Hardware", "Hardware"),
        ("https://wiki.archlinux.org/title/Desktop_environment", "Desktop environment"),
    ];

    for (url, title) in common_pages {
        // Check if already cached and fresh
        if let Some(existing) = cache.get_by_url(url) {
            let age = chrono::Utc::now() - existing.cached_at;
            if age.num_days() < 7 {
                info!("Page '{}' is fresh, skipping", title);
                continue;
            }
        }

        match fetch_and_cache_page(url, title).await {
            Ok(entry) => {
                info!("Cached: {}", title);
                cache.upsert(entry);
            }
            Err(e) => {
                warn!("Failed to cache {}: {}", title, e);
            }
        }

        // Small delay to be nice to wiki servers
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    cache.save()?;
    info!("Wiki cache updated successfully");

    Ok(())
}

/// Get wiki content for a URL (from cache or fetch)
pub async fn get_wiki_content(url: &str) -> Option<String> {
    // Try to load from cache first
    if let Ok(cache) = WikiCache::load() {
        if let Some(entry) = cache.get_by_url(url) {
            // Check if not too old (30 days)
            let age = chrono::Utc::now() - entry.cached_at;
            if age.num_days() < 30 {
                return Some(entry.content.clone());
            }
        }
    }

    // Not in cache or too old, try to fetch
    // Extract title from URL
    let title = url.split('/').last().unwrap_or("Unknown");

    if let Ok(entry) = fetch_and_cache_page(url, title).await {
        // Update cache
        if let Ok(mut cache) = WikiCache::load() {
            cache.upsert(entry.clone());
            let _ = cache.save();
        }
        return Some(entry.content);
    }

    None
}

/// Check if wiki cache needs refresh
pub fn needs_refresh() -> bool {
    if let Ok(cache) = WikiCache::load() {
        return cache.needs_refresh();
    }
    true
}
