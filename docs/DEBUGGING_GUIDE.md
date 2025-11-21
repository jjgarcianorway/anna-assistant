# Anna Assistant Debugging Guide

**Version**: Beta.208
**Last Updated**: 2025-01-21

This guide provides comprehensive instructions for debugging Anna Assistant using environment variables and diagnostic tools.

## Environment Variables

Anna supports debug environment variables to enable detailed logging for specific subsystems. These variables are designed to help developers and QA engineers diagnose issues without modifying code.

### Available Debug Flags

#### ANNA_DEBUG_NORMALIZATION

Enables detailed logging for the canonical answer normalization pipeline.

**Usage:**
```bash
export ANNA_DEBUG_NORMALIZATION=1
annactl "what is my hostname"
```

**Output:**
```
[DEBUG] normalize_for_cli: Input length = 156 bytes
[DEBUG] normalize_markdown: Starting normalization
[DEBUG] normalize_markdown: Trimmed 8 lines
[DEBUG] normalize_markdown: Blocks normalized
[DEBUG] normalize_blocks: 8 lines → 7 lines
[DEBUG] normalize_for_cli: Output length = 142 bytes
```

**Use Cases:**
- Verify that answer text is being normalized correctly
- Diagnose formatting inconsistencies between CLI and TUI modes
- Debug blank line collapsing or section spacing issues
- Trace normalization pipeline execution

**Module**: `crates/annactl/src/output/normalizer.rs`

---

#### ANNA_DEBUG_LLM

Enables detailed logging for LLM interactions, including prompts and responses.

**Usage:**
```bash
export ANNA_DEBUG_LLM=1
annactl "install docker"
```

**Expected Output:**
```
[DEBUG LLM] Sending prompt to llama3.2:3b (538 tokens)
[DEBUG LLM] System prompt: You are Anna, an Arch Linux assistant...
[DEBUG LLM] User query: install docker
[DEBUG LLM] Response received (245 tokens, 1.2s)
[DEBUG LLM] Response text: [SUMMARY]\nDocker is available in official repos...
```

**Use Cases:**
- Debug LLM connection issues
- Verify prompt engineering and system prompts
- Analyze token usage and response times
- Diagnose hallucination or incorrect reasoning

**Module**: `crates/annactl/src/llm_integration.rs`

---

#### ANNA_DEBUG_TUI

Enables detailed logging for TUI rendering, events, and state management.

**Usage:**
```bash
export ANNA_DEBUG_TUI=1
annactl  # Interactive TUI mode
```

**Expected Output:**
```
[DEBUG TUI] Initializing TUI (terminal size: 120x40)
[DEBUG TUI] Rendering frame 1 (3 conversation items)
[DEBUG TUI] Scroll offset: 0, max scroll: 15, visible lines: 35
[DEBUG TUI] Input event: Key(Char('h'))
[DEBUG TUI] State update: input buffer changed (5 chars)
```

**Use Cases:**
- Debug TUI rendering issues (scroll jitter, off-by-one errors)
- Diagnose input handling problems
- Trace state transitions in TUI mode
- Verify frame rendering performance

**Module**: `crates/annactl/src/tui/`

---

### Combining Debug Flags

You can enable multiple debug flags simultaneously:

```bash
export ANNA_DEBUG_NORMALIZATION=1
export ANNA_DEBUG_LLM=1
export ANNA_DEBUG_TUI=1
annactl
```

Debug output is sent to **stderr**, so you can separate it from normal output:

```bash
# Capture only debug output
ANNA_DEBUG_LLM=1 annactl "free storage" 2>debug.log

# Show normal output in terminal, save debug to file
ANNA_DEBUG_NORMALIZATION=1 annactl "list packages" 2>>debug.log
```

---

## Diagnostic Commands

### Check System Health

```bash
annactl status
```

Provides comprehensive system health report including:
- Anna version
- LLM daemon status (ollama)
- Model availability (llama3.2:3b)
- CPU and RAM usage
- Disk space
- Network connectivity

