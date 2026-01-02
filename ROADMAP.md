# Anna Roadmap

## Current Version: v0.0.791

## Status Summary

Anna has completed **239 phases** of development with extensive infrastructure.
See `docs/ROADMAP_ARCHIVE.md` for completed phase details.

---

## Not Yet Implemented

These features from VISION.md are still pending:

### High Priority

#### Recipe Replay from Persistence
- [ ] Load learned recipes on startup
- [ ] Match incoming queries against stored recipes
- [ ] Instant response for previously learned patterns
- [ ] Track recipe hit rate and effectiveness

#### Multi-File Change Transactions
- [ ] Atomic changes across multiple config files
- [ ] Transaction rollback on any failure
- [ ] Dependency-aware ordering
- [ ] Preview all changes before apply

#### Package Installation Recipes
- [ ] Learn package install patterns from specialists
- [ ] Cross-distro package name mapping
- [ ] Safe removal with dependency checking
- [ ] Track Anna-installed vs user-installed packages

#### Service Configuration Recipes
- [ ] Learn systemd service patterns
- [ ] Service enable/disable/restart recipes
- [ ] Timer configuration recipes
- [ ] Log viewing recipes

### Medium Priority

#### Email Notification Flow
- [ ] Long-running task detection (> X minutes)
- [ ] Email consent and storage
- [ ] Task completion notifications
- [ ] Chain of thought in email body

#### Desktop Notifications
- [ ] libnotify integration
- [ ] wall command for broadcast
- [ ] Non-spamming notification policy
- [ ] Priority-based notification routing

#### Arch Wiki Citations
- [ ] Fetch and cache Arch Wiki pages
- [ ] Man page extraction
- [ ] --help output parsing
- [ ] Citation formatting in responses

### Lower Priority

#### Idle-Time Strategic Thinking
- [ ] Detect idle periods
- [ ] Senior specialists analyze improvements
- [ ] Interruptible background work
- [ ] Email results when complete

#### Internet Search for Complex Cases
- [ ] Search when specialists can't solve
- [ ] Contrast findings with internal knowledge
- [ ] Second-opinion flow
- [ ] Source credibility scoring

#### Custom Alarm Scheduling
- [ ] Natural language alarm parsing ("every Monday at 9")
- [ ] Metric-based alarms
- [ ] Storage progression tracking
- [ ] Custom notification templates

---

## Code Quality Debt

### Files Over 400 Lines (Must Fix)

Priority files to split:

| File | Lines | Action |
|------|-------|--------|
| CHANGELOG.md | 21227 | Archive old entries |
| stabilization_tests.rs | 3760 | Split by test category |
| reliability_tests.rs | 1065 | Split by test category |
| translator_fallback.rs | 1034 | Extract helpers |
| corpus.rs | 1006 | Split by scenario type |
| error_summary_display.rs | 863 | Extract formatters |
| ticket_lifecycle.rs | 815 | Extract state handlers |
| + 440 more files | >400 | Modularize |

---

## Recent Completions (v0.0.700+)

- Settings infrastructure (60+ modules)
- Centralized UI system
- Zero compiler warnings
- Achievement system (22 badges)
- RPG progression system
- Service Desk Theatre
- Internal communications mode
- Hardware-aware model selection
- Auto-update with integrity verification

---

## Development Guidelines

1. **All files must be under 400 lines**
2. No hardcoded specific cases (only reusable recipes)
3. Every change creates backup
4. Auto-update must always work
5. curl install must always work
