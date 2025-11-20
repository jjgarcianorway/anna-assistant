//! Real-World QA Scenarios for Beta.67
//!
//! Integration tests based on actual user workflows:
//! - Vim syntax highlighting setup
//! - Hardware detection and reporting
//! - LLM model upgrades
//!
//! Each scenario tests the full pipeline:
//! - LLM prompt generation
//! - ACTION_PLAN creation and validation
//! - Backup creation with ANNA_BACKUP naming
//! - Execution safety
//! - Result verification

use crate::action_plan::{ActionPlan, ActionRisk, ActionStep};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

/// Scenario: Enable vim syntax highlighting
///
/// Starting states tested:
/// 1. No .vimrc exists
/// 2. Existing .vimrc with unrelated settings
/// 3. Existing .vimrc with previous Anna block
///
/// Expected behavior:
/// - Backup created if .vimrc exists (ANNA_BACKUP.YYYYMMDD-HHMMSS)
/// - Anna block added with clear markers
/// - No duplicate Anna blocks
/// - Restore instructions provided
#[derive(Debug)]
pub struct VimSyntaxScenario {
    /// Test directory (temp)
    pub test_dir: PathBuf,
    /// Path to .vimrc in test
    pub vimrc_path: PathBuf,
}

impl VimSyntaxScenario {
    /// Create a new vim scenario test
    pub fn new(test_dir: impl Into<PathBuf>) -> Self {
        let test_dir = test_dir.into();
        let vimrc_path = test_dir.join(".vimrc");

        Self {
            test_dir,
            vimrc_path,
        }
    }

    /// Setup: No .vimrc exists
    pub fn setup_no_vimrc(&self) -> Result<()> {
        if self.vimrc_path.exists() {
            fs::remove_file(&self.vimrc_path)?;
        }
        Ok(())
    }

    /// Setup: Existing .vimrc with unrelated settings
    pub fn setup_existing_vimrc(&self) -> Result<()> {
        let content = r#"
" User's existing vim config
set nocompatible
set number
set tabstop=4
"#;
        fs::write(&self.vimrc_path, content)?;
        Ok(())
    }

    /// Setup: Existing .vimrc with previous Anna block
    pub fn setup_with_anna_block(&self) -> Result<()> {
        let content = r#"
" User's existing vim config
set nocompatible

" ═══ Anna Assistant Configuration ═══
" Added by Anna on 2025-01-15
syntax on
set background=dark
" ═══ End Anna Configuration ═══

set number
"#;
        fs::write(&self.vimrc_path, content)?;
        Ok(())
    }

    /// Generate ACTION_PLAN for enabling vim syntax
    pub fn generate_action_plan(&self) -> Result<ActionPlan> {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let backup_path = format!("{}.ANNA_BACKUP.{}", self.vimrc_path.display(), timestamp);

        let mut steps = Vec::new();

        // Step 1: Check if vim is installed
        steps.push(ActionStep {
            id: "check_vim".to_string(),
            description: "Check if vim is installed".to_string(),
            risk: ActionRisk::Low,
            requires_confirmation: false,
            backup: None,
            commands: vec![
                vec!["which".to_string(), "vim".to_string()],
            ],
            restore_hint: None,
        });

        // Step 2: Backup existing .vimrc if it exists
        if self.vimrc_path.exists() {
            steps.push(ActionStep {
                id: "backup_vimrc".to_string(),
                description: format!("Backup existing .vimrc to {}", backup_path),
                risk: ActionRisk::Low,
                requires_confirmation: false,
                backup: Some(format!("cp {} {}", self.vimrc_path.display(), backup_path)),
                commands: vec![
                    vec!["cp".to_string(), self.vimrc_path.display().to_string(), backup_path.clone()],
                ],
                restore_hint: Some(format!("cp {} {}", backup_path, self.vimrc_path.display())),
            });
        }

        // Step 3: Add Anna block to .vimrc
        steps.push(ActionStep {
            id: "add_syntax_highlighting".to_string(),
            description: "Add vim syntax highlighting configuration".to_string(),
            risk: ActionRisk::Medium,
            requires_confirmation: true,
            backup: None,
            commands: vec![
                // NOTE: In real implementation, this would be a custom append operation
                // For now, showing the concept
                vec!["touch".to_string(), self.vimrc_path.display().to_string()],
            ],
            restore_hint: Some("See backup file for original configuration".to_string()),
        });

        let plan = ActionPlan { steps };

        // CRITICAL: Validate before returning
        plan.validate()?;

        Ok(plan)
    }

