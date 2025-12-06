# Anna Roadmap

## Current Focus (v0.0.103+)

**Theme**: Hollywood IT Department Experience + Learning System

Anna aims to provide a cinematic, old-school terminal experience with ASCII art styling,
named IT personas, and RPG-style progression. The learning system allows Anna to
skip the slow LLM path for queries similar to ones she's seen before.

### Active Development
- [x] Recipe learning system (v0.0.94)
- [x] Safe change engine with rollback (v0.0.95)
- [x] Desktop team flow with CLI confirmation (v0.0.96)
- [x] Change history and undo (v0.0.97)
- [x] Multi-file change transactions (v0.0.98)
- [x] Package/service recipes (v0.0.98)
- [x] Natural language package install (v0.0.99)
- [x] Natural language service management (v0.0.99)
- [x] Recipe matcher for fast path (v0.0.100)
- [x] Shell config recipes (v0.0.100)
- [x] Git config recipes (v0.0.100)
- [x] Recipe fast path integration (v0.0.101)
- [x] Recipe-based direct answers (v0.0.102)
- [x] Recipe success tracking and feedback (v0.0.103)
- [x] User feedback improves recipe confidence (v0.0.103)

---

## Completed

### v0.0.103 - Recipe Feedback System (Phase 23) ✓
- [x] `FeedbackRequest` struct - Anna asks user for feedback when uncertain
- [x] `feedback_request` field in ServiceDeskResult
- [x] Anna asks for feedback on borderline confidence (60-75) or new recipes (<3 uses)
- [x] Interactive feedback handling in REPL and one-shot modes
- [x] Feedback adjusts recipe reliability_score (+1 helpful, -5 not-helpful)
- [x] Feedback history logged to ~/.anna/feedback_history.jsonl

### v0.0.102 - Recipe Direct Answers (Phase 22) ✓
- [x] Direct answer from recipe template (skip probes too)
- [x] `build_recipe_result()` creates ServiceDeskResult from recipe
- [x] `can_answer_directly()` checks for answer template
- [x] Instant responses for learned patterns

### v0.0.101 - Recipe Fast Path Integration (Phase 21) ✓
- [x] Recipe index built at daemon startup
- [x] Recipe check BEFORE calling LLM translator
- [x] High-confidence recipes skip LLM (score >= 70)
- [x] ConfigureShell and ConfigureGit query classes
- [x] Shell/git config query routing

### v0.0.100 - Recipe Matcher & Config Recipes (Phase 20) ✓
- [x] Recipe matcher for translator fast-path
- [x] Shell configuration recipes (bash, zsh, fish)
- [x] Git configuration recipes
- [x] New RecipeKind variants (ShellConfig, GitConfig)

### v0.0.99 - Natural Language Package & Service Management (Phase 19) ✓
- [x] Package install via natural language ("install htop")
- [x] Service management via natural language ("restart docker")
- [x] Cross-distro package name mapping
- [x] Protected service detection
- [x] QueryClass: InstallPackage, ManageService

### v0.0.98 - Multi-file Transactions & Recipe Systems (Phase 18) ✓
- [x] ChangeTransaction for atomic multi-file changes
- [x] Automatic rollback on failure
- [x] Package recipes with multi-manager support (pacman, apt, dnf, flatpak, snap)
- [x] Service recipes with risk levels and protected services
- [x] Cross-distro package name mapping

### v0.0.97 - Change History and Undo (Phase 17) ✓
- [x] Change history tracking in ~/.anna/change_history.jsonl
- [x] `annactl history` command
- [x] `annactl undo <id>` command
- [x] Backup-based restoration

### v0.0.96 - Desktop Team Editor Config Flow (Phase 16) ✓
- [x] Natural language editor configuration ("enable syntax highlighting")
- [x] proposed_change field in ServiceDeskResult
- [x] CLI confirmation flow for config changes
- [x] Integration with Safe Change Engine

### v0.0.95 - Safe Change Engine (Phase 15) ✓
- [x] PlanChange, ApplyChange, RollbackChange RPC methods
- [x] Backup-first, idempotent config modifications
- [x] Extracted editor_recipe_data.rs module
- [x] All files under 400 lines

### v0.0.94 - Recipe Learning System (Phase 14) ✓
- [x] Automatic recipe learning from successful queries
- [x] Learning criteria: verified=true, reliability >= 80
- [x] Recipe persistence in ~/.anna/recipes/
- [x] Team assignment from domain

### v0.0.93 - Documentation Update (Phase 13) ✓
- [x] Updated README, ROADMAP, FEATURES for current version
- [x] Hollywood IT aesthetic documentation

### v0.0.92 - Codebase Hygiene (Phase 12) ✓
- [x] Zero compiler warnings across entire workspace
- [x] Fixed unused methods, variables, and imports
- [x] Applied cargo fix to test files

### v0.0.91 - ASCII-Style Achievement Badges (Phase 11) ✓
- [x] Replaced emoji badges with ASCII art symbols
- [x] Badge styles: `[1]` `<3d>` `(90+)` `{*}` `~00~` `|7d|`
- [x] Hollywood IT aesthetic consistency

### v0.0.90 - Achievement Badges (Phase 10) ✓
- [x] 22 unique achievements across 6 categories
- [x] Milestones, Streaks, Quality, Teams, Special, Tenure
- [x] Integration with stats display

