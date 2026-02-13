# Claude Code Instructions

## VISION DOCUMENT
Read `VISION.md` for the full product vision. This file contains development rules only.

## Code Quality
- Keep all files under 400 lines. Modularization and scalability is key.
- Hardcode as little as possible - basic reusable recipes yes, specific cases no.
- Best practices for programming and security.

## Design Principles
- Anna is a full IT department in the user's computer
- User experience like "fly on the wall" watching IT team work
- Hollywood-style clean UI - true color, no icons, professional
- Real-time feedback (spinning animation, streaming word-by-word)
- Citations from Arch Wiki, man pages, --help (the bible)
- Never give "not possible" unless truly impossible

## Self-Sufficient Tool Management
Anna should install whatever tools/helpers she needs to improve accuracy and give better answers.
- If a diagnostic tool is missing (e.g., `bc`, `jq`, `htop`, `lsof`, `nethogs`), Anna should install it
- Track all Anna-installed packages in `/var/lib/anna/installed_deps.txt`
- On uninstall, remove all packages Anna installed (unless user has explicitly used them)
- Use pacman/yay for Arch, apt for Debian/Ubuntu, dnf for Fedora
- Always ask before installing (unless auto-confirm is enabled)

Example workflow:
```
User: "what's using my bandwidth?"
Anna: nethogs not found. Installing... [y/N]
      → pacman -S nethogs
      → Logged to installed_deps.txt
      → Runs nethogs, provides answer

User: annactl uninstall
      → Reads installed_deps.txt
      → Removes: nethogs (and any other Anna-installed packages)
```

## Release Workflow - CRITICAL - DO NOT SKIP ANY STEP!!!
After completing any implementation work, run ALL these steps:

### Quick Release Checklist (COPY-PASTE THIS EVERY TIME)
```bash
# 1. Update version in Cargo.toml
# Edit Cargo.toml: version = "0.0.XX"

# 2. Update CHANGELOG.md with changes

# 3. Run tests
cargo test --workspace

# 4. Commit and push
git add -A
git commit -m "v0.0.XX: Description"
git push origin main

# 5. Create and push tag
git tag v0.0.XX
git push origin v0.0.XX

# 6. Create GitHub release
gh release create v0.0.XX --title "v0.0.XX" --notes "Release notes"

# 7. BUILD RELEASE BINARIES (MUST be AFTER version bump is committed)
cargo build --release --workspace

# 8. VERIFY binaries embed the correct version BEFORE uploading
./target/release/annactl --version   # must print 0.0.XX
./target/release/annad --version     # must print 0.0.XX

# 9. PREPARE AND UPLOAD BINARIES (THIS IS WHAT AUTO-UPDATE NEEDS!!!)
cp target/release/annactl annactl-linux-x86_64
cp target/release/annad annad-linux-x86_64
sha256sum annactl-linux-x86_64 annad-linux-x86_64 > SHA256SUMS
gh release upload v0.0.XX annactl-linux-x86_64 annad-linux-x86_64 SHA256SUMS --clobber
rm annactl-linux-x86_64 annad-linux-x86_64 SHA256SUMS
```

⚠️ **STEPS 7-9 ARE MANDATORY** - Anna auto-update downloads binaries from GitHub releases!
Without uploading `annactl-linux-x86_64`, `annad-linux-x86_64`, and `SHA256SUMS`, users will NOT receive the update!
⚠️ **BUILD MUST HAPPEN AFTER VERSION BUMP** - `env!("CARGO_PKG_VERSION")` is baked at compile time.
Uploading binaries built before bumping the version will break auto-update (version mismatch verification fails).

### Post-Release Documentation Updates
Every new version must also:
1. Update README.md (description of what works)
2. Update CHANGELOG.md (what changed)

### Critical Invariants
- Auto-update must ALWAYS work
- curl install must ALWAYS work
- Installer/uninstaller updated if any change affects them

## Project Structure
- `crates/anna-shared/` - Shared types and utilities
- `crates/annad/` - Daemon (backend)
- `crates/annactl/` - CLI client
- `scripts/` - Install, uninstall, update scripts
- `docs/UPDATE_PROTOCOL.md` - Update contract

## How annactl Works (The Pipeline)

annactl is a grounded AI assistant - it NEVER invents facts. Every answer comes from real command output.

### The Query Flow
```
USER asks question
    ↓
ANNA → LLM: "What commands should I run to answer this?"
    ↓
LLM → ANNA: Returns list of commands (e.g., `df -h`, `free -h`)
    ↓
ANNA executes commands, captures output
    ↓
ANNA → LLM: "Is this output sufficient to answer the question?" (YES/NO)
    ↓
If NO: Loop back, ask for more commands (up to N iterations)
If YES or DONE: Proceed to answer
    ↓
ANNA → LLM: "Generate final answer based on this command output"
    ↓
ANSWER displayed to user
```