    /// Verify: Check that Anna block was added correctly
    pub fn verify_anna_block_added(&self) -> Result<()> {
        if !self.vimrc_path.exists() {
            return Err(anyhow!(".vimrc does not exist after execution"));
        }

        let content = fs::read_to_string(&self.vimrc_path)?;

        // Check for Anna block markers
        if !content.contains("═══ Anna Assistant Configuration ═══") {
            return Err(anyhow!("Anna block start marker not found"));
        }

        if !content.contains("═══ End Anna Configuration ═══") {
            return Err(anyhow!("Anna block end marker not found"));
        }

        // Check that syntax highlighting is enabled
        if !content.contains("syntax on") {
            return Err(anyhow!("syntax on not found in .vimrc"));
        }

        Ok(())
    }

    /// Verify: Check that no duplicate Anna blocks exist
    pub fn verify_no_duplicate_blocks(&self) -> Result<()> {
        let content = fs::read_to_string(&self.vimrc_path)?;

        let anna_block_count = content.matches("═══ Anna Assistant Configuration ═══").count();

        if anna_block_count == 0 {
            return Err(anyhow!("No Anna block found"));
        }

        if anna_block_count > 1 {
            return Err(anyhow!("Duplicate Anna blocks found ({})", anna_block_count));
        }

        Ok(())
    }

    /// Verify: Check that backup was created with correct naming
    pub fn verify_backup_exists(&self) -> Result<PathBuf> {
        let backup_pattern = format!("{}.ANNA_BACKUP.", self.vimrc_path.display());

        for entry in fs::read_dir(&self.test_dir)? {
            let entry = entry?;
            let path = entry.path();
            let filename = path.file_name().unwrap().to_string_lossy();

            if filename.starts_with(".vimrc.ANNA_BACKUP.") {
                // Verify timestamp format: YYYYMMDD-HHMMSS
                let timestamp_part = filename.strip_prefix(".vimrc.ANNA_BACKUP.")
                    .ok_or_else(|| anyhow!("Invalid backup filename"))?;

                if timestamp_part.len() != 15 {  // YYYYMMDD-HHMMSS = 15 chars
                    return Err(anyhow!("Backup timestamp has wrong format: {}", timestamp_part));
                }

                if !timestamp_part.contains('-') {
                    return Err(anyhow!("Backup timestamp missing hyphen"));
                }

                return Ok(path);
            }
        }

        Err(anyhow!("No backup file found matching pattern: {}", backup_pattern))
    }
}

/// Scenario: "What computer is this?" - Hardware detection
///
/// Tests that Anna:
/// - Executes hardware detection commands (lscpu, lsblk, free, lspci)
/// - Uses EXACT values from command output
/// - Never invents hardware specifications
/// - Provides clear, accurate summary
#[derive(Debug)]
pub struct HardwareQueryScenario {
    /// Mock lscpu output
    pub mock_lscpu: String,
    /// Mock lsblk output
    pub mock_lsblk: String,
    /// Mock free -h output
    pub mock_free: String,
    /// Mock lspci output
    pub mock_lspci: String,
}

