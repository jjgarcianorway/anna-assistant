//! GPU Setup - Automatic detection and configuration for GPU acceleration.
//!
//! This module ensures Ollama can use the GPU:
//! 1. Detects NVIDIA GPU via lspci
//! 2. Checks if ollama-cuda is installed (vs CPU-only ollama)
//! 3. Installs cuda and ollama-cuda if needed
//! 4. Adds ollama user to video/render groups
//! 5. Restarts Ollama to pick up GPU access

use std::process::Command;
use tracing::{info, warn, error};

/// GPU detection result
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub detected: bool,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub cuda_available: bool,
    pub ollama_cuda_installed: bool,
    pub ollama_has_gpu_access: bool,
}

/// Check if NVIDIA GPU is present
pub fn detect_nvidia_gpu() -> Option<String> {
    let output = Command::new("lspci")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let lower = line.to_lowercase();
        if (lower.contains("vga") || lower.contains("3d") || lower.contains("display"))
            && lower.contains("nvidia")
        {
            // Extract model name
            if let Some(idx) = line.find("NVIDIA") {
                return Some(line[idx..].to_string());
            }
            return Some(line.to_string());
        }
    }
    None
}

/// Check if ollama-cuda package is installed
pub fn is_ollama_cuda_installed() -> bool {
    let output = Command::new("pacman")
        .args(["-Q", "ollama-cuda"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

/// Check if cuda package is installed
pub fn is_cuda_installed() -> bool {
    let output = Command::new("pacman")
        .args(["-Q", "cuda"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

/// Check if ollama user is in video and render groups
pub fn ollama_has_gpu_groups() -> bool {
    let output = Command::new("groups")
        .arg("ollama")
        .output();

    match output {
        Ok(o) => {
            let groups = String::from_utf8_lossy(&o.stdout);
            groups.contains("video") && groups.contains("render")
        }
        Err(_) => false,
    }
}

/// Check if CPU-only ollama package is installed (conflicts with ollama-cuda)
pub fn is_ollama_cpu_installed() -> bool {
    let output = Command::new("pacman")
        .args(["-Q", "ollama"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

/// Check if ollama binary was installed manually (not via pacman)
pub fn is_ollama_manually_installed() -> bool {
    // Check if /usr/bin/ollama exists but isn't owned by any package
    let exists = std::path::Path::new("/usr/bin/ollama").exists();
    if !exists {
        return false;
    }

    let output = Command::new("pacman")
        .args(["-Qo", "/usr/bin/ollama"])
        .output();

    match output {
        Ok(o) => !o.status.success(), // If pacman can't find owner, it's manual
        Err(_) => false,
    }
}

/// Clean up ALL manually installed Ollama files (from curl installer)
/// v0.0.820: These files conflict with pacman packages
fn cleanup_manual_ollama_files() {
    use std::path::Path;

    let paths = [
        "/usr/bin/ollama",
        "/usr/lib/ollama",
        "/usr/lib/systemd/system/ollama.service",
        "/usr/lib/sysusers.d/ollama.conf",
        "/usr/lib/tmpfiles.d/ollama.conf",
        "/usr/share/licenses/ollama",
        "/usr/share/ollama",
    ];

    // Stop and kill ollama first
    let _ = Command::new("systemctl").args(["stop", "ollama"]).output();
    let _ = Command::new("pkill").args(["-9", "ollama"]).output();
    std::thread::sleep(std::time::Duration::from_secs(1));

    for path in &paths {
        let p = Path::new(path);
        if p.exists() {
            if p.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(p) {
                    warn!("Failed to remove dir {}: {}", path, e);
                } else {
                    info!("Removed {}", path);
                }
            } else if let Err(e) = std::fs::remove_file(p) {
                warn!("Failed to remove file {}: {}", path, e);
            } else {
                info!("Removed {}", path);
            }
        }
    }
}

/// Install CUDA and ollama-cuda packages
/// v0.0.820: Handle conflict with CPU-only ollama package and manual installs
pub fn install_cuda_packages() -> Result<(), String> {
    info!("Installing CUDA packages for GPU acceleration...");

    // v0.0.820: Clean up ALL manual install files first (not just binary)
    if is_ollama_manually_installed() {
        info!("Detected manually installed ollama, cleaning up...");
        cleanup_manual_ollama_files();
    }

    // v0.0.820: Remove CPU-only ollama package if installed (conflicts with ollama-cuda)
    if is_ollama_cpu_installed() && !is_ollama_cuda_installed() {
        info!("Removing CPU-only ollama package to install ollama-cuda...");
        let remove_output = Command::new("pacman")
            .args(["-R", "--noconfirm", "ollama"])
            .output()
            .map_err(|e| format!("Failed to run pacman -R: {}", e))?;

        if !remove_output.status.success() {
            let stderr = String::from_utf8_lossy(&remove_output.stderr);
            warn!("Failed to remove ollama package (may not be an issue): {}", stderr);
        }
    }

    // Install cuda and ollama-cuda
    // v0.0.824: Use --overwrite to handle gcc/gcc-libs file conflicts
    let output = Command::new("pacman")
        .args(["-S", "--noconfirm", "--overwrite", "*", "cuda", "ollama-cuda"])
        .output()
        .map_err(|e| format!("Failed to run pacman: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to install CUDA packages: {}", stderr));
    }

    info!("CUDA packages installed successfully");
    Ok(())
}

/// Add ollama user to video and render groups
pub fn add_ollama_to_gpu_groups() -> Result<(), String> {
    info!("Adding ollama user to GPU groups...");

    let output = Command::new("usermod")
        .args(["-aG", "video,render", "ollama"])
        .output()
        .map_err(|e| format!("Failed to run usermod: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to add ollama to groups: {}", stderr));
    }

    info!("ollama user added to video and render groups");
    Ok(())
}

/// Restart Ollama service to pick up new GPU access
pub fn restart_ollama_service() -> Result<(), String> {
    info!("Restarting Ollama service...");

    let output = Command::new("systemctl")
        .args(["restart", "ollama"])
        .output()
        .map_err(|e| format!("Failed to run systemctl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to restart ollama: {}", stderr));
    }

    info!("Ollama service restarted");
    Ok(())
}

/// Full GPU setup: detect, install, configure
/// Returns true if GPU is now available, false otherwise
pub fn ensure_gpu_acceleration() -> Result<bool, String> {
    // Step 1: Detect NVIDIA GPU
    let gpu = match detect_nvidia_gpu() {
        Some(model) => {
            info!("Detected NVIDIA GPU: {}", model);
            model
        }
        None => {
            info!("No NVIDIA GPU detected, skipping GPU setup");
            return Ok(false);
        }
    };

    let mut needs_restart = false;

    // Step 2: Check if CUDA is installed
    if !is_cuda_installed() {
        info!("CUDA not installed, installing...");
        install_cuda_packages()?;
        needs_restart = true;
    } else {
        info!("CUDA already installed");
    }

    // Step 3: Check if ollama-cuda is installed
    if !is_ollama_cuda_installed() {
        info!("ollama-cuda not installed, installing...");
        install_cuda_packages()?;
        needs_restart = true;
    } else {
        info!("ollama-cuda already installed");
    }

    // Step 4: Check if ollama user has GPU access
    if !ollama_has_gpu_groups() {
        info!("ollama user missing GPU groups, adding...");
        add_ollama_to_gpu_groups()?;
        needs_restart = true;
    } else {
        info!("ollama user already has GPU access");
    }

    // Step 5: Restart Ollama if any changes were made
    if needs_restart {
        restart_ollama_service()?;
        // Give Ollama time to start
        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    info!("GPU acceleration configured for: {}", gpu);
    Ok(true)
}

/// Get current GPU status without making changes
pub fn get_gpu_status() -> GpuInfo {
    let nvidia = detect_nvidia_gpu();

    GpuInfo {
        detected: nvidia.is_some(),
        vendor: nvidia.as_ref().map(|_| "NVIDIA".to_string()),
        model: nvidia,
        cuda_available: is_cuda_installed(),
        ollama_cuda_installed: is_ollama_cuda_installed(),
        ollama_has_gpu_access: ollama_has_gpu_groups(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_status() {
        let status = get_gpu_status();
        // Just verify the function runs without crashing
        println!("GPU detected: {}", status.detected);
        println!("CUDA available: {}", status.cuda_available);
        println!("ollama-cuda installed: {}", status.ollama_cuda_installed);
        println!("ollama has GPU access: {}", status.ollama_has_gpu_access);
    }
}
