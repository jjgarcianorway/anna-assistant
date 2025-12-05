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
