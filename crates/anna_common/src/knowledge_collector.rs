//! Knowledge Collector v5.1.0 - Full System Inventory
//!
//! Collects data from:
//! - Package managers (pacman + package files)
//! - ALL binaries on PATH (not just known categories)
//! - Systemd services
//! - Running processes
//! - Resource usage
//!
//! v5.1.0: Paranoid archivist mode - tracks EVERYTHING executable

use crate::knowledge_core::{
    Category, DetectionSource, KnowledgeObject, KnowledgeStore,
    TelemetryAggregates, classify_tool, get_config_paths,
    ObjectType, InventoryProgress, InventoryPhase,
};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
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

/// v5.1.0: Parse package info to extract version and install date
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub install_date: Option<u64>,
}

/// v5.1.0: Get all installed packages with version info
pub fn discover_pacman_packages_full() -> Vec<PackageInfo> {
    let output = Command::new("pacman")
        .args(["-Qi"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            parse_pacman_qi_output(&String::from_utf8_lossy(&out.stdout))
        }
        _ => {
            debug!("pacman -Qi not available or failed");
            vec![]
        }
    }
}

fn parse_pacman_qi_output(output: &str) -> Vec<PackageInfo> {
    let mut packages = Vec::new();
    let mut current_name = String::new();
    let mut current_version = String::new();
    let mut current_date: Option<u64> = None;

    for line in output.lines() {
        if line.starts_with("Name            :") {
            // Save previous package if any
            if !current_name.is_empty() {
                packages.push(PackageInfo {
                    name: current_name.clone(),
                    version: current_version.clone(),
                    install_date: current_date,
                });
            }
            current_name = line.trim_start_matches("Name            :").trim().to_string();
            current_version.clear();
            current_date = None;
        } else if line.starts_with("Version         :") {
            current_version = line.trim_start_matches("Version         :").trim().to_string();
        } else if line.starts_with("Install Date    :") {
            let date_str = line.trim_start_matches("Install Date    :").trim();
            current_date = parse_pacman_date(date_str);
        }
    }

    // Don't forget the last package
    if !current_name.is_empty() {
        packages.push(PackageInfo {
            name: current_name,
            version: current_version,
            install_date: current_date,
        });
    }

    packages
}

fn parse_pacman_date(date_str: &str) -> Option<u64> {
    // pacman dates look like: "Sun 01 Dec 2024 10:30:00 AM UTC"
    // or "2024-12-01T10:30:00"
    // We'll do a simple heuristic parse
    use chrono::{DateTime, Utc, NaiveDateTime};

    // Try RFC2822-ish format first
    if let Ok(dt) = DateTime::parse_from_str(date_str, "%a %d %b %Y %I:%M:%S %p %Z") {
        return Some(dt.timestamp() as u64);
    }

    // Try ISO format
    if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt.and_utc().timestamp() as u64);
    }

    // Try another common pacman format
    if let Ok(dt) = DateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M") {
        return Some(dt.timestamp() as u64);
    }

    None
}

/// v5.1.0: List files owned by a package
pub fn get_package_files(pkg: &str) -> Vec<String> {
    let output = Command::new("pacman")
        .args(["-Ql", pkg])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|line| {
                    // Lines are: "packagename /path/to/file"
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    parts.get(1).map(|s| s.to_string())
                })
                .filter(|p| {
                    // Only keep executable files
                    p.starts_with("/usr/bin/") || p.starts_with("/usr/sbin/") ||
                    p.starts_with("/bin/") || p.starts_with("/sbin/")
                })
                .collect()
        }
        _ => vec![],
    }
}

// ============================================================================
// Systemd Service Discovery (v5.1.0)
// ============================================================================

/// v5.1.0: Systemd service info
#[derive(Debug, Clone)]
pub struct SystemdServiceInfo {
    pub name: String,
    pub unit_file: String,
    pub enabled: bool,
    pub active: bool,
}

/// v5.1.0: Discover systemd services
pub fn discover_systemd_services() -> Vec<SystemdServiceInfo> {
    let output = Command::new("systemctl")
        .args(["list-unit-files", "--type=service", "--no-legend", "--no-pager"])
        .output();

    let mut services = Vec::new();

    if let Ok(out) = output {
        if out.status.success() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let unit = parts[0];
                    let state = parts[1];

                    // Extract service name from unit file (e.g., "nginx.service" -> "nginx")
                    let name = unit.trim_end_matches(".service").to_string();

                    // Check if it's enabled
                    let enabled = matches!(state, "enabled" | "enabled-runtime");

                    // Check if it's active
                    let active = is_service_active(unit);

                    services.push(SystemdServiceInfo {
                        name,
                        unit_file: unit.to_string(),
                        enabled,
                        active,
                    });
                }
            }
        }
    }

    services
}

