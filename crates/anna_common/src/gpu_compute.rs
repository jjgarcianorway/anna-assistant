use serde::{Deserialize, Serialize};
use std::process::Command;

/// GPU compute capabilities detection (CUDA, OpenCL, ROCm, oneAPI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuComputeCapabilities {
    pub cuda_support: Option<CudaSupport>,
    pub opencl_support: Option<OpenClSupport>,
    pub rocm_support: Option<RocmSupport>,
    pub oneapi_support: Option<OneApiSupport>,
    pub has_compute_capability: bool,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CudaSupport {
    pub cuda_available: bool,
    pub cuda_version: Option<String>,
    pub driver_version: Option<String>,
    pub compute_capability: Vec<String>, // Per-GPU compute capability (e.g., "8.6")
    pub gpu_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClSupport {
    pub opencl_available: bool,
    pub opencl_version: Option<String>,
    pub platforms: Vec<OpenClPlatform>,
    pub device_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClPlatform {
    pub platform_name: String,
    pub platform_version: String,
    pub devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocmSupport {
    pub rocm_available: bool,
    pub rocm_version: Option<String>,
    pub hip_version: Option<String>,
    pub gpu_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneApiSupport {
    pub oneapi_available: bool,
    pub oneapi_version: Option<String>,
    pub level_zero_available: bool,
}

impl GpuComputeCapabilities {
    /// Detect GPU compute capabilities across all frameworks
    pub fn detect() -> Self {
        let cuda_support = detect_cuda_support();
        let opencl_support = detect_opencl_support();
        let rocm_support = detect_rocm_support();
        let oneapi_support = detect_oneapi_support();

        let has_compute_capability = cuda_support
            .as_ref()
            .map(|c| c.cuda_available)
            .unwrap_or(false)
            || opencl_support
                .as_ref()
                .map(|o| o.opencl_available)
                .unwrap_or(false)
            || rocm_support
                .as_ref()
                .map(|r| r.rocm_available)
                .unwrap_or(false)
            || oneapi_support
                .as_ref()
                .map(|o| o.oneapi_available)
                .unwrap_or(false);

        let mut recommendations = Vec::new();

        if let Some(ref cuda) = cuda_support {
            if cuda.cuda_available {
                recommendations.push(format!(
                    "CUDA {} available with {} GPU(s) - ML/AI workloads ready",
                    cuda.cuda_version.as_deref().unwrap_or("unknown"),
                    cuda.gpu_count
                ));
            }
        }

        if let Some(ref opencl) = opencl_support {
            if opencl.opencl_available {
                recommendations.push(format!(
                    "OpenCL available with {} device(s) - cross-platform compute ready",
                    opencl.device_count
                ));
            }
        }

        if let Some(ref rocm) = rocm_support {
            if rocm.rocm_available {
                recommendations.push(format!(
                    "ROCm {} available - AMD GPU compute ready",
                    rocm.rocm_version.as_deref().unwrap_or("unknown")
                ));
            }
        }

        if let Some(ref oneapi) = oneapi_support {
            if oneapi.oneapi_available {
                recommendations.push("Intel oneAPI available - unified compute ready".to_string());
            }
        }

        if !has_compute_capability {
            recommendations.push("No GPU compute frameworks detected - install CUDA/OpenCL/ROCm for GPU acceleration".to_string());
        }

        Self {
            cuda_support,
            opencl_support,
            rocm_support,
            oneapi_support,
            has_compute_capability,
            recommendations,
        }
    }
}

fn detect_cuda_support() -> Option<CudaSupport> {
    // Check for nvidia-smi
    let check = Command::new("which").arg("nvidia-smi").output().ok()?;
    if !check.status.success() {
        return None;
    }

    // Query CUDA version via nvidia-smi
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=name,compute_cap", "--format=csv,noheader"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let mut compute_capability = Vec::new();
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            compute_capability.push(parts[1].to_string());
        }
    }

    // Get CUDA version
    let version_output = Command::new("nvidia-smi")
        .args(["--query-gpu=driver_version", "--format=csv,noheader"])
        .output()
        .ok()?;

    let driver_version = if version_output.status.success() {
        String::from_utf8_lossy(&version_output.stdout)
            .lines()
            .next()
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    // Try to get CUDA runtime version via nvcc
    let cuda_version = Command::new("nvcc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let output_str = String::from_utf8_lossy(&o.stdout);
                // Parse "Cuda compilation tools, release X.Y"
                output_str
                    .lines()
                    .find(|line| line.contains("release"))
                    .and_then(|line| {
                        line.split("release")
                            .nth(1)
                            .and_then(|s| s.split(',').next())
                            .map(|s| s.trim().to_string())
                    })
            } else {
                None
            }
        });

    Some(CudaSupport {
        cuda_available: true,
        cuda_version,
        driver_version,
        gpu_count: compute_capability.len(),
        compute_capability,
    })
}

fn detect_opencl_support() -> Option<OpenClSupport> {
    // Check for clinfo utility
    let check = Command::new("which").arg("clinfo").output().ok()?;
    if !check.status.success() {
        return None;
    }

    let output = Command::new("clinfo").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut platforms = Vec::new();
    let mut current_platform: Option<String> = None;
    let mut current_version: Option<String> = None;
    let mut current_devices: Vec<String> = Vec::new();
    let mut device_count = 0;

    for line in stdout.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Platform Name") {
            // Save previous platform if exists
            if let (Some(name), Some(version)) = (current_platform.clone(), current_version.clone())
            {
                platforms.push(OpenClPlatform {
                    platform_name: name,
                    platform_version: version,
                    devices: current_devices.clone(),
                });
                current_devices.clear();
            }

            current_platform = trimmed.split_once(':').map(|(_, v)| v.trim().to_string());
        } else if trimmed.starts_with("Platform Version") {
            current_version = trimmed.split_once(':').map(|(_, v)| v.trim().to_string());
        } else if trimmed.starts_with("Device Name") {
            let device_name = trimmed.split_once(':').map(|(_, v)| v.trim().to_string());
            if let Some(name) = device_name {
                current_devices.push(name);
                device_count += 1;
            }
        }
    }

    // Save last platform
    if let (Some(name), Some(version)) = (current_platform, current_version) {
        platforms.push(OpenClPlatform {
            platform_name: name,
            platform_version: version,
            devices: current_devices,
        });
    }

    if platforms.is_empty() {
        return None;
    }

    // Try to extract OpenCL version
    let opencl_version = stdout
        .lines()
        .find(|line| line.contains("OpenCL C"))
        .and_then(|line| {
            line.split("OpenCL C")
                .nth(1)
                .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
        });

    Some(OpenClSupport {
        opencl_available: true,
        opencl_version,
        platforms,
        device_count,
    })
}

