# Anna Vision Document

This is the authoritative vision for Anna. All development must align with this document.

## The User Experience

Anna is experienced as a full IT department living inside the user's computer. The user perceives interactions like being a fly on the wall, watching the workflows needed to solve a case, like in real life.

### Example Interaction

```
lhoqvso@razorback: annactl

Hello lhoqvso,

It's been a while since you reached out! (Almost a day!)

Here are a few updates since the last time:

Your boot time has increased by 2 seconds. This seems to be mainly due to the
trim.service we enabled. But don't worry! Trim is enabled for a reason. It helps
maintain a healthy hard drive and improves access to data. It's worth losing a
couple of seconds, isn't it?

There are no warnings or errors detected, and nothing critical.

I noticed that you've been using Vim more frequently lately compared to Nano
(which was the other way around before). Maybe you're learning how to use it?
If you're interested, I can offer some tips and improvements. Just let me know!

But I think you're asking me something specific, aren't you?

You: Yes, Anna, I'd like to know if I can enable syntax highlighting in my editor.

Anna: Sure, let me check. I think you're referring to Vim (since it's more
commonly used lately?). Please confirm.

You: Yes, Vim, please.

Anna: Sure, that shouldn't be too complicated, but I'm not sure how to do it.
So, I'll create a ticket with the appropriate team and get back to you.

--- Internal communication ---

Anna to Desktop Administrator: Hey! My user wants to enable syntax highlighting
in Vim (confirmed). I've checked, and it's installed and used the most recently.
According to my knowledge, Vim relies on a .vimrc file in the user's home folder.
The file exists, but I'm not sure how to enable it. Can you help me?
Case number CN-0003-05122025

Desktop Administrator to Anna: Sure, you've done your homework. Yes, the
information about the config file is correct, but you should add the following
to that file: "syntax on" (without the quotes).

This answer is reliable and risk-free.

Anna to Desktop Administrator: Thank you! Since you've confirmed its high
reliability, I'll store it in my knowledge base so I don't have to bother
you again next time.

Anna: I apologize for the waiting time. I've checked and it should be quite
straightforward. I need to run a command:

printf '\nsyntax enable\n' >> "$HOME/.vimrc"

Would you like me to execute it for you? Please confirm.

You: Yes, please. Thanks a lot.

Anna: The command has been successfully executed.

You: Thank you, Anna. :)
```

## annactl status Requirements

Anna's status command must show:

### Core Information
- Installed version of software
- Available GitHub version (verify all necessary files present)
- Update check pace
- Last check time and next scheduled check

### Permissions & Environment
- User groups
- Folder permissions
- Daemon status
- Ollama status
- List of helpers (tools Anna discovered that provide more data)
  - Show whether installed by Anna or by user
  - Anna-installed helpers are removed on uninstall (with confirmation)
  - User-installed helpers are NOT removed on uninstall

### LLM Configuration
- Current LLMs installed by Anna
- Which specialist uses which LLM

### Errors & Warnings
- Any errors or warnings in the Anna service
- REPL greeting should announce errors
- One-time question if errors are the only issue

### Configuration Display
- Debug mode status
- All other relevant settings

## annactl stats Requirements (RPG System)

### Anna's XP and Progression
- XP from 0 to 100, non-linear RPG-style progression
- Funny titles based on level

### Performance Metrics
- Number of questions/requests
- Average reliability
- Escalated tickets
- Number of new recipes created
- Recipes categorized by type
- Average interaction count between Anna and specialists (detailed)
- Number of times Anna managed on her own
- Number of recipes learned (not in initial 0 XP knowledge)
- Longest resolution time
- Shortest resolution time

### Fun Statistics
- Most consulted team
- Repeated questions
- Topic most asked about
- Longest reply
- Shortest reply
- Installation date
- And more interesting data

### Glance vs Detail
- `annactl stats` shows summary
- `annactl stats <category>` shows details (sound/video/network/logs/...)

## IT Department Structure

### Team Roles
- Each department has Junior and Senior specialists
- Different prompts with the same model
- Junior uses lighter model, Senior uses "deep thinking" model
- Models adjusted by Anna based on hardware available
- Permanent names for each person (human names, diverse)
- Stats show tickets closed per expert (junior or senior)

### Escalation Flow
- Only juniors can dispatch to seniors
- Complex cases may require multiple teams
- Every time junior/senior solves with high confidence, Anna learns the recipe

### The Translator
- Very fast, light model
- Understands original user message
- Converts to format Anna can check recipes or dispatch
- Also converts JSON/internal formats to natural language for user

## Capabilities

