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
- Track all Anna-installed packages in `~/.anna/installed_deps.txt` (user) or `/var/lib/anna/installed_deps.txt` (system)
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

# 7. BUILD RELEASE BINARIES
cargo build --release --workspace

# 8. PREPARE AND UPLOAD BINARIES (THIS IS WHAT AUTO-UPDATE NEEDS!!!)
cp target/release/annactl annactl-linux-x86_64
cp target/release/annad annad-linux-x86_64
sha256sum annactl-linux-x86_64 annad-linux-x86_64 > SHA256SUMS
gh release upload v0.0.XX annactl-linux-x86_64 annad-linux-x86_64 SHA256SUMS --clobber
rm annactl-linux-x86_64 annad-linux-x86_64 SHA256SUMS
```

⚠️ **STEPS 7-8 ARE MANDATORY** - Anna auto-update downloads binaries from GitHub releases!
Without uploading `annactl-linux-x86_64`, `annad-linux-x86_64`, and `SHA256SUMS`, users will NOT receive the update!

### Post-Release Documentation Updates
Every new version must also:
1. Update README.md (description of what works)
2. Update ROADMAP.md (remove implemented, keep detailed missing items)
3. Update FEATURES.md (tested and verified features)

### Critical Invariants
- Auto-update must ALWAYS work
- curl install must ALWAYS work
- Installer/uninstaller updated if any change affects them

## Project Structure
- `crates/anna-shared/` - Shared types and utilities
- `crates/annad/` - Daemon (backend)
- `crates/annactl/` - CLI client
- `VISION.md` - Full product vision (authoritative)
- `ROADMAP.md` - Planned features by phase
- `FEATURES.md` - Implemented features

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
- `tests/tricky_100_questions.txt` - 100 challenging questions by category
- `tests/comparison_test.sh` - Test runner script
- `tests/COMPARISON_REPORT.md` - Full analysis with recommendations