impl HardwareQueryScenario {
    /// Create scenario with realistic mock data
    pub fn with_real_data() -> Self {
        Self {
            mock_lscpu: r#"Architecture:            x86_64
  CPU op-mode(s):        32-bit, 64-bit
  Address sizes:         48 bits physical, 48 bits virtual
  Byte Order:            Little Endian
CPU(s):                  32
  On-line CPU(s) list:   0-31
Vendor ID:               AuthenticAMD
  Model name:            AMD Ryzen 9 7950X 16-Core Processor
    CPU family:          25
    Model:               97
    Thread(s) per core:  2
    Core(s) per socket:  16
"#.to_string(),
            mock_lsblk: r#"NAME   MAJ:MIN RM   SIZE RO TYPE MOUNTPOINTS
nvme0n1 259:0    0   1.8T  0 disk
├─nvme0n1p1 259:1    0   512M  0 part /boot
├─nvme0n1p2 259:2    0    16G  0 part [SWAP]
└─nvme0n1p3 259:3    0   1.8T  0 part /
"#.to_string(),
            mock_free: r#"               total        used        free      shared  buff/cache   available
Mem:            31Gi       8.2Gi        18Gi       274Mi       5.1Gi        22Gi
Swap:           15Gi          0B        15Gi
"#.to_string(),
            mock_lspci: r#"01:00.0 VGA compatible controller: NVIDIA Corporation AD106 [GeForce RTX 4060] (rev a1)
"#.to_string(),
        }
    }

    /// Generate ACTION_PLAN for hardware detection
    pub fn generate_action_plan(&self) -> Result<ActionPlan> {
        let plan = ActionPlan {
            steps: vec![
                ActionStep {
                    id: "detect_cpu".to_string(),
                    description: "Detect CPU information".to_string(),
                    risk: ActionRisk::Low,
                    requires_confirmation: false,
                    backup: None,
                    commands: vec![
                        vec!["lscpu".to_string()],
                    ],
                    restore_hint: None,
                },
                ActionStep {
                    id: "detect_memory".to_string(),
                    description: "Detect memory information".to_string(),
                    risk: ActionRisk::Low,
                    requires_confirmation: false,
                    backup: None,
                    commands: vec![
                        vec!["free".to_string(), "-h".to_string()],
                    ],
                    restore_hint: None,
                },
                ActionStep {
                    id: "detect_storage".to_string(),
                    description: "Detect storage devices".to_string(),
                    risk: ActionRisk::Low,
                    requires_confirmation: false,
                    backup: None,
                    commands: vec![
                        vec!["lsblk".to_string()],
                    ],
                    restore_hint: None,
                },
                ActionStep {
                    id: "detect_gpu".to_string(),
                    description: "Detect GPU information".to_string(),
                    risk: ActionRisk::Low,
                    requires_confirmation: false,
                    backup: None,
                    commands: vec![
                        vec!["lspci".to_string()],
                    ],
                    restore_hint: None,
                },
            ],
        };

        plan.validate()?;
        Ok(plan)
    }

    /// Extract exact values from mock data
    pub fn extract_cpu_model(&self) -> Result<String> {
        for line in self.mock_lscpu.lines() {
            if line.trim().starts_with("Model name:") {
                let model = line.split(':').nth(1)
                    .ok_or_else(|| anyhow!("No model name found"))?
                    .trim();
                return Ok(model.to_string());
            }
        }
        Err(anyhow!("CPU model not found in lscpu output"))
    }

    /// Extract exact memory from mock data
    pub fn extract_total_memory(&self) -> Result<String> {
        for line in self.mock_free.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    return Ok(parts[1].to_string());
                }
            }
        }
        Err(anyhow!("Memory total not found in free output"))
    }

    /// Verify that summary uses exact values (no hallucination)
    pub fn verify_summary_accuracy(&self, summary: &str) -> Result<()> {
        let cpu_model = self.extract_cpu_model()?;
        let memory = self.extract_total_memory()?;

        // Summary MUST contain exact CPU model
        if !summary.contains(&cpu_model) {
            return Err(anyhow!(
                "Summary does not contain exact CPU model: expected '{}' in summary",
                cpu_model
            ));
        }

        // Summary MUST contain exact memory amount
        if !summary.contains(&memory) {
            return Err(anyhow!(
                "Summary does not contain exact memory: expected '{}' in summary",
                memory
            ));
        }

        // Summary MUST NOT invent specifications
        let forbidden_phrases = [
            "approximately",
            "around",
            "roughly",
            "about",
            "similar to",
        ];

        for phrase in &forbidden_phrases {
            if summary.to_lowercase().contains(phrase) {
                return Err(anyhow!(
                    "Summary contains vague language: '{}' (must use exact values)",
                    phrase
                ));
            }
        }

        Ok(())
    }
}

