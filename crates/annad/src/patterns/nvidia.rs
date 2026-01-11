//! NVIDIA/GPU patterns for nvidia-smi, drivers, Optimus, PRIME.
//! v0.0.977: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create an nvidia-related DeepUnderstanding
fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::Factual,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

type NvidiaPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match nvidia-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_nvidia_smi(q)
        .or_else(|| match_nvidia_driver(q))
        .or_else(|| match_optimus(q))
        .or_else(|| match_nvidia_config(q))
        .or_else(|| match_nvidia_troubleshoot(q))
}

/// nvidia-smi patterns
fn match_nvidia_smi(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NvidiaPattern] = &[
        // nvidia-smi basic
        (&["nvidia", "smi"], "run nvidia-smi", "nvidia",
         &["nvidia-smi"]),
        (&["nvidia-smi"], "run nvidia-smi", "nvidia",
         &["nvidia-smi"]),
        // GPU usage
        (&["gpu", "usage"], "show GPU usage", "nvidia",
         &["nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader", "nvidia-smi"]),
        (&["gpu", "utilization"], "show GPU utilization", "nvidia",
         &["nvidia-smi --query-gpu=utilization.gpu,utilization.memory --format=csv"]),
        // GPU memory
        (&["gpu", "memory"], "show GPU memory usage", "nvidia",
         &["nvidia-smi --query-gpu=memory.used,memory.total --format=csv"]),
        (&["vram", "usage"], "show VRAM usage", "nvidia",
         &["nvidia-smi --query-gpu=memory.used,memory.total,memory.free --format=csv"]),
        // GPU temperature
        (&["gpu", "temp"], "show GPU temperature", "nvidia",
         &["nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader"]),
        (&["gpu", "temperature"], "show GPU temperature", "nvidia",
         &["nvidia-smi --query-gpu=temperature.gpu --format=csv", "sensors | grep -i gpu"]),
        // GPU power
        (&["gpu", "power"], "show GPU power draw", "nvidia",
         &["nvidia-smi --query-gpu=power.draw --format=csv"]),
        (&["gpu", "wattage"], "show GPU power consumption", "nvidia",
         &["nvidia-smi --query-gpu=power.draw,power.limit --format=csv"]),
        // GPU clock
        (&["gpu", "clock"], "show GPU clock speeds", "nvidia",
         &["nvidia-smi --query-gpu=clocks.gr,clocks.mem --format=csv"]),
        (&["gpu", "frequency"], "show GPU frequencies", "nvidia",
         &["nvidia-smi --query-gpu=clocks.gr,clocks.mem,clocks.video --format=csv"]),
        // GPU processes
        (&["gpu", "processes"], "show GPU processes", "nvidia",
         &["nvidia-smi --query-compute-apps=pid,name,used_memory --format=csv"]),
        (&["using", "gpu"], "show what is using GPU", "nvidia",
         &["nvidia-smi pmon -c 1", "nvidia-smi --query-compute-apps=pid,name --format=csv"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// NVIDIA driver patterns
fn match_nvidia_driver(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NvidiaPattern] = &[
        // Driver version
        (&["nvidia", "driver", "version"], "show NVIDIA driver version", "nvidia",
         &["nvidia-smi --query-gpu=driver_version --format=csv,noheader", "modinfo nvidia | grep ^version"]),
        (&["nvidia", "version"], "show NVIDIA version", "nvidia",
         &["nvidia-smi | head -3"]),
        // Driver installed
        (&["nvidia", "driver", "installed"], "check if NVIDIA driver installed", "nvidia",
         &["pacman -Q nvidia 2>/dev/null || pacman -Q nvidia-dkms 2>/dev/null", "lsmod | grep nvidia"]),
        (&["nvidia", "installed"], "check NVIDIA installation", "nvidia",
         &["pacman -Qs nvidia | grep -E 'nvidia|local'", "lsmod | grep nvidia"]),
        // Driver status
        (&["nvidia", "driver", "status"], "show NVIDIA driver status", "nvidia",
         &["lsmod | grep nvidia", "nvidia-smi 2>&1 | head -5"]),
        (&["nvidia", "module"], "show NVIDIA kernel modules", "nvidia",
         &["lsmod | grep -E 'nvidia|nouveau'"]),
        // CUDA version
        (&["cuda", "version"], "show CUDA version", "nvidia",
         &["nvidia-smi | grep CUDA", "nvcc --version 2>/dev/null"]),
        (&["cuda", "installed"], "check if CUDA is installed", "nvidia",
         &["pacman -Q cuda 2>/dev/null", "nvcc --version 2>/dev/null"]),
        // Nouveau vs NVIDIA
        (&["nouveau", "nvidia"], "check nouveau vs nvidia", "nvidia",
         &["lsmod | grep -E 'nouveau|nvidia'"]),
        (&["nouveau", "status"], "check nouveau status", "nvidia",
         &["lsmod | grep nouveau", "cat /etc/modprobe.d/*.conf 2>/dev/null | grep nouveau"]),
        // Blacklist nouveau
        (&["blacklist", "nouveau"], "check nouveau blacklist", "nvidia",
         &["cat /etc/modprobe.d/*.conf 2>/dev/null | grep nouveau", "cat /usr/lib/modprobe.d/*.conf 2>/dev/null | grep nouveau"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Optimus/PRIME patterns
fn match_optimus(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NvidiaPattern] = &[
        // PRIME status
        (&["prime", "status"], "show PRIME status", "nvidia",
         &["prime-select query 2>/dev/null || echo 'prime-select not found'", "cat /proc/driver/nvidia/gpus/*/information 2>/dev/null"]),
        (&["prime", "select"], "show PRIME selection", "nvidia",
         &["prime-select query 2>/dev/null", "optimus-manager --status 2>/dev/null"]),
        // Optimus manager
        (&["optimus", "manager"], "show optimus-manager status", "nvidia",
         &["optimus-manager --status 2>/dev/null || echo 'optimus-manager not installed'"]),
        (&["optimus", "status"], "show Optimus status", "nvidia",
         &["optimus-manager --status 2>/dev/null", "prime-select query 2>/dev/null"]),
        // Bumblebee
        (&["bumblebee", "status"], "show Bumblebee status", "nvidia",
         &["systemctl status bumblebeed 2>/dev/null", "optirun --status 2>/dev/null"]),
        // Hybrid graphics
        (&["hybrid", "graphics"], "show hybrid graphics status", "nvidia",
         &["glxinfo | grep -i vendor", "prime-select query 2>/dev/null || optimus-manager --status 2>/dev/null"]),
        // Which GPU active
        (&["which", "gpu", "active"], "show active GPU", "nvidia",
         &["glxinfo | grep -i 'opengl renderer'", "prime-select query 2>/dev/null"]),
        (&["active", "gpu"], "show active GPU", "nvidia",
         &["glxinfo | grep -i 'opengl renderer'"]),
        // Offload
        (&["prime", "offload"], "show PRIME offload status", "nvidia",
         &["echo 'Use: __NV_PRIME_RENDER_OFFLOAD=1 command'", "cat /proc/driver/nvidia/gpus/*/information 2>/dev/null"]),
        (&["nvidia", "offload"], "show NVIDIA offload", "nvidia",
         &["echo 'Use: __NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia command'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// NVIDIA config patterns
fn match_nvidia_config(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NvidiaPattern] = &[
        // nvidia-settings
        (&["nvidia", "settings"], "show nvidia-settings info", "nvidia",
         &["nvidia-settings --query all 2>/dev/null | head -30"]),
        // Xorg nvidia
        (&["xorg", "nvidia"], "show Xorg NVIDIA config", "nvidia",
         &["cat /etc/X11/xorg.conf 2>/dev/null | head -50", "cat /etc/X11/xorg.conf.d/*nvidia*.conf 2>/dev/null"]),
        (&["nvidia", "xorg"], "show NVIDIA Xorg config", "nvidia",
         &["ls /etc/X11/xorg.conf.d/ 2>/dev/null", "cat /etc/X11/xorg.conf 2>/dev/null | grep -A10 -i nvidia"]),
        // Persistence mode
        (&["nvidia", "persistence"], "show NVIDIA persistence mode", "nvidia",
         &["nvidia-smi --query-gpu=persistence_mode --format=csv"]),
        // Power management
        (&["nvidia", "power", "mode"], "show NVIDIA power mode", "nvidia",
         &["nvidia-smi --query-gpu=power.management,power.limit --format=csv"]),
        // GPU info
        (&["nvidia", "gpu", "info"], "show NVIDIA GPU info", "nvidia",
         &["nvidia-smi --query-gpu=name,driver_version,memory.total,pci.bus_id --format=csv"]),
        (&["nvidia", "card"], "show NVIDIA card info", "nvidia",
         &["nvidia-smi --query-gpu=name,memory.total --format=csv", "lspci | grep -i nvidia"]),
        // Multiple GPUs
        (&["nvidia", "gpus"], "list NVIDIA GPUs", "nvidia",
         &["nvidia-smi -L", "lspci | grep -i nvidia"]),
        (&["multiple", "gpu"], "show multiple GPUs", "nvidia",
         &["nvidia-smi -L", "lspci | grep -E 'VGA|3D'"]),
        // EGL
        (&["nvidia", "egl"], "show NVIDIA EGL status", "nvidia",
         &["eglinfo 2>/dev/null | head -20", "ls /usr/share/glvnd/egl_vendor.d/"]),
        // Vulkan
        (&["nvidia", "vulkan"], "show NVIDIA Vulkan status", "nvidia",
         &["vulkaninfo --summary 2>/dev/null | head -20"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// NVIDIA troubleshooting patterns
fn match_nvidia_troubleshoot(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[NvidiaPattern] = &[
        // NVIDIA errors
        (&["nvidia", "errors"], "show NVIDIA errors", "nvidia",
         &["dmesg | grep -i nvidia | tail -20", "journalctl -b | grep -i nvidia | grep -iE 'error|fail' | tail -10"]),
        (&["nvidia", "logs"], "show NVIDIA logs", "nvidia",
         &["journalctl -b | grep -i nvidia | tail -30", "dmesg | grep -i nvidia"]),
        // NVIDIA not working
        (&["nvidia", "not", "working"], "troubleshoot NVIDIA", "nvidia",
         &["lsmod | grep nvidia", "nvidia-smi 2>&1", "dmesg | grep -i nvidia | tail -10"]),
        (&["nvidia", "failed"], "check NVIDIA failures", "nvidia",
         &["dmesg | grep -iE 'nvidia.*error|nvidia.*fail'", "journalctl -b | grep -i 'nvidia.*error' | tail -10"]),
        // GPU not detected
        (&["gpu", "not", "detected"], "troubleshoot GPU detection", "nvidia",
         &["lspci | grep -i nvidia", "lsmod | grep nvidia", "dmesg | grep -i nvidia"]),
        (&["nvidia", "not", "detected"], "troubleshoot NVIDIA detection", "nvidia",
         &["lspci | grep -i nvidia", "lsmod | grep -E 'nvidia|nouveau'"]),
        // Screen tearing
        (&["nvidia", "tearing"], "check NVIDIA screen tearing", "nvidia",
         &["nvidia-settings --query CurrentMetaMode 2>/dev/null", "cat /etc/X11/xorg.conf.d/*nvidia* 2>/dev/null | grep -i sync"]),
        (&["screen", "tearing", "nvidia"], "fix NVIDIA screen tearing", "nvidia",
         &["echo 'Add ForceCompositionPipeline=On to nvidia-settings or xorg.conf'"]),
        // Performance mode
        (&["nvidia", "performance"], "set NVIDIA performance mode", "nvidia",
         &["nvidia-smi --query-gpu=power.management --format=csv", "echo 'Use: nvidia-smi -pm 1' for persistence"]),
        // Fan control
        (&["nvidia", "fan"], "show NVIDIA fan info", "nvidia",
         &["nvidia-smi --query-gpu=fan.speed --format=csv", "nvidia-settings --query all 2>/dev/null | grep -i fan"]),
        // DRM modeset
        (&["nvidia", "drm"], "check NVIDIA DRM modeset", "nvidia",
         &["cat /sys/module/nvidia_drm/parameters/modeset", "cat /etc/modprobe.d/*.conf 2>/dev/null | grep nvidia.*modeset"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvidia_smi() {
        assert!(match_patterns("nvidia smi").is_some());
        assert!(match_patterns("gpu usage").is_some());
        assert!(match_patterns("gpu memory").is_some());
        assert!(match_patterns("gpu temperature").is_some());
    }

    #[test]
    fn test_nvidia_driver() {
        assert!(match_patterns("nvidia driver version").is_some());
        assert!(match_patterns("cuda version").is_some());
        assert!(match_patterns("nvidia module").is_some());
    }

    #[test]
    fn test_optimus() {
        assert!(match_patterns("prime status").is_some());
        assert!(match_patterns("optimus manager").is_some());
        assert!(match_patterns("hybrid graphics").is_some());
    }

    #[test]
    fn test_nvidia_config() {
        assert!(match_patterns("nvidia settings").is_some());
        assert!(match_patterns("nvidia gpu info").is_some());
        assert!(match_patterns("nvidia vulkan").is_some());
    }

    #[test]
    fn test_nvidia_troubleshoot() {
        assert!(match_patterns("nvidia errors").is_some());
        assert!(match_patterns("nvidia not working").is_some());
        assert!(match_patterns("gpu not detected").is_some());
    }
}
