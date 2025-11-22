# Answer Format Specification

**Version:** Beta.245
**Purpose:** Canonical format for all Anna diagnostic and status answers

---

## Overview

Anna uses a structured three-section format for deterministic diagnostic and status answers:

```
[SUMMARY]
<1-2 sentence high-level status>

[DETAILS]
<Detailed insights, issues, or information>

[COMMANDS]
<Recommended commands or actions>
```

This format is used consistently across:
- `annactl "check my system health"` (diagnostic queries)
- `annactl "run a full diagnostic"` (diagnostic queries)
- `annactl status` (condensed status view)
- System report queries

---

## Section Details

### [SUMMARY]

**Purpose:** Immediate, decisive status assessment

**Format:**
- First line: `System health: **<status description>**`
- Follow-up: 1-2 sentences explaining the status
- Voice: Clear, decisive, sysadmin tone

**Examples:**

Healthy system:
```
[SUMMARY]
System health: **all clear, no critical issues detected.**

All diagnostic checks passed. System is operating normally.
```

Critical issues:
```
[SUMMARY]
System health: **2 critical issue(s) and 1 warning(s) detected.**

Immediate attention required for critical issues.
```

Warnings only:
```
[SUMMARY]
System health: **3 warning(s) detected, no critical issues.**

System is stable but warnings should be investigated.
```

### [DETAILS]

**Purpose:** Specific diagnostic insights and issues

**Format:**
- No redundant "Diagnostic Insights:" header (removed in Beta.245)
- Numbered list of issues with severity markers:
  - ✗ for critical issues
  - ⚠ for warnings
  - ℹ for info
- Each issue includes:
  - Summary line with severity marker
  - Details paragraph (indented)
  - Diagnostic commands if applicable (prefixed with $)

**Example:**
```
[DETAILS]

1. ✗ **Failed service detected**
   The systemd service 'example.service' has failed. Check logs for details.

   $ systemctl status example.service
   $ journalctl -u example.service

2. ⚠ **High disk usage**
   Root partition is 85% full. Consider cleaning up old files.

   $ df -h
   $ du -sh /* | sort -h
```

### [COMMANDS]

**Purpose:** Recommended next actions

**Format:**
- Empty line after section header
- Commands listed with $ prefix
- No inline comments for healthy systems
- Brief explanatory text for healthy systems

**Examples:**

With issues:
```
[COMMANDS]

$ annactl status
$ journalctl -xe
$ systemctl --failed
```

Healthy system:
```
[COMMANDS]

No actions required - system is healthy.

$ annactl status        # View current status
```

---

## Voice and Tone Guidelines (Beta.245)

### Deterministic Answers

When answers come from the **internal diagnostic engine** or **system telemetry**:

**Do:**
- Use clear, decisive language
- State facts directly
- Use sysadmin terminology
- Be concise and professional

**Don't:**
- Use hedging words ("might", "probably", "I think")
- Reference LLM sources
- Add unnecessary explanations
- Use conversational filler

### Source Attribution (Beta.245)

**Deterministic sources** (High confidence):
```
Source: internal diagnostic engine (9 deterministic checks)
Source: verified system telemetry (direct system query)
Source: system telemetry (direct system query)
```

**LLM sources** (Medium/Low confidence):
```
🔍 Confidence: Medium | Sources: LLM
```

---

## Formatting Rules

### Markdown Conversion

The `normalize_for_cli()` function applies terminal formatting:
- `**bold**` → ANSI bold
- `[SECTION]` → cyan + bold
- `$ command` → green
- Triple backticks stripped

### Whitespace

- Section headers followed by blank line
- Blank line between issues in [DETAILS]
- Blank line before commands in [COMMANDS]
- No trailing whitespace

---

## Consistency Requirements (Beta.245)

### Cross-Command Consistency

`annactl status` and diagnostic one-shot answers must use:

**Compatible health wording:**
- ✅ "System health: all clear"
- ✅ "System health: no critical issues detected"
- ❌ "System is healthy" vs "All systems nominal" (inconsistent)

**Consistent severity language:**
- "critical issue(s)"
- "warning(s)"
- Not: "severe problems", "alerts", "errors"

**Same issue ordering:**
- By severity (critical → warning → info)
- Then by importance

### Status vs Diagnostic Distinction

**`annactl status`:**
- Condensed view (top 2-3 issues)
- One-line issue summaries
- Clear that full diagnostic exists

**Diagnostic queries:**
- Full expanded report
- All issues with details
- Complete command recommendations

---

## Implementation Files

- **Format generation:** `crates/annactl/src/unified_query_handler.rs:format_diagnostic_report()`
- **CLI rendering:** `crates/annactl/src/output/normalizer.rs:normalize_for_cli()`
- **TUI rendering:** `crates/annactl/src/output/normalizer.rs:normalize_for_tui()`
- **Status command:** `crates/annactl/src/status_command.rs`

---

## Examples

### Complete Diagnostic Report (Healthy)

```
[SUMMARY]
System health: **all clear, no critical issues detected.**

All diagnostic checks passed. System is operating normally.

[COMMANDS]

No actions required - system is healthy.

$ annactl status        # View current status
```

**Source:** `Source: internal diagnostic engine (9 deterministic checks)`

### Complete Diagnostic Report (Issues)

```
[SUMMARY]
System health: **1 critical issue(s) and 2 warning(s) detected.**

Immediate attention required for critical issues.

[DETAILS]

1. ✗ **Failed systemd service**
   The service 'postgresql.service' is in failed state.

   $ systemctl status postgresql.service
   $ journalctl -u postgresql.service -n 50

2. ⚠ **High memory usage**
   RAM usage at 87%. Consider investigating memory-intensive processes.

   $ free -h
   $ ps aux --sort=-%mem | head -10

3. ⚠ **Old package cache**
   Package cache using 2.3 GB. Safe to clean.

   $ paccache -r

[COMMANDS]

$ annactl status
$ journalctl -xe
$ systemctl --failed
```

**Source:** `Source: internal diagnostic engine (9 deterministic checks)`

---

**Document Version:** Beta.245
**Last Updated:** 2025-11-22
**Maintained By:** Anna development team
