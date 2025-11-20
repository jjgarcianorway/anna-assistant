# JSON Runtime Contract - Anna LLM Interface

**Version:** Beta.143
**Status:** Foundation Implemented
**Purpose:** Define the strict contract between Anna and the runtime LLM

---

## 🎯 Overview

This document defines the **JSON Runtime Contract** - the formal interface between Anna (annad + annactl) and the runtime LLM (Ollama with Llama/Qwen/etc).

### Key Principle

**The runtime LLM is a planner, not an actor.**

- Anna executes commands
- The LLM only proposes action plans
- All plans are JSON, validated, and gated by risk level

---

## 📋 The Contract

### Input to LLM

The runtime LLM receives:

```
[SYSTEM MESSAGE]
- Identity: "ANNA_RUNTIME_PLANNER"
- Schema definition
- Safety rules
- Output requirements
- Environment detection rules
- Examples

[USER MESSAGE]
- User request
- Interaction mode (one-shot, repl, tui)
- Telemetry JSON (from annad)
```

### Output from LLM

The LLM must output **ONLY** valid JSON matching this schema:

```json
{
  "analysis": "string - High level reasoning using telemetry",
  "goals": [
    "string - User-visible goal 1",
    "string - User-visible goal 2"
  ],
  "necessary_checks": [
    {
      "id": "string - unique check ID",
      "description": "string - What this checks",
      "command": "string - Safe diagnostic command",
      "risk_level": "INFO|LOW|MEDIUM|HIGH",
      "required": true|false
    }
  ],
  "command_plan": [
    {
      "id": "string - unique step ID",
      "description": "string - What this step does",
      "command": "string - Exact shell command",
      "risk_level": "INFO|LOW|MEDIUM|HIGH",
      "rollback_id": "string|null - ID of rollback step",
      "requires_confirmation": true|false
    }
  ],
  "rollback_plan": [
    {
      "id": "string - unique rollback ID",
      "description": "string - What this undoes",
      "command": "string - Command to restore state"
    }
  ],
  "notes_for_user": "string - Plain English explanation",
  "meta": {
    "detection_results": {
      "de": "string|null - Desktop environment",
      "wm": "string|null - Window manager",
      "wallpaper_backends": ["string"],
      "display_protocol": "wayland|x11|null"
    },
    "template_used": "string|null - Template identifier",
    "llm_version": "anna_runtime_v3"
  }
}
```

---

## 🎚️ Risk Levels

### INFO (Blue)
- Pure inspection, no changes
- Examples: `ls`, `cat`, `ps`, `systemctl status`, `pacman -Q`
- **No confirmation required**

### LOW (Green)
- Reversible user config
- Examples: wallpaper, fonts, theming, user dotfiles
- **Confirmation required**

### MEDIUM (Yellow)
- Package installs, service changes
- Must have clear rollback
- **Confirmation required**

### HIGH (Red)
- Bootloader, network, filesystem
- Could lock user out
- **Double confirmation required**

---

## 🔍 Environment Detection

For environment-dependent tasks (wallpaper, DE settings), the LLM must:

### 1. Check Telemetry First
```json
{
  "display_protocol": "wayland",
  "de": null,
  "wm": "hyprland",
  "installed_packages": ["hyprpaper", "swaybg"]
}
```

### 2. Add Detection Checks if Needed
```json
{
  "necessary_checks": [
    {
      "id": "detect_wm",
      "description": "Detect window manager",
      "command": "ps -e | grep -Ei 'hyprland|sway|i3'",
      "risk_level": "INFO",
      "required": true
    },
    {
      "id": "detect_wallpaper",
      "description": "Detect wallpaper backend",
      "command": "ps -e | grep -Ei 'swaybg|hyprpaper|feh'",
      "risk_level": "INFO",
      "required": true
    }
  ]
}
```

### 3. Generate Environment-Specific Commands

**Hyprland + hyprpaper:**
```bash
cp ~/.config/hypr/hyprpaper.conf ~/.config/hypr/hyprpaper.conf.anna_backup.$(date +%Y%m%d_%H%M%S)
sed -i 's|^preload = .*|preload = /path/to/image.png|' ~/.config/hypr/hyprpaper.conf
hyprctl hyprpaper wallpaper ',/path/to/image.png'
```

