# Claude Workflow for Anna Project

## 📋  Project Rules

- Never release without testing - never claim something is implemented without testing
- Ensure no file has more than 400 lines - modularization is key
- Use best practices for coding, security, documentation
- Ensure the software is always scalable
- Beautiful UX/UI is mandatory - use TRUE COLOR, Bold, emojis/icons with 2 spaces after each
- Always release when bumping a version (commit, upload, release, push, tag, update README.md)
- Every release must include binaries

## 📁  Canonical Files

| File | Purpose |
|------|---------|
| `CLAUDE.md` | This file - workflow contract |
| `docs/ANNA_SPEC.md` | Technical and product specification |
| `docs/ANNA_PROGRESS.md` | Roadmap and progress checklist |
| `docs/ANNA_TEST_PLAN.md` | Test strategy and coverage |
| `docs/ANNA_BUGLOG.md` | Bug tracker and regression log |

## ✅  Task Lifecycle

1. **Read context**: Open CLAUDE.md, ANNA_SPEC.md, ANNA_PROGRESS.md, ANNA_TEST_PLAN.md
2. **Clarify scope**: Identify version/milestone, affected checklist items
3. **Plan**: Write numbered plan before coding
4. **Implement**: Small, cohesive changes respecting constraints
5. **Test**: Run `cargo test --workspace`, document expected outcomes
6. **Update tracking**: Update progress, test plan, buglog as needed
7. **Report**: Summarize changes, files affected, tests run

## 🔒  "Done" Semantics

- Never say "implemented" without showing relevant code
- Never say "all tests pass" without running them
- Treat logs and user feedback as ground truth
- Prefer under-claiming over over-claiming

## 🐛  Bug Handling

- Log bugs in `docs/ANNA_BUGLOG.md` with GitHub issue reference
- Mirror status in ANNA_PROGRESS.md for relevant version
- When fixing: update code, tests, ANNA_TEST_PLAN.md, ANNA_BUGLOG.md

## 🚫  Anna Constraints (from ANNA_SPEC.md)

- CLI surface: `annactl` only (REPL, one-shot, status, version, help)
- No hardcoded system facts - probes and learned facts only
- Separate system knowledge from user knowledge
- Command whitelist only - no arbitrary shell execution

## 🧠  v0.50.0 Brain Upgrade Spec

### Question Classification (5 Types)

```rust
enum QuestionType {
    FactFromKnowledge,      // Answerable from stored knowledge
    SimpleProbe,            // Single probe needed (e.g., "What CPU?")
    ComplexDiagnosis,       // Multiple probes + reasoning
    DangerousOrHighRisk,    // Safety check required
    NeedsUserClarification, // Ambiguous question
}
```

### Safe Command Policy

Commands are classified by safety level:

| Safety Level | Auto-Execute | Examples |
|-------------|--------------|----------|
| `read_only` | ✅ Yes | `ls`, `cat`, `lscpu`, `free`, `df` |
| `low_risk` | ✅ Yes | `pacman -Q`, `systemctl status` |
| `dangerous` | ❌ Never | `rm`, `mv`, `chmod`, `dd`, `kill` |

### 11 Safe Command Categories

1. **File Inspection**: `ls`, `file`, `stat`, `wc`, `du`
2. **Shell Builtins**: `pwd`, `echo`, `type`, `which`
3. **File Reading**: `cat`, `head`, `tail`, `less`
4. **Text Processing**: `grep`, `awk`, `sed` (read-only), `cut`, `sort`, `uniq`
5. **Searching**: `find`, `locate`, `whereis`
6. **System Info**: `uname`, `hostname`, `uptime`, `date`, `timedatectl`
7. **Package Queries**: `pacman -Q`, `pacman -Si`, `dpkg -l`, `rpm -qi`
8. **Networking**: `ip addr`, `ip route`, `ss`, `ping` (limited)
9. **Archives**: `tar -tf`, `unzip -l`, `zcat`, `gunzip -c`
10. **Shell Infrastructure**: `env`, `printenv`, `locale`
11. **Hardware Queries**: `lscpu`, `lsblk`, `lspci`, `lsusb`, `free`, `df`

### Generic Command Probe

```json
{
  "probe_id": "system.command.run",
  "params": {
    "command": "pacman -Qi linux",
    "timeout_secs": 30
  }
}
```

### Never Safe Commands (Dangerous)

```
rm, mv, cp, chmod, chown, chgrp, dd, mkfs, fdisk,
parted, mount, umount, kill, pkill, killall, reboot,
shutdown, poweroff, systemctl start/stop/enable/disable,
pacman -S, pacman -R, apt install, apt remove
```

### LLM Orchestration Flow

```
Question → Classify → Route:
  ├─ FactFromKnowledge → Return from cache (no LLM)
  ├─ SimpleProbe → Execute probe → Junior summarize
  ├─ ComplexDiagnosis → Junior plan → Execute → Senior synthesize
  ├─ DangerousOrHighRisk → Block with explanation
  └─ NeedsUserClarification → Ask clarifying question
```

### Junior/Senior Optimization

- **Junior (Fast)**: Command parsing, probe execution, draft answers
- **Senior (Smart)**: Reasoning, synthesis, verification, user-facing answers
- Local tools first: `--help`, `man`, local docs before LLM calls

## 🎭  v0.60.0 Conversational UX Spec

### Principles

1. **No frozen UI**: Progress messages for any operation > 1s
2. **No extra LLM tokens**: Narrative from structured events, not LLM calls
3. **Readable conversation logs**: Anna/Junior/Senior dialog from real steps
4. **No slowdowns**: Event generation must be cheap (templates only)

### Actors (Narrative Personas)

| Actor | Role | Style |
|-------|------|-------|
| **Anna** | Orchestrator, user voice | Short, clear, occasional dry humor |
| **Junior** | Planning, probe selection | Technical, step-by-step |
| **Senior** | Supervisor, auditor | Only when reviewing/scoring |

### Event Types

```rust
enum EventKind {
    QuestionReceived,       // User asked something
    ClassificationStarted,  // Analyzing question
    ClassificationDone,     // Type determined
    ProbesPlanned,          // Commands selected
    CommandRunning,         // Executing command
    CommandDone,            // Command finished
    SeniorReviewStarted,    // Senior checking
    SeniorReviewDone,       // Review complete
    UserClarificationNeeded,// Need user input
    AnswerSynthesizing,     // Building answer
    AnswerReady,            // Done
}
```

### Progress Message Templates

```
[Anna]   Reading your question and planning next steps.
[Junior] Classifying question: looks like a simple safe probe.
[Anna]   Running safe command: journalctl -u annad --since '6 hours ago'.
[Senior] Double-checking the answer and scoring reliability.
[Anna]   Done. Reliability: 93% (GREEN).
```

### Conversation Log (Debug Mode)

```
[Anna]   I parsed your question and handed it to Junior.
[Junior] I classified it as a simple safe probe (journalctl).
[Anna]   I ran: journalctl -u annad --since '6 hours ago'.
[Senior] I reviewed the logs - reliable at 93%.
[Anna]   I summarized the key lines for you.
```

### Rules

- Messages: Short, informative, no fluff
- No LLM calls for formatting events
- Progress lines streamed in order
- Conversation log from real events only
