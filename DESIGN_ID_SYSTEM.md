# Category-Based ID System Design

## Problem Statement

Current ID system has critical flaws:
1. Numbers change when filtering by category
2. User applies wrong items due to numbering confusion
3. No stable, permanent identifiers
4. Multi-word categories require quotes

## Solution: Category-Based Permanent IDs

### Format: `CAT-NNN`

Examples:
- `NET-001` - Network category, advice #1
- `SEC-042` - Security category, advice #42
- `PKG-015` - Packages category, advice #15
- `HW-008` - Hardware category, advice #8

### Category Codes (Aligned with Arch Wiki)

| Code | Category | Description |
|------|----------|-------------|
| SYS | system | System administration |
| NET | network | Networking |
| SEC | security | Security |
| PKG | packages | Package management |
| HW | hardware | Hardware support |
| DESK | desktop | Desktop environment |
| DEV | development | Development tools |
| MEDIA | multimedia | Audio/Video |
| PERF | optimization | Performance tuning |
| BOOT | boot | Boot process |
| FS | filesystem | Filesystem |
| USER | users | User management |

### ID Assignment Strategy

**Option A: Registry-Based Mapping** (RECOMMENDED)
- Keep internal string IDs: "amd-microcode", "python-lsp"
- Maintain stable mapping file: `id_registry.json`
- Display layer converts to category IDs
- Apply accepts both formats

```json
{
  "amd-microcode": "HW-001",
  "intel-microcode": "HW-002",
  "python-lsp": "DEV-042",
  "ssh-config": "SEC-018"
}
```

**Benefits:**
- Minimal code changes
- IDs stable across updates
- Backward compatible
- Easy to maintain

**Option B: Direct Category IDs** (More invasive)
- Replace all string IDs with category IDs in source
- Requires massive refactor
- Hard to maintain

### Implementation Plan

#### Phase 1: Foundation (Beta 100)
1. ✅ Change version to Beta 1.0.0-beta.100
2. Create ID registry system
3. Category code enum
4. ID mapping functions

#### Phase 2: Display Layer (Beta 101)
5. Convert IDs for display in advise command
6. Update apply to accept both formats
7. Show both formats initially for transition

#### Phase 3: Standardization (Beta 102)
8. Align all categories with Arch Wiki
9. Single-word category names
10. Shell completion generation

#### Phase 4: Intelligence (Beta 103)
11. Telemetry-based relevance scoring
12. Remove irrelevant recommendations
13. Contextual chaining

### Migration Path

**Week 1: Beta 100**
- ID system infrastructure
- Registry-based mapping
- Both old and new IDs work

**Week 2: Beta 101**
- Display shows new IDs
- Apply accepts both
- Documentation updated

**Week 3: Beta 102**
- Category standardization
- Remove multi-word categories
- Shell completions

**Week 4: Beta 103**
- Intelligence improvements
- Telemetry enhancement
- Final polish

### Apply Workflow Enhancement

**Before:**
```bash
annactl apply 1  # Applies immediately, might be wrong item
```

**After:**
```bash
annactl apply NET-006

→ NET-006: Enable firewall with UFW

  Category: Network Security
  Risk: Low
  Time: ~30 seconds

  Steps:
  1. Install ufw package
  2. Enable ufw.service
  3. Configure default deny incoming
  4. Allow SSH (port 22)
  5. Enable at boot

  [A]pply  [D]ismiss  [N]ever show  [?]Details: _
```

### Informative Advice Handling

New advice types:
- **Actionable** - Can be applied
- **Informative** - FYI only, auto-dismiss when viewed
- **Warning** - Critical info, requires acknowledgment

**Workflow:**
```bash
annactl advise
# Shows: INFO-005 [Informative] SSH using weak ciphers

annactl show INFO-005
# Displays full details
# Marks as "read", won't show again unless new scan detects it
```

## Success Criteria

- ✅ IDs never change regardless of filter
- ✅ Categories single words (no quotes)
- ✅ Apply shows preview before action
- ✅ Telemetry drives recommendations
- ✅ Only relevant advice shown
- ✅ Shell completion works
- ✅ Arch Wiki categories match
