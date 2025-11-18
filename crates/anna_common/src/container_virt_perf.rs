use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerVirtPerformance {
    pub broken_containers: Vec<BrokenContainer>,
    pub high_cpu_containers: Vec<HighCpuContainer>,
    pub missing_limits: Vec<MissingResourceLimit>,
    pub nested_virtualization: Option<NestedVirtualization>,
    pub qemu_performance: Option<QemuPerformance>,
    pub performance_score: u8,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenContainer {
    pub runtime: ContainerRuntime,
    pub container_id: String,
    pub name: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub created: String,
    pub image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighCpuContainer {
    pub runtime: ContainerRuntime,
    pub container_id: String,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_usage: Option<String>,
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingResourceLimit {
    pub runtime: ContainerRuntime,
    pub container_id: String,
    pub name: String,
    pub missing_limits: Vec<ResourceLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedVirtualization {
    pub kvm_available: bool,
    pub kvm_intel_nested: Option<bool>,
    pub kvm_amd_nested: Option<bool>,
    pub nested_enabled: bool,
    pub hypervisor_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QemuPerformance {
    pub qemu_installed: bool,
    pub kvm_enabled: bool,
    pub cpu_flags: CpuVirtualizationFlags,
    pub libvirt_installed: bool,
    pub libvirt_running: bool,
    pub performance_features: Vec<String>,
    pub missing_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuVirtualizationFlags {
    pub vmx: bool,  // Intel VT-x
    pub svm: bool,  // AMD-V
    pub ept: bool,  // Intel EPT (Extended Page Tables)
    pub npt: bool,  // AMD NPT (Nested Page Tables)
    pub vpid: bool, // Virtual Processor ID
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerRuntime {
    Docker,
    Podman,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceLimit {
    Memory,
    Cpu,
    PidsLimit,
}

impl ContainerVirtPerformance {
    pub fn detect() -> Self {
        let broken_containers = detect_broken_containers();
        let high_cpu_containers = detect_high_cpu_containers();
        let missing_limits = detect_missing_limits();
        let nested_virtualization = detect_nested_virtualization();
        let qemu_performance = detect_qemu_performance();

        let performance_score = calculate_performance_score(
            &broken_containers,
            &high_cpu_containers,
            &missing_limits,
            &nested_virtualization,
            &qemu_performance,
        );

        let recommendations = generate_recommendations(
            &broken_containers,
            &high_cpu_containers,
            &missing_limits,
            &nested_virtualization,
            &qemu_performance,
        );

        ContainerVirtPerformance {
            broken_containers,
            high_cpu_containers,
            missing_limits,
            nested_virtualization,
            qemu_performance,
            performance_score,
            recommendations,
        }
    }

    pub fn has_issues(&self) -> bool {
        !self.broken_containers.is_empty()
            || !self.high_cpu_containers.is_empty()
            || !self.missing_limits.is_empty()
    }
}

fn detect_broken_containers() -> Vec<BrokenContainer> {
    let mut broken = Vec::new();

    // Check Docker
    if let Some(docker_broken) = check_docker_containers() {
        broken.extend(docker_broken);
    }

    // Check Podman
    if let Some(podman_broken) = check_podman_containers() {
        broken.extend(podman_broken);
    }

    broken
}

fn check_docker_containers() -> Option<Vec<BrokenContainer>> {
    if !is_command_available("docker") {
        return None;
    }

    let output = Command::new("docker")
        .args([
            "ps",
            "-a",
            "--format",
            "{{.ID}}|{{.Names}}|{{.Status}}|{{.Image}}|{{.CreatedAt}}",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let list = String::from_utf8(output.stdout).ok()?;
    let mut broken = Vec::new();

    for line in list.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 5 {
            let status = parts[2];

            // Check for exited containers
            if status.starts_with("Exited") {
                // Extract exit code
                let exit_code = status
                    .strip_prefix("Exited (")
                    .and_then(|s| s.split(')').next())
                    .and_then(|s| s.parse::<i32>().ok());

                // Non-zero exit codes indicate failures
                if exit_code.unwrap_or(0) != 0 {
                    broken.push(BrokenContainer {
                        runtime: ContainerRuntime::Docker,
                        container_id: parts[0].to_string(),
                        name: parts[1].to_string(),
                        status: status.to_string(),
                        exit_code,
                        created: parts[4].to_string(),
                        image: parts[3].to_string(),
                    });
                }
            }
        }
    }

    Some(broken)
}

fn check_podman_containers() -> Option<Vec<BrokenContainer>> {
    if !is_command_available("podman") {
        return None;
    }

    let output = Command::new("podman")
        .args([
            "ps",
            "-a",
            "--format",
            "{{.ID}}|{{.Names}}|{{.Status}}|{{.Image}}|{{.CreatedAt}}",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let list = String::from_utf8(output.stdout).ok()?;
    let mut broken = Vec::new();

    for line in list.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 5 {
            let status = parts[2];

            if status.starts_with("Exited") {
                let exit_code = status
                    .strip_prefix("Exited (")
                    .and_then(|s| s.split(')').next())
                    .and_then(|s| s.parse::<i32>().ok());

                if exit_code.unwrap_or(0) != 0 {
                    broken.push(BrokenContainer {
                        runtime: ContainerRuntime::Podman,
                        container_id: parts[0].to_string(),
                        name: parts[1].to_string(),
                        status: status.to_string(),
                        exit_code,
                        created: parts[4].to_string(),
                        image: parts[3].to_string(),
                    });
                }
            }
        }
    }

    Some(broken)
}

fn detect_high_cpu_containers() -> Vec<HighCpuContainer> {
    let mut high_cpu = Vec::new();

    // Check Docker stats
    if let Some(docker_stats) = check_docker_stats() {
        high_cpu.extend(docker_stats);
    }

    // Check Podman stats
    if let Some(podman_stats) = check_podman_stats() {
        high_cpu.extend(podman_stats);
    }

    high_cpu
}

fn check_docker_stats() -> Option<Vec<HighCpuContainer>> {
    if !is_command_available("docker") {
        return None;
    }

    // Get stats for running containers (no-stream for single snapshot)
    let output = Command::new("docker")
        .args([
            "stats",
            "--no-stream",
            "--format",
            "{{.ID}}|{{.Name}}|{{.CPUPerc}}|{{.MemUsage}}",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stats = String::from_utf8(output.stdout).ok()?;
    let mut high_cpu = Vec::new();

    for line in stats.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            // Parse CPU percentage (remove % sign)
            if let Some(cpu_str) = parts[2].strip_suffix('%') {
                if let Ok(cpu_percent) = cpu_str.parse::<f64>() {
                    // Consider >80% CPU as high
                    if cpu_percent > 80.0 {
                        high_cpu.push(HighCpuContainer {
                            runtime: ContainerRuntime::Docker,
                            container_id: parts[0].to_string(),
                            name: parts[1].to_string(),
                            cpu_percent,
                            memory_usage: Some(parts[3].to_string()),
                            duration: None,
                        });
                    }
                }
            }
        }
    }

    Some(high_cpu)
}

fn check_podman_stats() -> Option<Vec<HighCpuContainer>> {
    if !is_command_available("podman") {
        return None;
    }

    let output = Command::new("podman")
        .args([
            "stats",
            "--no-stream",
            "--format",
            "{{.ID}}|{{.Name}}|{{.CPU}}|{{.MemUsage}}",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stats = String::from_utf8(output.stdout).ok()?;
    let mut high_cpu = Vec::new();

    for line in stats.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            if let Some(cpu_str) = parts[2].strip_suffix('%') {
                if let Ok(cpu_percent) = cpu_str.parse::<f64>() {
                    if cpu_percent > 80.0 {
                        high_cpu.push(HighCpuContainer {
                            runtime: ContainerRuntime::Podman,
                            container_id: parts[0].to_string(),
                            name: parts[1].to_string(),
                            cpu_percent,
                            memory_usage: Some(parts[3].to_string()),
                            duration: None,
                        });
                    }
                }
            }
        }
    }

    Some(high_cpu)
}

fn detect_missing_limits() -> Vec<MissingResourceLimit> {
    let mut missing = Vec::new();

    // Check Docker containers
    if let Some(docker_limits) = check_docker_limits() {
        missing.extend(docker_limits);
    }

    // Check Podman containers
    if let Some(podman_limits) = check_podman_limits() {
        missing.extend(podman_limits);
    }

    missing
}

fn check_docker_limits() -> Option<Vec<MissingResourceLimit>> {
    if !is_command_available("docker") {
        return None;
    }

    // Get list of running containers
    let output = Command::new("docker")
        .args(["ps", "--format", "{{.ID}}|{{.Names}}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let containers = String::from_utf8(output.stdout).ok()?;
    let mut missing_limits = Vec::new();

    for line in containers.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 2 {
            let container_id = parts[0];
            let name = parts[1];

            // Inspect container for resource limits
            if let Some(limits) = inspect_docker_container_limits(container_id) {
                if !limits.is_empty() {
                    missing_limits.push(MissingResourceLimit {
                        runtime: ContainerRuntime::Docker,
                        container_id: container_id.to_string(),
                        name: name.to_string(),
                        missing_limits: limits,
                    });
                }
            }
        }
    }

    Some(missing_limits)
}

fn inspect_docker_container_limits(container_id: &str) -> Option<Vec<ResourceLimit>> {
    let output = Command::new("docker")
        .args([
            "inspect",
            container_id,
            "--format",
            "{{.HostConfig.Memory}}|{{.HostConfig.NanoCpus}}|{{.HostConfig.PidsLimit}}",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let limits_str = String::from_utf8(output.stdout).ok()?;
    let parts: Vec<&str> = limits_str.trim().split('|').collect();

    let mut missing = Vec::new();

    if parts.len() >= 3 {
        // Memory limit (0 means unlimited)
        if parts[0] == "0" {
            missing.push(ResourceLimit::Memory);
        }
        // CPU limit (0 means unlimited)
        if parts[1] == "0" {
            missing.push(ResourceLimit::Cpu);
        }
        // Pids limit (0 or -1 means unlimited)
        if parts[2] == "0" || parts[2] == "-1" {
            missing.push(ResourceLimit::PidsLimit);
        }
    }

    Some(missing)
}

fn check_podman_limits() -> Option<Vec<MissingResourceLimit>> {
    // Similar to Docker but using podman commands
    // For brevity, simplified implementation
    None
}

fn detect_nested_virtualization() -> Option<NestedVirtualization> {
    let kvm_available = std::path::Path::new("/dev/kvm").exists();

    if !kvm_available {
        return None;
    }

    let kvm_intel_path = "/sys/module/kvm_intel/parameters/nested";
    let kvm_amd_path = "/sys/module/kvm_amd/parameters/nested";

    let kvm_intel_nested = std::fs::read_to_string(kvm_intel_path)
        .ok()
        .map(|s| s.trim() == "Y" || s.trim() == "1");

    let kvm_amd_nested = std::fs::read_to_string(kvm_amd_path)
        .ok()
        .map(|s| s.trim() == "Y" || s.trim() == "1");

    let nested_enabled = kvm_intel_nested.unwrap_or(false) || kvm_amd_nested.unwrap_or(false);

    // Check if we're running in a hypervisor
    let hypervisor_detected = detect_hypervisor();

    Some(NestedVirtualization {
        kvm_available,
        kvm_intel_nested,
        kvm_amd_nested,
        nested_enabled,
        hypervisor_detected,
    })
}

fn detect_hypervisor() -> bool {
    // Check for hypervisor CPU flag
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        if cpuinfo.contains("hypervisor") {
            return true;
        }
    }

    // Check systemd-detect-virt
    if let Ok(output) = Command::new("systemd-detect-virt").output() {
        if output.status.success() {
            let virt_type = String::from_utf8_lossy(&output.stdout);
            return virt_type.trim() != "none";
        }
    }

    false
}

fn detect_qemu_performance() -> Option<QemuPerformance> {
    let qemu_installed = is_command_available("qemu-system-x86_64");

    if !qemu_installed {
        return None;
    }

    let kvm_enabled = std::path::Path::new("/dev/kvm").exists();
    let cpu_flags = detect_cpu_virt_flags();

    let libvirt_installed = is_command_available("virsh");
    let libvirt_running = if libvirt_installed {
        check_libvirt_running()
    } else {
        false
    };

    let mut performance_features = Vec::new();
    let mut missing_features = Vec::new();

    if kvm_enabled {
        performance_features.push("KVM acceleration".to_string());
    } else {
        missing_features.push("KVM acceleration (install kvm modules)".to_string());
    }

    if cpu_flags.vmx || cpu_flags.svm {
        performance_features.push("Hardware virtualization".to_string());
    } else {
        missing_features.push("Hardware virtualization (enable in BIOS)".to_string());
    }

    if cpu_flags.ept || cpu_flags.npt {
        performance_features.push("Nested page tables".to_string());
    }

    if cpu_flags.vpid {
        performance_features.push("VPID support".to_string());
    }

    Some(QemuPerformance {
        qemu_installed,
        kvm_enabled,
        cpu_flags,
        libvirt_installed,
        libvirt_running,
        performance_features,
        missing_features,
    })
}

fn detect_cpu_virt_flags() -> CpuVirtualizationFlags {
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();

    CpuVirtualizationFlags {
        vmx: cpuinfo.contains(" vmx"),
        svm: cpuinfo.contains(" svm"),
        ept: cpuinfo.contains(" ept"),
        npt: cpuinfo.contains(" npt"),
        vpid: cpuinfo.contains(" vpid"),
    }
}

fn check_libvirt_running() -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", "libvirtd"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn calculate_performance_score(
    broken: &[BrokenContainer],
    high_cpu: &[HighCpuContainer],
    missing_limits: &[MissingResourceLimit],
    nested: &Option<NestedVirtualization>,
    qemu: &Option<QemuPerformance>,
) -> u8 {
    let mut score = 100u8;

    // Deduct for broken containers
    score = score.saturating_sub((broken.len() as u8) * 10);

    // Deduct for high CPU containers
    score = score.saturating_sub((high_cpu.len() as u8) * 5);

    // Deduct for missing limits
    score = score.saturating_sub((missing_limits.len() as u8) * 3);

    // Bonus for nested virtualization
    if let Some(n) = nested {
        if !n.nested_enabled && n.kvm_available {
            score = score.saturating_sub(10);
        }
    }

    // Bonus for QEMU optimization
    if let Some(q) = qemu {
        if !q.kvm_enabled && q.qemu_installed {
            score = score.saturating_sub(20);
        }
    }

    score
}

fn generate_recommendations(
    broken: &[BrokenContainer],
    high_cpu: &[HighCpuContainer],
    missing_limits: &[MissingResourceLimit],
    nested: &Option<NestedVirtualization>,
    qemu: &Option<QemuPerformance>,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !broken.is_empty() {
        recommendations.push(format!(
            "Remove or restart {} broken container(s): {}",
            broken.len(),
            broken
                .iter()
                .take(3)
                .map(|c| &c.name)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if !high_cpu.is_empty() {
        recommendations.push(format!(
            "Investigate {} high CPU container(s) - consider resource limits",
            high_cpu.len()
        ));
    }

    if !missing_limits.is_empty() {
        recommendations.push(format!(
            "{} container(s) lack resource limits - add memory/CPU limits for stability",
            missing_limits.len()
        ));
    }

    if let Some(n) = nested {
        if n.kvm_available && !n.nested_enabled {
            recommendations
                .push("Enable nested virtualization for better VM-in-VM performance".to_string());
        }
    }

    if let Some(q) = qemu {
        if !q.missing_features.is_empty() {
            recommendations.push(format!(
                "QEMU performance: {}",
                q.missing_features.join(", ")
            ));
        }
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_detection() {
        let perf = ContainerVirtPerformance::detect();
        assert!(perf.performance_score <= 100);
    }
}
