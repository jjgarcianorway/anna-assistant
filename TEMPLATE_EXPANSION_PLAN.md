# Template Expansion Plan - Beta.112

**Date:** 2025-11-19
**Goal:** Increase template utilization from <10% to 80%+
**Current State:** 102 templates exist but only ~5 are mapped to keywords

---

## Problem Analysis

### Root Cause
**Manual keyword matching** in main.rs, repl.rs, and tui_v2.rs only maps a handful of templates:
- `check_swap_status` → "swap" keyword
- `check_gpu_memory` → "gpu", "vram" keywords
- `wifi_diagnostics` → "wifi", "wireless" keywords
- ~5-10 total mappings out of 102 templates

**Result:** 97+ templates are defined but NEVER USED

### QA Test Evidence (Beta.111)
- **100 questions tested**
- **100% PARTIAL** - Helpful context but no actionable commands
- **0% FULL ANSWER** - No complete command sequences
- **Root cause:** Queries fall through to LLM instead of using templates
- **Templates provide commands, LLM provides explanations**

---

## Available Templates (102 Total)

### System Diagnostics (15 templates)
- `check_swap_status`, `check_package_installed`, `check_service_status`
- `check_kernel_version`, `check_disk_space`, `check_memory`, `check_uptime`
- `check_cpu_model`, `check_cpu_load`, `check_distro`
- `check_failed_services`, `check_journal_errors`
- `system_weak_points_diagnostic`, `check_boot_time`, `check_dmesg_errors`

### Package Management (13 templates)
- `list_orphaned_packages`, `check_package_integrity`, `clean_package_cache`
- `list_package_files`, `find_file_owner`, `list_explicit_packages`
- `check_package_updates`, `list_aur_packages`, `package_depends`
- `package_reverse_depends`, `check_pacman_status`, `check_pacman_locks`
- `check_dependency_conflicts`

### Network Diagnostics (10 templates)
- `check_dns_resolution`, `check_network_interfaces`, `check_routing_table`
- `check_firewall_rules`, `test_port_connectivity`, `check_wifi_signal`
- `check_network_latency`, `check_listening_ports`, `wifi_diagnostics`
- `check_networkmanager_status`

### Service Management (8 templates)
- `restart_service`, `enable_service`, `disable_service`
- `check_service_logs`, `list_enabled_services`, `list_running_services`
- `check_failed_systemd_units`, `check_systemd_version`

### Boot & System (8 templates)
- `analyze_boot_time`, `check_boot_errors`, `show_boot_log`
- `analyze_boot_critical_chain`, `check_systemd_timers`
- `analyze_journal_size`, `show_recent_journal_errors`
- `check_recent_kernel_updates`

### CPU & Performance (8 templates)
- `check_cpu_frequency`, `check_cpu_governor`, `analyze_cpu_usage`
- `check_cpu_temperature`, `detect_cpu_throttling`, `show_top_cpu_processes`
- `check_load_average`, `analyze_context_switches`

### Memory Management (8 templates)
- `check_memory_usage`, `check_swap_usage`, `analyze_memory_pressure`
- `show_top_memory_processes`, `check_oom_killer`, `analyze_swap_activity`
- `check_huge_pages`, `show_memory_info`

### GPU & Graphics (10 templates)
- `check_gpu_info`, `check_gpu_drivers`, `check_nvidia_status`, `check_amd_gpu`
- `check_gpu_processes`, `check_gpu_temperature`, `check_gpu_errors`
- `analyze_graphics_performance`, `check_gpu_memory`

### Display & Desktop (8 templates)
- `check_display_server`, `check_desktop_environment`, `check_display_manager`
- `analyze_xorg_errors`, `check_wayland_compositor`, `check_desktop_session`
- `analyze_desktop_performance`, `check_window_manager`

### Hardware (5 templates)
- `check_disk_health`, `check_temperature`, `check_usb_devices`
- `check_pci_devices`, `check_hostname`

### Configuration (3 templates)
- `backup_config_file`, `show_config_file`, `check_config_syntax`

### Modules (2 templates)
- `list_loaded_modules`, `check_archlinux_keyring`

### Pacman Specific (6 templates)
- `check_pacman_cache_size`, `show_recent_pacman_operations`
- `check_pending_updates`, `check_pacman_mirrors`

---

## Implementation Strategy

### Phase 1: Keyword Mapping Expansion (Beta.112)
**Goal:** Map common question patterns to existing templates

**High-Priority Mappings** (Target: 40+ templates):