**sway + swaybg:**
```bash
pkill swaybg
swaybg -i /path/to/image.png -m fill & disown
```

**GNOME:**
```bash
gsettings set org.gnome.desktop.background picture-uri file:///path/to/image.png
```

**XFCE:**
```bash
xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/workspace0/last-image -s /path/to/image.png
```

---

## 🛡️ Safety Rules (Hard Bans)

The LLM must NEVER:

1. ❌ Use `rm -rf /`
2. ❌ Touch `/boot` without explicit telemetry confirmation
3. ❌ Modify partition tables
4. ❌ Disable security mechanisms
5. ❌ Pipe untrusted input to `sh`
6. ❌ Output markdown, backticks, or comments (JSON only)

---

## 📝 Complete Example: Wallpaper Change

### Request
```
User: "Change my wallpaper to /home/user/besseggen.png"
Telemetry: {
  "display_protocol": "wayland",
  "wm": "hyprland",
  "installed_packages": ["hyprpaper"]
}
```

### Response
```json
{
  "analysis": "User wants to change wallpaper to /home/user/besseggen.png. Telemetry shows Wayland with Hyprland WM and hyprpaper backend.",
  "goals": [
    "Set wallpaper to /home/user/besseggen.png"
  ],
  "necessary_checks": [
    {
      "id": "check_file",
      "description": "Verify image file exists",
      "command": "test -f /home/user/besseggen.png && echo 'exists' || echo 'missing'",
      "risk_level": "INFO",
      "required": true
    },
    {
      "id": "check_hyprpaper",
      "description": "Verify hyprpaper is running",
      "command": "pgrep -x hyprpaper > /dev/null && echo 'running' || echo 'not running'",
      "risk_level": "INFO",
      "required": true
    }
  ],
  "command_plan": [
    {
      "id": "backup_config",
      "description": "Backup current hyprpaper config",
      "command": "cp ~/.config/hypr/hyprpaper.conf ~/.config/hypr/hyprpaper.conf.anna_backup.$(date +%Y%m%d_%H%M%S)",
      "risk_level": "LOW",
      "rollback_id": null,
      "requires_confirmation": false
    },
    {
      "id": "update_config",
      "description": "Update hyprpaper preload and wallpaper directives",
      "command": "sed -i 's|^preload = .*|preload = /home/user/besseggen.png|' ~/.config/hypr/hyprpaper.conf && sed -i 's|^wallpaper = .*|wallpaper = ,/home/user/besseggen.png|' ~/.config/hypr/hyprpaper.conf",
      "risk_level": "LOW",
      "rollback_id": "restore_config",
      "requires_confirmation": true
    },
    {
      "id": "reload_hyprpaper",
      "description": "Reload hyprpaper to apply new wallpaper",
      "command": "hyprctl hyprpaper wallpaper ',/home/user/besseggen.png'",
      "risk_level": "LOW",
      "rollback_id": null,
      "requires_confirmation": false
    }
  ],
  "rollback_plan": [
    {
      "id": "restore_config",
      "description": "Restore previous hyprpaper config",
      "command": "LATEST=$(ls -t ~/.config/hypr/hyprpaper.conf.anna_backup.* 2>/dev/null | head -1); [ -n \"$LATEST\" ] && cp \"$LATEST\" ~/.config/hypr/hyprpaper.conf && hyprctl hyprpaper reload"
    }
  ],
  "notes_for_user": "I'll change your wallpaper to /home/user/besseggen.png using hyprpaper (Hyprland's wallpaper backend). This involves backing up your current config, updating the wallpaper path, and reloading hyprpaper. Risk: LOW (cosmetic change, easy to revert).",
  "meta": {
    "detection_results": {
      "de": null,
      "wm": "hyprland",
      "wallpaper_backends": ["hyprpaper"],
      "display_protocol": "wayland"
    },
    "template_used": "wallpaper_change_hyprland_v1",
    "llm_version": "anna_runtime_v3"
  }
}
```

---

## 📝 Example: System Status Query

### Request
```
User: "How is my system?"
Telemetry: {
  "cpu": {"model": "Intel i7-9700K", "cores": 8, "load": 15},
  "ram": {"total_gb": 32, "used_gb": 14.4, "percent": 45},
  "failed_services": []
}
```

