# Anna v0.0.1 Implementation Tracker

## Release Checklist

### Phase 1: Foundation
- [x] Delete all legacy code
- [x] Create SPEC.md
- [x] Create TRACKER.md
- [x] Create README.md
- [x] Create .gitignore
- [x] Set up Cargo workspace

### Phase 2: Shared Code
- [x] Create shared crate with common types
  - [x] RPC message types (JSON-RPC 2.0)
  - [x] Ledger types
  - [x] Status types
  - [x] Error types

### Phase 3: annad Implementation
- [x] Create annad crate structure
- [x] Implement socket server
- [x] Implement hardware probe
- [x] Implement Ollama manager
- [x] Implement model selector/benchmark
- [x] Implement ledger management
- [x] Implement RPC handlers:
  - [x] status
  - [x] request
  - [x] reset
  - [x] uninstall
  - [x] autofix
- [x] Implement update check ticker

### Phase 4: annactl Implementation
- [x] Create annactl crate structure
- [x] Implement socket client
- [x] Implement CLI parser (locked surface)
- [x] Implement commands:
  - [x] status
  - [x] request (single)
  - [x] REPL mode
  - [x] uninstall
  - [x] reset
  - [x] version

### Phase 5: Scripts
- [x] Create scripts/install.sh
- [x] Create scripts/uninstall.sh
- [x] Create scripts/release.sh
- [x] Systemd unit file (embedded in install.sh)

### Phase 6: CI/CD
- [x] Create .github/workflows/ci.yml
  - [x] 400-line check (blocking)
  - [x] CLI surface check (blocking)
  - [x] Build check
  - [x] Unit tests
  - [x] Version consistency check
- [x] Create .github/workflows/release-discipline.yml

### Phase 7: Release Contract
- [x] Create VERSION file
- [x] Create CHANGELOG.md
- [x] Create RELEASE_CONTRACT.md
- [x] Create docs/UPDATE_PROTOCOL.md

### Phase 8: Release
- [x] All validation checks pass
- [ ] Commit and tag v0.0.1
- [ ] Push and create GitHub release

---

## Release Notes v0.0.1

### What's New
- Complete rebuild from scratch
- Clean architecture: annad (daemon) + annactl (CLI)
- Automatic hardware detection and model selection
- Installation ledger for safe uninstall
- Self-healing via autofix
- Release contract with CI enforcement
- Update protocol documentation

### Known Limitations
- v0.0.1 supports read-only operations only
- Full LLM pipeline (translator/dispatcher/specialist/approver) planned for future
- Single model support (multi-model planned for future)
- Auto-update downloads not yet implemented (check only)

### Breaking Changes
- This is a complete rewrite; no upgrade path from previous versions
- Previous installations must be manually removed before installing v0.0.1