Anna must be able to:
- Run commands as any user or root
- Restart services
- Edit config files
- Install packages
- Whatever the user asks

### Helper Management
- Install helpers for her own knowledge (learned from specialists)
- Track what helpers she installed vs user installed
- Remove Anna-installed helpers on uninstall (with confirmation)
- Never install useless helpers (no ethtool if no ethernet)

### Available Teams
- If user has no sound, no Sound specialist needed
- Number of available team members shown in status

## Display Modes

### Debug Mode OFF (Fly-on-the-Wall)
- Show natural language dialog between players
- User reads the whole communication like a fly on the wall
- Clean, professional, Hollywood IT aesthetic
- True color, hacker-style tool
- No icons allowed
- All output follows same rules (version, status, stats, install, uninstall)

### Debug Mode ON (Troubleshooting)
- Show what is really happening
- For troubleshooting purposes

## UI/UX Requirements

### Real-Time Feedback
- Spinning ASCII animation when user is waiting
- Screen updated after each LLM call (not after whole thing)
- User sees progress in real-time
- Streaming from LLM word by word (where possible)

### Visual Style
- Extremely clean, professional
- True color, fashion hacker style
- Like a Hollywood movie IT department
- NO icons allowed
- Consistent across all output types

## Learning System

### Knowledge Sources
- Arch Wiki (the bible)
- Man pages, help commands, -h, --help
- Local copies of official pages linked from Arch Wiki
- Never browse web for general searching
- Patterns and telemetry from user and machine

### Recipe Learning
- Anna learns recipes from successful specialist interactions
- Reuse recipes for different goals
- Track amount of tickets closed by Anna vs specialists
- Anna's ticket count should increase over time

### Fact Management
- Learn facts from experiences and interactions
- Update or delete facts when not relevant
- Archive old facts
- Never hardcode specific cases (only basic reusable recipes)

## Communication

### Information Density
- Minimal information needed to resolve a case
- Tier 2/3 (management) get facts only, no bullshit
- Network person doesn't need CPU info unless relevant

### User Notifications
- Email (with user permission, stored for reuse)
- Greetings
- libnotify
- wall
- Any other appropriate method (without spamming)
- Notify when tickets updated or important events

### Long-Running Tasks
- If task takes > X minutes:
  1. Inform user task needs research
  2. Ask for email (store for future)
  3. Investigate when machine is idle
  4. Send email with chain of thoughts and conclusion
- If still unable to solve:
  1. Search internet for similar cases
  2. Contrast with technicians and senior management
  3. Send second email with solution

## Personality & Configuration

### Configurable via Natural Language
- Personality traits
- Learning mode
- Risk levels for confirmation skipping
- Verbosity and detail level
- All settings changeable through `annactl` in natural language

### Tips System
- Greetings can show tips about config options
- Not debug mode, but personality traits, learning mode, risk levels, etc.

## Citations & Evidence

Anna must always:
- Have citations from Arch Wiki, man help, -h, --help commands
- Store local copies of wiki pages if linked from Arch Wiki
- Give exact answer without extra information
- Explain why she runs commands (if learning mode enabled)
- Explain how commands work (if learning mode enabled)

## Problem Solving

- Never give "not possible to fix" unless:
  - Contradictory to terms
  - Really complex involving hardcore solutions
- Keep user informed of drastic problems
- Detect all telemetry from user and machine
- Know the system better than the kernel itself
- Solutions must be short, clear, exactly to the point

## Idle Time Usage

- Seniors can think strategically about improvements during idle time
- If interrupted, Anna can resume later
- Can send email to notify user when complete

## Monitoring & Alarms

- Impressive system monitoring capabilities
- User can set specific alarms via natural language:
  - "Please notify me every Monday at 9 about storage progression"
  - Custom notifications for any metric

## Development Rules (Hardcoded for Claude)

### Code Quality
- Keep files under 400 lines
- Hardcode as little as possible
- Basic reusable recipes yes, specific cases no
- Best practices for programming and security

### Release Process
Every new version must:
1. Tag, push, commit, release with checksums
2. Update README.md (description of what works)
3. Update ROADMAP.md (remove implemented, keep detailed missing items)
4. Update FEATURES.md (tested and verified features)
5. Auto-update must ALWAYS work
6. curl install must ALWAYS work
7. Installer/uninstaller updated if any change affects them
8. Provide summary for ChatGPT/user after each version bump

### Questioning Changes
- Question back if something is unclear or conflicts
- If prompter confirms, update this document
- If not confirmed, discard idea as violation of terms

### Performance
- Anna must be as fast as possible
- Benchmark computer to select optimal models
- Hardware-aware model selection
