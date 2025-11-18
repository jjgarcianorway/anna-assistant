//! CPU performance detection
//!
//! Detects CPU performance configuration:
//! - CPU governor settings
//! - Microcode version
//! - CPU flags (SSE, AVX, etc.)

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

/// CPU performance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuPerformance {
    /// CPU governor per core (or global if all cores use same governor)
    pub governor: GovernorInfo,
    /// Microcode package installed
    pub microcode_package: Option<String>,
    /// Microcode version
    pub microcode_version: Option<String>,
    /// Important CPU flags
    pub cpu_flags: CpuFlags,
}

/// CPU governor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorInfo {
    /// All cores use the same governor
    pub uniform: bool,
    /// Governor name (if uniform) or most common governor
    pub governor: String,
    /// Per-core governors (if not uniform)
    pub per_core: Option<Vec<String>>,
}

/// Important CPU flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuFlags {
    /// SSE support
    pub sse: bool,
    /// SSE2 support
    pub sse2: bool,
    /// SSE3 support
    pub sse3: bool,
    /// SSSE3 support
    pub ssse3: bool,
    /// SSE4.1 support
    pub sse4_1: bool,
    /// SSE4.2 support
    pub sse4_2: bool,
    /// AVX support
    pub avx: bool,
    /// AVX2 support
    pub avx2: bool,
    /// AVX-512 support
    pub avx512f: bool,
    /// AES-NI support
    pub aes: bool,
    /// Hardware virtualization (VMX for Intel, SVM for AMD)
    pub virtualization: bool,
}

impl CpuPerformance {
    /// Detect CPU performance configuration
    pub fn detect() -> Self {
        let governor = detect_cpu_governor();
        let (microcode_package, microcode_version) = detect_microcode();
        let cpu_flags = detect_cpu_flags();

        Self {
            governor,
            microcode_package,
            microcode_version,
            cpu_flags,
        }
    }
}

/// Detect CPU governor
fn detect_cpu_governor() -> GovernorInfo {
    let mut governors = Vec::new();

    // Read governor from each CPU
    let mut cpu_num = 0;
    loop {
        let governor_path = format!(
            "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
            cpu_num
        );
        if let Ok(governor) = fs::read_to_string(&governor_path) {
            governors.push(governor.trim().to_string());
            cpu_num += 1;
        } else {
            break;
        }
    }

    if governors.is_empty() {
        // No cpufreq info available
        return GovernorInfo {
            uniform: true,
            governor: "unknown".to_string(),
            per_core: None,
        };
    }

    // Check if all governors are the same
    let first_governor = &governors[0];
    let uniform = governors.iter().all(|g| g == first_governor);

    if uniform {
        GovernorInfo {
            uniform: true,
            governor: first_governor.clone(),
            per_core: None,
        }
    } else {
        // Find most common governor
        let mut counts = std::collections::HashMap::new();
        for gov in &governors {
            *counts.entry(gov.clone()).or_insert(0) += 1;
        }
        let most_common = counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(gov, _)| gov.clone())
            .unwrap_or_else(|| "mixed".to_string());

        GovernorInfo {
            uniform: false,
            governor: most_common,
            per_core: Some(governors),
        }
    }
}

/// Detect microcode package and version
fn detect_microcode() -> (Option<String>, Option<String>) {
    let mut package = None;
    let mut version = None;

    // Check which microcode package is installed
    if let Ok(output) = Command::new("pacman").arg("-Q").arg("intel-ucode").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = stdout.split_whitespace().collect();
            if parts.len() >= 2 {
                package = Some("intel-ucode".to_string());
                version = Some(parts[1].to_string());
            }
        }
    }

    if package.is_none() {
        if let Ok(output) = Command::new("pacman").arg("-Q").arg("amd-ucode").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = stdout.split_whitespace().collect();
                if parts.len() >= 2 {
                    package = Some("amd-ucode".to_string());
                    version = Some(parts[1].to_string());
                }
            }
        }
    }

    // Also try to get the loaded microcode version from kernel
    if version.is_none() {
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if line.starts_with("microcode") {
                    if let Some(value) = line.split(':').nth(1) {
                        version = Some(value.trim().to_string());
                        break;
                    }
                }
            }
        }
    }

    (package, version)
}

/// Detect important CPU flags
fn detect_cpu_flags() -> CpuFlags {
    let mut flags_set = std::collections::HashSet::new();

    // Read flags from /proc/cpuinfo
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("flags") || line.starts_with("Features") {
                if let Some(flags_str) = line.split(':').nth(1) {
                    for flag in flags_str.split_whitespace() {
                        flags_set.insert(flag.to_lowercase());
                    }
                }
                break; // Only need to check first CPU
            }
        }
    }

    CpuFlags {
        sse: flags_set.contains("sse"),
        sse2: flags_set.contains("sse2"),
        sse3: flags_set.contains("pni") || flags_set.contains("sse3"),
        ssse3: flags_set.contains("ssse3"),
        sse4_1: flags_set.contains("sse4_1"),
        sse4_2: flags_set.contains("sse4_2"),
        avx: flags_set.contains("avx"),
        avx2: flags_set.contains("avx2"),
        avx512f: flags_set.contains("avx512f"),
        aes: flags_set.contains("aes") || flags_set.contains("aes-ni"),
        virtualization: flags_set.contains("vmx") || flags_set.contains("svm"),
    }
}

impl std::fmt::Display for GovernorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.uniform {
            write!(f, "{}", self.governor)
        } else {
            write!(f, "{} (mixed)", self.governor)
        }
    }
}
