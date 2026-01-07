# Stats Redesign Plan - Service Desk Vision

## Current Problems
1. XP/Level system is for "Anna" overall - makes no sense
2. Achievements/Suggestions are gamification bloat
3. Staff shown as "Desktop Jr" not by name (Sofia, Michael)
4. No staff progression/XP - staff should learn and grow
5. Debug mode doesn't show real LLM prompts/outputs

## Vision: A Full Service Desk
Anna is a **service desk with real employees**:
- **Departments**: Desktop, Network, Storage, Security, Services
- **Staff**: Each has a name, role, tier (Jr/Sr), and their own metrics
- **Progression**: Juniors learn and grow, eventually could become seniors
- **Real work**: Show actual tickets handled, success rates, response times

## New `annactl stats` Design

```
──────────────────────────────────────────────────────────────────────────────
Anna Service Desk  |  Staff Performance Report
──────────────────────────────────────────────────────────────────────────────

[service desk]
  total_tickets         15
  resolved              12
  escalated             3
  avg_response          4.2s

[departments]
  Desktop               tickets: 8   resolved: 7   avg: 3.1s
  Network               tickets: 4   resolved: 3   avg: 5.2s
  Storage               tickets: 2   resolved: 2   avg: 4.8s
  Services              tickets: 1   resolved: 0   avg: 8.1s

[staff roster]
  DESKTOP
    Sofia (Jr)          tickets: 7   xp: 120   rate: 86%   level: Apprentice
    Erik (Sr)           tickets: 1   xp: 25    rate: 100%  level: Expert

  NETWORK
    Michael (Jr)        tickets: 3   xp: 45    rate: 67%   level: Novice
    Ana (Sr)            tickets: 1   xp: 30    rate: 100%  level: Expert

  STORAGE
    Lars (Jr)           tickets: 2   xp: 40    rate: 100%  level: Apprentice

[recent activity]
  Sofia resolved DSK-0042 "vim config" in 2.1s
  Michael escalated NET-0015 to Ana
  Lars resolved STR-0008 "disk space" in 3.4s

──────────────────────────────────────────────────────────────────────────────
```

## Implementation Steps

### Phase 1: Add XP to Staff (StaffMetrics)
- Add `xp: u64` and `level: u8` to `StaffMetrics`
- XP calculation: 10 per ticket handled, +20 bonus if resolved, +5 per reliability point above 60
- Juniors: level 1-3 (Novice, Apprentice, Competent)
- Seniors: level 4-6 (Expert, Master, Principal)

### Phase 2: Rewrite stats_display_v2.rs
- Remove: achievements, suggestions, profile (user XP)
- Add: Service desk summary, department breakdown, staff roster with XP
- Show staff by **name** (Sofia) with tier (Jr/Sr) and XP/level
- Recent activity section (last 3-5 tickets)

### Phase 3: Track Staff Names Properly
- Store `staff_name` alongside `person_id` in records
- Use roster lookup: `person_id: "desktop_jr"` → `name: "Sofia"`

### Phase 4: Debug Mode - Real LLM Output
- When debug_mode=ON, show actual LLM prompts and responses
- "Fly on the wall" view of the LLM conversation
- This requires changes to transcript rendering

## Files to Modify
1. `anna-shared/src/staff_stats.rs` - Add XP/level to StaffMetrics
2. `annactl/src/stats_display_v2.rs` - Complete rewrite for new design
3. `anna-shared/src/event_log/types.rs` - Add staff_name field
4. `annad/src/theatre.rs` - Record staff_name with ticket
5. `annactl/src/transcript_render.rs` - Debug mode shows raw LLM

## Questions for User
- Should seniors start at higher XP/level, or earn it fresh?
- Should staff XP persist across resets, or reset with data?
- How detailed should "recent activity" be?