### Response
```json
{
  "analysis": "User wants system status overview. Telemetry shows CPU at 15% load, RAM 45% used, no failed services.",
  "goals": [
    "Provide system health summary"
  ],
  "necessary_checks": [],
  "command_plan": [],
  "rollback_plan": [],
  "notes_for_user": "Your system is healthy. CPU: Intel Core i7-9700K (8 cores) at 15% load. RAM: 14.4GB / 32GB used (45%). No failed services detected.",
  "meta": {
    "detection_results": {},
    "template_used": "system_status_overview",
    "llm_version": "anna_runtime_v3"
  }
}
```

---

## 🏗️ Implementation (Beta.143)

### Files Created

**Rust Schema:**
- `crates/anna_common/src/action_plan_v3.rs`
  - `ActionPlan` struct
  - `CommandStep`, `NecessaryCheck`, `RollbackStep` structs
  - `RiskLevel` enum with validation
  - Complete validation logic

**Runtime Prompt:**
- `crates/annactl/src/system_prompt_v3_json.rs`
  - `build_runtime_system_prompt()` - Complete system message
  - `build_user_prompt()` - User message with telemetry
  - Embedded examples and safety rules

---

## 🔄 Workflow

### 1. User Request
```
annactl: "change my wallpaper to /path/to/image.png"
```

### 2. Anna Prepares Context
```rust
let telemetry = annad.get_telemetry();
let system_prompt = build_runtime_system_prompt();
let user_prompt = build_user_prompt(request, telemetry, "one-shot");
```

### 3. Query Runtime LLM
```rust
let response = ollama.query(system_prompt, user_prompt).await?;
```

### 4. Parse and Validate JSON
```rust
let plan: ActionPlan = serde_json::from_str(&response)?;
plan.validate()?;
```

### 5. Display in TUI
```
┌─ Action Plan ─────────────────────────┐
│ Change wallpaper                      │
│                                        │
│ Risk: LOW ✅                          │
│ Steps: 3                               │
│ Rollback: Available                   │
│                                        │
│ [Show Details] [Approve] [Cancel]    │
└────────────────────────────────────────┘
```

### 6. Execute on Approval
```rust
if user_approves {
    annad.execute_plan(plan).await?;
}
```

---

## ✅ Validation Rules

Anna validates every plan:

1. **Schema compliance** - All required fields present
2. **Non-empty commands** - No empty command strings
3. **Rollback references** - All rollback_ids exist
4. **Risk classification** - All steps have risk_level
5. **Dangerous commands** - Check against hard bans
6. **Command realism** - Validate commands are Arch-native

---

## 🚧 Status (Beta.143)

### ✅ Completed
- JSON schema defined (`ActionPlan` structs)
- Runtime system prompt created
- Safety rules embedded
- Environment detection patterns
- Validation logic implemented
- Examples provided (wallpaper, status)

### ⏳ Next Steps (Beta.144+)
- Create dialogue runner that uses v3
- Implement JSON response parser
- Create TUI display for action plans
- Implement execution engine
- Add actual LLM integration
- Test with real Ollama queries

---

## 📊 Comparison: V2 vs V3

| Aspect | V2 (Beta.142) | V3 (Beta.143) |
|--------|---------------|---------------|
| **Output Format** | Markdown | JSON |
| **Parsing** | Text parsing | serde_json |
| **Validation** | Manual checks | Rust type system |
| **Execution** | Not implemented | Clear interface |
| **TUI Display** | Render markdown | Render structured plan |
| **Rollback** | Text only | Executable commands |
| **Risk** | In separate section | Per-command field |
| **Status** | Design doc | Production contract |

---

## 🎯 Why JSON?

1. **Validation** - Rust structs enforce schema
2. **Parsing** - serde_json is reliable
3. **Execution** - Clear command → action mapping
4. **TUI** - Structured data easy to display
5. **Logging** - Serializable action plans
6. **Rollback** - Executable rollback commands
7. **Safety** - Type-safe risk levels

---

## 📚 References

- **Schema:** `crates/anna_common/src/action_plan_v3.rs`
- **Prompt:** `crates/annactl/src/system_prompt_v3_json.rs`
- **Tests:** In both files
- **Related:** `PROMPT_V2_SYSTEM.md` (principles, now with JSON output)

---

**Version:** Beta.143
**Last Updated:** 2025-11-20
**Status:** Foundation implemented, integration pending
