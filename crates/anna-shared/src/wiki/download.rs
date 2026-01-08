//! Download Arch Wiki for offline access.
//!
//! Uses arch-wiki-docs package or direct download.

use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

use super::wiki_articles_dir;
use crate::helpers::{self, HelpersRegistry, InstallSource};

/// Source for wiki content
const ARCH_WIKI_DOCS_PATH: &str = "/usr/share/doc/arch-wiki/html";

/// Download/sync Arch Wiki locally
pub async fn download_wiki() -> Result<()> {
    let articles_dir = wiki_articles_dir();
    std::fs::create_dir_all(&articles_dir)?;

    // Strategy 1: Use arch-wiki-docs if available or can be installed
    if try_use_arch_wiki_docs(&articles_dir).await? {
        return Ok(());
    }

    // Strategy 2: Download directly from git mirror
    download_from_git(&articles_dir).await?;

    Ok(())
}

/// Try to use arch-wiki-docs package
async fn try_use_arch_wiki_docs(articles_dir: &Path) -> Result<bool> {
    let docs_path = Path::new(ARCH_WIKI_DOCS_PATH);

    // Check if already installed
    if docs_path.exists() {
        tracing::info!("Using existing arch-wiki-docs");
        copy_wiki_docs(docs_path, articles_dir).await?;
        return Ok(true);
    }

    // Try to install arch-wiki-docs (non-interactively - will fail if no passwordless sudo)
    tracing::info!("Trying to install arch-wiki-docs package...");

    // Use --noconfirm and timeout to avoid blocking. This will fail if sudo needs a password.
    match Command::new("sudo")
        .args(["-n", "pacman", "-S", "--noconfirm", "arch-wiki-docs"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            // Register as installed by Anna
            if let Ok(mut registry) = HelpersRegistry::load() {
                let _ = registry.register_anna_installed(
                    "arch-wiki-docs",
                    "Offline Arch Wiki for RAG search",
                    None,
                );
            }

            // Copy the docs
            if docs_path.exists() {
                copy_wiki_docs(docs_path, articles_dir).await?;
                return Ok(true);
            }
        }
        Ok(_) => {
            tracing::info!("Could not install arch-wiki-docs (requires sudo), will use git download");
        }
        Err(e) => {
            tracing::info!("sudo not available: {}, will use git download", e);
        }
    }

    Ok(false)
}

/// Copy wiki docs from arch-wiki-docs to our directory
async fn copy_wiki_docs(source: &Path, dest: &Path) -> Result<()> {
    // Convert HTML to simple text/markdown for easier processing
    let entries = std::fs::read_dir(source)?;
    let mut count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "html") {
            let filename = path.file_stem().unwrap().to_string_lossy();
            let dest_file = dest.join(format!("{}.txt", filename));

            // Read HTML and extract text (simple extraction)
            if let Ok(content) = std::fs::read_to_string(&path) {
                let text = html_to_text(&content);
                if !text.is_empty() {
                    std::fs::write(&dest_file, &text)?;
                    count += 1;
                }
            }
        }
    }

    tracing::info!("Processed {} wiki articles", count);
    Ok(())
}

/// Download key wiki articles from API (fallback when arch-wiki-docs unavailable)
async fn download_from_git(articles_dir: &Path) -> Result<()> {
    tracing::info!("Downloading key wiki articles from Arch Wiki API...");

    // Key articles that are most useful for system administration
    let key_articles = [
        "Pacman", "Pacman/Tips_and_tricks", "AUR", "Arch_User_Repository",
        "Systemd", "Systemd/User", "GRUB", "Kernel",
        "Network_configuration", "Wireless_network_configuration", "NetworkManager",
        "Display_manager", "GDM", "SDDM", "LightDM", "Xorg", "Wayland",
        "KDE", "GNOME", "Hyprland", "Sway", "i3",
        "PulseAudio", "PipeWire", "ALSA",
        "NVIDIA", "AMDGPU", "Intel_graphics",
        "SSH", "Firewall", "UFW", "Iptables",
        "Fstab", "Mount", "LVM", "Btrfs", "Ext4",
        "Users_and_groups", "Sudo", "File_permissions_and_attributes",
        "Arch_Linux", "Installation_guide", "General_recommendations",
        "System_maintenance", "Improving_performance",
        "Locale", "Keyboard_configuration", "Console_fonts",
        "Power_management", "Laptop", "Battery",
        "Bluetooth", "Printing", "Scanner",
        "Vim", "Neovim", "Bash", "Zsh", "Fish",
        "Git", "Docker", "Flatpak", "Snap",
        "Fonts", "Cursor_themes", "HiDPI",
    ];

    let client = reqwest::Client::builder()
        .user_agent("Anna-Assistant/1.0 (Arch Linux helper)")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let mut success_count = 0;
    let total = key_articles.len();

    for (i, article) in key_articles.iter().enumerate() {
        if i % 10 == 0 {
            tracing::info!("Downloading articles: {}/{}", i, total);
        }

        // Use the MediaWiki API to get plain text content
        let url = format!(
            "https://wiki.archlinux.org/api.php?action=query&titles={}&prop=extracts&explaintext=true&format=json",
            article.replace(' ', "_")
        );

        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    // Extract the text content from the API response
                    if let Some(pages) = json.get("query").and_then(|q| q.get("pages")) {
                        if let Some(page) = pages.as_object().and_then(|p| p.values().next()) {
                            if let Some(extract) = page.get("extract").and_then(|e| e.as_str()) {
                                let filename = article.replace('/', "_");
                                let dest_file = articles_dir.join(format!("{}.txt", filename));
                                if let Ok(()) = std::fs::write(&dest_file, extract) {
                                    success_count += 1;
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                tracing::debug!("Could not download article: {}", article);
            }
        }

        // Small delay to be nice to the wiki servers
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    tracing::info!("Downloaded {} of {} wiki articles", success_count, total);

    if success_count == 0 {
        anyhow::bail!("Could not download any wiki articles");
    }

    Ok(())
}

/// Simple HTML to text conversion
fn html_to_text(html: &str) -> String {
    // Remove script and style tags
    let mut text = html.to_string();

    // Remove script tags
    while let Some(start) = text.find("<script") {
        if let Some(end) = text[start..].find("</script>") {
            text = format!("{}{}", &text[..start], &text[start + end + 9..]);
        } else {
            break;
        }
    }

    // Remove style tags
    while let Some(start) = text.find("<style") {
        if let Some(end) = text[start..].find("</style>") {
            text = format!("{}{}", &text[..start], &text[start + end + 8..]);
        } else {
            break;
        }
    }

    // Remove all HTML tags
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_pre = false;

    for c in text.chars() {
        match c {
            '<' => {
                in_tag = true;
                // Check for <pre> or <code>
                let remaining: String = text.chars().skip(result.len()).take(10).collect();
                if remaining.starts_with("<pre") || remaining.starts_with("<code") {
                    in_pre = true;
                } else if remaining.starts_with("</pre") || remaining.starts_with("</code") {
                    in_pre = false;
                }
            }
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => {
                result.push(c);
            }
            _ => {}
        }
    }

    // Decode HTML entities
    result = result
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // Clean up whitespace
    let lines: Vec<&str> = result.lines().map(|l| l.trim()).collect();
    lines.join("\n")
}

/// Update wiki (re-download)
pub async fn update_wiki() -> Result<()> {
    let articles_dir = wiki_articles_dir();

    // Remove old articles
    if articles_dir.exists() {
        std::fs::remove_dir_all(&articles_dir)?;
    }

    // Re-download
    download_wiki().await?;

    Ok(())
}