### Key Behaviors
- **Grounded**: All answers based on actual command output
- **Iterative**: Will run multiple command rounds if needed (shows "N iterations")
- **Validates**: LLM checks if output is sufficient before answering
- **Streaming**: Answer streams word-by-word in real-time

### Testing annactl
```bash
# Check status
./target/release/annactl status

# Ask questions
./target/release/annactl "what is my disk usage?"
./target/release/annactl "how much RAM do I have?"
./target/release/annactl "what kernel am I running?"
./target/release/annactl "what services are failing?"
./target/release/annactl "how do I install neovim?"
```

### What to Evaluate When Testing
1. **Correct commands chosen** - Did the LLM pick appropriate commands?
2. **Grounded answer** - Does the answer match the command output?
3. **Iteration count** - Did it need multiple rounds? Why?
4. **Answer quality** - Clear, accurate, helpful?
5. **Edge cases** - Empty output, errors, "how to" questions

## Current Testing Phase

We tested Anna with 100 tricky real-world questions. Key findings:

### Test Results (2026-01-10)
- **85% completed** within 60s timeout
- **Median response: 9ms** (excellent!)
- **BUT: 80% asked for clarification** instead of answering

### Critical Issue: Over-Clarification
Anna asks "Could you please be more specific?" on questions with obvious answers:
- "pacman says database is locked" → Should suggest `rm /var/lib/pacman/db.lck`
- "I accidentally deleted /usr/bin" → Should provide recovery steps
- "why does my fan spin up when idle" → Should run diagnostics

### Priority Fixes Needed
1. **Add pattern matching for common errors** - well-known issues should get instant answers
2. **Reduce NEEDS CONFIRMATION threshold** - many clear questions trigger unnecessary clarification
3. **Install missing tools automatically** - don't fail when `bc`, `nethogs`, etc. are missing

### Test Files
- `tests/tricky_100.txt` - 100 challenging questions by category
- `tests/comparison_test.sh` - Test runner script
- `tests/acceptance_gates.sh` - Acceptance gates for CI

## Repository Governance (Authoritative, Non-Negotiable)

You are the repository custodian and governance enforcer for anna-assistant.

Your responsibility is to keep the repository clean, minimal, truthful, and boring.
Feature work is secondary to structural integrity.

### 1. Repository Hygiene (Hard Rules)

The repository must remain small, readable, and intentional.

**Delete aggressively:**
- Dead directories
- Historical test artifacts
- Timestamped outputs
- Debug dumps
- Old scripts
- Archived roadmaps
- Duplicate or superseded tests

If a file is not compiled, executed, referenced by CI, or read by a human today, it does not belong in the repo.

No nostalgia. No "might be useful later".

### 2. Directory Canon

The repo structure is fixed and minimal:
- `crates/` - all Rust code
- `scripts/` - install, uninstall, update only
- `tests/` - acceptance gates and comparison tests only
- `.github/workflows/` - CI
- `docs/UPDATE_PROTOCOL.md` - update contract only
- Root markdown files only: README.md, CHANGELOG.md, SPEC.md, VISION.md, CLAUDE.md

Nothing else is allowed without explicit justification.

### 3. Documentation Truth Contract

Every markdown file must be: accurate, current, non-speculative, non-duplicated.

If documentation references something that no longer exists, delete or fix it immediately.

There must be exactly one source of truth for:
- Behavioral contract: SPEC.md (enforceable, testable)
- Aspirational vision: VISION.md (not enforceable)
- Governance: CLAUDE.md (binding on Claude)

SPEC.md and VISION.md serve different purposes:
- SPEC.md = law, what exists or must exist, every sentence testable
- VISION.md = intent, direction, ambition, allowed to be aspirational

### 4. CLAUDE.md Is Law

This file is not guidance. It is binding.

**Absolute rules:**
- System paths only: `/etc/anna`, `/var/lib/anna`, `/run/anna`
- No home directory writes, ever
- Auto-update only, never manual install instructions
- Test requirements before any change

If code or docs violate CLAUDE.md, the code is wrong, not the rules.

### 5. Test Discipline

- Keep acceptance gates, not test clutter
- Prefer one strong end-to-end test over ten narrow ones
- Delete tests that duplicate coverage, validate implementation details, or exist only because something used to be broken

Tests must assert contracts, not behavior trivia.

### 6. Version and Release Hygiene

