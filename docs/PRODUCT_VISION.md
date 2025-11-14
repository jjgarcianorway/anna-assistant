# Anna Product Vision

## What is Anna?

Anna is a **local system and desktop caretaker** for Arch Linux.

She continuously analyzes your machine - hardware, software, services, and configuration - and helps you fix and improve everything in the simplest possible way.

## Who is Anna for?

- Individual Arch Linux users who want their system to "just work"
- Anyone who wants a knowledgeable sysadmin companion watching their back
- People who prefer talking to their computer in natural language

## Core Responsibilities

1. **Inspect** - Continuously monitor hardware, installed software, services, and configuration
2. **Detect** - Find concrete problems and risks on this specific machine
3. **Fix or Guide** - Offer to fix issues automatically, or explain exactly what to do
4. **Stay Silent** - When everything is OK, don't create noise

## What Anna is NOT

- ❌ Not a general monitoring platform (use Prometheus for that)
- ❌ Not an AI chatbot (she doesn't have conversations)
- ❌ Not a remote management server (she runs locally)
- ❌ Not a research playground (no exposed experimental features)

## UX Principles

### 1. Few Commands, High Value

80% of value comes from 3-4 commands:
- `annactl daily` - Quick morning health check
- `annactl status` - Detailed system snapshot
- `annactl repair` - Interactive problem fixing
- `annactl init` - Optional advanced setup

### 2. Two-Second Answer

Running `annactl daily` should give you an instant answer to: **"Is my system OK?"**

If yes: One line saying so.
If no: Top 1-3 issues with clear next steps.

### 3. Everything Explained in Human Terms

No internal jargon. No "Phase 3.7 Predictive Intelligence Engine".

Instead:
- "Your disk is 96% full. Run this command to free 30GB."
- "TLP is installed but not enabled. Your battery life could be better."
- "3 package updates available including a kernel security fix."

### 4. Opinionated and Confident

Anna makes recommendations based on Arch Wiki best practices. She doesn't ask "maybe you want to consider..." - she says "Here's what's wrong and here's the fix."

If the user disagrees, they can ignore it. But the default should be clear guidance, not wishy-washy options.

## The Caretaker Loop

Every piece of Anna's intelligence feeds into this loop:

```
1. Detect concrete issue on this machine
   ↓
2. Explain what's wrong in plain English
   ↓
3. Offer to fix it OR guide the user step-by-step
   ↓
4. Execute fix (with confirmation) OR explain why not
   ↓
5. Learn from the outcome to improve future detection
```

If a module doesn't end up in this loop, it's dead weight and should be removed.

## Product Guardrail

**Hard rule for future development:**

Any new feature must clearly answer:
1. What specific problem on the user's machine does it detect or fix?
2. How does it appear to the user through `daily`, `status`, `repair`, or `init`?

If you can't answer both questions, don't build it.

## Success Metrics

Anna is successful when:

1. Users run `annactl daily` every morning and trust the answer
2. When problems occur, `annactl repair` fixes them without the user needing to understand internals
3. Users recommend Anna to other Arch users as "essential"
4. The documentation is so clear that new users are productive in under 5 minutes

## User Experience Principles

### First Contact

On first contact Anna performs a one-time deep scan of the machine and presents a short list of concrete issues and improvements. **The user should never have to discover the initial setup command by reading documentation.**

When you run `annactl` or `annactl daily` for the first time:
- Anna automatically detects this is the first run
- Shows a friendly welcome message
- Runs a deeper system scan
- Presents prioritized findings with clear actions
- Remembers the scan results for future comparisons

### Daily Use

After first run, Anna is extremely lightweight:
- `annactl daily` - two-second morning check (the core workflow)
- `annactl status` - detailed view when you need it
- `sudo annactl repair` - fix issues interactively
- `annactl upgrade` - keep Anna updated

## What This Means for Development

- **Stop building infrastructure for infrastructure's sake**
- **Start every feature with "what problem does this solve for the user?"**
- **Keep the command surface small** - hide complexity behind `--help --all`
- **Write docs for users, not archaeologists** - no one cares about "Phase 3.7"
- **Make the default case trivial** - `annactl daily` should be all most users need
- **First run experience matters** - users form opinions in the first 60 seconds

## The Vision in One Sentence

> Anna is the knowledgeable sysadmin friend who silently watches your Arch machine, spots problems before they get bad, and either fixes them or tells you exactly what to do - with as little ceremony as possible.

---

Everything else is implementation detail.
