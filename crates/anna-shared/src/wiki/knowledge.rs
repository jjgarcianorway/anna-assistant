//! Wiki Knowledge Extractor - Learn from the Arch Wiki automatically.
//!
//! Instead of hardcoding config file locations and commands, Anna reads the wiki
//! and extracts this knowledge automatically:
//! - Config file locations per topic
//! - Common commands for different tasks
//! - Package -> config file mappings
//! - Troubleshooting patterns
//!
//! This makes Anna's knowledge grow with the wiki, not with hardcoding.

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::wiki_articles_dir;
use crate::config::anna_data_dir;

/// Extracted knowledge from wiki
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WikiKnowledge {
    /// Config files mentioned per topic
    pub config_files: HashMap<String, Vec<ConfigFileInfo>>,

    /// Commands commonly used per topic
    pub topic_commands: HashMap<String, Vec<String>>,

    /// Package to config file mappings
    pub package_configs: HashMap<String, Vec<String>>,

    /// Service to config file mappings
    pub service_configs: HashMap<String, Vec<String>>,

    /// When this knowledge was extracted
    pub extracted_at: Option<String>,

    /// Statistics
    pub stats: KnowledgeStats,
}

/// Information about a config file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileInfo {
    /// Path to the config file
    pub path: String,

    /// What this config file controls
    pub description: Option<String>,

    /// Article where this was found
    pub source_article: String,
}

/// Statistics about extracted knowledge
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeStats {
    /// Number of articles processed
    pub articles_processed: u32,

    /// Number of config files found
    pub config_files_found: u32,

    /// Number of commands extracted
    pub commands_extracted: u32,
}

