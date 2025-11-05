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
    update_common_pages_with_force(false).await
}

/// Update wiki cache with optional force flag
pub async fn update_common_pages_with_force(force: bool) -> Result<()> {
    info!("Updating wiki cache for common pages (force={})", force);

    let mut cache = WikiCache::load().unwrap_or_default();

    // Comprehensive list of essential and commonly referenced pages
    let common_pages = vec![
        // Essential/Installation
        ("https://wiki.archlinux.org/title/Installation_guide", "Installation guide"),
        ("https://wiki.archlinux.org/title/General_recommendations", "General recommendations"),

        // Security & System
        ("https://wiki.archlinux.org/title/Security", "Security"),
        ("https://wiki.archlinux.org/title/System_maintenance", "System maintenance"),
        ("https://wiki.archlinux.org/title/Improving_performance", "Improving performance"),
        ("https://wiki.archlinux.org/title/AppArmor", "AppArmor"),
        ("https://wiki.archlinux.org/title/Fail2ban", "Fail2ban"),
        ("https://wiki.archlinux.org/title/Audit_framework", "Audit framework"),
        ("https://wiki.archlinux.org/title/USBGuard", "USBGuard"),
        ("https://wiki.archlinux.org/title/Firejail", "Firejail"),
        ("https://wiki.archlinux.org/title/AIDE", "AIDE"),
        ("https://wiki.archlinux.org/title/Dnscrypt-proxy", "Dnscrypt-proxy"),

        // Package Management
        ("https://wiki.archlinux.org/title/Pacman", "Pacman"),
        ("https://wiki.archlinux.org/title/Pacman/Tips_and_tricks", "Pacman tips and tricks"),
        ("https://wiki.archlinux.org/title/AUR_helpers", "AUR helpers"),
        ("https://wiki.archlinux.org/title/Arch_User_Repository", "Arch User Repository"),
        ("https://wiki.archlinux.org/title/Official_repositories", "Official repositories"),

        // System Core
        ("https://wiki.archlinux.org/title/Systemd", "Systemd"),
        ("https://wiki.archlinux.org/title/Kernel_parameters", "Kernel parameters"),
        ("https://wiki.archlinux.org/title/Users_and_groups", "Users and groups"),
        ("https://wiki.archlinux.org/title/Sysctl", "Sysctl"),

        // Hardware & Drivers
        ("https://wiki.archlinux.org/title/Hardware", "Hardware"),
        ("https://wiki.archlinux.org/title/Xorg", "Xorg"),
        ("https://wiki.archlinux.org/title/Wayland", "Wayland"),
        ("https://wiki.archlinux.org/title/NVIDIA", "NVIDIA"),
        ("https://wiki.archlinux.org/title/Intel_graphics", "Intel graphics"),
        ("https://wiki.archlinux.org/title/AMDGPU", "AMDGPU"),
        ("https://wiki.archlinux.org/title/Hardware_video_acceleration", "Hardware video acceleration"),
        ("https://wiki.archlinux.org/title/Vulkan", "Vulkan"),
        ("https://wiki.archlinux.org/title/GPGPU", "GPGPU"),

        // Audio
        ("https://wiki.archlinux.org/title/PipeWire", "PipeWire"),
        ("https://wiki.archlinux.org/title/PulseAudio", "PulseAudio"),
        ("https://wiki.archlinux.org/title/ALSA", "ALSA"),
        ("https://wiki.archlinux.org/title/Bluetooth_headset", "Bluetooth headset"),

        // Network
        ("https://wiki.archlinux.org/title/Network_configuration", "Network configuration"),
        ("https://wiki.archlinux.org/title/Wireless_network_configuration", "Wireless network configuration"),
        ("https://wiki.archlinux.org/title/Firewall", "Firewall"),
        ("https://wiki.archlinux.org/title/SSH", "SSH"),
        ("https://wiki.archlinux.org/title/Domain_name_resolution", "Domain name resolution"),

        // Desktop Environment & Window Managers
        ("https://wiki.archlinux.org/title/Desktop_environment", "Desktop environment"),
        ("https://wiki.archlinux.org/title/GNOME", "GNOME"),
        ("https://wiki.archlinux.org/title/KDE", "KDE"),
        ("https://wiki.archlinux.org/title/Xfce", "Xfce"),
        ("https://wiki.archlinux.org/title/I3", "i3"),
        ("https://wiki.archlinux.org/title/Sway", "Sway"),
        ("https://wiki.archlinux.org/title/Hyprland", "Hyprland"),

        // Development - Languages
        ("https://wiki.archlinux.org/title/Python", "Python"),
        ("https://wiki.archlinux.org/title/Rust", "Rust"),
        ("https://wiki.archlinux.org/title/Node.js", "Node.js"),
        ("https://wiki.archlinux.org/title/Go", "Go"),
        ("https://wiki.archlinux.org/title/Java", "Java"),
        ("https://wiki.archlinux.org/title/PHP", "PHP"),
        ("https://wiki.archlinux.org/title/Ruby", "Ruby"),
        ("https://wiki.archlinux.org/title/GNU_Compiler_Collection", "GNU Compiler Collection"),
        ("https://wiki.archlinux.org/title/Clang", "Clang"),
        ("https://wiki.archlinux.org/title/CMake", "CMake"),

        // Development - Tools
        ("https://wiki.archlinux.org/title/Docker", "Docker"),
        ("https://wiki.archlinux.org/title/Podman", "Podman"),
        ("https://wiki.archlinux.org/title/Kubectl", "Kubectl"),
        ("https://wiki.archlinux.org/title/Git", "Git"),
        ("https://wiki.archlinux.org/title/PostgreSQL", "PostgreSQL"),

        // Gaming & Multimedia
        ("https://wiki.archlinux.org/title/Gaming", "Gaming"),
        ("https://wiki.archlinux.org/title/Steam", "Steam"),
        ("https://wiki.archlinux.org/title/Wine", "Wine"),
        ("https://wiki.archlinux.org/title/Gamemode", "Gamemode"),
        ("https://wiki.archlinux.org/title/MangoHud", "MangoHud"),
        ("https://wiki.archlinux.org/title/Gamescope", "Gamescope"),
        ("https://wiki.archlinux.org/title/Lutris", "Lutris"),
        ("https://wiki.archlinux.org/title/Gamepad", "Gamepad"),
        ("https://wiki.archlinux.org/title/RetroArch", "RetroArch"),
        ("https://wiki.archlinux.org/title/PCSX2", "PCSX2"),
        ("https://wiki.archlinux.org/title/Dolphin_emulator", "Dolphin emulator"),
        ("https://wiki.archlinux.org/title/Discord", "Discord"),

        // Power & Laptop
        ("https://wiki.archlinux.org/title/Power_management", "Power management"),
        ("https://wiki.archlinux.org/title/Laptop", "Laptop"),
        ("https://wiki.archlinux.org/title/TLP", "TLP"),

        // Filesystems & Storage
        ("https://wiki.archlinux.org/title/Btrfs", "Btrfs"),
        ("https://wiki.archlinux.org/title/Ext4", "Ext4"),
        ("https://wiki.archlinux.org/title/LUKS", "LUKS"),
        ("https://wiki.archlinux.org/title/Dm-crypt", "Dm-crypt"),

        // Troubleshooting
        ("https://wiki.archlinux.org/title/FAQ", "FAQ"),
        ("https://wiki.archlinux.org/title/Debugging", "Debugging"),
    ];

    let total = common_pages.len();
    let mut downloaded = 0;
    let mut skipped = 0;
    let mut failed = 0;

    info!("Processing {} wiki pages", total);

    for (index, (url, title)) in common_pages.iter().enumerate() {
        let progress = ((index + 1) as f32 / total as f32 * 100.0) as u32;

        // Check if already cached and fresh (unless force)
        if !force {
            if let Some(existing) = cache.get_by_url(url) {
                let age = chrono::Utc::now() - existing.cached_at;
                if age.num_days() < 7 {
                    info!("[{}/{}] Page '{}' is fresh, skipping", index + 1, total, title);
                    skipped += 1;
                    continue;
                }
            }
        }

        info!("[{}/{}] ({:3}%) Downloading: {}", index + 1, total, progress, title);

        match fetch_and_cache_page(url, title).await {
            Ok(entry) => {
                info!("✓ Cached: {}", title);
                cache.upsert(entry);
                downloaded += 1;
            }
            Err(e) => {
                warn!("✗ Failed to cache {}: {}", title, e);
                failed += 1;
            }
        }

        // Small delay to be nice to wiki servers
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    cache.save()?;
    info!("Wiki cache update complete: {} downloaded, {} skipped, {} failed",
          downloaded, skipped, failed);

    Ok(())
}

/// Get wiki content for a URL (from cache or fetch)
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn needs_refresh() -> bool {
    if let Ok(cache) = WikiCache::load() {
        return cache.needs_refresh();
    }
    true
}