#### Package Questions
- "package", "install", "pacman" → `check_package_installed`, `check_pacman_status`
- "update", "upgrade", "syu" → `check_package_updates`, `check_pending_updates`
- "aur" → `list_aur_packages`
- "orphan" → `list_orphaned_packages`
- "cache" → `check_pacman_cache_size`, `clean_package_cache`
- "mirror" → `check_pacman_mirrors`
- "keyring" → `check_archlinux_keyring`

#### System Diagnostics
- "kernel" → `check_kernel_version`, `check_recent_kernel_updates`
- "boot", "startup" → `analyze_boot_time`, `check_boot_errors`
- "service" → `check_service_status`, `list_running_services`
- "failed" → `check_failed_services`, `check_failed_systemd_units`
- "journal", "log" → `check_journal_errors`, `show_recent_journal_errors`
- "distro", "version" → `check_distro`
- "uptime" → `check_uptime`

#### Performance
- "slow", "performance" → `system_weak_points_diagnostic`
- "cpu" → `check_cpu_load`, `show_top_cpu_processes`, `check_cpu_temperature`
- "memory", "ram" → `check_memory_usage`, `show_top_memory_processes`
- "disk space" → `check_disk_space`
- "temperature", "temp", "heat" → `check_temperature`, `check_cpu_temperature`

#### Network
- "dns" → `check_dns_resolution`
- "network", "connection" → `check_network_interfaces`, `check_networkmanager_status`
- "port" → `check_listening_ports`, `test_port_connectivity`
- "latency", "ping" → `check_network_latency`
- "firewall" → `check_firewall_rules`

#### GPU/Graphics
- "nvidia" → `check_nvidia_status`, `check_gpu_memory`
- "amd", "radeon" → `check_amd_gpu`
- "display", "monitor" → `check_display_server`, `check_desktop_environment`
- "xorg", "x11" → `analyze_xorg_errors`
- "wayland" → `check_wayland_compositor`

### Phase 2: LLM Prompt Enhancement (Beta.112)
**Goal:** Make LLM provide actionable commands

**Current Problem:** LLM provides context but no commands

**Solution:** Update planner prompt to emphasize:
1. "Provide specific commands the user can run"
2. "Use concrete command examples, not conceptual explanations"
3. "Format responses with command blocks"
4. "Always include the actual command to execute"

### Phase 3: RecipePlanner Integration (Beta.113+)
**Goal:** Let LLM choose templates instead of keyword matching

**Current State:** Planner/Critic loop exists but LLM calls are placeholders

**Implementation:**
1. Wire up `call_planner_llm()` to actual LLM (lines 114-132 in recipe_planner.rs)
2. Wire up `call_critic_llm()` to actual LLM (lines 136-153)
3. Update prompts to list all 102 templates as options
4. Have LLM select template + parameters instead of raw commands

---

## Success Metrics

**Target Results (Beta.112+):**
- Template utilization: <10% → 40%+ (Phase 1)
- FULL ANSWER rate: 0% → 30%+ (Phase 1+2)
- PARTIAL rate: 100% → 65% (some will move to FULL)
- UNHELPFUL: Maintain at 0%

**Long-term (Beta.113+):**
- Template utilization: 40% → 80%+
- FULL ANSWER rate: 30% → 70%+

---

## Implementation Order

1. **Analyze QA failures** - Map question patterns to templates
2. **Expand keyword matching** - Add 35+ new keyword mappings
3. **Update LLM prompts** - Emphasize actionable command sequences
4. **Test with QA suite** - Verify improvement
5. **Document and commit** - Update README and CHANGELOG

---

## Files to Modify

- `crates/annactl/src/main.rs` - Add keyword mappings (one-shot mode)
- `crates/annactl/src/repl.rs` - Add keyword mappings (REPL mode)
- `crates/annactl/src/tui_v2.rs` - Add keyword mappings (TUI mode)
- `crates/anna_common/src/llm.rs` - Update LLM prompts for actionable responses
- `crates/anna_common/src/recipe_planner.rs` - Future: wire up LLM calls

---

## Risk Assessment

**Low Risk:**
- Adding keyword mappings (safe, tested pattern from Beta.101)
- Updating LLM prompts (reversible, no breaking changes)

**Medium Risk:**
- May create false positive matches (keyword too broad)
- Mitigation: Use word-boundary matching like Beta.101

**High Priority:**
- Consistency across all three modes (main.rs, repl.rs, tui_v2.rs)
- User requirement: "ensure that the replies from annactl, TUI or one-off are consistent!!!!!"