impl WikiKnowledge {
    /// Load knowledge from disk
    pub fn load() -> Result<Self> {
        let path = knowledge_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let knowledge: WikiKnowledge = serde_json::from_str(&content)?;
            Ok(knowledge)
        } else {
            Ok(WikiKnowledge::default())
        }
    }

    /// Save knowledge to disk
    pub fn save(&self) -> Result<()> {
        let path = knowledge_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Extract knowledge from all downloaded wiki articles
    pub fn extract_from_wiki(&mut self) -> Result<()> {
        let articles_dir = wiki_articles_dir();
        if !articles_dir.exists() {
            return Ok(());
        }

        // Config file patterns to look for
        let config_pattern = Regex::new(r"(?m)(?:^|\s)(/(?:etc|home/\.[a-z]|~)/[a-zA-Z0-9_./-]+(?:\.conf|\.cfg|\.ini|\.yaml|\.yml|\.json|\.toml|rc)?)\b").unwrap();

        // Command patterns (lines starting with $ or #)
        let command_pattern = Regex::new(r"(?m)^[$#]\s*(.+)$").unwrap();

        // Package patterns
        let package_pattern = Regex::new(r"(?:pacman -S(?:yu)?|install)\s+([a-z0-9-]+)").unwrap();

        // Service patterns
        let service_pattern = Regex::new(r"systemctl\s+(?:enable|start|restart)\s+([a-z0-9-]+)(?:\.service)?").unwrap();

        for entry in std::fs::read_dir(&articles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "txt").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let article_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // Infer topic from article name
                    let topic = infer_topic(&article_name);

                    // Extract config files
                    for cap in config_pattern.captures_iter(&content) {
                        let config_path = cap[1].to_string();
                        // Clean up the path
                        let clean_path = config_path
                            .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '-' && c != '_');

                        let info = ConfigFileInfo {
                            path: clean_path.to_string(),
                            description: None,
                            source_article: article_name.clone(),
                        };

                        self.config_files
                            .entry(topic.clone())
                            .or_default()
                            .push(info);

                        self.stats.config_files_found += 1;
                    }

                    // Extract commands
                    for cap in command_pattern.captures_iter(&content) {
                        let cmd = cap[1].trim().to_string();
                        if cmd.len() > 5 && !cmd.contains("...") && !cmd.starts_with('#') {
                            self.topic_commands
                                .entry(topic.clone())
                                .or_default()
                                .push(cmd);
                            self.stats.commands_extracted += 1;
                        }
                    }

                    // Extract package -> config mappings
                    for cap in package_pattern.captures_iter(&content) {
                        let package = cap[1].to_string();
                        // Find config files mentioned nearby
                        for cfg_cap in config_pattern.captures_iter(&content) {
                            let cfg_path = cfg_cap[1].to_string();
                            self.package_configs
                                .entry(package.clone())
                                .or_default()
                                .push(cfg_path);
                        }
                    }

                    // Extract service -> config mappings
                    for cap in service_pattern.captures_iter(&content) {
                        let service = cap[1].to_string();
                        for cfg_cap in config_pattern.captures_iter(&content) {
                            let cfg_path = cfg_cap[1].to_string();
                            self.service_configs
                                .entry(service.clone())
                                .or_default()
                                .push(cfg_path);
                        }
                    }

                    self.stats.articles_processed += 1;
                }
            }
        }

        // Deduplicate
        for configs in self.config_files.values_mut() {
            configs.sort_by(|a, b| a.path.cmp(&b.path));
            configs.dedup_by(|a, b| a.path == b.path);
        }

        for commands in self.topic_commands.values_mut() {
            commands.sort();
            commands.dedup();
        }

        for configs in self.package_configs.values_mut() {
            configs.sort();
            configs.dedup();
        }

        for configs in self.service_configs.values_mut() {
            configs.sort();
            configs.dedup();
        }

        self.extracted_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// Get config files related to a topic
    pub fn get_config_files(&self, topic: &str) -> Vec<&ConfigFileInfo> {
        let topic_lower = topic.to_lowercase();

        let mut results = Vec::new();

        // Direct match
        if let Some(configs) = self.config_files.get(&topic_lower) {
            results.extend(configs.iter());
        }

        // Fuzzy match - find topics containing the query
        for (key, configs) in &self.config_files {
            if key.contains(&topic_lower) || topic_lower.contains(key) {
                results.extend(configs.iter());
            }
        }

        results
    }

    /// Get commands commonly used for a topic
    pub fn get_topic_commands(&self, topic: &str) -> Vec<&String> {
        let topic_lower = topic.to_lowercase();

        let mut results = Vec::new();

        if let Some(cmds) = self.topic_commands.get(&topic_lower) {
            results.extend(cmds.iter());
        }

        // Fuzzy match
        for (key, cmds) in &self.topic_commands {
            if key.contains(&topic_lower) || topic_lower.contains(key) {
                results.extend(cmds.iter());
            }
        }

        results
    }

    /// Get config files for a package
    pub fn get_package_configs(&self, package: &str) -> Vec<&String> {
        self.package_configs
            .get(package)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Get config files for a service
    pub fn get_service_configs(&self, service: &str) -> Vec<&String> {
        // Try with and without .service suffix
        let service_clean = service.trim_end_matches(".service");

        self.service_configs
            .get(service_clean)
            .or_else(|| self.service_configs.get(service))
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Check if knowledge needs refresh (older than 7 days or empty)
    pub fn needs_refresh(&self) -> bool {
        if self.stats.articles_processed == 0 {
            return true;
        }

        if let Some(ref extracted) = self.extracted_at {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(extracted) {
                let age = chrono::Utc::now().signed_duration_since(dt);
                return age.num_days() > 7;
            }
        }

        true
    }
}

/// Infer topic from article name
fn infer_topic(article_name: &str) -> String {
    let name_lower = article_name.to_lowercase();

    // Map common article names to topics
    let topic_mappings = [
        ("network", &["networkmanager", "systemd-networkd", "netctl", "wifi", "ethernet", "wireless"][..]),
        ("audio", &["pulseaudio", "pipewire", "alsa", "sound"]),
        ("display", &["xorg", "wayland", "sway", "gnome", "kde", "display"]),
        ("boot", &["grub", "systemd-boot", "refind", "boot", "kernel"]),
        ("storage", &["btrfs", "ext4", "zfs", "lvm", "mount", "fstab", "partition"]),
        ("gpu", &["nvidia", "amd", "intel", "gpu", "graphics"]),
        ("security", &["firewall", "iptables", "nftables", "security", "ssh", "gpg"]),
        ("shell", &["bash", "zsh", "fish", "shell"]),
        ("editor", &["vim", "neovim", "emacs", "nano", "editor"]),
    ];

    for (topic, keywords) in topic_mappings {
        if keywords.iter().any(|k| name_lower.contains(k)) {
            return topic.to_string();
        }
    }

    // Default to the article name itself as topic
    name_lower.replace('_', "-")
}

/// Get knowledge storage path
pub fn knowledge_path() -> PathBuf {
    anna_data_dir().join("wiki_knowledge.json")
}
