# Archive - Legacy Design Documents

This directory contains historical design documents and specifications that have been superseded by newer implementations.

These files are kept for historical reference and learning, but **are not used in production**.

---

## Files

### PROMPT_V2_SYSTEM.md (Beta.142)
**Status:** Superseded by JSON Runtime Contract (Beta.143)

**What it was:** Complete rewrite of Anna's LLM prompting system with 17 strict rules enforcing disciplined systems engineering thinking.

**Why archived:** V2 used **markdown output**, but production requires **JSON output** for:
- Validation (Rust structs)
- Parsing (serde_json)
- Execution (clear command → action mapping)
- TUI display (structured data)

**What survived:**
- All 17 principles and rules → Integrated into v3 JSON prompt
- Telemetry-first reasoning → Core of v3
- Environment auto-detection → Detailed in v3
- Safety philosophy → Enforced in v3
- Risk classification → Per-command in v3

**Replacement:** See `docs/runtime_llm_contract.md` for the production JSON contract

---

## Why Archive Instead of Delete?

1. **Historical Context** - Shows evolution of Anna's intelligence architecture
2. **Learning** - Good design principles worth preserving
3. **Reference** - V2's 17 rules informed v3's design
4. **Accountability** - Transparent development history

---

## Current Production System

**See:** `docs/runtime_llm_contract.md`
**Code:** `crates/annactl/src/system_prompt_v3_json.rs`

---

**Last Updated:** 2025-11-20 (Beta.143)