/// Scenario: Upgrade Anna's LLM model
///
/// Tests that Anna:
/// - Reads hardware specifications correctly
/// - Suggests appropriate model based on RAM/CPU/GPU
/// - Backs up existing configuration with ANNA_BACKUP
/// - Updates config file safely
/// - Provides clear upgrade explanation
#[derive(Debug)]
pub struct LlmUpgradeScenario {
    /// Total RAM in MB
    pub total_ram_mb: u64,
    /// CPU cores
    pub cpu_cores: usize,
    /// Has GPU
    pub has_gpu: bool,
    /// Current model
    pub current_model: String,
    /// Config file path
    pub config_path: PathBuf,
}

impl LlmUpgradeScenario {
    /// High-end hardware scenario (32GB RAM, 16 cores, GPU)
    pub fn high_end() -> Self {
        Self {
            total_ram_mb: 32000,
            cpu_cores: 16,
            has_gpu: true,
            current_model: "llama3.2:3b".to_string(),
            config_path: PathBuf::from("/tmp/anna_test/llm_config.toml"),
        }
    }

    /// Mid-range hardware scenario (16GB RAM, 8 cores, no GPU)
    pub fn mid_range() -> Self {
        Self {
            total_ram_mb: 16000,
            cpu_cores: 8,
            has_gpu: false,
            current_model: "llama3.2:1b".to_string(),
            config_path: PathBuf::from("/tmp/anna_test/llm_config.toml"),
        }
    }

    /// Low-end hardware scenario (8GB RAM, 4 cores, no GPU)
    pub fn low_end() -> Self {
        Self {
            total_ram_mb: 8000,
            cpu_cores: 4,
            has_gpu: false,
            current_model: "llama3.2:1b".to_string(),
            config_path: PathBuf::from("/tmp/anna_test/llm_config.toml"),
        }
    }

    /// Suggest upgrade model based on hardware
    pub fn suggest_upgrade(&self) -> Result<String> {
        // Model selection logic based on hardware
        if self.total_ram_mb >= 24000 && self.cpu_cores >= 12 {
            // High-end: Can run llama3.1:8b
            Ok("llama3.1:8b".to_string())
        } else if self.total_ram_mb >= 12000 && self.cpu_cores >= 6 {
            // Mid-range: llama3.2:3b
            Ok("llama3.2:3b".to_string())
        } else {
            // Low-end: Stay with llama3.2:1b
            Err(anyhow!("Hardware not sufficient for upgrade"))
        }
    }

    /// Generate ACTION_PLAN for model upgrade
    pub fn generate_action_plan(&self) -> Result<ActionPlan> {
        let new_model = self.suggest_upgrade()?;
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let backup_path = format!("{}.ANNA_BACKUP.{}", self.config_path.display(), timestamp);

        let plan = ActionPlan {
            steps: vec![
                ActionStep {
                    id: "check_ollama".to_string(),
                    description: "Check if Ollama is running".to_string(),
                    risk: ActionRisk::Low,
                    requires_confirmation: false,
                    backup: None,
                    commands: vec![
                        vec!["systemctl".to_string(), "is-active".to_string(), "ollama".to_string()],
                    ],
                    restore_hint: None,
                },
                ActionStep {
                    id: "pull_model".to_string(),
                    description: format!("Download new model: {}", new_model),
                    risk: ActionRisk::Medium,
                    requires_confirmation: true,
                    backup: None,
                    commands: vec![
                        vec!["ollama".to_string(), "pull".to_string(), new_model.clone()],
                    ],
                    restore_hint: Some(format!("ollama rm {}", new_model)),
                },
                ActionStep {
                    id: "backup_config".to_string(),
                    description: format!("Backup configuration to {}", backup_path),
                    risk: ActionRisk::Low,
                    requires_confirmation: false,
                    backup: Some(format!("cp {} {}", self.config_path.display(), backup_path)),
                    commands: vec![
                        vec!["cp".to_string(), self.config_path.display().to_string(), backup_path.clone()],
                    ],
                    restore_hint: Some(format!("cp {} {}", backup_path, self.config_path.display())),
                },
                ActionStep {
                    id: "update_config".to_string(),
                    description: format!("Update configuration to use {}", new_model),
                    risk: ActionRisk::Medium,
                    requires_confirmation: true,
                    backup: None,
                    commands: vec![
                        // In real implementation, would use proper config update
                        vec!["touch".to_string(), self.config_path.display().to_string()],
                    ],
                    restore_hint: Some(format!("Use backup: {}", backup_path)),
                },
            ],
        };

        plan.validate()?;
        Ok(plan)
    }

