# Evaluation Rules for Anna QA Suite

## Overview

This document codifies the rules used to evaluate Anna's answers against golden reference answers. Every verdict (PASS/PARTIAL/FAIL) must be based on these explicit, testable criteria.

## Scoring Formula

```
score = 1.0 - (failed_checks / total_checks)

where total_checks =
  + number of required_commands
  + number of required_files
  + number of required_concepts
  + 1 (if warnings are required)

and failed_checks =
  + number of missing required_commands
  + number of missing required_files
  + number of missing required_concepts
  + 1 (if warnings required but not present)
```

## Verdict Thresholds

### PASS (✅)

All of the following must be true:

1. **Score ≥ 0.9** (90% of checks passed)
2. **Zero critical issues**
3. **All required commands mentioned**
4. **All required files/paths mentioned**
5. **All key concepts covered**
6. **Safety warnings present** (if golden answer specifies warnings)
7. **No error patterns** in output (see below)
8. **Output length ≥ 50 characters**

Example:
```
Question: How do I enable a systemd service?
Anna output contains:
✅ "systemctl enable <service>"
✅ "systemctl start <service>"
✅ Mentions systemd
✅ Output is 200+ characters
✅ No errors
→ Verdict: PASS
```

### PARTIAL (⚠️)

Either of the following:

1. **Score ≥ 0.6** (60% of checks passed) AND **no error patterns**
2. **Most critical steps present** but missing minor details
3. **Correct general approach** but too generic or incomplete

Example:
```
Question: How do I install from AUR?
Anna output contains:
✅ "git clone"
✅ "makepkg"
❌ Missing "review PKGBUILD" warning
❌ Missing "base-devel" requirement
→ Score: 0.5, Verdict: PARTIAL
```

### FAIL (❌)

Any of the following:

1. **Score < 0.6** (less than 60% of checks passed)
2. **Error patterns detected** (see below)
3. **Wrong commands for Arch Linux** (e.g., apt-get instead of pacman)
4. **Dangerous operations without warnings**
5. **Output too short** (< 50 characters)
6. **Timeout or crash**
7. **Hallucinated information** (fake commands, wrong file paths)

Example:
```
Question: How do I install a package?
Anna output:
"apt-get install package"
→ Wrong distro! Verdict: FAIL
```

### SKIP (⏭️)

Used when:
- No golden answer available yet
- Question malformed or ambiguous
- Testing infrastructure issue

## Error Patterns

Anna's output is checked for these error patterns (case-insensitive):

- `error:`
- `failed:`
- `cannot`
- `unknown command`
- `not found`
- `planner error`
- `llm call failed`
- `timeout`
- `http timeout`
- `failed to parse`

If any error pattern is detected, the verdict is automatically **FAIL** regardless of score.

## Required Elements Check

### Required Commands

Commands must appear in Anna's output either:
- With or without `sudo` prefix
- In code blocks or inline
- In any order

Example matches:
```
Required: "systemctl enable service"
✅ Matches: "systemctl enable service"
✅ Matches: "sudo systemctl enable service"
✅ Matches: "`systemctl enable <service>`"
❌ No match: "enable the service"  (missing systemctl)
```

### Required Files

File paths must appear exactly as specified:

```
Required: "/etc/systemd/network/*.network"
✅ Matches: "/etc/systemd/network/20-wired.network"
✅ Matches: "edit /etc/systemd/network/10-eth0.network"
❌ No match: "/etc/systemd/network"  (missing .network extension)
```

### Required Concepts

Concepts must be mentioned (case-insensitive):

```
Required: "PKGBUILD"
✅ Matches: "Review the PKGBUILD"
✅ Matches: "check PKGBUILD for security"
❌ No match: "review the build script"  (PKGBUILD not mentioned)
```

## Safety Warning Requirements

If golden answer includes `warnings` field, Anna's output must contain **at least one** of these safety keywords:

- backup
- warning
- careful
- caution
- risk
- danger
- critical
- important
- note

