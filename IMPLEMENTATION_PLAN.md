# Anna Implementation Plan: From 810 Versions of Chaos to the Vision

## The Problem

After 810 versions, Anna is:
- A hardcoded pattern matcher pretending to be intelligent
- 5+ overlapping recipe/learning systems that confuse each other
- README promises things that don't exist
- No real learning, no real knowledge base, no real IT department experience

## What's Actually Salvageable

After analyzing the codebase, these components WORK and should be kept:

| Component | Location | Status |
|-----------|----------|--------|
| Translator | `crates/annad/src/translator.rs` | Good - LLM query understanding |
| Probe System | `crates/annad/src/probe_registry.rs` | Good - 50+ probes, clean execution |
| Recipe Schema | `crates/anna-shared/src/learning_engine/recipe.rs` | Good - comprehensive schema |
| Ollama Streaming | `crates/annad/src/ollama_streaming.rs` | Good - word-by-word display |
| Fast Path Logic | `crates/annad/src/recipe_fast_path.rs` | Good - recipes-before-LLM approach |

## What Must Be Thrown Away

- `query_classify/` - 800+ lines of hardcoded patterns that fail on slight variations
- Duplicate recipe systems - `recipe_v2/`, `recipe_v3/`, `recipe_store/`, etc.
- The deterministic router's hardcoded `QueryClass` enum - doesn't scale

## The Core Loop (MUST WORK FIRST)

```
User: "enable syntax highlighting in vim"
         │
         ▼
┌─────────────────┐
│   Translator    │  ← LLM understands natural language
│   (qwen3-vl)    │
└────────┬────────┘
         │ ParsedQuery { intent: "configure_editor",
         │               target: "vim",
         │               action: "enable_syntax_highlighting" }
         ▼
┌─────────────────┐
│  Recipe Store   │  ← Check if Anna knows this
│  (JSON files)   │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
 Found?    Not Found
    │         │
    ▼         ▼
Execute   ┌─────────────────┐
Recipe    │   Specialist    │  ← Sofia (Desktop Admin)
    │     │   (LLM call)    │
    │     └────────┬────────┘
    │              │
    │              ▼
    │     ┌─────────────────┐
    │     │  Learn Recipe   │  ← Anna learns for next time
    │     └────────┬────────┘
    │              │
    ▼              ▼
┌─────────────────────────┐
│      Answer to User     │
│  (with citations)       │
└─────────────────────────┘
```

## Phase 1: Clean Foundation (THIS WEEK)

### Goal: The core loop works for ONE query type

**Step 1: Unify Recipe System**
- Pick `learning_engine/recipe.rs` as THE definition
- Create ONE `RecipeStore` at `/var/lib/anna/recipes/`
- Delete all other recipe code

**Step 2: Create Simple Handler**
```rust
// The ENTIRE flow in one readable function
pub async fn handle(query: &str) -> Answer {
    // 1. Translate
    let parsed = translator::translate(query).await;

    // 2. Check recipes
    if let Some(recipe) = recipes::find(&parsed) {
        return execute_recipe(recipe).await;
    }

    // 3. Ask specialist
    let solution = specialist::solve(&parsed).await;

    // 4. Learn
    if solution.confidence > 0.8 {
        recipes::save(create_recipe(&parsed, &solution));
    }

    solution.answer
}
```

**Step 3: One Working Example**
- "how much RAM do I have?"
- Works end-to-end
- Second time: uses learned recipe, instant

**Deliverable**: Demo showing:
1. First query → specialist solves → Anna learns
2. Same query → recipe found → instant answer

## Phase 2: Specialist System (NEXT WEEK)

### The IT Department (per VISION.md)

```
┌─────────────────────────────────────────────────┐
│               ANNA SERVICE DESK                 │
│                                                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │ SYSTEM  │  │ NETWORK │  │ STORAGE │  ...   │
│  │         │  │         │  │         │        │
│  │ Jr: Wei │  │ Jr: Eva │  │ Jr: Tom │        │
│  │ Sr: Lin │  │ Sr: Raj │  │ Sr: Amy │        │
│  └─────────┘  └─────────┘  └─────────┘        │
└─────────────────────────────────────────────────┘
```

Each specialist:
- Has a name (for dialog display)
- Has a domain (system, network, storage, services, packages, desktop)
- Junior uses light model (qwen3-vl:4b)
- Senior uses deep model (qwen2.5:7b-instruct)
- Same prompt templates, different model