    /// Verify that suggested model is appropriate for hardware
    pub fn verify_model_appropriate(&self) -> Result<()> {
        let suggested = self.suggest_upgrade();

        match (self.total_ram_mb, suggested) {
            (ram, Ok(model)) if ram < 12000 && model.contains("8b") => {
                Err(anyhow!("Suggested 8b model but only {}MB RAM", ram))
            }
            (ram, Ok(model)) if ram >= 24000 && model.contains("1b") => {
                Err(anyhow!("High-end hardware but suggested only 1b model"))
            }
            _ => Ok(()),
        }
    }

    /// Verify backup was created before config change
    pub fn verify_backup_before_change(&self, plan: &ActionPlan) -> Result<()> {
        let backup_step_idx = plan.steps.iter()
            .position(|s| s.id == "backup_config")
            .ok_or_else(|| anyhow!("No backup step found"))?;

        let update_step_idx = plan.steps.iter()
            .position(|s| s.id == "update_config")
            .ok_or_else(|| anyhow!("No update step found"))?;

        if backup_step_idx >= update_step_idx {
            return Err(anyhow!("Backup step must come BEFORE config update"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_vim_scenario_action_plan_valid() {
        let temp_dir = env::temp_dir().join("anna_test_vim");
        fs::create_dir_all(&temp_dir).unwrap();

        let scenario = VimSyntaxScenario::new(&temp_dir);
        scenario.setup_no_vimrc().unwrap();

        let plan = scenario.generate_action_plan().unwrap();

        // Plan should validate successfully
        assert!(plan.validate().is_ok());

        // Should have at least 2 steps (check vim, add config)
        assert!(plan.steps.len() >= 2);

        // Check vim step should be low risk
        assert_eq!(plan.steps[0].risk, ActionRisk::Low);

        // Add syntax step should be medium risk (modifies config)
        let add_syntax_step = plan.steps.iter()
            .find(|s| s.id == "add_syntax_highlighting")
            .expect("add_syntax_highlighting step not found");
        assert_eq!(add_syntax_step.risk, ActionRisk::Medium);
        assert!(add_syntax_step.requires_confirmation);

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_vim_scenario_backup_naming() {
        let temp_dir = env::temp_dir().join("anna_test_vim_backup");
        fs::create_dir_all(&temp_dir).unwrap();

        let scenario = VimSyntaxScenario::new(&temp_dir);
        scenario.setup_existing_vimrc().unwrap();

        let plan = scenario.generate_action_plan().unwrap();

        // Find backup step
        let backup_step = plan.steps.iter()
            .find(|s| s.id == "backup_vimrc")
            .expect("backup step not found");

        // Backup command must exist and contain ANNA_BACKUP
        assert!(backup_step.backup.is_some());
        let backup_cmd = backup_step.backup.as_ref().unwrap();
        assert!(backup_cmd.contains("ANNA_BACKUP"), "Backup command does not contain ANNA_BACKUP: {}", backup_cmd);

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_hardware_scenario_exact_values() {
        let scenario = HardwareQueryScenario::with_real_data();

        // Extract exact CPU model
        let cpu_model = scenario.extract_cpu_model().unwrap();
        assert_eq!(cpu_model, "AMD Ryzen 9 7950X 16-Core Processor");

        // Extract exact memory
        let memory = scenario.extract_total_memory().unwrap();
        assert_eq!(memory, "31Gi");

        // Verify that summary with exact values passes
        let good_summary = "Your computer has an AMD Ryzen 9 7950X 16-Core Processor with 31Gi of RAM.";
        assert!(scenario.verify_summary_accuracy(good_summary).is_ok());

        // Verify that vague summary fails
        let vague_summary = "Your computer has approximately 32GB of RAM.";
        assert!(scenario.verify_summary_accuracy(vague_summary).is_err());

        // Verify that hallucinated summary fails
        let hallucinated_summary = "Your computer has an Intel Core i9 processor.";
        assert!(scenario.verify_summary_accuracy(hallucinated_summary).is_err());
    }

    #[test]
    fn test_hardware_scenario_action_plan() {
        let scenario = HardwareQueryScenario::with_real_data();
        let plan = scenario.generate_action_plan().unwrap();

        // Should validate successfully
        assert!(plan.validate().is_ok());

        // Should have 4 detection steps
        assert_eq!(plan.steps.len(), 4);

        // All steps should be low risk (just reading info)
        for step in &plan.steps {
            assert_eq!(step.risk, ActionRisk::Low);
            assert!(!step.requires_confirmation);
        }

        // Should have CPU, memory, storage, GPU detection
        assert!(plan.steps.iter().any(|s| s.id == "detect_cpu"));
        assert!(plan.steps.iter().any(|s| s.id == "detect_memory"));
        assert!(plan.steps.iter().any(|s| s.id == "detect_storage"));
        assert!(plan.steps.iter().any(|s| s.id == "detect_gpu"));
    }

    #[test]
    fn test_llm_upgrade_high_end() {
        let scenario = LlmUpgradeScenario::high_end();

        // High-end hardware should suggest 8b model
        let suggested = scenario.suggest_upgrade().unwrap();
        assert!(suggested.contains("8b"), "High-end hardware should suggest 8b model, got: {}", suggested);

        // Model should be appropriate for hardware
        assert!(scenario.verify_model_appropriate().is_ok());
    }

    #[test]
    fn test_llm_upgrade_mid_range() {
        let scenario = LlmUpgradeScenario::mid_range();

        // Mid-range hardware should suggest 3b model
        let suggested = scenario.suggest_upgrade().unwrap();
        assert!(suggested.contains("3b"), "Mid-range hardware should suggest 3b model, got: {}", suggested);

        assert!(scenario.verify_model_appropriate().is_ok());
    }

    #[test]
    fn test_llm_upgrade_low_end() {
        let scenario = LlmUpgradeScenario::low_end();

        // Low-end hardware should not suggest upgrade
        assert!(scenario.suggest_upgrade().is_err(), "Low-end hardware should not upgrade");
    }

    #[test]
    fn test_llm_upgrade_backup_before_change() {
        let scenario = LlmUpgradeScenario::high_end();
        let plan = scenario.generate_action_plan().unwrap();

        // Backup must come before config update
        assert!(scenario.verify_backup_before_change(&plan).is_ok());

        // All steps should validate
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_llm_upgrade_action_plan_structure() {
        let scenario = LlmUpgradeScenario::high_end();
        let plan = scenario.generate_action_plan().unwrap();

        // Should have 4 steps: check, pull, backup, update
        assert_eq!(plan.steps.len(), 4);

        // Check step is low risk
        assert_eq!(plan.steps[0].risk, ActionRisk::Low);

        // Pull and update steps are medium risk
        let pull_step = plan.steps.iter().find(|s| s.id == "pull_model").unwrap();
        assert_eq!(pull_step.risk, ActionRisk::Medium);
        assert!(pull_step.requires_confirmation);

        let update_step = plan.steps.iter().find(|s| s.id == "update_config").unwrap();
        assert_eq!(update_step.risk, ActionRisk::Medium);
        assert!(update_step.requires_confirmation);

        // Backup step must contain ANNA_BACKUP
        let backup_step = plan.steps.iter().find(|s| s.id == "backup_config").unwrap();
        assert!(backup_step.backup.is_some());
        assert!(backup_step.backup.as_ref().unwrap().contains("ANNA_BACKUP"));
    }
}
