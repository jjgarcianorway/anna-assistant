# READ-ONLY Contract (Phase 22)

## Overview

User questions are classified into two categories:
- **READ_ONLY**: Inspection, diagnosis, reporting - no confirmation needed
- **MUTATING**: Changes configuration, files, services - requires confirmation

## Intent Classification

### READ_ONLY Keywords (default for diagnostic questions)
- detect, show, check, list, diagnose, display, report, describe
- "is my", "what is", "why does", "how much", "what's using"
- recommend (without explicit change request)

### MUTATING Keywords (require ActionPlan)
- change, set, configure, install, uninstall, remove, disable, enable
- prevent, ensure, fix (when explicitly requesting changes)
- mask, unmask, modify, edit, create, delete

### Classification Rules
1. If MUTATING keyword present AND explicitly targets a file/service → MUTATING
2. If only READ_ONLY keywords → READ_ONLY
3. "recommend actions" without "apply/do it" → READ_ONLY
4. Default for troubleshoot/diagnose → READ_ONLY

## READ_ONLY Answer Contract

For READ_ONLY intents, Anna MUST:
- Run probes automatically (no confirmation)
- Answer directly using probe results
- NOT ask "Would you like me to..." or similar
- NOT output shell commands, code blocks, or terminal instructions
- NOT claim to "handle it myself" for read-only info

### Evidence Format (non-Debug)
```
Evidence:
  - probe_name: result_summary
  - probe_name: result_summary
  - probe_name: result_summary
```
Max 3 lines. No raw commands.

### Forbidden in READ_ONLY Answers
- `sudo`, `systemctl`, `pacman`, `apt`, `nano`, `vim`
- Code fences with commands
- "Run this:", "Execute:", "Type:"
- "cat /proc", "free -h", "journalctl"
- Step-by-step terminal directions

## MUTATING Answer Contract

For MUTATING intents, Anna MUST:
- Generate ActionPlan with steps
- Present plan and ask "Proceed? (yes/no)"
- Execute only after "yes"
- Verify and rollback on failure

## Probe Deduplication

Per-request ProbeLedger prevents:
- Same probe executed twice per request
- Repeated output in iterations
- "(N iterations)" visible only at Debug level

## Enforcement Chain

1. Intent classifier determines READ_ONLY vs MUTATING
2. Probe dispatcher checks ProbeLedger for duplicates
3. LLM output filtered by sanitize_dialogue()
4. FinalAnswer filtered by filter_final_answer()
5. If READ_ONLY + commands detected → replace with clean answer

## Do Not Regress

- [ ] READ_ONLY questions get direct answers
- [ ] No commands in non-Debug answers
- [ ] No confirmation flow for READ_ONLY
- [ ] No "Would you like me to..." for READ_ONLY
- [ ] Probes not repeated per request
- [ ] Evidence max 3 lines in non-Debug
