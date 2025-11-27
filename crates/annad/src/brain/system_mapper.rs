//! System Mapper v0.11.0
//!
//! Maps system configuration on first install and incremental refreshes.

use anna_common::{Fact, KnowledgeStore, MappingPhase};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// System mapper - discovers and records system facts
pub struct SystemMapper {
    store: Arc<RwLock<KnowledgeStore>>,
}

impl SystemMapper {
    pub fn new(store: Arc<RwLock<KnowledgeStore>>) -> Self {
        Self { store }
    }

    /// Run a mapping phase and return discovered facts
    pub async fn run_phase(&self, phase: MappingPhase) -> anyhow::Result<Vec<Fact>> {
        info!("Running mapping phase: {:?}", phase);

        let facts = match phase {
            MappingPhase::Hardware => self.map_hardware().await?,
            MappingPhase::CoreSoftware => self.map_core_software().await?,
            MappingPhase::Desktop => self.map_desktop().await?,
            MappingPhase::UserContext => self.map_user_context().await?,
            MappingPhase::Network => self.map_network().await?,
            MappingPhase::Services => self.map_services().await?,
        };

        // Store discovered facts
        {
            let store = self.store.write().await;
            for fact in &facts {
                if let Err(e) = store.upsert(fact) {
                    tracing::warn!("Failed to store fact: {}", e);
                }
            }
        }

        info!(
            "Phase {:?} complete, {} facts discovered",
            phase,
            facts.len()
        );
        Ok(facts)
    }

    /// Map hardware configuration
    async fn map_hardware(&self) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        // CPU via lscpu
        if let Some(cpu_facts) = self.probe_cpu().await {
            facts.extend(cpu_facts);
        }

        // Memory
        if let Some(mem_facts) = self.probe_memory().await {
            facts.extend(mem_facts);
        }

        // Disks
        if let Some(disk_facts) = self.probe_disks().await {
            facts.extend(disk_facts);
        }

        // GPU
        if let Some(gpu_facts) = self.probe_gpu().await {
            facts.extend(gpu_facts);
        }

