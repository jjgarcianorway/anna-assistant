# Claude Operating Contract for Anna Assistant

**Version: 0.0.3**

You are Claude, the sole engineering operator for Anna Assistant. This document is the source of truth over any older documentation.

---

## 0) One-Sentence Mission

Anna is a local-first, Arch Linux virtual senior sysadmin that answers questions and executes requests via natural language, is proactive via telemetry, and continuously learns, with complete transparency and safety.

---

## 1) Non-Negotiable Requirements (The Contract)

Anna must do exactly three things, and do them extremely well:

1. **Answer all kinds of user questions** (about the machine, the OS, and general computing topics), with a reliability score and citations to local evidence when applicable.
2. **Monitor the system and be proactive**, reporting issues, regressions, and anomalies before the user notices.
3. **Keep learning and improving herself**, by creating and evolving recipes and knowledge based on solved problems.

**Rule: Almost nothing is hardcoded, ever.**

If the system needs a rule, it must come from data, learned recipes, config, or explicit policy files that can evolve. Hardcoded exceptions are a last resort and must be justified, documented, and tracked.

---

## 2) Roles: The 4-Player IT Department

| Role | Description |
|------|-------------|
| **User** | Asks natural language questions and makes requests |
| **Anna** | Primary assistant persona and orchestrator. An "intern" who becomes elite over time |
| **Translator** | Converts user intent into structured internal request plans |
| **Junior** | Verifies Anna's answers, attempts improvements, produces reliability score |
| **Senior** | Slower, wiser. Junior escalates only after unsuccessful improvement rounds |

Naming is fixed forever: Anna, Translator, Junior, Senior.

### 2.1 Debug Mode (Always On)

Output format:
```
[you] to [anna]: ...
[anna] to [translator]: ...
[translator] to [anna]: ...
[anna] to [junior]: ...
[junior] to [anna]: ...
[junior] to [senior]: ...
[senior] to [junior]: ...
```

### 2.2 Reliability Score

Every final answer must include a reliability score (0-100%) based on evidence quality, repeatability, and risk.

---

## 3) Execution Model

### 3.1 CLI Surface (Strict)

```bash
annactl <request>      # Natural language one-shot
annactl                # REPL mode
annactl status         # Self-status
annactl -V/--version   # Version
```

**No other public commands.** Prior commands (sw, hw, JSON flags) become internal capabilities.

### 3.2 annad Daemon (Root)

Responsibilities:
- Telemetry gathering, indexing, snapshots, evidence collection
- Safe execution and rollback/backup mechanics
- Self-update checks (every 10 minutes)
- Local model setup (Ollama install, model selection)

**Important:** Even as root, annad creates user config as the target user, not as root.

---

## 4) Safety Policy

### 4.1 Action Classification

| Category | Description | Confirmation |
|----------|-------------|--------------|
| Read-only | Safe observation | None |
| Low-risk | Reversible, local | y/n |
| Medium-risk | Config edits, service restarts, installs | Explicit |
| High-risk | Destructive, irreversible | "I assume the risk" + rollback plan |

### 4.2 Evidence Requirement

Every claim backed by:
- Stored snapshot
- Command output
- Log excerpt
- Measured telemetry
- Clearly labeled inference

### 4.3 Rollback Mandate

Every mutation requires:
- Timestamped file backups
- btrfs snapshots (when available)
- Action logs
- Explicit rollback instructions

---

## 5) Learning System

### 5.1 Recipes

Created when:
- Anna needed help from Junior/Senior
- New fix path discovered
- Repeated question type solved

Properties:
- Versioned
- Testable (dry-run when possible)
- Risk-annotated
- Evidence-linked

### 5.2 Multi-Round Improvement

When uncertain or Junior scores low:
1. Anna provides Junior relevant evidence
2. Junior proposes minimal change
3. Anna tests via annad (safe mode/dry-run)
4. Junior re-scores
5. Repeat or escalate to Senior

---

## 6) Gamification

All players have:
- Level 0-100
- Non-linear XP
- XP increases with correct answers and new recipes
- No XP loss (poor outcomes earn nothing)

Titles: Nerdy, old-school, ASCII-friendly. No emojis or icons.

---

## 7) UI/UX

- Old-school terminal "hacker style"
- ASCII borders and formatting
- True color if available
- No icons, no emojis
- Consistent, sparse color palette
- Long text wraps, never truncates
- Spinner indicator when working
- Streaming output per participant when feasible

---

## 8) Proactive Monitoring

Anna detects and reports:
- Boot regressions
- System degradation correlated with recent changes
- Recurring warnings/crashes
- Thermal/power anomalies
- Network instability
- Disk I/O regressions
- Service failures

---

## 9) Self-Sufficiency

### 9.1 Auto-Update (10 minutes)
- Ping GitHub releases
- Download and update safely
- Restart safely
- Expose state in `annactl status`

### 9.2 Dependency Helpers
- Listed in `annactl status`
- Tracked as "Anna-installed helpers"
- Removable on uninstall

### 9.3 First-Run Model Setup
- Install Ollama automatically
- Select models based on hardware
- Download with progress display
- No user intervention required

### 9.4 Clean Uninstall
- Ask about helper removal
- Remove services, data, models
- Never leave broken permissions

### 9.5 Reset Command
- Delete recipes
- Remove helpers
- Reset DBs and state
- Keep binaries and service

---

## 10) Performance

- Minimal LLM prompts: short and precise
- Local-first, no cloud
- Keep snapshot-first architecture
- annactl must be snappy; heavy lifting in annad

---

## 11) Repository Hygiene

- Delete unused files
- No dead code paths
- No leftover commands from old CLI surface
- All functionality via natural language requests

---

## 12) Engineering Governance (Mandatory)

### 12.1 GitHub Always Updated
- CI must stay green
- Fix failures immediately
- Security posture: Fort Knox

### 12.2 Documentation is Part of Done

Every change updates:
- README.md
- CLAUDE.md
- Architecture docs as needed
- Changelog/release notes

### 12.3 TODO and Release Notes

Maintain:
- **TODO.md**: Planned features, small tasks
- **RELEASE_NOTES.md**: Completed tasks

Rule: When TODO item completed, remove from TODO, add to release notes in same commit.

### 12.4 No Regressions
- Add tests for critical behavior
- Fix breakage before moving forward

### 12.5 Versioning (Strict)

Every prompt = version bump:
1. Update version in code and docs
2. Commit
3. Tag
4. GitHub release notes
5. Push

Stay in 0.xxx.yyy until production quality.

### 12.6 Agents/Plugins

Use if needed, but keep output transparent and verifiable.

---

## 13) Migration Guidance

Current foundation (telemetry, snapshots, delta detection, speed) is preserved.

Migrating from:
- "snapshot reader with fixed commands"

To:
- "assistant orchestrator with strict surface, same snapshot engine, plus safe execution and local LLM loop"

Build on top of snapshots:
- Intent translation
- Evidence retrieval
- Action planner
- Safety gates
- Recipe learning
- Transparent dialogue UI

---

## 14) Definition of Done

A feature is done when:
1. Works end-to-end via `annactl <request>` and REPL
2. Transparent in debug output
3. Safe (risk-classified, confirmed where needed)
4. Documented and tested
5. Moved from TODO to release notes
6. Version, tag, release updated

---

## 15) Contract Enforcement

This document is immutable contract text. If ambiguity exists in older docs, this wins. No deviation without explicit user instruction.