fn is_service_active(unit: &str) -> bool {
    let output = Command::new("systemctl")
        .args(["is-active", "--quiet", unit])
        .output();

    matches!(output, Ok(out) if out.status.success())
}

/// v5.1.0: Count systemd services (quick method)
pub fn count_systemd_services() -> usize {
    let output = Command::new("systemctl")
        .args(["list-unit-files", "--type=service", "--no-legend", "--no-pager"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).lines().count()
        }
        _ => 0,
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

/// v5.1.0: Count total binaries on PATH (quick method)
pub fn count_path_binaries() -> usize {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let mut seen = HashSet::new();

    for dir in path_var.split(':') {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        seen.insert(name.to_string());
                    }
                }
            }
        }
    }

    seen.len()
}

/// v5.1.0: Discover ALL binaries (not just known categories)
pub fn discover_all_binaries() -> Vec<(String, String)> {
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
// Knowledge Builder v5.1.0
// ============================================================================

/// Build knowledge from discovered data
pub struct KnowledgeBuilder {
    store: KnowledgeStore,
    telemetry: TelemetryAggregates,
    /// v5.1.0: Inventory progress tracking
    progress: InventoryProgress,
    /// v5.1.0: Previously known packages (for change detection)
    prev_packages: HashSet<String>,
}

impl KnowledgeBuilder {
    pub fn new() -> Self {
        let store = KnowledgeStore::load();
        // Snapshot current packages for change detection
        let prev_packages: HashSet<String> = store.objects
            .values()
            .filter(|o| o.object_types.contains(&ObjectType::Package))
            .map(|o| o.name.clone())
            .collect();

        Self {
            store,
            telemetry: TelemetryAggregates::load(),
            progress: InventoryProgress::new(),
            prev_packages,
        }
    }

    pub fn from_stores(store: KnowledgeStore, telemetry: TelemetryAggregates) -> Self {
        Self {
            store,
            telemetry,
            progress: InventoryProgress::new(),
            prev_packages: HashSet::new(),
        }
    }

    /// Get the knowledge store
    pub fn store(&self) -> &KnowledgeStore {
        &self.store
    }

    /// Get mutable knowledge store
    pub fn store_mut(&mut self) -> &mut KnowledgeStore {
        &mut self.store
    }

    /// Get the telemetry
    pub fn telemetry(&self) -> &TelemetryAggregates {
        &self.telemetry
    }

    /// v5.1.0: Get inventory progress
    pub fn progress(&self) -> &InventoryProgress {
        &self.progress
    }

    /// Consume and return stores
    pub fn into_stores(self) -> (KnowledgeStore, TelemetryAggregates) {
        (self.store, self.telemetry)
    }

    /// Collect from packages (v5.0.0 - known categories only)
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

    /// v5.1.0: Collect ALL packages with full info
    pub fn collect_packages_full(&mut self) {
        info!("[COLLECT] Full package scan...");
        let packages = discover_pacman_packages_full();
        let total = packages.len();

        self.progress.start_phase(InventoryPhase::ScanningPackages, total);

        let mut discovered = 0;
        let mut current_packages = HashSet::new();

        for (i, pkg_info) in packages.iter().enumerate() {
            current_packages.insert(pkg_info.name.clone());

            let (category, wiki_ref) = classify_tool(&pkg_info.name);
            let is_new = !self.store.objects.contains_key(&pkg_info.name);

            let obj = self.store.objects.entry(pkg_info.name.clone()).or_insert_with(|| {
                let mut o = KnowledgeObject::new(&pkg_info.name, category.clone());
                o.wiki_ref = wiki_ref.map(|s| s.to_string());
                o
            });

            // Update with full info
            obj.installed = true;
            obj.package_name = Some(pkg_info.name.clone());
            obj.package_version = Some(pkg_info.version.clone());
            obj.installed_at = pkg_info.install_date;
            obj.removed_at = None; // It's installed

            // Add Package type if not present
            if !obj.object_types.contains(&ObjectType::Package) {
                obj.object_types.push(ObjectType::Package);
            }

            // Add inventory source
            if !obj.inventory_source.contains(&"pacman_db".to_string()) {
                obj.inventory_source.push("pacman_db".to_string());
            }

            obj.detected_as = if obj.binary_path.is_some() {
                DetectionSource::Both
            } else {
                DetectionSource::Package
            };

            // Find config files
            let config_paths = get_config_paths(&pkg_info.name);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            for path in config_paths {
                if Path::new(&path).exists() && !obj.config_paths.contains(&path) {
                    obj.config_paths.push(path);
                    obj.config_discovered_at = Some(now);
                }
            }

            if is_new {
                discovered += 1;
            }

            // Update progress every 100 items
            if i % 100 == 0 {
                self.progress.update(i);
            }
        }

        // Detect removed packages
        let mut removed = 0;
        for old_pkg in &self.prev_packages {
            if !current_packages.contains(old_pkg) {
                if let Some(obj) = self.store.objects.get_mut(old_pkg) {
                    if obj.removed_at.is_none() {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        obj.removed_at = Some(now);
                        obj.installed = false;
                        removed += 1;
                    }
                }
            }
        }

        // Update prev_packages for next cycle
        self.prev_packages = current_packages;

        self.progress.update(total);

        if discovered > 0 {
            info!("[COLLECT] {} new packages", discovered);
        }
        if removed > 0 {
            info!("[COLLECT] {} packages removed", removed);
        }
    }

    /// Collect from binaries (v5.0.0 - known categories only)
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

    /// v5.1.0: Collect ALL binaries on PATH
    pub fn collect_all_binaries(&mut self) {
        info!("[COLLECT] Full PATH scan...");
        let binaries = discover_all_binaries();
        let total = binaries.len();

        self.progress.start_phase(InventoryPhase::ScanningPath, total);

        let mut discovered = 0;

        for (i, (name, path)) in binaries.iter().enumerate() {
            let (category, wiki_ref) = classify_tool(name);
            let is_new = !self.store.objects.contains_key(name);

            let obj = self.store.objects.entry(name.clone()).or_insert_with(|| {
                let mut o = KnowledgeObject::new(name, category.clone());
                o.wiki_ref = wiki_ref.map(|s| s.to_string());
                o
            });

            // Add Command type if not present
            if !obj.object_types.contains(&ObjectType::Command) {
                obj.object_types.push(ObjectType::Command);
            }

            // Add path if not already tracked
            if !obj.paths.contains(path) {
                obj.paths.push(path.clone());
            }

            // Keep binary_path as first path
            if obj.binary_path.is_none() {
                obj.binary_path = Some(path.clone());
            }

            // Add inventory source
            if !obj.inventory_source.contains(&"path_scan".to_string()) {
                obj.inventory_source.push("path_scan".to_string());
            }

            obj.detected_as = if obj.package_name.is_some() {
                DetectionSource::Both
            } else {
                DetectionSource::Binary
            };

            if is_new {
                discovered += 1;
            }

            // Update progress every 100 items
            if i % 100 == 0 {
                self.progress.update(i);
            }
        }

        self.progress.update(total);

        if discovered > 0 {
            info!("[COLLECT] {} new commands from PATH", discovered);
        }
    }

    /// v5.1.0: Collect systemd services
    pub fn collect_services(&mut self) {
        info!("[COLLECT] Scanning systemd services...");
        let services = discover_systemd_services();
        let total = services.len();

        self.progress.start_phase(InventoryPhase::ScanningServices, total);

        let mut discovered = 0;

        for (i, svc) in services.iter().enumerate() {
            let (category, wiki_ref) = classify_tool(&svc.name);
            let is_new = !self.store.objects.contains_key(&svc.name);

            let obj = self.store.objects.entry(svc.name.clone()).or_insert_with(|| {
                let mut o = KnowledgeObject::new(&svc.name, category.clone());
                o.wiki_ref = wiki_ref.map(|s| s.to_string());
                o
            });

            // Add Service type if not present
            if !obj.object_types.contains(&ObjectType::Service) {
                obj.object_types.push(ObjectType::Service);
            }

            obj.service_unit = Some(svc.unit_file.clone());
            obj.service_enabled = Some(svc.enabled);
            obj.service_active = Some(svc.active);

            // Add inventory source
            if !obj.inventory_source.contains(&"systemd".to_string()) {
                obj.inventory_source.push("systemd".to_string());
            }

            if is_new {
                discovered += 1;
            }

            // Update progress every 50 items
            if i % 50 == 0 {
                self.progress.update(i);
            }
        }

        self.progress.update(total);

        if discovered > 0 {
            info!("[COLLECT] {} new services", discovered);
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

    /// Run full collection cycle (v5.0.0 compatible)
    pub fn collect_all(&mut self) {
        self.collect_packages();
        self.collect_binaries();
        self.collect_processes();
    }

    /// v5.1.0: Run full inventory scan with progress tracking
    pub fn collect_full_inventory(&mut self) {
        info!("[INVENTORY] Starting full system inventory...");

        // Phase 1: PATH binaries
        self.collect_all_binaries();

        // Phase 2: Packages with full info
        self.collect_packages_full();

        // Phase 3: Systemd services
        self.collect_services();

        // Mark complete
        self.progress.complete();

        let (commands, packages, services) = self.store.count_by_type();
        info!("[INVENTORY] Complete: {} commands, {} packages, {} services",
            commands, packages, services);
    }

    /// v5.1.1: Targeted discovery for a single object (priority scan)
    ///
    /// Performs deep scan for a specific name across:
    /// - PATH binaries
    /// - pacman database
    /// - systemd services
    /// - config locations
    ///
    /// Returns true if the object was found and updated
    pub fn targeted_discovery(&mut self, name: &str) -> bool {
        info!("[PRIORITY] Targeted discovery for '{}'", name);
        self.progress.start_priority_scan(name);

        let mut found = false;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Check 1: PATH binary
        if let Some(path) = binary_exists(name) {
            found = true;
            let (category, wiki_ref) = classify_tool(name);
            let obj = self.store.objects.entry(name.to_string()).or_insert_with(|| {
                let mut o = KnowledgeObject::new(name, category.clone());
                o.wiki_ref = wiki_ref.map(|s| s.to_string());
                o
            });

            if !obj.object_types.contains(&ObjectType::Command) {
                obj.object_types.push(ObjectType::Command);
            }
            if !obj.paths.contains(&path) {
                obj.paths.push(path.clone());
            }
            if obj.binary_path.is_none() {
                obj.binary_path = Some(path);
            }
            if !obj.inventory_source.contains(&"priority_scan".to_string()) {
                obj.inventory_source.push("priority_scan".to_string());
            }
        }

        // Check 2: pacman package
        if let Some(pkg_info) = get_pacman_package_info(name) {
            found = true;
            let (category, wiki_ref) = classify_tool(name);
            let obj = self.store.objects.entry(name.to_string()).or_insert_with(|| {
                let mut o = KnowledgeObject::new(name, category.clone());
                o.wiki_ref = wiki_ref.map(|s| s.to_string());
                o
            });

            obj.installed = true;
            obj.package_name = Some(name.to_string());
            if !obj.object_types.contains(&ObjectType::Package) {
                obj.object_types.push(ObjectType::Package);
            }
            if !obj.inventory_source.contains(&"priority_scan".to_string()) {
                obj.inventory_source.push("priority_scan".to_string());
            }

            // Parse version from pkg_info
            for line in pkg_info.lines() {
                if line.starts_with("Version") {
                    if let Some(ver) = line.split(':').nth(1) {
                        obj.package_version = Some(ver.trim().to_string());
                    }
                }
            }
        }

        // Check 3: systemd service
        let services = discover_systemd_services();
        for svc in &services {
            if svc.name == name || svc.name.starts_with(name) {
                found = true;
                let (category, wiki_ref) = classify_tool(name);
                let obj = self.store.objects.entry(name.to_string()).or_insert_with(|| {
                    let mut o = KnowledgeObject::new(name, category.clone());
                    o.wiki_ref = wiki_ref.map(|s| s.to_string());
                    o
                });

                if !obj.object_types.contains(&ObjectType::Service) {
                    obj.object_types.push(ObjectType::Service);
                }
                obj.service_unit = Some(svc.unit_file.clone());
                obj.service_enabled = Some(svc.enabled);
                obj.service_active = Some(svc.active);
                if !obj.inventory_source.contains(&"priority_scan".to_string()) {
                    obj.inventory_source.push("priority_scan".to_string());
                }
                break;
            }
        }

        // Check 4: Config files
        let config_paths = get_config_paths(name);
        for path in config_paths {
            if Path::new(&path).exists() {
                if let Some(obj) = self.store.objects.get_mut(name) {
                    if !obj.config_paths.contains(&path) {
                        obj.config_paths.push(path);
                        obj.config_discovered_at = Some(now);
                    }
                }
            }
        }

        // Update detection source
        if let Some(obj) = self.store.objects.get_mut(name) {
            obj.detected_as = if obj.package_name.is_some() && obj.binary_path.is_some() {
                DetectionSource::Both
            } else if obj.package_name.is_some() {
                DetectionSource::Package
            } else if obj.binary_path.is_some() {
                DetectionSource::Binary
            } else {
                DetectionSource::Unknown
            };
            obj.last_seen_at = now;
        }

        self.progress.end_priority_scan();

        if found {
            info!("[PRIORITY] Found and updated '{}'", name);
        } else {
            debug!("[PRIORITY] '{}' not found on system", name);
        }

        found
    }

    /// v5.1.1: Check if an object exists and is complete
    pub fn is_object_complete(&self, name: &str) -> bool {
        if let Some(obj) = self.store.get(name) {
            // Consider complete if we have at least one type and some info
            !obj.object_types.is_empty() &&
            (obj.binary_path.is_some() || obj.package_name.is_some() || obj.service_unit.is_some())
        } else {
            false
        }
    }

    /// v5.1.1: Ensure object is up-to-date, performing targeted discovery if needed
    pub fn ensure_object_fresh(&mut self, name: &str) -> bool {
        if !self.is_object_complete(name) {
            self.targeted_discovery(name)
        } else {
            true
        }
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
        // Using explicit variable to avoid warning
        let _count = binaries.len();
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
