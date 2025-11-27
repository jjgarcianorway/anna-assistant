//! Learning Job Processor v0.11.0
//!
//! Processes learning jobs from the queue using LLM-A/LLM-B.

use anna_common::{Fact, KnowledgeStore, LearningEvent, LearningJob, MappingPhase};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// Maximum jobs to process per cycle
const MAX_JOBS_PER_CYCLE: usize = 5;

/// Job processor for learning tasks
pub struct JobProcessor {
    /// Job queue (priority ordered)
    queue: Mutex<VecDeque<LearningJob>>,
    /// Knowledge store reference
    store: Arc<RwLock<KnowledgeStore>>,
    /// Whether processor is running
    running: Mutex<bool>,
}

impl JobProcessor {
    /// Create a new job processor
    pub fn new(store: Arc<RwLock<KnowledgeStore>>) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            store,
            running: Mutex::new(false),
        }
    }

    /// Enqueue a learning job
    pub async fn enqueue(&self, job: LearningJob) {
        let mut queue = self.queue.lock().await;

        // Insert by priority (higher priority first)
        let insert_pos = queue
            .iter()
            .position(|j| j.priority.as_i32() < job.priority.as_i32())
            .unwrap_or(queue.len());

        queue.insert(insert_pos, job);

        debug!("Enqueued learning job, queue size: {}", queue.len());
    }

    /// Get queue size
    pub async fn queue_size(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Process pending jobs (called periodically by annad)
    pub async fn process_pending(&self) -> Vec<String> {
        let mut processed_ids = Vec::new();

        // Check if already running
        {
            let mut running = self.running.lock().await;
            if *running {
                return processed_ids;
            }
            *running = true;
        }

        // Process up to MAX_JOBS_PER_CYCLE jobs
        for _ in 0..MAX_JOBS_PER_CYCLE {
            let job = {
                let mut queue = self.queue.lock().await;
                queue.pop_front()
            };

            match job {
                Some(mut job) => {
                    info!(
                        "Processing learning job: {} ({:?})",
                        job.id,
                        job.event.event_type()
                    );
                    job.start();

                    match self.process_job(&mut job).await {
                        Ok(facts) => {
                            job.complete(facts.iter().map(|f| f.id.clone()).collect());
                            processed_ids.push(job.id.clone());
                            info!("Job {} completed, {} facts affected", job.id, facts.len());
                        }
                        Err(e) => {
                            error!("Job {} failed: {}", job.id, e);
                            job.fail(&e.to_string());

                            // Re-enqueue if retryable
                            if job.can_retry() {
                                warn!("Job {} will be retried (attempt {})", job.id, job.retries);
                                self.enqueue(job).await;
                            }
                        }
                    }
                }
                None => break,
            }
        }

        // Mark as not running
        *self.running.lock().await = false;

        processed_ids
    }

    /// Process a single job
    async fn process_job(&self, job: &mut LearningJob) -> anyhow::Result<Vec<Fact>> {
        match &job.event {
            LearningEvent::InitialMapping { phase } => self.process_mapping_phase(*phase).await,
            LearningEvent::PackageAdded { name, version } => {
                self.process_package_added(name, version.as_deref()).await
            }
            LearningEvent::PackageRemoved { name } => self.process_package_removed(name).await,
            LearningEvent::PackageUpgraded {
                name,
                old_version,
                new_version,
            } => {
                self.process_package_upgraded(name, old_version, new_version)
                    .await
            }
            LearningEvent::ServiceChanged { name, state } => {
                self.process_service_changed(name, state).await
            }
            LearningEvent::ScheduledRefresh { target } => {
                self.process_scheduled_refresh(target).await
            }
            _ => {
                debug!("Unhandled event type: {:?}", job.event.event_type());
                Ok(Vec::new())
            }
        }
    }

    /// Process initial system mapping for a phase
    async fn process_mapping_phase(&self, phase: MappingPhase) -> anyhow::Result<Vec<Fact>> {
        info!("Running system mapping phase: {:?}", phase);
        let mut facts = Vec::new();

        match phase {
            MappingPhase::Hardware => {
                facts.extend(self.map_hardware().await?);
            }
            MappingPhase::CoreSoftware => {
                facts.extend(self.map_core_software().await?);
            }
            MappingPhase::Network => {
                facts.extend(self.map_network().await?);
            }
            MappingPhase::Desktop => {
                facts.extend(self.map_desktop().await?);
            }
            MappingPhase::UserContext => {
                facts.extend(self.map_user_context().await?);
            }
            MappingPhase::Services => {
                facts.extend(self.map_services().await?);
            }
        }

        // Store facts
        let store = self.store.write().await;
        for fact in &facts {
            store.upsert(fact)?;
        }

        Ok(facts)
    }

    /// Map hardware info
    async fn map_hardware(&self) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        // CPU info via lscpu
        if let Ok(output) = tokio::process::Command::new("lscpu")
            .arg("-J")
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(lscpu) = json.get("lscpu").and_then(|v| v.as_array()) {
                        for item in lscpu {
                            let field = item.get("field").and_then(|v| v.as_str()).unwrap_or("");
                            let data = item.get("data").and_then(|v| v.as_str()).unwrap_or("");

                            match field {
                                "CPU(s):" => {
                                    facts.push(Fact::from_probe(
                                        "cpu:0".to_string(),
                                        "cores".to_string(),
                                        data.to_string(),
                                        "cpu.info",
                                        0.95,
                                    ));
                                }
                                "Model name:" => {
                                    facts.push(Fact::from_probe(
                                        "cpu:0".to_string(),
                                        "model".to_string(),
                                        data.to_string(),
                                        "cpu.info",
                                        0.95,
                                    ));
                                }
                                "Architecture:" => {
                                    facts.push(Fact::from_probe(
                                        "cpu:0".to_string(),
                                        "architecture".to_string(),
                                        data.to_string(),
                                        "cpu.info",
                                        0.95,
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Memory info
        if let Ok(output) = tokio::process::Command::new("cat")
            .arg("/proc/meminfo")
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.starts_with("MemTotal:") {
                        let value = line.replace("MemTotal:", "").trim().to_string();
                        facts.push(Fact::from_probe(
                            "system:memory".to_string(),
                            "total".to_string(),
                            value,
                            "mem.info",
                            0.95,
                        ));
                    }
                }
            }
        }

        Ok(facts)
    }

    /// Map core software
    async fn map_core_software(&self) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        // Kernel info
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

        Ok(facts)
    }

    /// Map network configuration
    async fn map_network(&self) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        // Get network interfaces
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

        Ok(facts)
    }

    /// Map desktop environment
    async fn map_desktop(&self) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        // Detect from environment
        if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
            facts.push(Fact::from_probe(
                "desktop:current".to_string(),
                "name".to_string(),
                de,
                "env.XDG_CURRENT_DESKTOP",
                0.9,
            ));
        }

        // Check for common WMs/DEs
        let wms = ["hyprland", "sway", "i3", "gnome-shell", "plasmashell"];
        for wm in wms {
            if let Ok(output) = tokio::process::Command::new("pgrep")
                .arg("-x")
                .arg(wm)
                .output()
                .await
            {
                if output.status.success() {
                    facts.push(Fact::from_probe(
                        format!("app:{}", wm),
                        "running".to_string(),
                        "true".to_string(),
                        "pgrep",
                        0.85,
                    ));
                }
            }
        }

        Ok(facts)
    }

    /// Map user context
    async fn map_user_context(&self) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        // Current shell
        if let Ok(shell) = std::env::var("SHELL") {
            facts.push(Fact::from_probe(
                "user:current".to_string(),
                "shell".to_string(),
                shell,
                "env.SHELL",
                0.95,
            ));
        }

        // Editor preference
        if let Ok(editor) = std::env::var("EDITOR") {
            facts.push(Fact::from_probe(
                "user:current".to_string(),
                "editor".to_string(),
                editor,
                "env.EDITOR",
                0.9,
            ));
        }

        Ok(facts)
    }

    /// Map services
    async fn map_services(&self) -> anyhow::Result<Vec<Fact>> {
        // Services require systemctl access which may not be available
        // This is a placeholder for now
        Ok(Vec::new())
    }

    /// Process package added event
    async fn process_package_added(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> anyhow::Result<Vec<Fact>> {
        let mut facts = Vec::new();

        facts.push(Fact::from_probe(
            format!("pkg:{}", name),
            "installed".to_string(),
            "true".to_string(),
            "pacman.log",
            0.95,
        ));

        if let Some(v) = version {
            facts.push(Fact::from_probe(
                format!("pkg:{}", name),
                "version".to_string(),
                v.to_string(),
                "pacman.log",
                0.95,
            ));
        }

        // Store facts
        let store = self.store.write().await;
        for fact in &facts {
            store.upsert(fact)?;
        }

        Ok(facts)
    }

    /// Process package removed event
    async fn process_package_removed(&self, name: &str) -> anyhow::Result<Vec<Fact>> {
        let store = self.store.write().await;
        store.delete(&format!("pkg:{}", name), "installed", "package removed")?;
        store.delete(&format!("pkg:{}", name), "version", "package removed")?;
        Ok(Vec::new())
    }

    /// Process package upgraded event
    async fn process_package_upgraded(
        &self,
        name: &str,
        _old_version: &str,
        new_version: &str,
    ) -> anyhow::Result<Vec<Fact>> {
        let fact = Fact::from_probe(
            format!("pkg:{}", name),
            "version".to_string(),
            new_version.to_string(),
            "pacman.log",
            0.95,
        );

        let store = self.store.write().await;
        store.upsert(&fact)?;

        Ok(vec![fact])
    }

    /// Process service state change
    async fn process_service_changed(&self, name: &str, state: &str) -> anyhow::Result<Vec<Fact>> {
        let fact = Fact::from_probe(
            format!("svc:{}", name),
            "state".to_string(),
            state.to_string(),
            "systemd",
            0.9,
        );

        let store = self.store.write().await;
        store.upsert(&fact)?;

        Ok(vec![fact])
    }

    /// Process scheduled refresh
    async fn process_scheduled_refresh(&self, target: &str) -> anyhow::Result<Vec<Fact>> {
        if target == "hygiene" {
            // Run hygiene check
            let store = self.store.write().await;
            let stale_count = store.mark_stale_by_age(24)?; // 24 hours
            info!("Knowledge hygiene: marked {} facts as stale", stale_count);
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::JobPriority;
    use tempfile::tempdir;

    async fn test_processor() -> (JobProcessor, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_knowledge.db");
        let store = KnowledgeStore::open(&path).unwrap();
        let processor = JobProcessor::new(Arc::new(RwLock::new(store)));
        (processor, dir)
    }

    #[tokio::test]
    async fn test_enqueue_by_priority() {
        let (processor, _dir) = test_processor().await;

        let low = LearningJob::new(LearningEvent::ScheduledRefresh {
            target: "test".to_string(),
        });
        let high = LearningJob::new(LearningEvent::GpuDriverChanged {
            available: true,
            driver: None,
        });

        processor.enqueue(low).await;
        processor.enqueue(high).await;

        let queue = processor.queue.lock().await;
        assert_eq!(queue[0].priority, JobPriority::High);
    }
}
