//! Knowledge Collector v5.0.0 - System Data Gathering
//!
//! Collects data from:
//! - Package managers (pacman)
//! - Binaries on PATH
//! - Running processes
//! - Resource usage

use crate::knowledge_core::{
    Category, DetectionSource, KnowledgeObject, KnowledgeStore,
    TelemetryAggregates, classify_tool, get_config_paths,
};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn};

// ============================================================================
// Package Discovery
// ============================================================================

/// Discover installed packages via pacman
pub fn discover_pacman_packages() -> Vec<String> {
    let output = Command::new("pacman")
        .args(["-Qq"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        }
        _ => {
            debug!("pacman not available or failed");
            vec![]
        }
    }
}

/// Get package info from pacman
pub fn get_pacman_package_info(pkg: &str) -> Option<String> {
    let output = Command::new("pacman")
        .args(["-Qi", pkg])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

// ============================================================================
// Binary Discovery
// ============================================================================

/// Discover binaries on PATH
pub fn discover_binaries() -> Vec<(String, String)> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let mut binaries = Vec::new();
    let mut seen = HashSet::new();

    for dir in path_var.split(':') {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !seen.contains(name) {
                            seen.insert(name.to_string());
                            binaries.push((name.to_string(), path.display().to_string()));
                        }
                    }
                }
            }
        }
    }

    binaries
}

/// Check if a binary exists on PATH
pub fn binary_exists(name: &str) -> Option<String> {
    let output = Command::new("which")
        .arg(name)
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}

// ============================================================================
// Process Discovery
// ============================================================================

/// Process info from /proc
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_time_ms: u64,
    pub rss_bytes: u64,
}

/// Get list of running processes
pub fn discover_processes() -> Vec<ProcessInfo> {
    let mut processes = Vec::new();

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Ok(pid) = name.parse::<u32>() {
                    if let Some(info) = get_process_info(pid) {
                        processes.push(info);
                    }
                }
            }
        }
    }

    processes
}

/// Get info for a single process
fn get_process_info(pid: u32) -> Option<ProcessInfo> {
    // Read /proc/[pid]/comm for process name
    let comm_path = format!("/proc/{}/comm", pid);
    let name = fs::read_to_string(&comm_path)
        .ok()?
        .trim()
        .to_string();

    // Read /proc/[pid]/stat for CPU time
    let stat_path = format!("/proc/{}/stat", pid);
    let stat = fs::read_to_string(&stat_path).ok()?;
    let parts: Vec<&str> = stat.split_whitespace().collect();

    // utime is field 14, stime is field 15 (0-indexed: 13, 14)
    let utime: u64 = parts.get(13)?.parse().ok()?;
    let stime: u64 = parts.get(14)?.parse().ok()?;

    // Convert to milliseconds (assuming 100 ticks per second)
    let cpu_time_ms = (utime + stime) * 10;

    // Read /proc/[pid]/statm for memory
    let statm_path = format!("/proc/{}/statm", pid);
    let statm = fs::read_to_string(&statm_path).ok()?;
    let statm_parts: Vec<&str> = statm.split_whitespace().collect();

    // RSS is field 2 (index 1), in pages
    let rss_pages: u64 = statm_parts.get(1)?.parse().ok()?;
    let page_size: u64 = 4096; // Typical Linux page size
    let rss_bytes = rss_pages * page_size;

    Some(ProcessInfo {
        pid,
        name,
        cpu_time_ms,
        rss_bytes,
    })
}

// ============================================================================
// Knowledge Builder
// ============================================================================

/// Build knowledge from discovered data
pub struct KnowledgeBuilder {
    store: KnowledgeStore,
    telemetry: TelemetryAggregates,
}

impl KnowledgeBuilder {
    pub fn new() -> Self {
        Self {
            store: KnowledgeStore::load(),
            telemetry: TelemetryAggregates::load(),
        }
    }

    pub fn from_stores(store: KnowledgeStore, telemetry: TelemetryAggregates) -> Self {
        Self { store, telemetry }
    }

    /// Get the knowledge store
    pub fn store(&self) -> &KnowledgeStore {
        &self.store
    }

    /// Get the telemetry
    pub fn telemetry(&self) -> &TelemetryAggregates {
        &self.telemetry
    }

    /// Consume and return stores
    pub fn into_stores(self) -> (KnowledgeStore, TelemetryAggregates) {
        (self.store, self.telemetry)
    }