        Ok(facts)
    }

    async fn probe_cpu(&self) -> Option<Vec<Fact>> {
        let output = tokio::process::Command::new("lscpu")
            .arg("-J")
            .output()
            .await
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;
        let lscpu = json.get("lscpu")?.as_array()?;

        let mut facts = Vec::new();

        for item in lscpu {
            let field = item.get("field")?.as_str()?;
            let data = item.get("data")?.as_str()?;

            let (attr, conf) = match field {
                "CPU(s):" => ("cores", 0.95),
                "Model name:" => ("model", 0.95),
                "Architecture:" => ("architecture", 0.95),
                "Thread(s) per core:" => ("threads_per_core", 0.95),
                "CPU max MHz:" => ("max_mhz", 0.9),
                _ => continue,
            };

            facts.push(Fact::from_probe(
                "cpu:0".to_string(),
                attr.to_string(),
                data.to_string(),
                "cpu.info",
                conf,
            ));
        }

        Some(facts)
    }

    async fn probe_memory(&self) -> Option<Vec<Fact>> {
        let output = tokio::process::Command::new("cat")
            .arg("/proc/meminfo")
            .output()
            .await
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut facts = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() != 2 {
                continue;
            }

            let (key, attr) = match parts[0] {
                "MemTotal" => ("total", true),
                "MemFree" => ("free", true),
                "SwapTotal" => ("swap_total", true),
                _ => continue,
            };

            if attr {
                facts.push(Fact::from_probe(
                    "system:memory".to_string(),
                    key.to_string(),
                    parts[1].trim().to_string(),
                    "mem.info",
                    0.95,
                ));
            }
        }

        Some(facts)
    }

    async fn probe_disks(&self) -> Option<Vec<Fact>> {
        let output = tokio::process::Command::new("lsblk")
            .args(["-J", "-b", "-o", "NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT"])
            .output()
            .await
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;
        let devices = json.get("blockdevices")?.as_array()?;

        let mut facts = Vec::new();

        for dev in devices {
            let name = dev.get("name")?.as_str()?;
            let dtype = dev
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let size = dev.get("size").and_then(|v| v.as_u64()).unwrap_or(0);

            if dtype == "disk" {
                facts.push(Fact::from_probe(
                    format!("disk:{}", name),
                    "size_bytes".to_string(),
                    size.to_string(),
                    "disk.lsblk",
                    0.95,
                ));
                facts.push(Fact::from_probe(
                    format!("disk:{}", name),
                    "type".to_string(),
                    dtype.to_string(),
                    "disk.lsblk",
                    0.95,
                ));
            }

            // Check partitions
            if let Some(children) = dev.get("children").and_then(|v| v.as_array()) {
                for child in children {
                    let cname = child.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let mountpoint = child.get("mountpoint").and_then(|v| v.as_str());
                    let fstype = child.get("fstype").and_then(|v| v.as_str());

                    if let Some(mp) = mountpoint {
                        facts.push(Fact::from_probe(
                            format!("fs:{}", mp),
                            "device".to_string(),
                            format!("/dev/{}", cname),
                            "disk.lsblk",
                            0.95,
                        ));
                    }

                    if let Some(fs) = fstype {
                        facts.push(Fact::from_probe(
                            format!("disk:{}", cname),
                            "fstype".to_string(),
                            fs.to_string(),
                            "disk.lsblk",
                            0.95,
                        ));
                    }
                }
            }
        }

        Some(facts)
    }

    async fn probe_gpu(&self) -> Option<Vec<Fact>> {
        let output = tokio::process::Command::new("lspci").output().await.ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut facts = Vec::new();
        let mut gpu_idx = 0;

        for line in stdout.lines() {
            if line.contains("VGA") || line.contains("3D controller") {
                facts.push(Fact::from_probe(
                    format!("gpu:{}", gpu_idx),
                    "description".to_string(),
                    line.to_string(),
                    "hardware.gpu",
                    0.9,
                ));

                // Detect vendor
                let vendor = if line.to_lowercase().contains("nvidia") {
                    "nvidia"
                } else if line.to_lowercase().contains("amd")
                    || line.to_lowercase().contains("radeon")
                {
                    "amd"
                } else if line.to_lowercase().contains("intel") {
                    "intel"
                } else {
                    "unknown"
                };

                facts.push(Fact::from_probe(
                    format!("gpu:{}", gpu_idx),
                    "vendor".to_string(),
                    vendor.to_string(),
                    "hardware.gpu",
                    0.85,
                ));

                gpu_idx += 1;
            }
        }

        Some(facts)
    }

    /// Map core software
    async fn map_core_software(&self) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        // Kernel
        if let Ok(output) = tokio::process::Command::new("uname")
            .arg("-r")
            .output()
            .await
        {
            if output.status.success() {
                let kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
                facts.push(Fact::from_probe(
                    "system:kernel".to_string(),
                    "version".to_string(),
                    kernel,
                    "system.kernel",
                    0.95,
                ));
            }
        }

        // Distribution
        if let Ok(output) = tokio::process::Command::new("cat")
            .arg("/etc/os-release")
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.starts_with("NAME=") {
                        let name = line.replace("NAME=", "").replace('"', "");
                        facts.push(Fact::from_probe(
                            "system:distro".to_string(),
                            "name".to_string(),
                            name,
                            "os-release",
                            0.95,
                        ));
                    } else if line.starts_with("VERSION_ID=") {
                        let version = line.replace("VERSION_ID=", "").replace('"', "");
                        facts.push(Fact::from_probe(
                            "system:distro".to_string(),
                            "version".to_string(),
                            version,
                            "os-release",
                            0.95,
                        ));
                    }
                }
            }
        }

        Ok(facts)
    }

    /// Map desktop environment
    async fn map_desktop(&self) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        // XDG_CURRENT_DESKTOP
        if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
            facts.push(Fact::from_probe(
                "desktop:current".to_string(),
                "name".to_string(),
                de,
                "env.XDG_CURRENT_DESKTOP",
                0.9,
            ));
        }

        // XDG_SESSION_TYPE
        if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
            facts.push(Fact::from_probe(
                "desktop:current".to_string(),
                "session_type".to_string(),
                session,
                "env.XDG_SESSION_TYPE",
                0.9,
            ));
        }

        Ok(facts)
    }

    /// Map user context
    async fn map_user_context(&self) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        // Shell
        if let Ok(shell) = std::env::var("SHELL") {
            facts.push(Fact::from_probe(
                "user:current".to_string(),
                "shell".to_string(),
                shell,
                "env.SHELL",
                0.95,
            ));
        }

        // Editor
        if let Ok(editor) = std::env::var("EDITOR") {
            facts.push(Fact::from_probe(
                "user:current".to_string(),
                "editor".to_string(),
                editor,
                "env.EDITOR",
                0.9,
            ));
        }

        // Home directory
        if let Ok(home) = std::env::var("HOME") {
            facts.push(Fact::from_probe(
                "user:current".to_string(),
                "home".to_string(),
                home,
                "env.HOME",
                0.95,
            ));
        }

        Ok(facts)
    }

    /// Map network configuration
    async fn map_network(&self) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        // Network interfaces
        if let Ok(output) = tokio::process::Command::new("ip")
            .args(["-j", "link", "show"])
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(interfaces) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                    for iface in interfaces {
                        let name = iface.get("ifname").and_then(|v| v.as_str()).unwrap_or("");
                        let state = iface
                            .get("operstate")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        if !name.is_empty() && name != "lo" {
                            facts.push(Fact::from_probe(
                                format!("net:{}", name),
                                "state".to_string(),
                                state.to_string(),
                                "net.links",
                                0.9,
                            ));
                        }
                    }
                }
            }
        }

        // DNS servers
        if let Ok(output) = tokio::process::Command::new("cat")
            .arg("/etc/resolv.conf")
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut nameservers = Vec::new();

                for line in stdout.lines() {
                    if line.starts_with("nameserver") {
                        if let Some(ns) = line.split_whitespace().nth(1) {
                            nameservers.push(ns.to_string());
                        }
                    }
                }

                if !nameservers.is_empty() {
                    facts.push(Fact::from_probe(
                        "net:dns".to_string(),
                        "nameservers".to_string(),
                        nameservers.join(","),
                        "dns.resolv",
                        0.9,
                    ));
                }
            }
        }

        Ok(facts)
    }

    /// Map services
    async fn map_services(&self) -> anyhow::Result<Vec<Fact>> {
        // Service mapping requires systemctl which may not be accessible
        Ok(Vec::new())
    }
}
