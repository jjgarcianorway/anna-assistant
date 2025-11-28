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