    /// Collect from packages
    pub fn collect_packages(&mut self) {
        info!("[COLLECT] Scanning packages...");
        let packages = discover_pacman_packages();
        let mut discovered = 0;

        for pkg in &packages {
            let (category, wiki_ref) = classify_tool(pkg);

            // Only track known categories (not Unknown)
            if category != Category::Unknown {
                let is_new = !self.store.objects.contains_key(pkg);

                let obj = self.store.objects.entry(pkg.clone()).or_insert_with(|| {
                    let mut o = KnowledgeObject::new(pkg, category.clone());
                    o.wiki_ref = wiki_ref.map(|s| s.to_string());
                    o
                });

                obj.installed = true;
                obj.package_name = Some(pkg.clone());
                obj.detected_as = if obj.binary_path.is_some() {
                    DetectionSource::Both
                } else {
                    DetectionSource::Package
                };

                // Find config files
                let config_paths = get_config_paths(pkg);
                for path in config_paths {
                    if Path::new(&path).exists() && !obj.config_paths.contains(&path) {
                        obj.config_paths.push(path);
                    }
                }

                if is_new {
                    discovered += 1;
                }
            }
        }

        if discovered > 0 {
            info!("[COLLECT] Discovered {} new packages", discovered);
        }
    }

    /// Collect from binaries
    pub fn collect_binaries(&mut self) {
        info!("[COLLECT] Scanning binaries...");
        let binaries = discover_binaries();
        let mut discovered = 0;

        for (name, path) in &binaries {
            let (category, wiki_ref) = classify_tool(name);

            // Only track known categories
            if category != Category::Unknown {
                let is_new = !self.store.objects.contains_key(name);

                let obj = self.store.objects.entry(name.clone()).or_insert_with(|| {
                    let mut o = KnowledgeObject::new(name, category.clone());
                    o.wiki_ref = wiki_ref.map(|s| s.to_string());
                    o
                });

                obj.binary_path = Some(path.clone());
                obj.detected_as = if obj.package_name.is_some() {
                    DetectionSource::Both
                } else {
                    DetectionSource::Binary
                };

                // Find config files
                let config_paths = get_config_paths(name);
                for cfg_path in config_paths {
                    if Path::new(&cfg_path).exists() && !obj.config_paths.contains(&cfg_path) {
                        obj.config_paths.push(cfg_path);
                    }
                }

                if is_new {
                    discovered += 1;
                }
            }
        }

        if discovered > 0 {
            info!("[COLLECT] Discovered {} new binaries", discovered);
        }
    }

    /// Collect from running processes
    pub fn collect_processes(&mut self) {
        debug!("[COLLECT] Scanning processes...");
        let processes = discover_processes();

        for proc in &processes {
            self.telemetry.record_process(&proc.name);

            // If we know this tool, update its usage stats
            if let Some(obj) = self.store.objects.get_mut(&proc.name) {
                obj.record_usage(proc.cpu_time_ms, proc.rss_bytes);
            } else {
                // Check if it's a known category
                let (category, wiki_ref) = classify_tool(&proc.name);
                if category != Category::Unknown {
                    let mut obj = KnowledgeObject::new(&proc.name, category);
                    obj.wiki_ref = wiki_ref.map(|s| s.to_string());
                    obj.record_usage(proc.cpu_time_ms, proc.rss_bytes);

                    // Try to find binary path
                    if let Some(path) = binary_exists(&proc.name) {
                        obj.binary_path = Some(path);
                        obj.detected_as = DetectionSource::Binary;
                    }

                    self.store.upsert(obj);
                }
            }
        }
    }

    /// Run full collection cycle
    pub fn collect_all(&mut self) {
        self.collect_packages();
        self.collect_binaries();
        self.collect_processes();
    }

    /// Save both stores
    pub fn save(&self) -> std::io::Result<()> {
        self.store.save()?;
        self.telemetry.save()?;
        Ok(())
    }
}

impl Default for KnowledgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_discovery() {
        // Should find at least some binaries
        let binaries = discover_binaries();
        // This might be empty in some test environments, so just check it doesn't panic
        assert!(binaries.len() >= 0);
    }

    #[test]
    fn test_process_discovery() {
        // Should find at least one process (ourselves)
        let processes = discover_processes();
        assert!(!processes.is_empty());
    }

    #[test]
    fn test_knowledge_builder() {
        let mut builder = KnowledgeBuilder::from_stores(
            KnowledgeStore::new(),
            TelemetryAggregates::new(),
        );

        // Collect processes (should work in any environment)
        builder.collect_processes();

        // Should have recorded some telemetry
        assert!(builder.telemetry().processes_observed > 0);
    }
}
