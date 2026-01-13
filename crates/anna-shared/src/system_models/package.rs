//! Package Model - Official vs AUR, file ownership, hooks, partial upgrade risks.
//!
//! Models the package state for understanding:
//! - What packages are installed and their sources
//! - File ownership and conflicts
//! - Pre/post transaction hooks
//! - Partial upgrade risks

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Complete package model
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageModel {
    /// Installed packages
    pub packages: HashMap<String, Package>,
    /// Package groups
    pub groups: HashMap<String, Vec<String>>,
    /// Orphaned packages
    pub orphans: HashSet<String>,
    /// Explicitly installed packages
    pub explicit: HashSet<String>,
    /// Foreign packages (AUR, manual, etc.)
    pub foreign: HashSet<String>,
    /// Known upgrade risks
    pub upgrade_risks: Vec<UpgradeRisk>,
    /// Package hooks
    pub hooks: Vec<PackageHook>,
    /// Last sync time
    pub last_sync: Option<String>,
    /// Packages with available updates
    pub updates_available: HashMap<String, PackageUpdate>,
}

/// A package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Package name
    pub name: String,
    /// Installed version
    pub version: String,
    /// Package description
    pub description: String,
    /// Package source
    pub source: PackageSource,
    /// Repository (for official packages)
    pub repository: Option<String>,
    /// Dependencies
    pub depends: Vec<String>,
    /// Optional dependencies
    pub optdepends: Vec<String>,
    /// Packages this provides
    pub provides: Vec<String>,
    /// Packages this conflicts with
    pub conflicts: Vec<String>,
    /// Install reason
    pub reason: InstallReason,
    /// Install date
    pub install_date: Option<String>,
    /// Installed size in bytes
    pub size: u64,
    /// Files owned by this package
    pub files: Vec<String>,
    /// Package URL
    pub url: Option<String>,
    /// Packager
    pub packager: Option<String>,
}

/// Package source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageSource {
    /// Official Arch repository
    Official,
    /// Arch User Repository
    Aur,
    /// Manually installed
    Manual,
    /// From a local file
    Local,
    /// Unknown source
    Unknown,
}

impl PackageSource {
    pub fn from_repo(repo: Option<&str>) -> Self {
        match repo {
            Some("core") | Some("extra") | Some("multilib") => PackageSource::Official,
            Some("aur") => PackageSource::Aur,
            Some("local") => PackageSource::Local,
            None => PackageSource::Unknown,
            _ => PackageSource::Unknown,
        }
    }

    /// Risk level for packages from this source
    pub fn risk_level(&self) -> SourceRisk {
        match self {
            PackageSource::Official => SourceRisk::Low,
            PackageSource::Aur => SourceRisk::Medium,
            PackageSource::Manual => SourceRisk::High,
            PackageSource::Local => SourceRisk::Medium,
            PackageSource::Unknown => SourceRisk::High,
        }
    }
}

/// Source risk level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceRisk {
    Low,
    Medium,
    High,
}

/// Install reason
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallReason {
    Explicit,
    Dependency,
    Unknown,
}

impl InstallReason {
    pub fn from_str(s: &str) -> Self {
        if s.contains("Explicitly") || s.contains("explicit") {
            InstallReason::Explicit
        } else if s.contains("dependency") {
            InstallReason::Dependency
        } else {
            InstallReason::Unknown
        }
    }
}

/// Available package update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageUpdate {
    /// Package name
    pub name: String,
    /// Current version
    pub current: String,
    /// New version
    pub new: String,
    /// Download size
    pub download_size: u64,
    /// Is this a security update?
    pub security: bool,
}

/// Upgrade risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeRisk {
    /// Risk type
    pub risk_type: UpgradeRiskType,
    /// Affected packages
    pub packages: Vec<String>,
    /// Risk description
    pub description: String,
    /// Mitigation steps
    pub mitigation: Vec<String>,
    /// Risk severity
    pub severity: RiskSeverity,
}

