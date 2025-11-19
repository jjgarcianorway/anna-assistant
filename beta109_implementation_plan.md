# Beta.109 Implementation Plan
**Date:** 2025-11-19
**Priority:** CRITICAL - Fix Consistency Violations
**Target:** Deliver word-by-word streaming to ALL three modes

## Executive Summary

Beta.108 QA testing revealed a **critical consistency failure**: Beautiful word-by-word streaming only works in one-shot mode. REPL and TUI modes still use blocking LLM calls, violating the user's explicit requirement:

> "ensure that the replies from annactl, TUI or one-off are consistent!!!!!"

**Current State:**
| Mode | Streaming | Status |
|------|-----------|--------|
| One-shot | ✅ `chat_stream()` | Working |
| REPL | ❌ Blocking | **BROKEN** |
| TUI | ❌ Blocking | **BROKEN** |

## Problem Analysis

### REPL Mode Issue (repl.rs:512-517)

**Current Code:**
```rust
// Query LLM (blocking for now, streaming to be added in future update)
match crate::llm_integration::query_llm_with_context(user_message, db).await {
    Ok(response) => {
        // Clear thinking line and show response
        print!("\r{}", " ".repeat(50));  // Clear line
        println!("\r{} {}", "anna:".bright_magenta().bold(), response.white());
    }
```

**Problems:**
- Uses `query_llm_with_context()` which returns full response
- No streaming callback
- User waits 10+ seconds staring at "anna (thinking):"
- Response dumps all at once

### TUI Mode Issue (tui_v2.rs:606-612)

**Current Code:**
```rust
// Call LLM (blocking call in spawn_blocking for async context)
let llm_response = tokio::task::spawn_blocking(move || {
    llm_client.chat(&prompt)
}).await;

match llm_response {
    Ok(Ok(response)) => response.text,
```

**Problems:**
- Uses `llm_client.chat()` which is blocking
- Wrapped in `spawn_blocking` but still no streaming
- TUI freezes during LLM query (or shows loading spinner)
- Response appears all at once

## Solution: Implement Streaming in REPL and TUI

### Part 1: Fix REPL Mode Streaming

**File:** `crates/annactl/src/repl.rs`
**Lines:** 504-527

**Current Implementation:**
```rust
// Beta.108: Beautiful output with colors
use owo_colors::OwoColorize;
use std::io::{self, Write};

// Show thinking indicator
print!("{} ", "anna (thinking):".bright_magenta().dimmed());
io::stdout().flush().unwrap();

// Query LLM (blocking for now, streaming to be added in future update)
match crate::llm_integration::query_llm_with_context(user_message, db).await {
    Ok(response) => {
        // Clear thinking line and show response
        print!("\r{}", " ".repeat(50));  // Clear line
        println!("\r{} {}", "anna:".bright_magenta().bold(), response.white());
    }
    Err(e) => {
        print!("\r{}", " ".repeat(50));  // Clear thinking line
        println!();
        ui.error(&format!("❌ LLM query failed: {}", e));
        ui.info("💡 Try: 'annactl repair' to check LLM setup");
        println!();
    }
}
```

**New Implementation (Beta.109):**
```rust
// Beta.109: Word-by-word streaming like one-shot mode
use owo_colors::OwoColorize;
use std::io::{self, Write};
use anna_common::llm::{LlmClient, ChatMessage, ChatPrompt};

// Show thinking indicator
print!("{} ", "anna (thinking):".bright_magenta().dimmed());
io::stdout().flush().unwrap();

// Build conversation messages
let mut messages = conversation_history.clone();
messages.push(ChatMessage::user(user_message.clone()));

// Build LLM prompt with context
let llm_config = if let Some(db) = db {
    // Get LLM config from database
    db.execute(|conn| {
        let mut stmt = conn.prepare("SELECT model, base_url FROM llm_config ORDER BY updated_at DESC LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let model: String = row.get(0)?;
            let base_url: String = row.get(1)?;
            Ok(anna_common::llm::LlmConfig::local(&base_url, &model))
        } else {
            Ok(anna_common::llm::LlmConfig::default())
        }
    }).await.unwrap_or_else(|_| anna_common::llm::LlmConfig::default())
} else {
    anna_common::llm::LlmConfig::default()
};

let mut llm_client = match LlmClient::from_config(&llm_config) {
    Ok(client) => client,
    Err(e) => {
        print!("\r{}", " ".repeat(50));  // Clear thinking line
        println!();
        ui.error(&format!("❌ Failed to create LLM client: {}", e));
        ui.info("💡 Try: 'annactl repair' to check LLM setup");
        println!();
        return;
    }
};

let prompt = ChatPrompt {
    messages: messages.clone(),
    system_prompt: Some(
        "You are Anna, a helpful Linux system assistant. \
         Provide clear, actionable answers about Linux systems and operations."
            .to_string()
    ),
    temperature: Some(0.7),
    max_tokens: Some(2000),
    conversation_history: None,
};

// Clear thinking line and show streaming response
print!("\r{}", " ".repeat(50));  // Clear thinking indicator
print!("\r{} ", "anna:".bright_magenta().bold());
io::stdout().flush().unwrap();

// Stream response word-by-word
let mut full_response = String::new();
match llm_client.chat_stream(&prompt, &mut |word: &str| {
    print!("{}", word.white());
    io::stdout().flush().unwrap();
    full_response.push_str(word);
}).await {
    Ok(_) => {
        println!();  // Newline after response

        // Add to conversation history
        conversation_history.push(ChatMessage::user(user_message));
        conversation_history.push(ChatMessage::assistant(full_response));
    }
    Err(e) => {
        println!();
        ui.error(&format!("❌ LLM streaming failed: {}", e));
        ui.info("💡 Try: 'annactl repair' to check LLM setup");
        println!();
    }
}
```

