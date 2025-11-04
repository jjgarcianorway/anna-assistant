# Anna Roadmap

This document outlines Anna's development roadmap from v1.0 to future releases.

---

## Version History

### ✅ v1.0 "Production Ready" (Released)

**Goal**: Stable, beautiful, production-ready system companion

**Key Features**:
- CLI simplification (6 core commands)
- Experimental command gating (`ANNA_EXPERIMENTAL=1`)
- Beautiful output library standardization
- 20+ advisor rules with Arch Wiki citations
- Hardware/software/storage awareness
- Doctor system with health checks
- Report generation with recommendations

**Architecture**:
- Clean separation: annactl (CLI) + annad (daemon)
- JSON-RPC communication over Unix socket
- Beautiful output library (`anna_common::beautiful`)
- Consistent pastel color palette
- Modular command structure

---

### ✅ v1.1 "Advisor Intelligence" (Released)

**Goal**: Autonomous action with rollback safety

**Key Features**:
- `apply` command for autonomous action
  - Interactive mode (ask for each recommendation)
  - Dry-run mode (preview without executing)
  - Auto mode (apply low-risk only)
  - Specific mode (apply by ID)
- Rollback token generation
- Audit logging for autonomous actions
- Risk-based filtering (Low/Medium/High)
- State snapshot foundation

**Safety Model**:
- Only low-risk actions auto-apply
- Every action generates rollback token
- Complete audit trail
- Arch Wiki citations in logs

---

### 🚧 v1.2 "Rollback & Foresight" (In Progress)

**Goal**: Complete rollback system and behavioral awareness

**Phase 1: Rollback System** ✅
- [x] `rollback` command with three modes:
  - `--last` → Undo last applied action
  - `--id <advice_id>` → Undo specific action
  - `--list` → Show rollback history
- [x] Rollback token management
- [x] Rollback strategy detection (package removal, service restart, etc.)
- [x] Audit logging for rollback operations
- [x] Beautiful output for rollback command

**Phase 2: State Snapshots** (Pending)
- [ ] Package state capture (before/after)
- [ ] File modification tracking
- [ ] Configuration backup
- [ ] True state restoration in rollback

**Phase 3: Extended Advisor Rules** (Pending)
- [ ] 10+ new low-risk automation rules
- [ ] Editor defaults (vim syntax highlighting, git config)
- [ ] System limits (journald size, coredump settings)
- [ ] Security defaults (firewall status, SSH hardening)
- [ ] Performance tuning (swappiness, I/O scheduler)

**Phase 4: Behavioral Awareness** (Pending)
- [ ] Shell history parsing (bash, zsh, fish)
- [ ] Process usage analysis
- [ ] Workflow pattern detection (developer, server, creative)
- [ ] Personalized recommendation prioritization
- [ ] Privacy-first design (all data local)

**Phase 5: Documentation & Testing** (Partial)
- [x] CONTRIBUTING.md with build/testing standards
- [x] ARCHITECTURE.md with system design
- [ ] Automated test suite for apply/rollback
- [ ] Integration tests for advisor rules
- [ ] Performance benchmarks

---

## v1.3 "Predictive Intelligence" (Planned Q2 2024)

**Goal**: Learn from user behavior and predict future needs

**Key Features**:
- ML-based recommendation prioritization
- Historical pattern analysis
- Proactive maintenance suggestions
- Anomaly detection in system metrics
- Forecast engine for predictive warnings

**Architecture**:
- Local-only ML models (no cloud)
- Historical data in SQLite
- Recommendation feedback loop
- Success/failure learning
- Confidence scoring

**Safety**:
- Predictions are advisory only (no auto-action)
- User can accept/reject with feedback
- Model retraining based on user preferences

---

## v1.4 "Multi-Distro Expansion" (Planned Q3 2024)

**Goal**: Support beyond Arch Linux

**Target Distributions**:
- Debian/Ubuntu family
- Fedora/RHEL family
- OpenSUSE family
- Gentoo (community-driven)

**Architecture**:
- Distro detection framework (already exists)
- Pluggable advisor engines
- Distro-specific rule sets
- Package manager abstraction
- Distribution-specific wiki citations

**Implementation**:
- Debian: apt-based rules, Debian Wiki refs
- Fedora: dnf-based rules, Fedora Docs refs
- OpenSUSE: zypper-based rules, openSUSE Wiki refs

---

## v2.0 "Cloud Sync & Collaboration" (Planned Q4 2024)

**Goal**: Optional cloud features for multi-machine management

**Key Features**:
- Optional cloud sync (user opt-in)
- Multi-machine dashboard
- Shared recommendation database
- Team collaboration features
- Fleet management mode

**Privacy**:
- Cloud features are 100% opt-in
- Self-hosting option available
- End-to-end encryption
- Anonymous telemetry only (no PII)
- Data deletion on request

**Architecture**:
- Cloud sync via REST API
- Optional daemon component
- Encrypted storage
- WebSocket real-time updates
- Web dashboard (React/TypeScript)

---

## Long-Term Vision (v3.0+)

### Advanced Features

**System Recovery**:
- Automatic system snapshots before changes
- One-click system restore
- Bootable recovery environment
- Emergency repair mode

**Smart Automation**:
- Scheduled maintenance tasks
- Automatic update windows
- Dependency resolution
- Conflict detection

**Community Features**:
- Shared rule repository
- User-contributed advisor rules
- Crowdsourced recommendations
- Community voting on advice quality

**Enterprise Features**:
- Role-based access control
- Compliance reporting
- Policy enforcement
- Audit compliance (SOC 2, ISO 27001)

---

## Community Roadmap

### Open Source Milestones

**100 Stars** 🌟
- Dedicated Discord server
- Monthly community calls
- Official Docker images

**500 Stars** 🌟🌟
- Professional website
- Video tutorials
- Official documentation site

**1000 Stars** 🌟🌟🌟
- Anna Foundation established
- Full-time maintainer
- Corporate sponsorships

---

## Feature Requests

Have an idea? Open an issue on GitHub with the label `feature-request`.

Popular community requests:
- [ ] Desktop notifications
- [ ] System tray icon
- [ ] GUI dashboard
- [ ] Plugin system
- [ ] Custom advisor rules (user-defined)
- [ ] Backup integration (Timeshift, Borg)

---

## Deprecation Policy

Anna follows semantic versioning. Breaking changes are:

1. **Announced** in release notes
2. **Deprecated** for one major version
3. **Removed** in next major version

**Example**:
- v1.5: Feature deprecated (warning shown)
- v2.0: Feature removed

---

## Release Cadence

- **Major releases** (X.0.0): Every 6 months
- **Minor releases** (1.X.0): Every month
- **Patch releases** (1.1.X): As needed (bugs/security)

---

## How to Contribute

See `CONTRIBUTING.md` for development guidelines.

**Priority Areas**:
1. Advisor rules (Arch-specific)
2. Testing and validation
3. Documentation improvements
4. Bug fixes
5. Multi-distro support

---

## Design Principles (Never Compromise)

1. **Beauty** — Calm, elegant, consistent
2. **Intelligence** — Context-aware, helpful
3. **Safety** — Rollback-safe, auditable
4. **Privacy** — Local-first, transparent
5. **Community** — Open, inclusive, respectful

---

**Built with ❤️ and Rust**

**Guided by the Arch Wiki**

**Designed for humans**