### Verify Binary Integrity

```bash
annactl --version
sha256sum /usr/local/bin/annactl
```

Compare checksum against official release (see `SHA256SUMS` in GitHub releases).

### Test LLM Connection

```bash
# Direct ollama test
ollama list
ollama run llama3.2:3b "test"

# Anna LLM integration test
ANNA_DEBUG_LLM=1 annactl "what is 2+2"
```

### Verify Normalization

```bash
# Test normalization with debug output
ANNA_DEBUG_NORMALIZATION=1 annactl "free disk space"
```

Expected: Debug output showing byte counts and normalization steps.

---

## Common Issues and Solutions

### Issue: CLI and TUI show different output

**Diagnosis:**
```bash
ANNA_DEBUG_NORMALIZATION=1 annactl "install nginx"  # CLI mode
ANNA_DEBUG_NORMALIZATION=1 annactl  # TUI mode, then ask "install nginx"
```

Compare normalization logs. Both should show identical byte counts.

**Solution:**
- Ensure `unified_query_handler.rs` calls `normalizer::normalize_for_cli()`
- Verify TUI receives normalized text from unified handler
- Check for additional text processing in TUI rendering pipeline

### Issue: LLM returns incorrect answers

**Diagnosis:**
```bash
ANNA_DEBUG_LLM=1 annactl "your query here"
```

Check:
1. Is the prompt being sent correctly?
2. What is the system prompt content?
3. Is telemetry data included in the prompt?
4. What does the raw LLM response look like?

**Solution:**
- Verify system prompt in `system_prompt_v2.rs` or `system_prompt_v3_json.rs`
- Check telemetry data completeness
- Consider fine-tuning or adjusting temperature/top_p parameters

### Issue: TUI scroll jitter or off-by-one errors

**Diagnosis:**
```bash
ANNA_DEBUG_TUI=1 annactl
```

Monitor scroll offset calculations and visible line counts.

**Solution:**
- Check `draw_conversation_panel()` in `tui/render.rs`
- Verify text wrapping logic in `tui/utils.rs`
- Ensure scroll offset clamping is correct

---

## Performance Profiling

### Measure Build Time

```bash
cargo clean
time cargo build --release
```

Expected: ~45-60 seconds on modern hardware.

### Measure Startup Time

```bash
time annactl "what is my hostname"
```

Expected: <2 seconds for simple queries.

### Measure LLM Response Time

```bash
ANNA_DEBUG_LLM=1 annactl "complex query"
```

Check debug output for response time. Expected: 1-3 seconds for llama3.2:3b.

---

## Test Coverage

### Run All Tests

```bash
cargo test --workspace
```

Expected: 300+ tests passing (77 recipe tests + anna_common tests).

### Run Specific Module Tests

```bash
cargo test -p anna_common action_plan
cargo test -p annactl normalizer
cargo test -p annactl recipes::rust
```

### Test with Debug Flags

```bash
ANNA_DEBUG_NORMALIZATION=1 cargo test normalizer -- --nocapture
```

---

## Reporting Bugs

When reporting bugs, include:

1. **Anna version**: `annactl --version`
2. **System info**: `uname -a`
3. **Debug logs**: Output from relevant `ANNA_DEBUG_*` flags
4. **Steps to reproduce**: Exact commands used
5. **Expected vs actual behavior**
6. **System health**: Output from `annactl status`

**Submit issues at**: https://github.com/jjgarcianorway/anna-assistant/issues

---

## Additional Resources

- **Architecture docs**: `docs/ARCHITECTURE_BETA_200.md`
- **CHANGELOG**: `CHANGELOG.md`
- **Source code**: https://github.com/jjgarcianorway/anna-assistant

---

**Note**: Debug flags are designed for development and diagnostic purposes. They should not be enabled in production environments as they significantly increase log verbosity and may impact performance.