**Changes Required:**
1. Import `LlmClient`, `ChatMessage`, `ChatPrompt` from `anna_common::llm`
2. Build `ChatPrompt` with conversation history
3. Get LLM config from database (or use default)
4. Create `LlmClient` from config
5. Clear thinking line before streaming starts
6. Call `chat_stream()` with word-by-word callback
7. Accumulate full response for conversation history
8. Add both user message and assistant response to history

**Files to Modify:**
- `crates/annactl/src/repl.rs` (lines 504-527)

**Testing:**
```bash
# Test REPL streaming
annactl repl
anna> explain how the linux scheduler works
# Should see: word-by-word streaming with colors
```

### Part 2: Fix TUI Mode Streaming

**File:** `crates/annactl/src/tui_v2.rs`
**Lines:** 560-615

**Current Implementation:**
```rust
let llm_client = match LlmClient::from_config(&llm_config) {
    Ok(client) => client,
    Err(_) => {
        return format!("## ⚠ LLM Unavailable\n\n...");
    }
};

// Call LLM (blocking call in spawn_blocking for async context)
let llm_response = tokio::task::spawn_blocking(move || {
    llm_client.chat(&prompt)
}).await;

match llm_response {
    Ok(Ok(response)) => response.text,
    Ok(Err(e)) => format!("## LLM Error\n\nFailed to get response: {:?}", e),
    Err(_) => format!("## LLM Error\n\nTask panicked"),
}
```

**New Implementation (Beta.109):**
```rust
let mut llm_client = match LlmClient::from_config(&llm_config) {
    Ok(client) => client,
    Err(_) => {
        return format!("## ⚠ LLM Unavailable\n\n...");
    }
};

// Stream LLM response asynchronously
let mut accumulated_response = String::new();
let result = llm_client.chat_stream(&prompt, &mut |word: &str| {
    accumulated_response.push_str(word);

    // Send incremental update to TUI (via channel)
    // This requires adding a channel to pass updates back to the UI
    // For Beta.109, we'll accumulate and return full response
    // Beta.110 can add true TUI streaming with partial updates
}).await;

match result {
    Ok(_) => accumulated_response,
    Err(e) => format!("## LLM Error\n\nFailed to stream response: {:?}", e),
}
```

**Changes Required:**
1. Change `let llm_client` to `let mut llm_client`
2. Remove `spawn_blocking` wrapper (no longer needed)
3. Replace `llm_client.chat()` with `llm_client.chat_stream()`
4. Accumulate response in callback
5. Return accumulated response

**Advanced Option (Beta.110):**
For true streaming updates in TUI:
1. Add `tokio::sync::mpsc` channel to send word-by-word updates
2. Update TUI panel in real-time as words arrive
3. Show "streaming..." indicator

**Files to Modify:**
- `crates/annactl/src/tui_v2.rs` (lines 560-615)

**Testing:**
```bash
# Test TUI streaming
annactl tui
# Type: explain how the linux scheduler works
# Should see: response appear (full response in Beta.109, word-by-word in Beta.110)
```

## Implementation Checklist

### Phase 1: REPL Streaming (Priority 1)
- [ ] Read current REPL code (repl.rs:504-527)
- [ ] Implement streaming callback
- [ ] Test with simple question
- [ ] Test with complex question
- [ ] Test error handling
- [ ] Verify conversation history works
- [ ] Test colors and formatting

### Phase 2: TUI Streaming (Priority 2)
- [ ] Read current TUI code (tui_v2.rs:560-615)
- [ ] Implement streaming callback
- [ ] Test with simple question
- [ ] Test with complex question
- [ ] Test error handling
- [ ] Verify TUI doesn't freeze

