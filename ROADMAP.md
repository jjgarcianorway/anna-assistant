# Anna Roadmap

## 0.45.x: Stabilization (ACTIVE - No new features until complete)

**Focus**: Truthfulness, UX, probe spine, timeouts, learning loop

### Non-Negotiables
1. **LLM-first reasoning**: Deterministic code selects tools and enforces safety, but does NOT invent answers
2. **Probe spine**: Small deterministic mapping from evidence kinds to fast probes - no `probes=[]` for evidence-required queries
3. **No evidence, no specific claims**: Cannot verify = cannot state as fact
4. **Learning loop**: Recipes only persist on successful verified outcomes
5. **UX consistency**: Stable transcript formatting, correct speaker labels, no username/Anna confusion

### Checklist
- [ ] Probe spine: enforce minimum probes when evidence required
- [ ] Gate deterministic answers: only for explicitly allowed route capabilities
- [ ] Timeout behavior: evidence summary not generic rephrase
- [ ] Clarifications: installed-only options with escape hatches
- [ ] Recipe commit: only on verified success
- [ ] UX labels: [you], [anna], internal [translator], [probes], etc.
- [ ] Snapshot tests for all UX regression scenarios

### Test Scenarios (must pass before exiting stabilization)
1. `how is my computer doing?` → relevant issues only
2. `do I have nano?` → probes run, grounded yes/no
3. `what is my sound card?` → probes run, grounded answer
4. `how many cores has my cpu?` → lscpu probe, grounded answer
5. `what temperature has my cpu?` → sensors probe, NOT cpu model

---

## Completed

### v0.0.71 - Version Truth (Pure Hygiene) ✓
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

## On Hold (until 0.45.x stabilization complete)

### v0.0.27 - Recipes + Safe Change Engine
- [ ] Recipe Learning Loop: Structured, team-tagged recipes
- [ ] Safe Change Engine: Backup, apply, rollback for config edits
- [ ] Desktop team flow: "enable syntax highlighting in vim"
- [ ] Status/Stats UX: Team roster, per-team statistics
- [ ] User confirmation for system changes

## Planned

### v0.0.28+ - Future
- [ ] Recipe persistence and replay
- [ ] Multi-file change transactions
- [ ] Package installation recipes
- [ ] Service configuration recipes
- [ ] Undo history viewer in CLI
