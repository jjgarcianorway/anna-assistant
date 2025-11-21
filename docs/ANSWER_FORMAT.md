# Answer Format Specification

**Version**: Beta.207
**Date**: 2025-11-21

---

## Overview

All answers from Anna must follow a consistent structure across CLI and TUI modes. This ensures predictable, parseable, and user-friendly responses.

---

## Standard Answer Format

Every answer must follow this structure:

```
[SUMMARY]
One or two lines maximum.

[DETAILS]
...detailed explanation...

[COMMANDS]
...commands exactly as they must be executed...
```

### Format Rules

1. **SUMMARY section** (required)
   - Maximum 2 lines
   - States the key finding or result
   - No technical jargon unless necessary
   - Example: "Your system has 16 GB RAM with 8.2 GB currently in use (51%)."

2. **DETAILS section** (optional)
   - Extended explanation
   - Context and reasoning
   - Additional information not critical to the answer
   - Example: "RAM usage is normal for a development workstation. High usage may indicate memory leaks or too many applications running."

3. **COMMANDS section** (optional)
   - Executable commands only
   - Each command on its own line
   - Prefixed with `$ ` for user commands
   - Prefixed with `# ` for root commands
   - Example:
     ```
     $ free -h
     $ ps aux --sort=-%mem | head -10
     ```

---

## Answer Types

### Deterministic Answers (from telemetry)

**Structure**: SUMMARY only or SUMMARY + DETAILS

**Example**:
```
[SUMMARY]
Your CPU is Intel Core i7-9700K with 8 cores. Current load: 2.45 (1-min avg).

[DETAILS]
Load average below 8.0 indicates healthy CPU utilization. The 1-minute average represents recent activity.
```

**No COMMANDS section** - telemetry-based answers are informational, not actionable.

### Template Answers (from query_handler)

**Structure**: SUMMARY + COMMANDS or SUMMARY + DETAILS + COMMANDS

**Example**:
```
[SUMMARY]
Running systemctl to check failed services.

[COMMANDS]
$ systemctl --failed --no-pager
```

### Recipe Answers (from deterministic recipes)

**Structure**: SUMMARY + DETAILS + COMMANDS

**Example**:
```
[SUMMARY]
Installing Docker and Docker Compose with proper user permissions.

[DETAILS]
This will install Docker CE from official repositories and add your user to the docker group. You'll need to log out and back in for group changes to take effect.

[COMMANDS]
$ sudo pacman -S docker docker-compose
$ sudo systemctl enable docker.service
$ sudo systemctl start docker.service
$ sudo usermod -aG docker $USER
```

### LLM-Generated Answers (Tier 3/4 fallback)

**Structure**: SUMMARY + DETAILS or SUMMARY + DETAILS + COMMANDS

**Example**:
```
[SUMMARY]
Your GPU drivers need to be reinstalled after kernel update.

[DETAILS]
Kernel updates can break NVIDIA drivers if they're not DKMS-based. You'll need to reinstall the nvidia package to rebuild the kernel module.

[COMMANDS]
$ sudo pacman -S nvidia nvidia-utils
$ sudo mkinitcpio -P
$ reboot
```

---

## Fallback Messages

When telemetry is missing or commands fail, use this structure:

```
[SUMMARY]
Unable to determine [specific information].

[DETAILS]
Reason: [specific technical reason]
Missing: [list of missing telemetry keys or failed commands]

To collect this information:
[COMMANDS]
$ [command to gather telemetry]
$ [command to install missing tool]
```

**Example**:
```
[SUMMARY]
Unable to determine GPU VRAM usage.

[DETAILS]
Reason: nvidia-smi command not found
Missing: NVIDIA driver tools

To collect this information:
[COMMANDS]
$ sudo pacman -S nvidia-utils
$ nvidia-smi --query-gpu=memory.used,memory.total --format=csv
```

---

## Markdown Normalization

### Allowed Markdown

- **Bold**: `**text**` for emphasis
- *Italic*: `*text*` for secondary emphasis
- `Code`: `` `text` `` for inline commands or values
- Code blocks: ` ```language\n...\n``` ` for multi-line commands
- Lists: `- item` or `1. item` for structured data
- Headers: `## Header` (H2 only, H1 reserved for section markers)

### Forbidden Markdown

- H1 headers (`#`) - conflicts with section markers
- Horizontal rules (`---`) - reserved for section separators
- Blockquotes (`>`) - not needed for answer format
- Tables - too complex for terminal rendering
- Emojis - not allowed per project rules

### Whitespace Rules

1. **No trailing whitespace** at end of lines
2. **Single blank line** between sections
3. **No blank lines** within SUMMARY section
4. **Single blank line** between list items only if they contain multiple lines

---

## CLI vs TUI Rendering

### CLI Rendering

- Output formatted text directly to stdout
- Use `owo-colors` for syntax highlighting:
  - Section markers: Cyan bold
  - Commands: Green
  - Errors: Red bold
  - Warnings: Yellow