### Escalation Flow
```
User query → Anna → Junior attempts →
  If confident: Answer
  If not: Escalate to Senior → Answer
```

## Phase 3: Knowledge Base (WEEK AFTER)

### Sources (per VISION.md - "the bible")

1. **Arch Wiki** (LOCAL)
   - Clone wiki pages to `/var/lib/anna/wiki/`
   - Index by topic
   - Search before asking specialist

2. **Man Pages**
   - Parse relevant man pages
   - Cache parsed content
   - Use as citations

3. **Help Commands**
   - Run `command --help`
   - Parse and cache
   - Use as evidence

### Citation Requirement
EVERY answer must include:
```
Source: Arch Wiki - Vim#Syntax_highlighting
        man vim(1) - line 234
```

## Phase 4: Learning That Works

### Recipe Storage
```json
{
  "id": "vim-syntax-highlighting-001",
  "pattern": {
    "intent": "configure_editor",
    "keywords": ["vim", "syntax", "highlighting"],
    "target": "vim"
  },
  "solution": {
    "command": "echo 'syntax on' >> ~/.vimrc",
    "explanation": "Enable syntax highlighting in Vim",
    "citations": ["Arch Wiki - Vim#Syntax_highlighting"]
  },
  "stats": {
    "uses": 5,
    "successes": 5,
    "last_used": "2025-12-16"
  }
}
```

### Learning Loop
1. Specialist solves problem with high confidence
2. Anna extracts pattern + solution + citations
3. Recipe saved to store
4. Next time: recipe matches, instant answer

### Stats Tracking
- Tickets by Anna vs specialists
- Recipes learned over time
- Success rates
- This IS the RPG system (Anna gains XP by learning)

## Phase 5: The Experience (FINAL)

### Internal Comms (fly-on-the-wall)
```
--- internal comms ---
  [0.2s] Anna: Checking recipes... not found
  [0.3s] Anna to Sofia (Desktop): Need help with Vim syntax highlighting
  [0.8s] Sofia: Checking Arch Wiki... found it
  [1.2s] Sofia to Anna: Add 'syntax on' to ~/.vimrc (high confidence)
  [1.3s] Anna: Learning this for next time...
```

### Real-time Streaming
- Word-by-word display (ollama_streaming.rs already works)
- Progress indicators
- Stage visibility

### Hollywood UI
- True color
- Clean, professional
- No icons
- Spinning animations while waiting

## Implementation Order

1. **TODAY**: Create simplified handler.rs with core loop
2. **DAY 2**: Get "how much RAM" working end-to-end
3. **DAY 3**: Add recipe learning and verify it works
4. **DAY 4**: Add second example "disk space"
5. **WEEK 2**: Specialist system with names
6. **WEEK 3**: Knowledge base integration
7. **WEEK 4**: Stats/RPG system
8. **WEEK 5**: Internal comms display

## Success Criteria

### Phase 1 Complete When:
- [ ] One query works: ask → specialist solves → Anna learns
- [ ] Same query second time: instant from recipe
- [ ] No hardcoded patterns for this query

### Full Vision Complete When:
- [ ] "enable syntax highlighting in vim" works like VISION.md example
- [ ] Internal comms shows the dialog
- [ ] Citations from Arch Wiki
- [ ] Stats show tickets by Anna vs specialists
- [ ] Anna's recipe count grows over time

## What This Means for Current Code

### Keep
- `translator.rs` - works
- `probe_registry.rs` - works
- `ollama_streaming.rs` - works
- `learning_engine/recipe.rs` - good schema

### Refactor
- `handler.rs` - simplify to core loop
- `ticket_loop.rs` - extract specialist logic

### Delete
- `query_classify/` - hardcoded patterns
- `deterministic/` - hardcoded routes
- `recipe_v2/`, `recipe_v3/` - duplicate systems
- Any file doing substring pattern matching

## The Mindset Shift

**BEFORE**: "Let me add another hardcoded pattern for this query variation"
**AFTER**: "Let the translator understand it, let the specialist solve it, let Anna learn it"

The entire classification system (810 versions of it) was the wrong approach. The LLM IS the classifier. The translator already does this. We just weren't using it correctly.

---

*This plan created after deep analysis of codebase and VISION.md*
*Ready for implementation approval*