/// Types of upgrade risks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeRiskType {
    /// Partial upgrade (mixed package versions)
    PartialUpgrade,
    /// Library soname bump
    LibraryBump,
    /// Kernel update requiring reboot
    KernelUpdate,
    /// Init system changes
    InitChange,
    /// Package replacement
    PackageReplacement,
    /// Config file conflict
    ConfigConflict,
    /// AUR package needs rebuild
    AurRebuild,
}

/// Risk severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Package hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageHook {
    /// Hook name
    pub name: String,
    /// Hook type
    pub hook_type: HookType,
    /// When this hook runs
    pub when: HookTiming,
    /// Target packages/operations
    pub targets: Vec<String>,
    /// Script/command to run
    pub exec: String,
}

/// Hook type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookType {
    /// System hook (from packages)
    System,
    /// User hook
    User,
}

/// Hook timing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookTiming {
    PreTransaction,
    PostTransaction,
}

impl PackageModel {
    /// Create new empty package model
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a package
    pub fn upsert_package(&mut self, pkg: Package) {
        if pkg.reason == InstallReason::Explicit {
            self.explicit.insert(pkg.name.clone());
        }
        if pkg.source != PackageSource::Official {
            self.foreign.insert(pkg.name.clone());
        }
        self.packages.insert(pkg.name.clone(), pkg);
    }

    /// Get package by name
    pub fn get(&self, name: &str) -> Option<&Package> {
        self.packages.get(name)
    }

    /// Count risk factors
    pub fn count_risks(&self) -> usize {
        self.upgrade_risks
            .iter()
            .filter(|r| matches!(r.severity, RiskSeverity::High | RiskSeverity::Critical))
            .count()
    }

    /// Check for partial upgrade risk
    pub fn check_partial_upgrade(&self) -> Option<UpgradeRisk> {
        // Check if any core libraries have pending updates while
        // packages depending on them don't
        let core_libs = ["glibc", "gcc-libs", "openssl", "zlib"];
        let mut affected = Vec::new();

        for lib in core_libs {
            if self.updates_available.contains_key(lib) {
                // Find packages depending on this
                for (name, pkg) in &self.packages {
                    if pkg.depends.iter().any(|d| d.starts_with(lib)) {
                        if !self.updates_available.contains_key(name) {
                            affected.push(name.clone());
                        }
                    }
                }
            }
        }

        if !affected.is_empty() {
            Some(UpgradeRisk {
                risk_type: UpgradeRiskType::PartialUpgrade,
                packages: affected,
                description: "Core library update may break dependent packages".to_string(),
                mitigation: vec![
                    "Run full system upgrade: pacman -Syu".to_string(),
                    "Never upgrade individual packages".to_string(),
                ],
                severity: RiskSeverity::High,
            })
        } else {
            None
        }
    }

    /// Check for AUR packages needing rebuild after update
    pub fn check_aur_rebuild_needed(&self) -> Vec<UpgradeRisk> {
        let mut risks = Vec::new();

        for pkg_name in &self.foreign {
            if let Some(pkg) = self.packages.get(pkg_name) {
                // Check if any dependency has an update
                for dep in &pkg.depends {
                    let dep_name = dep.split(|c| c == '>' || c == '<' || c == '=').next().unwrap_or(dep);
                    if self.updates_available.contains_key(dep_name) {
                        risks.push(UpgradeRisk {
                            risk_type: UpgradeRiskType::AurRebuild,
                            packages: vec![pkg_name.clone()],
                            description: format!(
                                "AUR package {} may need rebuild after {} update",
                                pkg_name, dep_name
                            ),
                            mitigation: vec![
                                format!("Rebuild {} after system upgrade", pkg_name),
                                "Use AUR helper with rebuild support".to_string(),
                            ],
                            severity: RiskSeverity::Medium,
                        });
                        break;
                    }
                }
            }
        }

        risks
    }

