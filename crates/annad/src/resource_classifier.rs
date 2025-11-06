//! Resource-Aware Recommendation Classifier
//!
//! Analyzes system resources and adds warnings to recommendations
//! that may not be suitable for the current hardware.
//!
//! Principle: Still show options, but warn users about requirements!

use anna_common::types::{Advice, SystemFacts};

/// System resource profile
#[derive(Debug, Clone)]
pub struct ResourceProfile {
    pub ram_gb: f32,
    pub cpu_cores: usize,
    pub has_gpu: bool,
    pub gpu_vendor: Option<String>,
    pub is_vm: bool,
    pub disk_type: Option<String>, // "ssd", "hdd", "nvme"
}

impl ResourceProfile {
    /// Detect system resources from facts
    pub fn detect(facts: &SystemFacts) -> Self {
        let ram_gb = facts.total_memory_gb as f32;
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        
        let has_gpu = facts.gpu_vendor.is_some();
        let gpu_vendor = facts.gpu_vendor.as_ref().and_then(|g| {
            if g.to_lowercase().contains("nvidia") {
                Some("nvidia".to_string())
            } else if g.to_lowercase().contains("amd") || g.to_lowercase().contains("radeon") {
                Some("amd".to_string())
            } else if g.to_lowercase().contains("intel") {
                Some("intel".to_string())
            } else {
                None
            }
        });
        
        // Detect if running in VM
        let is_vm = std::process::Command::new("systemd-detect-virt")
            .output()
            .map(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                !output.trim().is_empty() && output.trim() != "none"
            })
            .unwrap_or(false);
        
        // Detect disk type (simple heuristic)
        let disk_type = if std::path::Path::new("/sys/block/nvme0n1").exists() {
            Some("nvme".to_string())
        } else if std::path::Path::new("/sys/block/sda/queue/rotational").exists() {
            std::fs::read_to_string("/sys/block/sda/queue/rotational")
                .ok()
                .and_then(|s| {
                    if s.trim() == "0" {
                        Some("ssd".to_string())
                    } else {
                        Some("hdd".to_string())
                    }
                })
        } else {
            None
        };
        
        Self {
            ram_gb,
            cpu_cores,
            has_gpu,
            gpu_vendor,
            is_vm,
            disk_type,
        }
    }
    
    /// Classify system into resource tier
    pub fn tier(&self) -> ResourceTier {
        if self.ram_gb < 2.0 || self.cpu_cores <= 2 {
            ResourceTier::VeryLow
        } else if self.ram_gb < 4.0 || self.cpu_cores <= 4 {
            ResourceTier::Low
        } else if self.ram_gb < 8.0 {
            ResourceTier::Medium
        } else if self.ram_gb < 16.0 {
            ResourceTier::High
        } else {
            ResourceTier::VeryHigh
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceTier {
    VeryLow,  // <2GB RAM or <=2 cores
    Low,      // 2-4GB RAM or 2-4 cores
    Medium,   // 4-8GB RAM
    High,     // 8-16GB RAM
    VeryHigh, // 16GB+ RAM
}

/// Resource requirements for software/bundles
#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub min_ram_gb: f32,
    pub recommended_ram_gb: f32,
    pub min_cpu_cores: usize,
    pub requires_gpu: bool,
    pub requires_disk_type: Option<String>,
}

impl ResourceRequirements {
    /// Check if current system meets requirements
    pub fn check(&self, profile: &ResourceProfile) -> ResourceCompatibility {
        let mut warnings = Vec::new();
        
        if profile.ram_gb < self.min_ram_gb {
            warnings.push(format!(
                "⚠️ Requires minimum {}GB RAM, your system has {:.1}GB",
                self.min_ram_gb, profile.ram_gb
            ));
        } else if profile.ram_gb < self.recommended_ram_gb {
            warnings.push(format!(
                "💡 Recommended {}GB RAM for best experience, your system has {:.1}GB",
                self.recommended_ram_gb, profile.ram_gb
            ));
        }
        
        if profile.cpu_cores < self.min_cpu_cores {
            warnings.push(format!(
                "⚠️ Requires minimum {} CPU cores, your system has {}",
                self.min_cpu_cores, profile.cpu_cores
            ));
        }
        
        if self.requires_gpu && !profile.has_gpu {
            warnings.push(
                "⚠️ Requires dedicated GPU, none detected".to_string()
            );
        }
        
        if let Some(ref required_disk) = self.requires_disk_type {
            if let Some(ref current_disk) = profile.disk_type {
                if required_disk != current_disk {
                    warnings.push(format!(
                        "💡 Works best with {}, you have {}",
                        required_disk, current_disk
                    ));
                }
            }
        }
        
        ResourceCompatibility {
            compatible: warnings.iter().any(|w| w.starts_with("⚠️")),
            warnings,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceCompatibility {
    pub compatible: bool,  // false if has ⚠️ warnings
    pub warnings: Vec<String>,
}

/// Known software/bundle resource requirements
pub fn get_requirements(identifier: &str) -> Option<ResourceRequirements> {
    match identifier {
        // Wayland Compositors
        "hyprland" | "hyprland-setup" => Some(ResourceRequirements {
            min_ram_gb: 2.0,
            recommended_ram_gb: 4.0,
            min_cpu_cores: 4,
            requires_gpu: false,
            requires_disk_type: None,
        }),
        
        // Heavy DEs
        "gnome" | "kde" | "plasma" => Some(ResourceRequirements {
            min_ram_gb: 2.0,
            recommended_ram_gb: 4.0,
            min_cpu_cores: 2,
            requires_gpu: false,
            requires_disk_type: None,
        }),
        
        // Gaming
        "steam" | "wine" | "proton" => Some(ResourceRequirements {
            min_ram_gb: 4.0,
            recommended_ram_gb: 8.0,
            min_cpu_cores: 4,
            requires_gpu: true,
            requires_disk_type: Some("ssd".to_string()),
        }),
        
        // Development
        "docker" => Some(ResourceRequirements {
            min_ram_gb: 4.0,
            recommended_ram_gb: 8.0,
            min_cpu_cores: 4,
            requires_gpu: false,
            requires_disk_type: Some("ssd".to_string()),
        }),
        
        "vscode" | "code" => Some(ResourceRequirements {
            min_ram_gb: 2.0,
            recommended_ram_gb: 4.0,
            min_cpu_cores: 2,
            requires_gpu: false,
            requires_disk_type: None,
        }),
        
        // Browsers
        "chromium" | "chrome" | "firefox" => Some(ResourceRequirements {
            min_ram_gb: 2.0,
            recommended_ram_gb: 4.0,
            min_cpu_cores: 2,
            requires_gpu: false,
            requires_disk_type: None,
        }),
        
        _ => None,
    }
}

/// Add resource warnings to advice items
pub fn annotate_advice(advice: &mut Vec<Advice>, facts: &SystemFacts) {
    let profile = ResourceProfile::detect(facts);
    
    for item in advice.iter_mut() {
        // Check bundle name or action for known requirements
        let identifiers: Vec<&str> = vec![
            item.bundle.as_deref().unwrap_or(""),
            &item.id,
            &item.action,
        ];
        
        for id in identifiers {
            if let Some(requirements) = get_requirements(id) {
                let compat = requirements.check(&profile);
                
                if !compat.warnings.is_empty() {
                    // Prepend warnings to the reason
                    let warning_text = compat.warnings.join("\n");
                    item.reason = format!(
                        "{}\n\n{}",
                        warning_text,
                        item.reason
                    );
                }
                
                break;  // Only check first match
            }
        }
    }
}
