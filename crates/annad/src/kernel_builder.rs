//! Kernel compilation assistance — hardware detection + wiki research + build plan.
//!
//! Detects hardware (CPU, GPU, drivers), reads Arch Wiki on kernel compilation,
//! LLM generates a hardware-specific build plan. Anna guides each step.
//!
//! Always confirms before install steps (make install, mkinitcpio, bootloader update).

use anyhow::{anyhow, Result};
use tracing::info;

/// Hardware profile for kernel configuration.
#[derive(Debug)]
pub struct HardwareProfile {
    pub cpu_vendor: String,     // intel, amd
    pub cpu_model: String,
    pub gpu_vendors: Vec<String>, // nvidia, amd, intel
    pub arch: String,           // x86_64, aarch64
    pub current_kernel: String,
}

impl HardwareProfile {
    pub fn detect() -> Self {
        let cpu_info = run_cmd("lscpu")
            .unwrap_or_default();
        let cpu_model = cpu_info.lines()
            .find(|l| l.starts_with("Model name"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".into());

        let cpu_vendor = if cpu_model.to_lowercase().contains("intel") {
            "intel"
        } else if cpu_model.to_lowercase().contains("amd") {
            "amd"
        } else {
            "unknown"
        }.to_string();

        let gpu_info = run_cmd("lspci")
            .unwrap_or_default();
        let mut gpu_vendors = Vec::new();
        if gpu_info.to_lowercase().contains("nvidia") { gpu_vendors.push("nvidia".into()); }
        if gpu_info.to_lowercase().contains("amd") || gpu_info.to_lowercase().contains("radeon") {
            gpu_vendors.push("amd".into());
        }
        if gpu_info.to_lowercase().contains("intel") && gpu_info.to_lowercase().contains("graphics") {
            gpu_vendors.push("intel".into());
        }

        let arch = run_cmd("uname -m").unwrap_or_else(|| "x86_64".into()).trim().to_string();
        let current_kernel = run_cmd("uname -r").unwrap_or_else(|| "unknown".into()).trim().to_string();

        Self { cpu_vendor, cpu_model, gpu_vendors, arch, current_kernel }
    }

    pub fn summary(&self) -> String {
        format!(
            "CPU: {} ({})\nGPU: {}\nArch: {}\nCurrent kernel: {}",
            self.cpu_model,
            self.cpu_vendor,
            if self.gpu_vendors.is_empty() { "unknown".into() } else { self.gpu_vendors.join(", ") },
            self.arch,
            self.current_kernel,
        )
    }
}

fn run_cmd(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() { return None; }
    std::process::Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

/// Check which kernel build tools are available.
fn check_build_deps() -> Vec<String> {
    let deps = ["base-devel", "bc", "cpio", "pahole", "python", "tar", "xz"];
    let mut missing = Vec::new();
    for dep in deps {
        let available = std::process::Command::new("pacman")
            .args(["-Q", dep])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !available {
            missing.push(dep.to_string());
        }
    }
    missing
}

/// Generate a hardware-specific kernel build plan via wiki + LLM.
pub async fn generate_kernel_build_plan(model: &str, kernel_preference: Option<&str>) -> Result<String> {
    let hw = HardwareProfile::detect();
    info!("Detected hardware: {}", hw.summary());

    // Research: Arch Wiki on kernel compilation
    let wiki_kernel = anna_shared::wiki::search::keyword_search_text("Kernel/Arch build system", 1200)
        .or_else(|| anna_shared::wiki::search::keyword_search_text("Kernel/Traditional compilation", 1200))
        .unwrap_or_default();

    // Check build deps
    let missing_deps = check_build_deps();

    let kernel_pref = kernel_preference.unwrap_or("linux");
    let has_nvidia = hw.gpu_vendors.contains(&"nvidia".to_string());

    let prompt = format!(
        "You are helping compile a Linux kernel on Arch Linux for this specific hardware.\n\
        \n\
        Hardware:\n{hw_summary}\n\
        Current kernel: {kernel}\n\
        Kernel to build: {kernel_pref}\n\
        NVIDIA GPU present: {nvidia}\n\
        Missing build dependencies: {missing}\n\
        \n\
        Arch Wiki on kernel compilation:\n{wiki_kernel}\n\
        \n\
        Generate a step-by-step kernel compilation plan for this specific hardware.\n\
        Include:\n\
        1. Install missing dependencies (if any)\n\
        2. Get kernel source (ABS or tarball method for Arch)\n\
        3. make localmodconfig (for hardware-specific config)\n\
        4. Hardware-specific kernel options for this CPU/GPU\n\
        5. Compilation command with appropriate -j flag for this CPU\n\
        6. [CONFIRM REQUIRED] Installation steps (make install, mkinitcpio)\n\
        7. Bootloader update step\n\
        \n\
        Mark steps requiring root with [ROOT].\n\
        Mark dangerous/irreversible steps with [CONFIRM REQUIRED].\n\
        Be specific to this hardware. Include exact commands.\n\
        \n\
        Format each step as:\n\
        STEP N: <description>\n\
        CMD: <command or 'interactive'>\n\
        RISK: LOW|MEDIUM|HIGH",
        hw_summary = hw.summary(),
        kernel = hw.current_kernel,
        kernel_pref = kernel_pref,
        nvidia = has_nvidia,
        missing = if missing_deps.is_empty() { "none".into() } else { missing_deps.join(", ") },
        wiki_kernel = wiki_kernel,
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 60).await
        .map_err(|e| anyhow!("LLM error generating kernel build plan: {}", e))?;

    // Record in registry
    let mut registry = crate::artifact_registry::ArtifactRegistry::load();
    let artifact = crate::artifact_registry::CreatedArtifact::new(
        crate::artifact_registry::ArtifactKind::KernelConfig,
        format!("kernel build plan ({})", kernel_pref),
        &format!("Compilation plan for {} on {}", kernel_pref, hw.cpu_model),
        vec![],
        vec![], // no removal for build plans
    );
    registry.add(artifact);

    let mut out = format!(
        "Kernel Build Plan for your hardware:\n{}\n\n{}\n\n",
        hw.summary(),
        "=".repeat(50)
    );
    out.push_str(&response);

    if !missing_deps.is_empty() {
        out.push_str(&format!(
            "\n\nRequired packages: {}\nInstall with: sudo pacman -S {}",
            missing_deps.join(", "),
            missing_deps.join(" ")
        ));
    }

    out.push_str("\n\nSteps marked [CONFIRM REQUIRED] will ask for your approval before execution.");

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_profile_detect_no_panic() {
        let hw = HardwareProfile::detect();
        let summary = hw.summary();
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_check_build_deps_no_panic() {
        let missing = check_build_deps();
        // Just ensure it runs without panic
        let _ = missing;
    }
}
