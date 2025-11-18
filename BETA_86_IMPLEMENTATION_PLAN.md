# Anna Beta.86 Implementation Plan
**Phase 2: Real-World Behavior, LLM Quality, UX & Telemetry**

**Date:** November 18, 2025
**Target Version:** 5.7.0-beta.86
**Build On:** beta.85 (file indexing + professional prompt)

---

## Executive Summary

Beta.86 focuses on **production-grade response quality** and **intelligent LLM management**. This phase implements:

1. **Perfect Answer Quality** - Structured, validated, zero-hallucination responses
2. **16-Trait Personality System** - Stored in database, fully customizable
3. **Smart LLM Selection** - Hardware-aware, auto-upgrade, no toy models
4. **Enhanced Telemetry** - Complete system awareness
5. **Answer Validation Pipeline** - Multi-pass LLM → validation → retry loop
6. **File Backup System** - Every modification tracked and reversible
7. **Beautiful TUI** - Clean sections, proper formatting
8. **Comprehensive Documentation** - All features documented

---

## Architecture Overview

### Current State (Beta.85)
```
✅ 36 database tables
✅ File-level indexing (file_index, file_changes)
✅ 200+ line professional prompt (4 critical sections)
✅ 100-question validation suite
✅ Auto-update system
✅ Basic 8-trait personality (TOML-based)
✅ Model profiles (model_profiles.rs)
```

### Target State (Beta.86)
```
🎯 16-trait personality system (database-backed)
🎯 Answer validation pipeline (hallucination prevention)
🎯 Smart LLM selection (hardware-aware + upgrade)
🎯 Structured answer format enforcement
🎯 File backup system with restore instructions
🎯 Enhanced telemetry (hardware, software, user behavior, problems)
🎯 Beautiful TUI with sections
🎯 Complete documentation
```

---

## Implementation Tasks

### 1. 16-Trait Personality System ⭐ HIGH PRIORITY

**Current Status:**
- `personality.rs` has 8 traits
- Saves to TOML files (~/.config/anna/personality.toml)
- Not stored in database

**Changes Required:**

#### 1.1 Expand to 16 Traits
**File:** `crates/anna_common/src/personality.rs`

Add 8 more traits to complete the 16-trait model:
```rust
// NEW TRAITS (add to default_traits())
PersonalityTrait::new("patient_vs_urgent", "Patient vs Urgent", 7),
PersonalityTrait::new("humble_vs_confident", "Humble vs Confident", 6),
PersonalityTrait::new("formal_vs_casual", "Formal vs Casual", 5),
PersonalityTrait::new("empathetic_vs_logical", "Empathetic vs Logical", 7),
PersonalityTrait::new("protective_vs_empowering", "Protective vs Empowering", 6),
PersonalityTrait::new("traditional_vs_innovative", "Traditional vs Innovative", 5),
PersonalityTrait::new("collaborative_vs_independent", "Collaborative vs Independent", 6),
PersonalityTrait::new("perfectionist_vs_pragmatic", "Perfectionist vs Pragmatic", 6),
```

#### 1.2 Add Trait Meanings
Extend `compute_meaning()` function with meaning ranges for all 16 traits.