### v0.0.89 - Personalized Greetings (Phase 9) ✓
- [x] Time-of-day awareness (Morning, Afternoon, Evening, Night)
- [x] User personalization from $USER
- [x] Domain-specific follow-up prompts
- [x] New greetings.rs module

### v0.0.88 - Warning Cleanup (Phase 8) ✓
- [x] Removed all compiler warnings
- [x] Fixed unused imports across workspace

### v0.0.87 - Dialogue Variety (Phase 7) ✓
- [x] Varied junior approval phrases
- [x] Varied escalation requests
- [x] Varied senior responses
- [x] Seed-based deterministic variety

### v0.0.81-86 - Service Desk Theatre ✓
- [x] Named IT personas with roles
- [x] Cinematic narrative rendering
- [x] Internal communications mode (-i flag)
- [x] Streak tracking and XP system

### v0.0.75 - RPG Stats System ✓
- [x] Event logging with JSONL store
- [x] XP calculation and level progression
- [x] Titles from Trainee to Principal Engineer
- [x] Stats display with progress bars

### v0.0.71 - Version Truth ✓
- [x] Single source of truth: workspace Cargo.toml version only
- [x] Unified version display: annactl/annad --version format consistent
- [x] Status shows: installed (annactl), daemon_ver (annad), available, last_check, next_check, auto_update
- [x] Hard gate tests: CI fails if annactl/annad version != workspace version
- [x] No hardcoded version strings in tests (compare against VERSION constant)
- [x] Auto-update semantic comparison with no-downgrade guarantee

### v0.0.70 - Version Unification + Release Hygiene ✓
- [x] Single source of truth: workspace Cargo.toml version is authoritative
- [x] All crates use version.workspace = true
- [x] anna_shared::VERSION uses env!("CARGO_PKG_VERSION")
- [x] install.sh fetches version from GitHub releases API (no hardcoding)
- [x] Version consistency tests validate all sources
- [x] Status output shows: installed, available, last_check, next_check, auto_update
- [x] Auto-update uses semantic version comparison (no string comparison)
- [x] No downgrade guarantee: newer installed version is never replaced

### v0.0.69 - Unified Versioning + REPL Enhancements ✓
- [x] Single source of truth for version (workspace Cargo.toml)
- [x] REPL "since last time" summary with snapshot comparison
- [x] Delta tracking for failed services, disk, memory changes
- [x] Version consistency tests
- [x] Documentation updates (CHANGELOG, FEATURES, README)

### v0.0.68 - Audio Parse Correctness + ConfigureEditor Grounding ✓
- [x] Audio deterministic answer handles "Multimedia audio controller"
- [x] ConfigureEditor uses full router probe list (skip spine override)
- [x] Clarification prompts end with period, not question mark

### v0.0.67 - Service Desk Theatre UX ✓
- [x] Service desk narrative renderer (render.rs)
- [x] REPL narrative header with boot status, critical issues
- [x] Stats RPG system with XP calculation
- [x] Local citations system (citations.rs)

### v0.0.66 - Version Normalization + Regressions ✓
- [x] Version consolidation across all sources
- [x] Audio evidence parsing for lspci PCI class codes
- [x] ConfigureEditor numbered options without question marks

### v0.0.63 - Service Desk Theatre Renderer ✓
- [x] Narrative flow in normal mode ("Checking X...")
- [x] Evidence source in footer when grounded
- [x] Clarification options numbered display
- [x] New transcript events (EvidenceSummary, DeterministicPath, ProposedAction, ActionConfirmationRequest)
- [x] Debug mode rendering for all new events

### v0.0.62 - ConfigureEditor Grounding ✓
- [x] Proper probe accounting with valid_evidence_count
- [x] Execution trace for all ConfigureEditor paths
- [x] Grounding signals based on valid evidence

### v0.0.61 - HardwareAudio Parser ✓
- [x] Content-based audio detection (not just command pattern)
- [x] pactl detection by "Card #" blocks
- [x] Evidence merge from lspci + pactl

### v0.0.45 - Query Classification & Probe Planning ✓
- [x] New QueryClass variants: InstalledToolCheck, HardwareAudio, CpuTemp, CpuCores, PackageCount, MemoryFree
- [x] Modularized router.rs + query_classify.rs
- [x] Stabilization golden tests
- [x] ReliabilityInput builder methods

### v0.0.26 - Team-Scoped Review System ✓
- [x] SPECIALISTS Registry: Team-scoped roles (Translator, Junior, Senior)
- [x] 8 Teams: Desktop, Storage, Network, Performance, Services, Security, Hardware, General
- [x] Deterministic Review Gate: Hybrid logic that minimizes LLM calls
- [x] Team-specific junior/senior review prompts
- [x] Review gate transcript events
- [x] Trace enhancements (ReviewerOutcome, FallbackUsed::Timeout)

### v0.0.23 - TRACE + TRUST+ + RESCUE ✓
- [x] Execution trace for debugging degraded paths
- [x] Enhanced reliability explanations
- [x] Explicit threshold constants for scoring

### v0.0.18 and earlier ✓
- [x] Core pipeline with grounded responses
- [x] Deterministic probe routing
- [x] Auto-update mechanism
- [x] Per-stage latency tracking
- [x] Hardware-aware model selection

## Planned

### v0.0.104+ - Future
- [ ] SSH key management recipes
- [ ] Systemd unit file recipes
- [ ] Cron job recipes
- [ ] Docker compose recipes