fn detect_rocm_support() -> Option<RocmSupport> {
    // Check for rocm-smi
    let check = Command::new("which").arg("rocm-smi").output().ok()?;
    if !check.status.success() {
        return None;
    }

    // Get ROCm version
    let version_output = Command::new("rocm-smi")
        .arg("--showproductname")
        .output()
        .ok()?;

    if !version_output.status.success() {
        return None;
    }

    // Count GPUs
    let list_output = Command::new("rocm-smi").arg("--showid").output().ok()?;

    let gpu_count = if list_output.status.success() {
        String::from_utf8_lossy(&list_output.stdout)
            .lines()
            .filter(|line| line.contains("GPU"))
            .count()
    } else {
        0
    };

    // Try to get ROCm version from package
    let rocm_version = Command::new("dpkg")
        .args(["-l", "rocm-libs"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .find(|line| line.contains("rocm-libs"))
                    .and_then(|line| line.split_whitespace().nth(2).map(|s| s.to_string()))
            } else {
                None
            }
        });

    // Try to get HIP version
    let hip_version = Command::new("hipcc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .find(|line| line.contains("HIP version"))
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    Some(RocmSupport {
        rocm_available: true,
        rocm_version,
        hip_version,
        gpu_count,
    })
}

fn detect_oneapi_support() -> Option<OneApiSupport> {
    // Check for oneAPI
    let check = Command::new("which").arg("dpcpp").output().ok()?;
    if !check.status.success() {
        return None;
    }

    // Get oneAPI version
    let version_output = Command::new("dpcpp").arg("--version").output().ok()?;

    let oneapi_version = if version_output.status.success() {
        String::from_utf8_lossy(&version_output.stdout)
            .lines()
            .next()
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    // Check for Level Zero
    let level_zero_available = Command::new("which")
        .arg("ze_loader")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    Some(OneApiSupport {
        oneapi_available: true,
        oneapi_version,
        level_zero_available,
    })
}