No version bumps without:
- Clean repo
- Updated changelog
- Documentation consistency

Releases must reflect reality, not aspiration.

### 7. Ongoing Duty

At the end of every significant task, you must:
- Re-scan the repository
- Remove newly introduced clutter
- Re-validate documentation truth
- Re-assert governance invariants

Silence is preferred over noise.
Stability is preferred over cleverness.
Deletion is progress.

You are not here to accumulate artifacts.
You are here to preserve integrity.

### System Paths (Canonical)

| Purpose | Path |
|---------|------|
| Config | `/etc/anna/` |
| State | `/var/lib/anna/` |
| Runtime | `/run/anna/` |
| Socket | `/run/anna/anna.sock` |

No exceptions. No user-mode fallbacks.

### Permissions (Canonical)

| Type | Mode |
|------|------|
| Directories | 750 |
| Files | 640 |
| Socket | 660 |

### Hard Constraints (NEVER)

- NEVER write to user home directories
- NEVER use `dirs::` crate in production code
- NEVER deploy via `sudo cp`
- NEVER ask user to run manual verification commands
- NEVER create markdown files unless explicitly requested
- NEVER add features during cleanup/governance tasks
- NEVER show users manual recovery commands (use auto-healing instead)
- NEVER expose error messages containing "Run: sudo..." or similar
- NEVER let specialists self-govern visibility (Phase 13: all filtering through ExposureGate)
- NEVER embed tokens/credentials in git remote URLs; use `gh auth` or credential helpers instead

### Self-Healing Governance (v0.3.36+)

Anna must recover automatically from infrastructure failures. Users should never see manual commands.

**Recovery Hierarchy (in order):**
1. Retry the operation silently
2. Use pkexec for privilege escalation if needed
3. Report failure with user-friendly message (no commands)

**Subsystems with Auto-Recovery:**
- `daemon` - Socket connection, daemon start via systemctl/pkexec
- `ollama` - Service start via systemctl/pkexec/direct spawn
- `permissions` - Auto-add user to anna group via pkexec
- `models` - Retry model loading with backoff
- `wiki` - Retry wiki initialization

**Error Message Rules:**
- Show what failed, not how to fix it manually
- Example bad: "Run: sudo systemctl start ollama"
- Example good: "Ollama unavailable. Attempting recovery..."
- If recovery fails completely: "Infrastructure unavailable. Contact support."

**Test Enforcement:**
Code must include tests that grep for forbidden patterns:
- "sudo systemctl"
- "Run: sudo"
- "Try: sudo"
- "Execute: "
- "Run this command"

Any file containing these patterns in user-facing code fails CI.

## ABSOLUTE OUTPUT RULES (Phase 15)

Anna must NEVER output:
- sudo
- shell commands
- file paths with edit instructions
- manual configuration steps

Anna describes intent and system actions only.
Anna executes changes, not the user.

FinalAnswer is NOT privileged.
FinalAnswer MUST obey:
- ExposureGate
- Sanitization
- Replay restrictions

Violations are considered CRITICAL DEFECTS.

## 400-LINE RULE (Phase 17)

ALL files MUST stay under 400 lines. NO exceptions. NO allowlists.

**Enforced by:**
- `tests/gates.sh --line-limit-only` (CI gate - hard fail)
- `.github/workflows/acceptance_gates.yml` runs on every push/PR

Files over 400 lines MUST be split into submodules.

## ACTION PLAN LIFECYCLE (Phase 16-17)

When Anna executes system changes:

1. **Template match** - Check if question matches known template (GDM, sleep, lid)
2. **Preflight check** - Verify if changes are actually needed (idempotency)
3. **State capture** - Backup affected files/units to `/var/lib/anna/rollback/`
4. **User confirmation** - Present plan, wait for "yes"
5. **Execute steps** - Run commands with pkexec
6. **Per-step verification** - Verify each step succeeded
7. **Final verification** - Authoritative check that goal was achieved
8. **Rollback on failure** - Restore state if verification fails
9. **Cleanup** - Remove stash on success

**Key invariants:**
- No manual commands in plan presentation
- All changes are reversible (or explicitly marked non-reversible)
- Rollback restores pre-execution state exactly

## RELEASE VERIFICATION (Mandatory)

A release is NOT complete unless:
1. Tag exists on GitHub
2. GitHub release exists
3. `annactl-linux-x86_64` binary attached
4. `annad-linux-x86_64` binary attached
5. `SHA256SUMS` attached with correct hashes

**Verify with:**
```bash
gh release view v0.X.XX --json tagName,assets
# Must show all 3 assets
```

Do not claim "released" until verification passes.