    /// Detect orphaned packages
    pub fn detect_orphans(&mut self) {
        let mut orphans = HashSet::new();

        for (name, pkg) in &self.packages {
            if pkg.reason == InstallReason::Dependency {
                // Check if anything depends on this
                let has_dependent = self.packages.values().any(|p| {
                    p.depends.iter().any(|d| {
                        let d_name = d.split(|c| c == '>' || c == '<' || c == '=').next().unwrap_or(d);
                        d_name == name || pkg.provides.iter().any(|pr| pr.starts_with(d_name))
                    })
                });

                if !has_dependent {
                    orphans.insert(name.clone());
                }
            }
        }

        self.orphans = orphans;
    }

    /// Get packages from a specific source
    pub fn packages_by_source(&self, source: PackageSource) -> Vec<&Package> {
        self.packages.values().filter(|p| p.source == source).collect()
    }

    /// Find packages owning a file
    pub fn find_file_owner(&self, path: &str) -> Option<&Package> {
        self.packages
            .values()
            .find(|p| p.files.iter().any(|f| f == path || path.starts_with(f)))
    }

    /// Analyze upgrade safety
    pub fn analyze_upgrade(&mut self) {
        self.upgrade_risks.clear();

        // Check partial upgrade
        if let Some(risk) = self.check_partial_upgrade() {
            self.upgrade_risks.push(risk);
        }

        // Check AUR rebuilds
        self.upgrade_risks.extend(self.check_aur_rebuild_needed());

        // Check for kernel updates
        if self.updates_available.contains_key("linux")
            || self.updates_available.contains_key("linux-lts")
        {
            self.upgrade_risks.push(UpgradeRisk {
                risk_type: UpgradeRiskType::KernelUpdate,
                packages: vec!["linux".to_string()],
                description: "Kernel update requires reboot".to_string(),
                mitigation: vec![
                    "Save all work before upgrading".to_string(),
                    "Plan for immediate reboot".to_string(),
                    "Update DKMS modules if applicable".to_string(),
                ],
                severity: RiskSeverity::Medium,
            });
        }
    }
}

impl Package {
    /// Create a new package
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: String::new(),
            source: PackageSource::Unknown,
            repository: None,
            depends: Vec::new(),
            optdepends: Vec::new(),
            provides: Vec::new(),
            conflicts: Vec::new(),
            reason: InstallReason::Unknown,
            install_date: None,
            size: 0,
            files: Vec::new(),
            url: None,
            packager: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_source_risk() {
        assert_eq!(PackageSource::Official.risk_level(), SourceRisk::Low);
        assert_eq!(PackageSource::Aur.risk_level(), SourceRisk::Medium);
        assert_eq!(PackageSource::Manual.risk_level(), SourceRisk::High);
    }

    #[test]
    fn test_orphan_detection() {
        let mut model = PackageModel::new();

        // Add a dependency package
        let mut dep_pkg = Package::new("libfoo", "1.0");
        dep_pkg.reason = InstallReason::Dependency;
        model.upsert_package(dep_pkg);

        // Add an explicit package that depends on it
        let mut main_pkg = Package::new("foo", "1.0");
        main_pkg.reason = InstallReason::Explicit;
        main_pkg.depends.push("libfoo".to_string());
        model.upsert_package(main_pkg);

        model.detect_orphans();

        // libfoo should not be an orphan
        assert!(!model.orphans.contains("libfoo"));

        // Now remove the dependent
        model.packages.remove("foo");
        model.detect_orphans();

        // libfoo should now be an orphan
        assert!(model.orphans.contains("libfoo"));
    }

    #[test]
    fn test_aur_package_tracking() {
        let mut model = PackageModel::new();

        let mut pkg = Package::new("yay", "12.0");
        pkg.source = PackageSource::Aur;
        model.upsert_package(pkg);

        assert!(model.foreign.contains("yay"));
        assert_eq!(model.packages_by_source(PackageSource::Aur).len(), 1);
    }
}