#### 1.3 Add Trait Interaction Constraints
Create new function:
```rust
impl PersonalityConfig {
    /// Validate trait interactions (some traits conflict)
    pub fn validate_interactions(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Example: Can't be both shy (introvert 10) and bold (bold 0)
        if let (Some(intro), Some(bold)) = (
            self.get_trait("introvert_vs_extrovert"),
            self.get_trait("cautious_vs_bold")
        ) {
            if intro.value >= 8 && bold.value <= 2 {
                errors.push("Conflicting traits: Very introverted but very bold".to_string());
            }
        }

        // Add more interaction rules...

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

#### 1.4 Add Database Storage
**File:** `crates/anna_common/src/context/db.rs`

Add new table in schema:
```sql
CREATE TABLE IF NOT EXISTS personality (
    id INTEGER PRIMARY KEY,
    trait_key TEXT NOT NULL UNIQUE,
    trait_name TEXT NOT NULL,
    value INTEGER NOT NULL CHECK (value >= 0 AND value <= 10),
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_personality_key ON personality(trait_key);
```

#### 1.5 Add Database Methods
**File:** `crates/anna_common/src/personality.rs`

```rust
impl PersonalityConfig {
    /// Load from database instead of TOML
    pub async fn load_from_db(db: &ContextDb) -> Result<Self> {
        // Query all traits from personality table
        // If empty, initialize with defaults
        // Return PersonalityConfig
    }

    /// Save to database
    pub async fn save_to_db(&self, db: &ContextDb) -> Result<()> {
        // Insert or update all traits in personality table
    }

    /// Migrate from TOML to database (one-time)
    pub async fn migrate_from_toml(db: &ContextDb) -> Result<()> {
        // Read old TOML file
        // Save to database
        // Optionally remove TOML file
    }
}
```

#### 1.6 Add CLI Commands
**File:** `crates/annactl/src/main.rs` (or new personality command module)

```bash
annactl personality show              # Display all traits with radar chart
annactl personality set <trait> <value>  # Set specific trait
annactl personality adjust <trait> <delta>  # Adjust by delta
annactl personality reset              # Reset to defaults
annactl personality export             # Export to TOML for backup
```

#### 1.7 Add Radar Chart Visualization
Create ASCII radar chart for trait display (or use plotters crate for terminal graphics).

---

### 2. Smart LLM Selection & Upgrade ⭐ HIGH PRIORITY

**Current Status:**
- `model_profiles.rs` has QualityTier enum (Tiny, Small, Medium, Large)
- Basic ModelProfile struct
- No auto-selection based on hardware

**Changes Required:**

#### 2.1 Hardware Detection
**File:** `crates/anna_common/src/hardware_capability.rs`

Enhance hardware detection:
```rust
pub struct HardwareProfile {
    pub ram_gb: f64,
    pub cpu_cores: usize,
    pub cpu_flags: Vec<String>,  // AVX2, AVX512, etc.
    pub gpu_available: bool,
    pub gpu_vram_gb: Option<f64>,
    pub tdp_watts: Option<u32>,
}

impl HardwareProfile {
    pub fn detect() -> Result<Self> {
        // Read /proc/cpuinfo for cores and flags
        // Read /proc/meminfo for RAM
        // Run lspci to detect GPU
        // Estimate TDP from CPU model
    }
}
```

#### 2.2 Model Catalog with Upgrade Paths
**File:** `crates/annactl/src/model_catalog.rs`

```rust
pub struct ModelCatalog {
    profiles: Vec<ModelProfile>,
}

impl ModelCatalog {
    pub fn load_available_models() -> Result<Self> {
        // Query Ollama for installed models
        // Load profiles for known models
    }

    pub fn recommend_model(&self, hardware: &HardwareProfile) -> Option<ModelProfile> {
        // Filter by hardware requirements
        // Reject models with QualityTier::Tiny if hardware allows better
        // Return best suitable model
    }

    pub fn find_upgrade(&self, current: &str, hardware: &HardwareProfile) -> Option<ModelProfile> {
        // Find better model than current
        // Check hardware compatibility
        // Return upgrade suggestion
    }
}
```

#### 2.3 Reject Toy Models
Add validation:
```rust
impl ModelProfile {
    pub fn is_production_grade(&self) -> bool {
        // Reject 1B and 3B models
        match self.quality_tier {
            QualityTier::Tiny | QualityTier::Small if self.model_name.contains("1b") => false,
            QualityTier::Tiny | QualityTier::Small if self.model_name.contains("3b") => false,
            _ => true,
        }
    }
}
```

#### 2.4 Store Current Model in Database
**File:** `crates/anna_common/src/context/db.rs`

Add to `llm_model_changes` table or create new `llm_current_model` table:
```sql
CREATE TABLE IF NOT EXISTS llm_current_model (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Only one row
    model_id TEXT NOT NULL,
    model_name TEXT NOT NULL,
    quality_tier TEXT NOT NULL,
    selected_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    auto_selected BOOLEAN DEFAULT FALSE
);
```

#### 2.5 CLI Commands for Model Management
```bash
annactl model show                 # Show current model
annactl model list                 # List all available models
annactl model recommend            # Show recommended model for hardware
annactl model upgrade              # Upgrade to better model
annactl model install <model>      # Install specific model via Ollama
annactl model switch <model>       # Switch to installed model
```

---

### 3. Answer Validation Pipeline ⭐ CRITICAL

**Current Status:**
- `llm_integration.rs` sends prompts to LLM
- No validation of LLM responses
- No retry mechanism

**Changes Required:**

#### 3.1 Create Validation Module
**File:** `crates/annactl/src/answer_validator.rs` (NEW)

```rust
pub struct AnswerValidator {
    telemetry: SystemTelemetry,
}

pub enum ValidationResult {
    Valid,
    Hallucination { reason: String },
    InvalidCommand { command: String, reason: String },
    MissingTelemetry { field: String },
}

impl AnswerValidator {
    pub fn validate(&self, answer: &str) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // Check 1: No "I think", "maybe", "probably"
        if self.contains_uncertain_language(answer) {
            results.push(ValidationResult::Hallucination {
                reason: "Contains uncertain language".to_string(),
            });
        }

        // Check 2: All mentioned system state exists in telemetry
        if let Some(missing) = self.check_telemetry_references(answer) {
            results.push(ValidationResult::MissingTelemetry { field: missing });
        }

        // Check 3: All commands are valid
        if let Some(invalid_cmd) = self.validate_commands(answer) {
            results.push(ValidationResult::InvalidCommand {
                command: invalid_cmd,
                reason: "Command does not exist".to_string(),
            });
        }

        // Check 4: Follows structured format [SUMMARY] [VALIDATION] [COMMAND PLAN] [BACKUP] [CITATION]
        if !self.has_required_sections(answer) {
            results.push(ValidationResult::Hallucination {
                reason: "Missing required answer sections".to_string(),
            });
        }

        results
    }

    fn contains_uncertain_language(&self, text: &str) -> bool {
        let forbidden = ["i think", "maybe", "probably", "might be", "could be"];
        let lower = text.to_lowercase();
        forbidden.iter().any(|phrase| lower.contains(phrase))
    }

    fn check_telemetry_references(&self, text: &str) -> Option<String> {
        // Parse answer for system state claims
        // Verify each claim against telemetry
        // Return first missing field
        None
    }

    fn validate_commands(&self, text: &str) -> Option<String> {
        // Extract all command references
        // Check if commands exist in $PATH
        // Return first invalid command
        None
    }

    fn has_required_sections(&self, text: &str) -> bool {
        text.contains("[SUMMARY]")
            && text.contains("[VALIDATION]")
            && text.contains("[COMMAND PLAN]")
            && text.contains("[BACKUP]")
            && text.contains("[CITATION]")
    }
}
```

#### 3.2 Multi-Pass LLM Loop
**File:** `crates/annactl/src/llm_integration.rs`

```rust
pub async fn query_with_validation(
    question: &str,
    telemetry: &SystemTelemetry,
    max_attempts: usize,
) -> Result<String> {
    let validator = AnswerValidator::new(telemetry.clone());

    for attempt in 1..=max_attempts {
        // Generate answer from LLM
        let answer = query_llm(question, telemetry).await?;

        // Validate answer
        let validation_results = validator.validate(&answer);

        if validation_results.is_empty() {
            // Answer is valid!
            return Ok(answer);
        }

        // Answer has issues - prepare correction prompt
        let correction_prompt = format!(
            "Your previous answer had these issues:\n{}\n\nPlease provide a corrected answer that:\n- Does not use uncertain language\n- Only references system state from telemetry\n- Uses valid commands only\n- Follows [SUMMARY] [VALIDATION] [COMMAND PLAN] [BACKUP] [CITATION] structure",
            validation_results.iter()
                .map(|r| format!("- {}", r))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Try again with correction instructions
        // (loop continues)
    }

    Err(anyhow::anyhow!("Failed to generate valid answer after {} attempts", max_attempts))
}
```

#### 3.3 Debug Mode for Pipeline Visibility
Add `--debug-llm` flag to annactl:
```bash
annactl --debug-llm "How do I check my GPU?"
```

Output shows:
```
[DEBUG] Attempt 1: Querying LLM...
[DEBUG] Validation: FAILED (uncertain language: "I think")
[DEBUG] Attempt 2: Querying LLM with corrections...
[DEBUG] Validation: PASSED
[RESPONSE] ...
```

---

### 4. Structured Answer Format Enforcement

**Changes Required:**

#### 4.1 Update INTERNAL_PROMPT.md
**File:** `INTERNAL_PROMPT.md`

Add new section:
```markdown
## ANNA_ANSWER_STRUCTURE

REQUIRED ANSWER FORMAT:

[SUMMARY]
Brief 1-2 sentence explanation of what needs to be done and why.

[VALIDATION]
Explicit statement of what telemetry Anna HAS about this situation:
- "I can see your GPU is: <model>"
- "I can see your service status is: <status>"
- "I do NOT have data about <X> yet"

If critical data is missing, stop here and say:
"I do not have enough telemetry to answer that yet. I need to collect <X>."

[COMMAND PLAN]
Step-by-step commands in correct execution order:
1. <command> - <purpose>
2. <command> - <purpose>
NEVER invent commands. ONLY use commands that exist.

[BACKUP]
Before executing any changes:
1. <backup command>
2. <verification command>
Restore procedure: <steps>

[CITATION]
Arch Wiki references:
- <command>: https://wiki.archlinux.org/title/<Page>
- <concept>: https://wiki.archlinux.org/title/<Page>
```

#### 4.2 Update Runtime Prompt Builder
**File:** `crates/annactl/src/runtime_prompt.rs`

Include ANNA_ANSWER_STRUCTURE in every prompt sent to LLM.

---

### 5. File Backup System

**Changes Required:**

#### 5.1 Create Backup Module
**File:** `crates/anna_common/src/file_backup.rs` (enhance existing or create new)

```rust
pub struct FileBackupManager {
    backup_dir: PathBuf,  // ~/.local/share/anna/backups or /var/lib/anna/backups
}

impl FileBackupManager {
    pub fn new() -> Result<Self> {
        let backup_dir = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local/share/anna/backups")
        } else {
            PathBuf::from("/var/lib/anna/backups")
        };

        std::fs::create_dir_all(&backup_dir)?;
        Ok(Self { backup_dir })
    }

    pub fn backup_file(&self, file_path: &Path) -> Result<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let filename = file_path.file_name().unwrap().to_str().unwrap();
        let backup_name = format!("{}.ANNA_BACKUP.{}", filename, timestamp);
        let backup_path = self.backup_dir.join(backup_name);

        std::fs::copy(file_path, &backup_path)?;
        Ok(backup_path)
    }

    pub fn add_anna_header(&self, file_path: &Path, description: &str) -> Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        let header = format!(
            "# Modified by Anna Assistant on {}\n# {}\n# Backup: {}\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            description,
            self.get_latest_backup(file_path)?.display()
        );

        let new_content = format!("{}{}", header, content);
        std::fs::write(file_path, new_content)?;
        Ok(())
    }

    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let mut backups = Vec::new();
        for entry in std::fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            if entry.path().to_str().unwrap().contains(".ANNA_BACKUP.") {
                backups.push(entry.path());
            }
        }
        backups.sort();
        Ok(backups)
    }
}
```

#### 5.2 Add CLI Command
```bash
annactl backup list              # List all backups
annactl backup restore <file>    # Restore specific backup
```

---

### 6. Enhanced Telemetry

**Current Status:**
- Beta.85 has file_index, file_changes
- Historian tracks CPU, memory, boot, services
- Missing: user behavior, recent problems

**Changes Required:**

#### 6.1 Add User Behavior Tracking
**File:** `crates/anna_common/src/user_behavior.rs` (enhance existing)

Track:
- Most used commands
- Common failure patterns
- Time-of-day preferences
- Ignored warnings

#### 6.2 Add Problem Detection
**File:** `crates/anna_common/src/problem_detector.rs` (NEW)

```rust
pub struct ProblemDetector;