### TUI Rendering

- Parse answer format before rendering
- Convert to ratatui `Paragraph` with `Text` spans
- Apply consistent styling:
  - Section markers: Bold
  - Commands: Monospace with background
  - Emphasis: Italic
  - Links: Underline

**No double rendering** - format once, render once.

---

## Error States

### Telemetry Collection Failure

```
[SUMMARY]
System telemetry unavailable.

[DETAILS]
Reason: Failed to read /proc/meminfo
Error: Permission denied

This may indicate:
- Insufficient permissions
- System misconfiguration
- Corrupted procfs

[COMMANDS]
$ ls -la /proc/meminfo
$ sudo cat /proc/meminfo
```

### Invalid Recipe JSON

```
[SUMMARY]
Action plan generation failed.

[DETAILS]
Reason: LLM returned invalid JSON structure
Validation errors:
- Missing required field: command_plan
- Invalid risk_level value: "super-safe"

Retry with simpler query or use deterministic templates.
```

### Command Execution Failure

```
[SUMMARY]
Failed to execute: systemctl --failed

[DETAILS]
Reason: systemctl command not found
Missing: systemd package

[COMMANDS]
$ pacman -Qi systemd
$ sudo pacman -S systemd
```

---

## Testing Format Compliance

### Test Cases

1. **Deterministic handler test**
   - Input: "how much RAM do I have?"
   - Expected: SUMMARY only, no COMMANDS
   - Verify: Same output in CLI and TUI

2. **Template handler test**
   - Input: "check failed services"
   - Expected: SUMMARY + COMMANDS
   - Verify: Command executable without modification

3. **Recipe handler test**
   - Input: "install docker"
   - Expected: SUMMARY + DETAILS + COMMANDS
   - Verify: Commands match recipe exactly

4. **Fallback test**
   - Input: Query for missing telemetry
   - Expected: SUMMARY + DETAILS with error explanation + COMMANDS to fix
   - Verify: Proposed commands actually collect the missing data

### Validation Script

```bash
# Test answer format parsing
annactl "how much RAM?" | grep -E '^\[SUMMARY\]$'
annactl "how much RAM?" | grep -E '^\[DETAILS\]$' && echo "FAIL: Telemetry should not have DETAILS"

# Test CLI vs TUI consistency
cli_answer=$(annactl "check swap")
# Compare with TUI output manually
```

---

## Implementation Notes

### Code Locations

- **Format parsing**: `crates/annactl/src/answer_format.rs` (new module)
- **Format generation**: All handlers in `unified_query_handler.rs`
- **Format rendering**: `crates/annactl/src/tui/render.rs` and `llm_query_handler.rs`

### Migration Path

1. Create `answer_format.rs` module with:
   - `AnswerFormat` struct
   - `parse_answer()` function
   - `format_answer()` function
   - `validate_format()` function

2. Update all handlers to emit structured format:
   - `try_answer_from_telemetry()` - SUMMARY only
   - Template handlers - SUMMARY + COMMANDS
   - Recipe handlers - SUMMARY + DETAILS + COMMANDS
   - LLM handlers - SUMMARY + DETAILS + optional COMMANDS

3. Update renderers:
   - CLI: Parse format, apply colors, output
   - TUI: Parse format, convert to ratatui Text, render

4. Add validation:
   - Unit tests for format parsing
   - Integration tests for CLI/TUI consistency
   - QA tests for all query types

---

## Examples

### Query: "what's my CPU?"

**Deterministic Handler Output**:
```
[SUMMARY]
Your CPU is Intel Core i7-9700K with 8 cores. Current load: 2.45 (1-min avg).
```

### Query: "check swap"

**Template Handler Output**:
```
[SUMMARY]
Running swapon to check swap configuration.

[COMMANDS]
$ swapon --show
```

### Query: "install docker"

**Recipe Handler Output**:
```
[SUMMARY]
Installing Docker and Docker Compose with proper user permissions.

[DETAILS]
This will install Docker CE from official repositories and add your user to the docker group. You'll need to log out and back in for group changes to take effect.

[COMMANDS]
$ sudo pacman -S docker docker-compose
$ sudo systemctl enable docker.service
$ sudo systemctl start docker.service
$ sudo usermod -aG docker $USER
```

### Query: "show GPU VRAM" (nvidia-smi not installed)

**Fallback Output**:
```
[SUMMARY]
Unable to determine GPU VRAM usage.

[DETAILS]
Reason: nvidia-smi command not found
Missing: NVIDIA driver tools

To collect this information:

[COMMANDS]
$ sudo pacman -S nvidia-utils
$ nvidia-smi --query-gpu=memory.used,memory.total --format=csv
```

---

## Summary

This format ensures:
- Consistent structure across all answer types
- Parseable output for automation
- Clear separation of information types
- User-friendly presentation
- CLI/TUI consistency
- Deterministic validation

All handlers must comply with this format starting in Beta.207.