Example:
```
Golden: "warnings": ["Backup /etc/fstab before editing"]
Anna must include: "IMPORTANT: Backup /etc/fstab first"
```

## Arch Linux Specific Rules

### Distro-Specific Commands

**FAIL immediately if Anna suggests:**
- `apt`, `apt-get`, `dpkg` (Debian/Ubuntu)
- `yum`, `dnf` (Fedora/RHEL)
- `zypper` (openSUSE)
- `emerge` (Gentoo)

**PASS only if Anna uses:**
- `pacman` (official repos)
- `makepkg` (AUR)
- `systemctl` (systemd)
- `journalctl` (logging)

### Config File Paths

**Must use Arch-specific paths:**
- ✅ `/etc/systemd/network/` (networkd)
- ❌ `/etc/network/interfaces` (Debian)
- ✅ `/etc/pacman.conf` (pacman)
- ❌ `/etc/apt/sources.list` (Debian)

## Edge Cases

### Generic vs Specific

**PARTIAL**: Anna gives correct but too generic advice
```
Question: How do I configure static IP with systemd-networkd?
Anna: "Edit network config files and restart networking"
→ Too generic! Missing specific files and commands
→ Verdict: PARTIAL
```

**FAIL**: Anna gives wrong specific advice
```
Question: How do I configure static IP with systemd-networkd?
Anna: "Edit /etc/network/interfaces"
→ Wrong distro/method!
→ Verdict: FAIL
```

### Multiple Valid Approaches

If multiple approaches are valid (e.g., NetworkManager vs systemd-networkd), and the question doesn't specify:

- Anna must pick ONE approach and explain it fully
- Anna can mention alternatives briefly
- Mixing approaches without clarification is PARTIAL

### Incomplete but Correct

If Anna's answer is correct but incomplete:
- Missing validation steps: PARTIAL
- Missing optional flags: PARTIAL
- Missing core steps: FAIL

## Human Review Override

Automated evaluation can be overridden by human review in these cases:

1. **False Positive PASS**: Human finds critical issue that automation missed
2. **False Negative FAIL**: Human determines Anna's alternate approach is valid
3. **Ambiguous Question**: Question itself is flawed or has multiple valid interpretations

All overrides must be documented in `HUMAN_REVIEW_SAMPLE.md` with justification.

## Updating These Rules

When updating evaluation rules:

1. Document the change in git commit
2. Re-run affected tests
3. Update `HUMAN_REVIEW_SAMPLE.md` if verdicts change
4. Bump the rule version in this file

**Current Rule Version**: 1.0 (2025-11-20)

## Examples

### Example 1: Clear PASS

```
Question: How do I check systemd service status?
Golden required_commands: ["systemctl status"]
Anna output: "Run 'systemctl status <service>' to check status"

Checks:
✅ Contains "systemctl status"
✅ Output > 50 chars
✅ No error patterns
→ Score: 1.0, Verdict: PASS
```

### Example 2: Clear FAIL

```
Question: How do I install nginx?
Golden required_commands: ["pacman -S nginx"]
Anna output: "Error: Planner LLM call failed"

Checks:
❌ Contains error pattern: "error"
❌ Missing required command
❌ Output too short
→ Score: 0.0, Verdict: FAIL
```

### Example 3: PARTIAL

```
Question: How do I configure static IP?
Golden required: ["systemd-networkd", "/etc/systemd/network/*.network", "systemctl restart"]
Anna output: "Edit network config in /etc/systemd/network/ and restart the network service"

Checks:
✅ Mentions systemd-networkd path
⚠️  Missing specific file extension (.network)
⚠️  Says "restart network service" not "systemctl restart systemd-networkd"
→ Score: 0.67, Verdict: PARTIAL
```

## Continuous Improvement

As we discover patterns where the automated evaluation is wrong:

1. Add test case to this document
2. Update evaluation logic in `run_qa_suite.py`
3. Re-run affected questions
4. Update human review sample

**The goal is high correlation between automated verdicts and human expert judgment.**