### Phase 3: Consistency Verification (Priority 3)
- [ ] Test same question in all three modes
- [ ] Verify identical streaming behavior
- [ ] Verify identical color formatting
- [ ] Verify identical timing/responsiveness
- [ ] Document differences (if any)

### Phase 4: QA Validation (Priority 4)
- [ ] Re-run 30-question validation
- [ ] Compare streaming experience across modes
- [ ] Measure response times
- [ ] User acceptance testing
- [ ] Update QA findings document

## Success Criteria

### Must Have (Beta.109)
1. ✅ REPL mode streams word-by-word
2. ✅ TUI mode streams responses (accumulated or word-by-word)
3. ✅ All three modes use `chat_stream()`
4. ✅ Beautiful colors in all modes
5. ✅ Thinking indicators in all modes
6. ✅ No blocking/freezing in any mode

### Should Have (Beta.109)
1. Identical user experience across modes
2. Consistent timing (words appear at same rate)
3. Error handling in all modes
4. Conversation history in REPL

### Nice to Have (Beta.110)
1. True word-by-word TUI updates (channel-based)
2. Streaming progress indicator in TUI
3. Ability to cancel streaming mid-response

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_repl_streaming() {
    // Test REPL streaming callback
    // Verify words accumulate correctly
    // Verify conversation history updated
}

#[tokio::test]
async fn test_tui_streaming() {
    // Test TUI streaming callback
    // Verify response accumulated
    // Verify no blocking
}
```

### Integration Tests
```bash
# Test 1: REPL streaming with LLM
annactl repl
anna> explain process scheduling
# Verify: word-by-word streaming, colors, no blocking

# Test 2: TUI streaming with LLM
annactl tui
# Type: explain process scheduling
# Verify: response appears, no freezing

# Test 3: One-shot (regression test)
annactl explain process scheduling
# Verify: still works as before
```

### Manual QA Checklist
- [ ] Same question in all 3 modes produces consistent output
- [ ] Streaming speed is similar across modes
- [ ] Colors match across modes
- [ ] No crashes or errors
- [ ] Conversation history works in REPL
- [ ] TUI remains responsive

## Risk Assessment

### High Risk
1. **Breaking REPL conversation history**
   - Mitigation: Careful testing with multi-turn conversations
   - Rollback plan: Keep `query_llm_with_context()` as fallback

2. **TUI freezing during streaming**
   - Mitigation: Proper async handling, avoid blocking
   - Rollback plan: Return to blocking with loading indicator

### Medium Risk
1. **Performance degradation**
   - Mitigation: Benchmark before/after
   - Acceptable: <10% slower, not noticeable to user

2. **LLM client compatibility**
   - Mitigation: Test with multiple models (llama3.1:8b, mistral, etc.)
   - Fallback: Graceful error messages

### Low Risk
1. **Color formatting issues**
   - Easy to debug visually
   - Easy to fix

## Rollback Plan

If Beta.109 has critical issues:

1. **Emergency Rollback:**
   ```bash
   # Revert streaming changes
   git revert <beta.109-commit>
   # Rebuild and re-release as Beta.109.1
   ```

2. **Partial Rollback:**
   - Keep one-shot streaming (working)
   - Revert REPL streaming (if broken)
   - Revert TUI streaming (if broken)
   - Release Beta.109.1 with notes

## Timeline Estimate

**Total Effort:** ~4-6 hours

| Phase | Time | Description |
|-------|------|-------------|
| REPL Implementation | 1-2 hours | Code changes, initial testing |
| TUI Implementation | 1-2 hours | Code changes, initial testing |
| Consistency Testing | 1 hour | Test all three modes |
| QA Validation | 1 hour | Re-run validation suite |
| Documentation | 30 min | Update CHANGELOG, QA report |

**Target Release:** 2025-11-19 (same day as Beta.108)

## Post-Release Actions

1. Monitor GitHub issues for streaming problems
2. User feedback on consistency
3. Performance metrics collection
4. Plan Beta.110 improvements (true TUI word-by-word streaming)

## References

- Beta.108 QA Findings: `qa_beta108_findings.md`
- One-shot streaming implementation: `crates/annactl/src/main.rs:266-293`
- REPL current code: `crates/annactl/src/repl.rs:504-527`
- TUI current code: `crates/annactl/src/tui_v2.rs:560-615`
- LLM client interface: `crates/anna_common/src/llm.rs`

## Conclusion

Beta.109 is a **critical consistency fix** that must be implemented to meet user requirements. The streaming interface works beautifully in one-shot mode - we just need to apply the same pattern to REPL and TUI modes.

**Priority:** Implement REPL streaming first (user-facing), then TUI streaming (also user-facing).

**Goal:** Achieve perfect consistency across all three modes as explicitly requested by the user.
