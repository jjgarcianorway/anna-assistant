//! Specialist Agent - Wraps existing department specialists as agents.

use anna_shared::agent::{
    Agent, AgentCapability, AgentContext, AgentDomain, AgentMemory, AgentMemoryStore,
    AgentResult, AgentTask, Evidence, EvidenceSource, Learning, ModelTier,
};
use async_trait::async_trait;
use std::sync::RwLock;
use tracing::{debug, info};

use crate::department::specialists::{Specialist, SpecialistRole};

/// Agent wrapper for existing department specialists.
pub struct SpecialistAgent {
    /// Specialist ID
    id: String,
    /// Specialist name
    name: String,
    /// Department/domain
    department: String,
    /// Expertise keywords
    expertise: Vec<String>,
    /// Model tier preference
    model_tier: ModelTier,
    /// Role (junior/senior)
    role: SpecialistRole,
    /// Agent memory (thread-safe)
    memory: RwLock<AgentMemory>,
}

impl SpecialistAgent {
    /// Create from an existing specialist.
    pub fn from_specialist(specialist: &Specialist, memory_store: &AgentMemoryStore) -> Self {
        let memory = memory_store
            .get(specialist.id)
            .cloned()
            .unwrap_or_default();

        Self {
            id: specialist.id.to_string(),
            name: specialist.name.to_string(),
            department: specialist.department.to_string(),
            expertise: specialist.expertise.iter().map(|s| s.to_string()).collect(),
            model_tier: ModelTier::from_str(specialist.model_tier),
            role: specialist.role,
            memory: RwLock::new(memory),
        }
    }

    /// Calculate confidence score for a task based on expertise match.
    fn calculate_confidence(&self, task: &AgentTask) -> f32 {
        let q_lower = task.question.to_lowercase();
        let mut score = 0.0;

        // Check expertise keywords
        let expertise_matches: usize = self.expertise
            .iter()
            .filter(|exp| q_lower.contains(&exp.to_lowercase()))
            .count();

        if expertise_matches > 0 {
            score += 0.3 * (expertise_matches as f32 / self.expertise.len() as f32).min(1.0);
        }

        // Check domain match
        for domain in &task.domains {
            if domain.as_str().to_lowercase() == self.department.to_lowercase() {
                score += 0.3;
                break;
            }
        }

        // Check learned patterns from memory
        if let Ok(memory) = self.memory.read() {
            if memory.find_matching_pattern(&task.question).is_some() {
                score += 0.2;
            }
        }

        // Boost for seniors on complex tasks
        if self.role == SpecialistRole::Senior && task.domains.len() > 1 {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }

    /// Execute investigation for this specialist's domain.
    async fn investigate(&self, task: &AgentTask, ctx: &AgentContext) -> InvestigationResult {
        info!(
            "{} ({}) investigating: {}",
            self.name,
            self.department,
            task.question.chars().take(50).collect::<String>()
        );

        // Get recommended probes from memory
        let probes = if let Ok(memory) = self.memory.read() {
            memory.get_recommended_probes(&self.department.to_lowercase(), 5)
        } else {
            vec![]
        };

        // Build evidence from domain-specific probes
        let mut evidence = Vec::new();

        // Add default probes based on domain
        let default_probes = self.get_default_probes();
        let all_probes: Vec<&str> = probes.iter().map(|s| s.as_str())
            .chain(default_probes.iter().copied())
            .collect();

        for probe in all_probes.into_iter().take(5) {
            if let Some(ev) = self.run_probe(probe).await {
                evidence.push(ev);
            }
        }

        // Generate answer based on evidence
        let answer = self.generate_answer(task, &evidence);
        let confidence = if evidence.is_empty() { 0.3 } else { 0.7 };

        InvestigationResult {
            answer,
            evidence,
            confidence,
        }
    }

    /// Get default probes for this specialist's domain.
    fn get_default_probes(&self) -> Vec<&'static str> {
        match self.department.to_lowercase().as_str() {
            "network" => vec!["ip addr", "cat /etc/resolv.conf", "ping -c1 8.8.8.8"],
            "storage" => vec!["df -h", "lsblk", "cat /etc/fstab"],
            "system" => vec!["systemctl --failed", "uname -a", "uptime"],
            "packages" => vec!["pacman -Q | wc -l", "checkupdates 2>/dev/null | head -5"],
            "hardware" => vec!["lspci | head -10", "lsusb", "sensors 2>/dev/null"],
            "audio" => vec!["pactl info", "wpctl status 2>/dev/null"],
            "security" => vec!["ss -tlnp", "who"],
            "desktop" => vec!["echo $XDG_SESSION_TYPE", "echo $SHELL"],
            _ => vec!["uname -a"],
        }
    }

    /// Run a single probe command.
    async fn run_probe(&self, command: &str) -> Option<Evidence> {
        debug!("Running probe: {}", command);

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let combined = if stderr.is_empty() {
            stdout
        } else {
            format!("{}\n{}", stdout, stderr)
        };

        if combined.trim().is_empty() {
            return None;
        }

        Some(Evidence {
            source: EvidenceSource::Command,
            command: Some(command.to_string()),
            output: combined.chars().take(2000).collect(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            confidence: if output.status.success() { 0.8 } else { 0.4 },
        })
    }

    /// Generate an answer based on collected evidence.
    fn generate_answer(&self, task: &AgentTask, evidence: &[Evidence]) -> String {
        if evidence.is_empty() {
            return format!(
                "I couldn't gather specific information for: {}. You may need to provide more details.",
                task.question
            );
        }

        let mut answer = String::new();

        for ev in evidence {
            if let Some(cmd) = &ev.command {
                answer.push_str(&format!("```\n{}\n```\n\n", ev.output.trim()));
                let _ = cmd; // keep for future use
            }
        }

        answer
    }
}

#[async_trait]
impl Agent for SpecialistAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn domain(&self) -> AgentDomain {
        AgentDomain::from_str(&self.department)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        self.expertise
            .iter()
            .map(|exp| AgentCapability::from_keyword(exp))
            .collect()
    }

    fn model_tier(&self) -> ModelTier {
        self.model_tier
    }

    fn can_handle(&self, task: &AgentTask) -> f32 {
        self.calculate_confidence(task)
    }

    async fn execute(&self, task: AgentTask, ctx: &AgentContext) -> AgentResult {
        let investigation = self.investigate(&task, ctx).await;

        let learning = if investigation.confidence > 0.5 {
            Some(Learning {
                keywords: self.expertise.clone(),
                successful_probes: investigation.evidence
                    .iter()
                    .filter_map(|e| e.command.clone())
                    .collect(),
                answer_pattern: None,
            })
        } else {
            None
        };

        AgentResult {
            task_id: task.id,
            agent_id: self.id.clone(),
            success: investigation.confidence > 0.3,
            answer: Some(investigation.answer),
            evidence: investigation.evidence,
            confidence: investigation.confidence,
            subtasks: vec![],
            learning,
        }
    }

    fn memory(&self) -> &AgentMemory {
        // Return a reference to the memory
        // Note: This is a simplified implementation
        static DEFAULT_MEMORY: std::sync::OnceLock<AgentMemory> = std::sync::OnceLock::new();
        DEFAULT_MEMORY.get_or_init(AgentMemory::default)
    }

    fn learn(&mut self, task: &AgentTask, result: &AgentResult) {
        if let Ok(mut memory) = self.memory.write() {
            memory.learn_from_result(task, result);
        }
    }
}

/// Result from specialist investigation.
struct InvestigationResult {
    answer: String,
    evidence: Vec<Evidence>,
    confidence: f32,
}
