# Anna Assistant - Implementation Roadmap

**Current Version: 0.0.3**

This roadmap migrates from the v7.42.5 snapshot-based architecture to the full natural language assistant while preserving performance.

---

## Phase 1: CLI Surface Lockdown (0.0.x)

### 0.0.2 - Strict CLI Surface (COMPLETED)
- [x] Remove `sw` command from public surface
- [x] Remove `hw` command from public surface
- [x] Remove all JSON flags from public surface
- [x] Keep only: `annactl`, `annactl <request>`, `annactl status`, `annactl --version`
- [x] Legacy commands route as natural language requests (no custom error)
- [x] REPL mode basic implementation (exit, quit, help, status)
- [x] CLI tests for new surface

### 0.0.3 - Request Pipeline Skeleton (COMPLETED)
- [x] Create DialogueActor enum (You, Anna, Translator, Junior, Annad)
- [x] Full multi-party dialogue transcript
- [x] Deterministic Translator mock (intent classification: question, system_query, action_request, unknown)
- [x] Target detection (cpu, memory, disk, docker, etc.)
- [x] Risk classification (read-only, low-risk, medium-risk, high-risk)
- [x] Evidence retrieval mock from snapshots
- [x] Junior scoring rubric (+40 evidence, +30 confident, +20 observational+cited, +10 read-only)
- [x] CLI tests for pipeline behavior

### 0.0.4 - REPL Enhancement (Planned)
- [ ] Add `annactl reset` command (stub)
- [ ] Add `annactl uninstall` command (stub)
- [ ] Improve REPL welcome message with version/level

---

## Phase 2: Dialogue System (0.1.x)

### 0.1.0 - LLM Integration
- [ ] Connect Translator to Ollama
- [ ] Connect Junior to Ollama
- [ ] Connect Anna response generation to Ollama
- [ ] Streaming output per participant

### 0.1.1 - Senior Escalation
- [ ] Create escalation criteria (confidence < threshold, needs_senior flag)
- [ ] Create Senior LLM prompt template
- [ ] Implement query_senior() function
- [ ] Multi-round improvement loop

---

## Phase 3: Evidence System (0.2.x)

### 0.2.0 - Snapshot Integration
- [ ] Route hardware queries to hw.json snapshot
- [ ] Route software queries to sw.json snapshot
- [ ] Route status queries to status_snapshot.json
- [ ] Add snapshot source citations to answers

### 0.2.1 - Command Execution
- [ ] Define safe command whitelist (read-only)
- [ ] Implement safe command runner
- [ ] Capture command output as evidence
- [ ] Add command output citations

### 0.2.2 - Log Evidence
- [ ] Query journalctl for relevant logs
- [ ] Extract error/warning patterns
- [ ] Add log excerpt citations

---

## Phase 4: Safety Gates (0.3.x)

### 0.3.0 - Action Classification
- [ ] Define ActionRisk enum (ReadOnly, LowRisk, MediumRisk, HighRisk)
- [ ] Create action classifier
- [ ] Implement confirmation prompts per risk level
- [ ] "I assume the risk" for high-risk actions

### 0.3.1 - Rollback Foundation
- [ ] Create backup before file modifications
- [ ] Store timestamped backups
- [ ] Store patch diffs
- [ ] Create rollback instruction set

### 0.3.2 - btrfs Integration
- [ ] Detect btrfs filesystem
- [ ] Create pre-action snapshots when available
- [ ] Expose snapshot in rollback plan

---

## Phase 5: Learning System (0.4.x)

### 0.4.0 - Recipe Storage
- [ ] Define Recipe struct (trigger, steps, verification, rollback, risk)
- [ ] Create recipe store (JSON files)
- [ ] Implement recipe save/load
- [ ] Recipe versioning

### 0.4.1 - Recipe Matching
- [ ] Match user intent to existing recipes
- [ ] Execute recipe steps
- [ ] Skip Junior/Senior when recipe exists
- [ ] Track recipe usage stats

### 0.4.2 - Recipe Learning
- [ ] Detect recipe-worthy interactions (Senior helped, repeated pattern)
- [ ] Extract recipe from successful answer
- [ ] Create new recipe
- [ ] Update existing recipes

---

## Phase 6: XP and Gamification (0.5.x)

### 0.5.0 - XP System
- [ ] Define XP curve (non-linear)
- [ ] Track XP for Anna, Junior, Senior
- [ ] Award XP for correct answers
- [ ] Award XP for new recipes

### 0.5.1 - Level and Titles
- [ ] Define level 0-100 progression
- [ ] Create title list (nerdy, ASCII-friendly)
- [ ] Display level/title in status
- [ ] Display level/title in REPL welcome

---

## Phase 7: Self-Sufficiency (0.6.x)

### 0.6.0 - Ollama Auto-Setup
- [ ] Detect Ollama installation
- [ ] Install Ollama if missing
- [ ] Detect hardware capabilities
- [ ] Select appropriate models
- [ ] Download models with progress

### 0.6.1 - Auto-Update
- [ ] Check GitHub releases every 10 minutes
- [ ] Download new version
- [ ] Verify checksum
- [ ] Restart annad safely
- [ ] Record update state

### 0.6.2 - Helper Tracking
- [ ] Track Anna-installed packages
- [ ] Display helpers in status
- [ ] Remove helpers on uninstall

### 0.6.3 - Clean Uninstall
- [ ] Implement `annactl uninstall`
- [ ] List helpers for removal choice
- [ ] Remove services, data, models
- [ ] Clean permissions

### 0.6.4 - Factory Reset
- [ ] Implement `annactl reset`
- [ ] Delete recipes
- [ ] Remove helpers
- [ ] Reset DBs
- [ ] Keep binaries

---

## Phase 8: Proactive Monitoring (0.7.x)

### 0.7.0 - Trend Detection
- [ ] Track boot time trends
- [ ] Track performance metrics over time
- [ ] Detect regressions

### 0.7.1 - Correlation Engine
- [ ] Correlate degradation with recent changes
- [ ] Package install timeline
- [ ] Service state changes

### 0.7.2 - Anomaly Alerts
- [ ] Thermal anomalies
- [ ] Network instability
- [ ] Disk I/O regressions
- [ ] Service failures

---

## Phase 9: Production Polish (0.8.x - 0.9.x)

### 0.8.0 - UI Polish
- [ ] ASCII borders and formatting
- [ ] True color support
- [ ] Spinner indicators
- [ ] Streaming output

### 0.8.1 - Performance Optimization
- [ ] Minimize LLM prompt sizes
- [ ] Cache frequent queries
- [ ] Optimize snapshot reads

### 0.9.0 - Testing and Hardening
- [ ] Integration tests for all flows
- [ ] Error handling coverage
- [ ] Edge case testing

### 0.9.1 - Documentation
- [ ] Complete README
- [ ] Architecture docs
- [ ] User guide

---

## Milestone: v1.0.0 - Production Ready

All phases complete, tested, documented. Ready for production use.

---

## Notes

- Each task should be small and verifiable
- Preserve snapshot performance throughout
- Debug mode always on for now
- No emojis or icons in output
- Every completed task moves to RELEASE_NOTES.md