impl ProblemDetector {
    pub fn detect_recent_problems() -> Vec<Problem> {
        let mut problems = Vec::new();

        // Check systemd failed units
        // Check journal errors (last 1 hour)
        // Check OOM events
        // Check disk space issues
        // Check network connectivity

        problems
    }
}

pub struct Problem {
    pub category: String,  // "systemd", "journal", "disk", "network"
    pub severity: u8,      // 1-10
    pub description: String,
    pub detected_at: DateTime<Utc>,
}
```

#### 6.3 Include in Telemetry Context
**File:** `crates/annactl/src/runtime_prompt.rs`

Add to telemetry section sent to LLM:
```rust
[RECENT_PROBLEMS]
- [systemd] Unit foo.service failed (severity: 8)
- [disk] /var at 95% capacity (severity: 7)
[/RECENT_PROBLEMS]
```

---

### 7. TUI Upgrade

**Changes Required:**

**File:** `crates/annactl/src/tui/*.rs` (various TUI modules)

- Add consistent section headers with color
- Align important values
- Use fenced code blocks for commands
- Add optional emoji mode toggle
- Remove text walls, use bullet points
- Add personality trait visualization

---

### 8. Documentation Updates

**Files to Update:**
1. `README.md` - Add beta.86 features
2. `ARCHITECTURE.md` - Document new modules
3. `INTERNAL_PROMPT.md` - Add ANNA_ANSWER_STRUCTURE section
4. `ANNA_COMPLETE_FEATURES.md` - Add 16-trait personality, validation pipeline
5. `TESTING_GUIDE.md` - Add new test scenarios
6. `WHATS_NEW_BETA_86.md` (NEW) - Feature highlights

---

### 9. Testing

**New Tests Required:**

#### 9.1 Personality Tests
**File:** `crates/anna_common/src/personality.rs` (extend tests)

```rust
#[test]
fn test_16_traits_present() {
    let config = PersonalityConfig::default();
    assert_eq!(config.traits.len(), 16);
}

#[test]
fn test_trait_interaction_validation() {
    let mut config = PersonalityConfig::default();
    // Set conflicting traits
    config.set_trait("introvert_vs_extrovert", 10).unwrap();
    config.set_trait("cautious_vs_bold", 0).unwrap();

    let result = config.validate_interactions();
    assert!(result.is_err());
}

#[test]
fn test_database_persistence() {
    // Test save and load from database
}
```

#### 9.2 Model Selection Tests
**File:** `crates/annactl/src/model_catalog.rs` (add tests)

```rust
#[test]
fn test_reject_toy_models() {
    let hardware = HardwareProfile {
        ram_gb: 16.0,
        cpu_cores: 8,
        ...
    };

    let catalog = ModelCatalog::load_available_models().unwrap();
    let recommended = catalog.recommend_model(&hardware).unwrap();

    assert!(recommended.is_production_grade());
    assert!(recommended.quality_tier >= QualityTier::Medium);
}
```

#### 9.3 Validation Pipeline Tests
**File:** `crates/annactl/src/answer_validator.rs` (add tests)

```rust
#[test]
fn test_detect_uncertain_language() {
    let validator = AnswerValidator::new(mock_telemetry());
    let answer = "I think you should run pacman -Syu";
    let results = validator.validate(answer);

    assert!(!results.is_empty());
    assert!(matches!(results[0], ValidationResult::Hallucination { .. }));
}

#[test]
fn test_detect_missing_sections() {
    let validator = AnswerValidator::new(mock_telemetry());
    let answer = "Just run this command"; // No [SUMMARY], etc.
    let results = validator.validate(answer);

    assert!(!results.is_empty());
}
```

---

## Version Updates

**Files to Modify:**

1. `Cargo.toml`
```toml
[package]
version = "5.7.0-beta.86"
```

2. `crates/anna_common/Cargo.toml`
```toml
[package]
version = "5.7.0-beta.86"
```

3. `crates/annactl/Cargo.toml`
```toml
[package]
version = "5.7.0-beta.86"
```

4. `crates/annad/Cargo.toml`
```toml
[package]
version = "5.7.0-beta.86"
```

---

## CHANGELOG Entry

**File:** `CHANGELOG.md`

```markdown
## [5.7.0-beta.86] - 2025-11-18

### Added
- **16-Trait Personality System**: Complete personality model stored in database
  - 16 configurable traits (0-10 scale)
  - Trait interaction validation
  - Radar chart visualization
  - CLI commands: `annactl personality show/set/adjust/reset`

- **Smart LLM Selection**: Hardware-aware model recommendations
  - Auto-detect system capabilities (RAM, CPU, GPU)
  - Recommend best suitable model
  - Reject toy models (1B, 3B) when hardware allows better
  - Upgrade suggestions
  - CLI commands: `annactl model show/list/recommend/upgrade/switch`

- **Answer Validation Pipeline**: Zero-hallucination guarantee
  - Multi-pass LLM → validation → retry loop
  - Detects uncertain language ("I think", "maybe")
  - Validates telemetry references
  - Validates command existence
  - Enforces structured answer format
  - Debug mode: `--debug-llm` for pipeline visibility

- **Structured Answer Format**: Consistent, professional responses
  - [SUMMARY]: Brief explanation
  - [VALIDATION]: Telemetry confirmation
  - [COMMAND PLAN]: Step-by-step commands
  - [BACKUP]: Restore instructions
  - [CITATION]: Arch Wiki links

- **File Backup System**: Every modification tracked and reversible
  - Automatic backups: `file.ANNA_BACKUP.YYYYMMDD-HHMMSS`
  - Anna header comments in modified files
  - CLI commands: `annactl backup list/restore`

- **Enhanced Telemetry**: Complete system awareness
  - User behavior tracking (command frequency, failures, preferences)
  - Problem detection (systemd failures, journal errors, OOM, disk space)
  - Recent changes tracking (/etc, /var, /home)

- **TUI Improvements**: Beautiful, clean interface
  - Consistent section headers with colors
  - Aligned values
  - Fenced code blocks for commands
  - Optional emoji mode

### Changed
- Personality system migrated from TOML files to database
- LLM integration now includes validation loop
- Runtime prompt includes ANNA_ANSWER_STRUCTURE section
- Telemetry context includes recent problems

### Fixed
- Uncertain language in LLM responses eliminated
- Command hallucinations prevented
- Missing telemetry explicitly surfaced

### Tests Added
- 16-trait personality tests
- Trait interaction validation tests
- Model selection tests (reject toy models)
- Answer validation pipeline tests
- Database persistence tests
```

---

## Build & Release Process

### 1. Build
```bash
cargo build --release
```

Expected: Clean build, 0 errors

### 2. Test
```bash
cargo test --workspace
```

Expected: All tests pass

### 3. Validate
```bash
./scripts/validate_post_install_qa.sh
```

Expected: Success rate ≥ 85%

### 4. Commit
```bash
git add -A
git commit -m "🚀 RELEASE: Beta.86 - LLM Quality & Personality System

Major Features:
- 16-trait personality system (database-backed)
- Smart LLM selection (hardware-aware, no toy models)
- Answer validation pipeline (zero hallucinations)
- Structured answer format enforcement
- File backup system with restore
- Enhanced telemetry (problems, user behavior)
- Beautiful TUI improvements

Code Changes:
- personality.rs: 16 traits + database storage
- model_catalog.rs: Smart selection + upgrade
- answer_validator.rs: Validation pipeline
- file_backup.rs: Automatic backups
- runtime_prompt.rs: ANNA_ANSWER_STRUCTURE
- db.rs: New personality table

Documentation:
- BETA_86_IMPLEMENTATION_PLAN.md
- WHATS_NEW_BETA_86.md
- Updated ARCHITECTURE.md
- Updated README.md
- CHANGELOG.md entry

Tests:
- Personality system tests
- Model selection tests
- Answer validation tests
- All tests passing

Version: 5.7.0-beta.86
Status: PRODUCTION READY"
```

### 5. Tag
```bash
git tag -a v5.7.0-beta.86 -m "Beta.86: LLM Quality & Personality System"
```

### 6. Release
```bash
gh release create v5.7.0-beta.86 \
  --title "v5.7.0-beta.86 - LLM Quality & Personality System" \
  --notes-file WHATS_NEW_BETA_86.md \
  --latest \
  ~/anna-assistant/target/release/annactl \
  ~/anna-assistant/target/release/annad
```

### 7. Push
```bash
git push origin main
git push origin v5.7.0-beta.86
```

---

## Success Metrics

**Must Achieve:**
- ✅ Build: 0 errors, 0 critical warnings
- ✅ Tests: 100% pass rate
- ✅ Validation: ≥85% success rate on post-install questions
- ✅ Documentation: All features documented
- ✅ No hallucinations in LLM responses (validated by pipeline)

**Deployment:**
- ✅ Binaries uploaded to GitHub release
- ✅ Auto-update will distribute to users within 10 minutes

---

## Timeline Estimate

**Phase 1: Core Implementation** (6-8 hours)
- Personality system expansion
- LLM selection logic
- Answer validation pipeline
- Database schema updates

**Phase 2: Integration** (3-4 hours)
- Telemetry enhancement
- File backup system
- TUI improvements
- CLI commands

**Phase 3: Testing & Documentation** (3-4 hours)
- Unit tests
- Integration tests
- Documentation updates
- CHANGELOG

**Phase 4: Build & Release** (1-2 hours)
- Build validation
- Test suite run
- GitHub release
- Push (when GitHub recovers)

**Total: 13-18 hours** (can be split across multiple sessions)

---

## Notes

- This plan builds directly on beta.85 foundation
- All existing features remain intact
- No breaking changes to user experience
- Focus on production-grade quality
- Zero-hallucination guarantee is the key differentiator

**Status:** Ready to implement
**Next Step:** Begin with Task 1 (16-Trait Personality System)

---

**Document Version:** 1.0
**Date:** November 18, 2025
**Author:** Claude (Anna Development Team)
