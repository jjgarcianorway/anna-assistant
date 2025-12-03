# Anna Assistant - Release Notes

---

## v0.0.43 - Doctor Registry + Unified Entry Flow (Auto-Detect Domain and Run the Right Doctor)

**Release Date:** 2025-12-03

### Summary

Anna now has a **Doctor Registry** that automatically selects the right diagnostic doctor based on your request. Instead of manually specifying which doctor to use, Anna analyzes keywords, intent tags, and symptoms to pick the best match. All doctors now follow a unified lifecycle with consistent output schemas.

### Key Features

**Doctor Registry (Data-Driven):**
- Registry loaded from `/etc/anna/policy/doctors.toml` (or `~/.config/anna/doctors.toml`)
- 5 doctors registered: Network, Storage, Audio, Boot, Graphics
- Each doctor entry includes:
  - Keywords (e.g., "wifi", "audio", "boot")
  - Intent tags (e.g., "network_diagnosis", "audio_problem")
  - Symptom patterns (e.g., "no sound", "slow boot")
  - Required/optional evidence bundles
  - Allowed playbooks
  - Case file name

**Doctor Selection Algorithm:**
- Keyword matching (10 points per match)
- Intent tag matching (15 points per tag)
- Symptom matching (20 points per symptom)
- Priority weighting for tie-breaking
- Max 2 doctors per request (1 primary + 1 optional secondary from different domain)
- Selection includes reasoning explaining why

**Unified Doctor Run Lifecycle:**
1. `select_doctor` - Pick the right doctor(s)
2. `collect_evidence` - Gather required evidence bundle
3. `diagnosis_flow` - Run deterministic diagnosis steps
4. `playbook_offer` - Suggest fix if applicable
5. `apply_fix` - Execute playbook if confirmed
6. `verify` - Check fix worked (or mark pending)
7. `close` - Complete run, capture recipe if eligible

**Doctor Run Output Schema (doctor_run.json):**
```json
{
  "schema_version": 1,
  "run_id": "dr-20231203-143022-123",
  "user_request": "my sound is broken",
  "selection": {
    "primary": {
      "doctor_id": "audio_doctor",
      "doctor_name": "Audio Doctor",
      "match_score": 85,
      "match_reason": "Matched keywords: sound, audio"
    },
    "reasoning": "Selected Audio Doctor based on keywords and symptoms"
  },
  "current_stage": "close",
  "stage_timings": [...],
  "key_findings": [...],
  "chosen_playbook": "restart_pipewire",
  "verification_status": { "verified": true },
  "result": "success",
  "reliability": 90,
  "junior_verification": {
    "doctor_choice_approved": true,
    "diagnosis_steps_followed": true,
    "fix_policy_compliant": true,
    "score": 92
  }
}
```

**Junior Verification Enforcement:**
- Verifies doctor choice makes sense for request
- Confirms diagnosis steps were followed
- Checks fix is policy-compliant and minimal
- Ensures final claim is evidence-backed or marked "verification pending"

**Status Integration:**
`annactl status` now shows:
- Last doctor run (ID, result, reliability)
- Doctor runs today count
- Success rate today

### doctors.toml Schema

```toml
schema_version = 1

[[doctors]]
id = "network_doctor"
name = "Network Doctor"
description = "Diagnoses network connectivity, WiFi, DNS, and routing issues"
domain = "network"
keywords = ["network", "wifi", "internet", "connection", "dns", "ping"]
intent_tags = ["network_diagnosis", "connectivity_issue"]
symptoms = ["no internet", "wifi disconnecting", "can't connect"]
required_evidence = ["interface_status", "ip_addresses", "routes", "dns_config"]
optional_evidence = ["wifi_signal", "network_manager_status"]
required_tools = ["ip", "ping"]
allowed_playbooks = ["restart_networkmanager", "renew_dhcp", "flush_dns"]
case_file_name = "networking_doctor.json"
priority = 80
enabled = true

[[doctors]]
id = "audio_doctor"
name = "Audio Doctor"
# ... similar structure
```

### Files Changed

- **NEW:** `crates/anna_common/src/doctor_registry.rs` - Full registry implementation (~1100 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.43
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `TODO.md` - Version bump
- **MODIFIED:** `RELEASE_NOTES.md` - This file

### Tests

20 new tests covering:
- Default registry creation
- Select network doctor ("wifi disconnecting")
- Select audio doctor ("no sound")
- Select boot doctor ("boot is slow")
- Select graphics doctor ("screen share broken")
- Select storage doctor ("disk full")
- Ambiguous request selects one and explains why
- Selection with intent tags
- No match returns None
- Doctor run lifecycle stages
- Doctor domain display
- Stage status display
- Doctor run result display
- Run ID generation
- Junior verification
- Verification pending
- Mark verified
- Config serialization
- Secondary doctor different domain
- Doctors by domain
- Selection explains why

### Example Transcript: "sound broken" - Doctor Selection and Flow

```
[you] to [anna]: my sound is broken

[anna] to [translator]: parse user intent
[translator] to [anna]: intent=fix_problem, targets=[audio], tags=[audio_problem]

[anna] to [doctor_registry]: select doctor for "my sound is broken"
[doctor_registry] to [anna]:
  Selection complete:
    Primary: Audio Doctor (score: 65)
    Matched keywords: sound
    Matched symptoms: no sound
    Reasoning: Selected Audio Doctor (score: 65) as primary based on
               keywords: sound and symptoms: no sound

[anna] to [you]:
  Selected: Audio Doctor
  Why: Your request mentions "sound" and matches the symptom "no sound".
  Evidence to collect: pipewire_status, audio_devices, default_sink

[anna] to [audio_doctor]: collect evidence
[audio_doctor] to [anna]:
  Evidence collected:
  - pipewire_status: NOT RUNNING
  - audio_devices: 2 detected (speakers, headphones)
  - default_sink: not set
  Run ID: dr-20231203-143022-123

[anna] to [audio_doctor]: run diagnosis
[audio_doctor] to [anna]:
  Step 1: Service Status - FAIL
    PipeWire service is not running
    Implication: Audio subsystem is down

  Step 2: Device Detection - PASS
    Found 2 audio devices
    Implication: Hardware detected correctly

  Step 3: Configuration - FAIL
    No default sink configured
    Implication: Even if service starts, no output selected

  Finding: pipewire-not-running (Error)
  Hypothesis: "PipeWire crashed or was never started" (90% confidence)
  Suggested playbook: restart_pipewire

[anna] to [you]:
  Diagnosis Complete

  Audio Health: BROKEN

  Key Findings:
  1. [Error] PipeWire service is not running
  2. [Warning] No default audio sink configured

  Top Hypothesis (90% confidence):
    PipeWire service has stopped. Restarting should restore audio.

  Recommended Fix:
    Restart PipeWire (Low risk)

  Type "I CONFIRM" to apply fix

[you] to [anna]: I CONFIRM

[anna] to [audio_doctor]: execute playbook restart_pipewire
[audio_doctor] to [anna]:
  Playbook executed:
  - Command: systemctl --user restart pipewire pipewire-pulse wireplumber
  - Exit code: 0
  - Post-check: PipeWire running = PASS

[anna] to [junior]: verify doctor run dr-20231203-143022-123
[junior] to [anna]:
  VERIFICATION:
  - Doctor choice approved: YES (audio keywords matched)
  - Diagnosis steps followed: YES (3 steps completed)
  - Fix policy-compliant: YES (restart_pipewire in allowed_playbooks)
  - Fix minimal: YES (single service restart)
  - Claim evidence-backed: YES (post-check confirms service running)
  SCORE: 92

[anna] to [you]:
  Audio Restored!

  PipeWire is now running. Try playing audio.

  Run ID: dr-20231203-143022-123
  Reliability: 92%
  Recipe captured: Yes (restart_pipewire for no-sound)

  Case file: audio_doctor.json
  Run file: doctor_run.json
```

### Example: Ambiguous Request Handled

```
[you] to [anna]: my system is slow

[anna] to [doctor_registry]: select doctor for "my system is slow"
[doctor_registry] to [anna]:
  Selection complete:
    Primary: Boot Doctor (score: 25)
    Matched keywords: slow
    Reasoning: Selected Boot Doctor (score: 25) as primary.
               "slow" matches boot diagnosis for startup slowness.
               Note: Score is low. Request is ambiguous.

[anna] to [you]:
  I'll use the Boot Doctor to check startup performance.

  Note: Your request "my system is slow" could mean several things:
  - Slow boot time → Boot Doctor (selected)
  - Slow disk access → Storage Doctor
  - Slow network → Network Doctor

  If this doesn't help, please clarify what feels slow
  (e.g., "boot takes forever" or "disk is slow").

  Collecting boot evidence...
```

---

## v0.0.42 - Arch GPU/Graphics Doctor v1 (Wayland/X11, Drivers, Compositor Health)

**Release Date:** 2025-12-03

### Summary

Anna can now diagnose common graphics issues on Arch Linux systems. The Graphics Doctor detects session type (Wayland/X11), compositor (Hyprland focus), GPU inventory, driver stack (NVIDIA/AMD/Intel), XDG portal health for screen sharing, and PipeWire status. It produces evidence-backed hypotheses for issues like black screen, stutter, tearing, and broken screen sharing.

### Key Features

**Evidence Bundle (Read-Only):**
- Session type detection (Wayland vs X11 vs TTY)
- Compositor detection (Hyprland, Sway, KDE Plasma, GNOME/Mutter, wlroots-based, X11 WMs)
- GPU inventory via lspci with vendor detection (NVIDIA, AMD, Intel)
- Kernel module detection (nvidia, nvidia-open, nouveau, amdgpu, radeon, i915, xe)
- Driver packages (nvidia, mesa, vulkan, libva, vdpau)
- XDG Portal stack health (xdg-desktop-portal, xdg-desktop-portal-hyprland, -gtk, -kde, etc.)
- PipeWire and WirePlumber status (required for screen sharing)
- Portal backend matching (correct backend for compositor)
- Monitor information (from hyprctl/swaymsg/xrandr)
- Compositor and portal logs from journalctl
- Graphics-relevant environment variables

**Deterministic Diagnosis Flow (6 steps):**
1. Session Detection - Identify Wayland/X11/TTY and compositor
2. GPU Detection - Inventory GPUs, detect vendor/driver
3. Package Verification - Confirm required packages for stack
4. Portal Health - Check XDG portal services, backend matching
5. Log Analysis - Scan for crash indicators, errors
6. Hypothesis Generation - Produce max 3 evidence-backed theories

Each step produces:
- Result: PASS/FAIL/PARTIAL/SKIPPED
- Details with evidence citations
- Implication explaining what it means

**Portal Backend Matching:**
| Compositor | Expected Portal Backend |
|------------|------------------------|
| Hyprland | xdg-desktop-portal-hyprland or -wlr |
| Sway | xdg-desktop-portal-wlr |
| KDE Plasma | xdg-desktop-portal-kde |
| GNOME | xdg-desktop-portal-gnome |
| wlroots-based | xdg-desktop-portal-wlr |
| X11 WMs | xdg-desktop-portal-gtk |

**Hypothesis Examples:**
- "Wrong portal backend for compositor" (90% confidence)
- "Portal service not running" (85%)
- "PipeWire not running for screen sharing" (85%)
- "NVIDIA driver not loaded" (80%)
- "Crash indicators in compositor logs" (75%)

**Fix Playbooks (with confirmation):**

| Playbook | Risk | Description |
|----------|------|-------------|
| `restart_portals` | Low | Restart xdg-desktop-portal services |
| `restart_pipewire_portals` | Low | Restart PipeWire, WirePlumber, and portals |
| `collect_crash_report` | Info | Collect crash logs for debugging |
| `restart_display_manager` | High | BLOCKED - Restart display manager (gdm, sddm, etc.) |

Each playbook has:
- Preflight checks (service exists, etc.)
- Commands with timeout
- Post-checks with wait times
- Rollback commands
- Policy blocking for high-risk actions
- Confirmation phrase: `I CONFIRM`

**Recipe Capture:**
When a fix playbook succeeds with reliability >= 80%:
- Automatic recipe creation request
- Problem description from diagnosis
- Solution from playbook
- Preconditions (session type, compositor)
- Evidence patterns for matching

**Case File Integration:**
- `graphics_doctor.json` in case files
- Evidence bundle stored
- Diagnosis result tracked
- Playbook execution recorded
- Notes and timestamps

### Files Changed

- **NEW:** `crates/anna_common/src/graphics_doctor.rs` - Full graphics doctor implementation (~1800 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.42
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `TODO.md` - Version bump
- **MODIFIED:** `RELEASE_NOTES.md` - This file

### Tests

18 new tests covering:
- Session type display
- Compositor display
- GPU vendor display
- Driver stack display
- Portal backend display
- Graphics health display
- Risk level display
- Session info collection
- GPU info collection
- Evidence health determination (broken/degraded/healthy)
- Portal backend correctness
- Diagnosis healthy system
- Diagnosis broken portal
- Wrong portal backend detection
- Restart portals playbook
- Display manager restart blocked by policy
- Case file workflow
- Recipe capture on success

### Example Transcript: "Screen Share Not Working"

```
[you] to [anna]: my screen share is not working in discord

[anna] to [translator]: parse user intent
[translator] to [anna]: graphics diagnosis request (screen sharing issue)

[anna] to [graphics_doctor]: collect evidence
[graphics_doctor] to [anna]:
  Evidence collected:
  - Session: Wayland (Hyprland)
  - GPU: AMD Radeon RX 7900 XTX (amdgpu driver)
  - Portals: xdg-desktop-portal running, but using GTK backend
  - PipeWire: running
  - WirePlumber: running

[anna] to [graphics_doctor]: run diagnosis
[graphics_doctor] to [anna]:
  Step 1: Session Detection - PASS
    Session type: Wayland
    Compositor: Hyprland
    Implication: Wayland compositor detected. Portal stack required for screen sharing.

  Step 2: GPU Detection - PASS
    Found 1 GPU(s):
    - AMD Radeon RX 7900 XTX (amdgpu driver loaded)
    Implication: GPU and driver correctly detected.

  Step 3: Package Verification - PASS
    Required packages present for AMD/Mesa stack.

  Step 4: Portal Health - FAIL
    Portal service running but wrong backend.
    Active backend: GTK
    Expected for Hyprland: Hyprland or wlr
    Implication: Screen sharing will likely fail or show black screen.

  Step 5: Log Analysis - PASS
    No crash indicators found in compositor logs.

  Step 6: Hypothesis Generation - PASS
    Generated 2 hypothesis(es):
    1. "Wrong portal backend for Hyprland" (95% confidence)
       GTK portal cannot capture Hyprland screens.
       Install xdg-desktop-portal-hyprland and restart portals.
       Suggested playbook: restart_portals
    2. "Portal may need restart after package change" (70% confidence)
       If portal was recently installed, restart may help.

[anna] to [you]:
  Graphics Health: DEGRADED

  Your screen sharing is likely failing because the wrong portal
  backend is active.

  Top hypothesis (95% confidence):
    Wrong portal backend for Hyprland compositor.
    You have GTK portal but Hyprland needs xdg-desktop-portal-hyprland
    or xdg-desktop-portal-wlr.

  Recommended fix:
    1. Install xdg-desktop-portal-hyprland (if not installed)
    2. Restart portal services
    Risk: Low - just restarts user services

  To apply restart playbook: type "I CONFIRM" when prompted

  Evidence: [ev-graphics-1733234567-session] [ev-graphics-1733234567-portals]

  Reliability: 90%
```

### Example Transcript: Portal Restart Fix

```
[anna] to [you]:
  Playbook: restart_portals
  Action: Restart XDG portal services for screen sharing

  This will:
  1. Preflight: Check portal services exist
  2. Execute: systemctl --user restart xdg-desktop-portal*
  3. Post-check: Verify portals running, wait 3s

  Rollback: Services will restart automatically on next login

  This is a Low-risk operation.
  Type "I CONFIRM" to proceed:

[you] to [anna]: I CONFIRM

[anna] to [annad]: execute playbook restart_portals
[annad] to [anna]:
  Playbook executed:
  - Commands: 1 executed
  - Post-checks: 1/1 passed
  - Result: SUCCESS
  - Reliability: 85%

[anna] to [you]:
  Portal services restarted successfully!

  Please try screen sharing again in Discord.

  If this fixes the issue, a recipe will be created for future use.

  Case file: graphics-20231203-143022.json
```

### Example: High-Risk Action Blocked

```
[you] to [anna]: restart my display manager

[anna] to [graphics_doctor]: generate playbook restart_display_manager
[graphics_doctor] to [anna]:
  Playbook: restart_display_manager
  POLICY BLOCKED: Restarting display manager will kill your session.
  This action requires manual intervention.

[anna] to [you]:
  Cannot restart display manager automatically.

  This is a HIGH-RISK action that would immediately terminate your
  graphical session, potentially causing data loss.

  If you need to restart the display manager, do it manually:
    sudo systemctl restart gdm  (or sddm, lightdm)

  Make sure to save all work first.

  Reliability: N/A (blocked by policy)
```

---

## v0.0.41 - Arch Boot Doctor v1 (Slow Boot + Service Regressions + "What Changed" Correlation)

**Release Date:** 2025-12-03

### Summary

Anna can now diagnose slow boot causes on Arch Linux systems. The Boot Doctor uses systemd tooling to identify offenders, detect regressions against baseline, and correlate issues with recent changes (package updates, service enables, config edits). Successful fixes with high reliability automatically create recipes for future reuse.

### Key Features

**Evidence Bundle (Read-Only):**
- `systemd-analyze time` - Boot timing breakdown (firmware, loader, kernel, initrd, userspace)
- `systemd-analyze blame` - Top N slowest units (default 20)
- `systemd-analyze critical-chain` - Critical path units (top 10)
- Enabled units snapshot (delta-friendly)
- Recent journal warnings/errors during boot
- "What changed" in last N days (default 14) - packages, services, configs
- Anna telemetry boot time trend (if tracked)
- Baseline for comparison (if available)

**Deterministic Diagnosis Flow (5 steps):**
1. Boot Time Summary - Analyze systemd-analyze output
2. Top Offenders - Identify units taking > 5s to start
3. Regression Check - Compare with baseline (new/regressed offenders)
4. Correlation Analysis - Link slow units to recent changes
5. Hypothesis Generation - Produce max 3 evidence-backed theories

Each step produces:
- Result: PASS/FAIL/PARTIAL/SKIPPED
- Details with evidence citations
- Implication explaining what it means

**"What Changed" Correlation Engine:**
- Parses `/var/log/pacman.log` for package installs/updates/removes
- Tracks kernel updates separately
- Detects service enable/disable events
- Ranks potential causes by evidence strength
- Every ranked item cites evidence IDs

**Hypothesis Generation (max 3):**
- "NetworkManager-wait-online is delaying boot" (90% confidence)
- "New service X is slowing boot" (85%)
- "Package update correlates with slow unit" (75%)
- "Service Y may be stuck or timing out" (70%)

**Fix Playbooks (with confirmation):**

| Playbook | Risk | Description |
|----------|------|-------------|
| `disable_wait_online` | Medium | Disable NetworkManager-wait-online.service |
| `restart_<unit>` | Low | Restart a stuck or slow service |
| `disable_<unit>` | Medium | Disable a non-essential slow service |

Each playbook has:
- Preflight checks (unit exists, etc.)
- Commands with timeout
- Post-checks with wait times
- Rollback commands
- Policy blocking for critical services (systemd-*, dbus, login, udev)
- Confirmation phrase: `I CONFIRM`
- Verification pending note for post-reboot checks

**Recipe Capture:**
When a fix playbook succeeds with reliability >= 80%:
- Automatic recipe creation request
- Problem description from diagnosis
- Solution from playbook
- Preconditions (boot time, target unit)
- Evidence patterns for matching

**Case File Integration:**
- `boot_doctor.json` in case files
- Evidence bundle stored
- Diagnosis result tracked
- Playbook execution recorded
- Verification pending items for post-reboot confirmation
- Notes and timestamps

### Files Changed

- **NEW:** `crates/anna_common/src/boot_doctor.rs` - Full boot doctor implementation (~1600 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.41
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `TODO.md` - Version bump
- **MODIFIED:** `RELEASE_NOTES.md` - This file

### Tests

18 new tests covering:
- Boot health display
- Risk level display
- Timing is_slow/is_very_slow checks
- Evidence health determination
- Healthy system diagnosis
- Slow boot diagnosis
- New offender detection
- Regression detection
- Correlation engine
- Wait-online playbook generation
- Restart service playbook
- Disable critical service blocked by policy
- Max 3 hypotheses limit
- Case file workflow
- Recipe capture on success
- Regression percent calculation
- Timing summary formatting

### Example Transcript: "Boot Got Slower" with Offender Ranking

```
[you] to [anna]: my boot got slower recently

[anna] to [translator]: parse user intent
[translator] to [anna]: boot diagnosis request

[anna] to [boot_doctor]: collect evidence (lookback: 14 days)
[boot_doctor] to [anna]:
  Evidence collected:
  - Boot timing: 45000ms total (userspace: 38000ms)
  - Top offenders: 3 units > 5s
  - Baseline available from 7 days ago (userspace: 15000ms)
  - Recent changes: 5 packages, 1 service enable

[anna] to [boot_doctor]: run diagnosis
[boot_doctor] to [anna]:
  Step 1: Boot Time Summary - PARTIAL
    Boot time: 45000ms (userspace 38000ms > 30s threshold)
    Implication: Boot is slower than ideal. Optimization possible.

  Step 2: Top Offenders - PARTIAL
    Found 3 slow units:
    - NetworkManager-wait-online.service (28000ms)
    - docker.service (6500ms) [NEW]
    - systemd-journal-flush.service (3200ms)

  Step 3: Regression Check - FAIL
    2 new slow units, 1 regressed unit
    - NEW: docker.service (6500ms) not in baseline
    - REGRESSED: NetworkManager-wait-online.service 28000ms (was 8000ms, +250%)

  Step 4: Correlation Analysis - PASS
    Found 2 correlation(s):
    - docker.service slow after Package Install docker
    - NetworkManager-wait-online regressed after networkmanager 1.46.0-1 update

  Step 5: Hypothesis Generation - PASS
    Generated 3 hypothesis(es):
    1. "NetworkManager-wait-online.service is delaying boot" (90% confidence)
       Suggested playbook: disable_wait_online
    2. "New service docker.service is slowing boot" (85% confidence)
       Correlated with: docker package install 2 days ago
    3. "NetworkManager update may have caused regression" (75% confidence)
       Evidence: pacman log shows upgrade on 2025-12-01

[anna] to [you]:
  Boot Health: DEGRADED

  Your boot time has regressed from ~15s to ~38s (userspace).

  Top hypothesis (90% confidence):
    NetworkManager-wait-online.service is delaying boot by 28s.
    This service waits for full network connectivity before proceeding.

  Correlated changes:
    - docker installed 2 days ago (new 6.5s offender)
    - networkmanager updated (wait-online regressed from 8s to 28s)

  Recommended fix:
    Disable NetworkManager-wait-online.service
    Risk: Medium - network-dependent services may start before network ready
    To apply: type "I CONFIRM" when prompted

  Evidence: [ev-boot-1733234567-timing] [ev-boot-1733234567-blame]
            [ev-pacman-1] [ev-pacman-2]

  Reliability: 85%
```

### Example Change Set: Disabling Non-Essential Slow Service (Policy-Gated)

```
[anna] to [you]:
  Playbook: disable_wait_online
  Action: Disable NetworkManager-wait-online.service

  This will:
  1. Run preflight: Verify unit exists
  2. Execute: systemctl disable --now NetworkManager-wait-online.service
  3. Post-check: Verify unit is disabled

  Rollback available:
    systemctl enable --now NetworkManager-wait-online.service

  WARNING: This is a Medium-risk operation.
  Network will still work, but network-dependent services may start
  before network is fully ready.

  Verification pending: Boot time improvement will be verified on next boot

  To proceed, type exactly: I CONFIRM

[you] to [anna]: I CONFIRM

[anna] to [boot_doctor]: execute playbook disable_wait_online
[boot_doctor] to [anna]:
  Preflight: PASS - Unit exists
  Executed: systemctl disable --now NetworkManager-wait-online.service
  Post-check: PASS - Unit disabled

  Result: SUCCESS
  Reliability: 75% (capped until verified on reboot)

  Next steps:
  - Reboot to verify boot time improvement
  - Run "systemd-analyze time" after reboot
  - Expected improvement: ~25-28 seconds

[anna] to [you]:
  Fix applied successfully.

  The service has been disabled. On next reboot:
  - Boot should be ~25-28 seconds faster
  - Run me again to verify and capture recipe if successful

  Verification pending item added to case file.
```

---

## v0.0.40 - Arch Audio Doctor v1 (PipeWire Focus + Fix-It Playbooks + Recipe Capture)

**Release Date:** 2025-12-03

### Summary

Anna can now diagnose and fix common audio issues on Arch Linux desktops. The Audio Doctor focuses on PipeWire (the modern audio stack) while providing legacy PulseAudio conflict detection. Successful fixes with high reliability automatically create recipes for future reuse.

### Key Features

**Supported Audio Stacks:**
- **PipeWire + WirePlumber** (primary, recommended)
- **PulseAudio** (legacy detection, conflict resolution)
- **ALSA** (basic hardware detection)
- **Bluetooth** (bluez integration)

**Common Issues Diagnosed:**
- "No sound" - Service not running, muted, wrong output
- "Mic not working" - Source issues, permissions
- "Bluetooth audio broken" - Service, adapter, connection, profile issues
- "Wrong output" - Default sink misconfigured
- "Crackling" - Service restart often helps

**Evidence Bundle:**
- Audio stack detection (pipewire vs pulseaudio vs alsa-only)
- Service states (pipewire, wireplumber, pulseaudio - user services)
- ALSA devices (aplay/arecord -l)
- PipeWire/WirePlumber nodes (wpctl status)
- Default sink/source with volume/mute state
- Bluetooth adapter and device states
- User permissions (audio/video/bluetooth groups, /dev/snd access)
- Recent journal logs

**Deterministic Diagnosis Flow (6 steps):**
1. Identify Audio Stack - Detect PipeWire/PulseAudio/ALSA
2. Verify Services - Check pipewire/wireplumber running
3. Confirm Devices - ALSA hardware + PipeWire nodes
4. Check Default Output - Sink selection, volume, mute
5. Check Conflicts - PulseAudio vs PipeWire
6. Check Bluetooth - Service, adapter, device, profile

Each step produces:
- Result: PASS/FAIL/PARTIAL/SKIPPED
- Details with evidence
- Implication explaining what it means

**Hypothesis Generation (max 3):**
- "PipeWire user service not running" (95% confidence)
- "WirePlumber session manager not running" (90%)
- "Default output is muted" (95%)
- "Volume too low" (85%)
- "PulseAudio running alongside PipeWire" (90%)
- "Bluetooth using low-quality profile" (80%)
- "Audio device permission issue" (60%)

**Fix Playbooks (with confirmation):**

| Playbook | Risk | Description |
|----------|------|-------------|
| `restart_pipewire` | Low | Restart PipeWire + WirePlumber user services |
| `restart_wireplumber` | Low | Restart WirePlumber only |
| `unmute_volume` | Low | Unmute + set volume to 50% |
| `set_default_sink` | Low | Set first available sink as default |
| `stop_pulseaudio` | Medium | Stop and mask PulseAudio (BLOCKED by default) |
| `set_bt_a2dp` | Low | Switch Bluetooth to high-quality A2DP profile |

Each playbook has:
- Preflight checks
- Commands (run as target user for user services)
- Post-checks with wait times
- Rollback commands
- Policy blocking for higher-risk operations
- Confirmation phrase: `I CONFIRM`

**Recipe Capture:**
When a fix playbook succeeds with reliability >= 80%:
- Automatic recipe creation request
- Problem description from diagnosis
- Solution from playbook
- Preconditions (audio stack, target user)
- Evidence patterns for matching

**Target User Correctness:**
All user service operations (pipewire, wireplumber, pulseaudio) are executed for the target user, not root.

**Case File Integration:**
- `audio_doctor.json` in case files
- Evidence bundle stored
- Diagnosis result tracked
- Playbook execution recorded
- Recipe capture requests saved

### Files Changed

- **NEW:** `crates/anna_common/src/audio_doctor.rs` - Full audio doctor implementation (~1400 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.40
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `TODO.md` - Version bump
- **MODIFIED:** `RELEASE_NOTES.md` - This file

### Tests

16 new tests covering:
- Audio stack display
- Audio health display
- Service state healthy check
- Audio node issue detection
- PulseAudio conflict detection
- Evidence health determination
- Healthy system diagnosis
- PipeWire stopped diagnosis
- Muted output diagnosis
- PulseAudio conflict diagnosis
- Playbook generation
- PulseAudio stop blocked by policy
- Case file workflow
- Recipe capture on success
- Bluetooth profile hypothesis
- Max 3 hypotheses limit

### Example Transcript: "No Sound" Resolved

```
[you] to [anna]: I have no sound

[anna] to [translator]: parse user intent
[translator] to [anna]: audio diagnosis request

[anna] to [audio_doctor]: collect evidence for testuser
[audio_doctor] to [anna]: Evidence collected. Stack: PipeWire

[anna] to [audio_doctor]: run diagnosis
[audio_doctor] to [anna]:
  Step 1: Identify Audio Stack - PASS
    PipeWire detected as active audio stack
  Step 2: Verify Services - FAIL
    PipeWire running but WirePlumber stopped
    Implication: Session/policy management missing
  Step 3: Confirm Devices - PASS
    Found 2 sink(s), 1 source(s)
  Step 4: Check Default Output - PASS
    Default sink 'Built-in Audio' at 50% volume, not muted
  Step 5: Check Conflicts - PASS
    No conflicts detected
  Step 6: Check Bluetooth - SKIPPED

  Hypothesis: WirePlumber session manager not running (90% confidence)
  Suggested fix: restart_wireplumber

[anna] to [you]: Your audio is not working because WirePlumber
                 (the session manager) is stopped. PipeWire itself
                 is running but cannot route audio without WirePlumber.

                 I can restart WirePlumber for you. Type:
                 I CONFIRM (apply fix)

[you] to [anna]: I CONFIRM

[anna] to [audio_doctor]: execute restart_wireplumber for testuser
[audio_doctor] to [anna]:
  Command: systemctl --user restart wireplumber
  Result: exit code 0
  Post-check: WirePlumber running - PASS
  Reliability: 92%

[anna] to [junior]: validate fix result
[junior] to [anna]: SCORE 92, fix verified, recipe creation triggered

[anna] to [you]: Fixed! WirePlumber is now running.
                 Reliability: 92%

                 Created recipe: "Fix: No sound because WirePlumber stopped"
```

### Example Transcript: Bluetooth Audio Profile Issue

```
[you] to [anna]: my bluetooth headphones sound terrible

[anna] to [audio_doctor]: collect evidence including bluetooth
[audio_doctor] to [anna]:
  Bluetooth: service active, adapter powered
  Device 'Sony WH-1000XM4' connected via HSP (low quality)
  A2DP profile available but not active

[anna] to [audio_doctor]: run diagnosis
[audio_doctor] to [anna]:
  Step 6: Check Bluetooth - PARTIAL
    Bluetooth audio device 'Sony WH-1000XM4' connected, profile: HSP
    Implication: Using low-quality profile

  Hypothesis: Bluetooth using low-quality profile (80% confidence)
    Device is using HSP (voice) instead of A2DP (high quality music)
  Suggested fix: set_bt_a2dp

[anna] to [you]: Your Bluetooth headphones are connected using the
                 HSP profile (meant for calls, low quality).
                 For music, you want A2DP (high fidelity).

                 I can switch to A2DP for you. Type:
                 I CONFIRM (apply fix)

[you] to [anna]: I CONFIRM

[anna] to [audio_doctor]: execute set_bt_a2dp for testuser
[audio_doctor] to [anna]:
  Command: wpctl set-profile ... 1
  Post-check: Profile set - PASS
  Reliability: 85%

[anna] to [you]: Switched to A2DP profile. Your headphones should
                 now sound much better for music playback.
                 Reliability: 85%
```

---

## v0.0.39 - Arch Storage Doctor v1 (BTRFS Focus + Safe Repair Plans)

**Release Date:** 2025-12-03

### Summary

Anna can now diagnose storage issues with a focus on BTRFS filesystems. The Storage Doctor collects comprehensive evidence, follows a deterministic diagnosis flow, generates risk-rated hypotheses, and offers safe repair plans with policy controls.

### Key Features

**Evidence Bundle:**
- Mount topology (lsblk, findmnt)
- Filesystem types (BTRFS, EXT4, XFS)
- Free space and metadata space (BTRFS-specific)
- Device errors (btrfs device stats)
- SMART health data (smartctl)
- Scrub and balance status (BTRFS)
- I/O errors from kernel log (journalctl)

**BTRFS-Specific Diagnostics:**
- Metadata pressure detection (critical when >90%)
- Device error tracking (corruption_errs, generation_errs = critical)
- Scrub status with uncorrected error detection
- Balance status monitoring
- RAID profile awareness

**Deterministic Diagnosis Flow (5 steps):**
1. Identify Filesystem Types - Detect BTRFS, EXT4, XFS
2. Check Space Usage - Free space and BTRFS metadata space
3. Check Device Health - BTRFS device stats and SMART data
4. Check BTRFS Maintenance - Scrub and balance status
5. Check I/O Error Logs - Kernel log analysis

Each step produces:
- Pass/Fail status
- Findings with risk levels (Info/Warning/Critical)
- Evidence IDs for traceability

**Hypothesis Generation (max 3):**
Each hypothesis includes:
- Summary and explanation
- Confidence percentage (0-100)
- Supporting evidence IDs
- Confirm/refute criteria
- Suggested repair plan

Example hypotheses:
- "Failing storage device" (SMART failure + I/O errors)
- "BTRFS metadata space exhaustion" (metadata >90%)
- "Data corruption detected" (scrub uncorrected errors)

**Safe Repair Plans:**

*Read-Only Plans (no confirmation needed):*
- SMART Extended Test - Run self-test on drive
- BTRFS Device Stats - View current error statistics

*Mutation Plans (confirmation required):*
- Start Scrub - Verify data integrity (allowed by default)
- Balance Metadata - Redistribute metadata chunks (BLOCKED by default)
- Clear Device Stats - Reset error counters after investigation

Each mutation plan has:
- Risk level
- Preflight checks
- Post-execution checks
- Rollback instructions
- Policy block status
- Confirmation phrase

**Policy Controls:**
- `allow_scrub: true` - Scrub operations allowed by default
- `allow_balance: false` - Balance operations BLOCKED by default (can take hours)

**Case File Integration:**
- `storage_doctor.json` in case files
- Full evidence bundle stored
- Diagnosis result tracked
- Repair history maintained
- Notes and status updates

### Storage Health Status

- **Healthy**: All checks pass, no warnings
- **Degraded**: Non-critical issues (I/O errors, SMART warnings, device errors)
- **Critical**: Data at risk (SMART failed, corruption, metadata exhausted)
- **Unknown**: Evidence collection failed

### Files Changed

- **NEW:** `crates/anna_common/src/storage_doctor.rs` - Full storage doctor implementation (~2100 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.39
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `RELEASE_NOTES.md` - This file

### Tests

17 new tests covering:
- Storage health display
- Filesystem type parsing
- Mount space checks
- BTRFS device stats error detection
- BTRFS metadata pressure detection
- SMART health warnings
- Diagnosis flow (healthy, degraded, critical)
- Hypothesis generation (max 3)
- Repair plan generation and policy blocking
- Case file workflow
- Scrub status with uncorrected errors

All 746 tests pass with zero regressions.

---

## v0.0.38 - Arch Networking Doctor v1 (WiFi/Ethernet Diagnosis + Fix-It Playbooks)

**Release Date:** 2025-12-03

### Summary

Anna can now diagnose and fix common Arch Linux networking issues. The Networking Doctor follows a deterministic diagnosis flow, generates evidence-backed hypotheses, and offers fix playbooks with confirmation, post-checks, and rollback support.

### Key Features

**Supported Network Managers:**
- NetworkManager (most common)
- iwd (Intel wireless daemon)
- systemd-networkd
- wpa_supplicant

**Deterministic Diagnosis Flow (6 steps):**
1. Physical Link - Check carrier/WiFi association
2. IP Address - Check for IPv4/IPv6 assignment
3. Default Route - Check for gateway configuration
4. IP Connectivity - Ping gateway and 1.1.1.1
5. DNS - Test name resolution
6. Manager Health - Check service status and conflicts

Each step produces:
- Result: PASS/FAIL/PARTIAL/SKIPPED
- Details with evidence
- Implication explaining what it means

**Evidence Collection Bundle:**
- Interface inventory (ip link, ip addr)
- Routes + default gateway (ip route)
- DNS config (resolvectl or resolv.conf)
- Link state + speed (ethtool)
- WiFi status (iw dev, signal strength)
- Manager state (systemctl status)
- Recent logs (journalctl filtered)

**Hypothesis Generation (1-3 max):**
Each hypothesis includes:
- Description
- Confidence percentage
- Supporting evidence IDs
- Next test to confirm/refute
- Suggested fix playbook

**Fix Playbooks (with confirmation):**
- `restart_manager` - Restart the active network manager
- `renew_dhcp` - Request new IP via DHCP
- `restart_resolver` - Restart systemd-resolved
- `start_manager` - Start stopped network manager
- `disable_conflicting` - Disable conflicting service (high risk)
- `toggle_wifi_powersave` - Disable WiFi power saving

Each playbook has:
- Risk level (Low/Medium/High)
- Confirmation phrase requirement
- Post-checks to verify fix
- Rollback steps if verification fails

**Case File Integration:**
- `networking_doctor.json` in case files
- Full diagnosis result stored
- Fix applied and result tracked
- Recipe creation on successful fix

### Files Changed

- **NEW:** `crates/anna_common/src/networking_doctor.rs` - Full networking doctor implementation (~1800 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added module and exports
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.38
- **MODIFIED:** `CLAUDE.md` - Version bump
- **MODIFIED:** `README.md` - Version bump and feature description
- **MODIFIED:** `TODO.md` - Version bump

### Example Transcript: WiFi Disconnect Diagnosis

```
=== NETWORKING DOCTOR DIAGNOSIS ===

Network Manager: NetworkManager (running)

Interfaces:
  wlan0 (wifi) - UP - 192.168.1.100/24 [WiFi: HomeNetwork]
  enp0s3 (ethernet) - DOWN - no IP

Diagnosis Flow:
  [OK] physical_link
    1 interface(s) up: wlan0 (WiFi: HomeNetwork)
    -> Physical layer is working
  [OK] ip_address
    wlan0: 192.168.1.100/24
    -> IP address acquired - DHCP or static config working
  [OK] default_route
    Default gateway: 192.168.1.1
    -> Routing table has a default route
  [FAIL] ip_connectivity
    Cannot ping gateway 192.168.1.1
    -> Gateway unreachable - check cable/WiFi or gateway device
  [SKIP] dns
    Skipped - no IP connectivity
    -> Cannot test DNS without IP connectivity
  [OK] manager_health
    NetworkManager running and healthy
    -> Network manager is functioning normally

Hypotheses:
  1. Gateway unreachable - network path issue (70% confidence)
     Fix available: restart_manager

Recommended Fix:
  Restart Network Manager - Restart the active network manager service
  Risk: low
  To apply, confirm with: I CONFIRM (apply fix)

Overall Status: DEGRADED
```

### Example Transcript: DNS Failure Diagnosis

```
=== NETWORKING DOCTOR DIAGNOSIS ===

Network Manager: NetworkManager (running)

Interfaces:
  enp0s3 (ethernet) - UP - 192.168.1.50/24

Diagnosis Flow:
  [OK] physical_link
    1 interface(s) up: enp0s3 (ethernet)
  [OK] ip_address
    enp0s3: 192.168.1.50/24
  [OK] default_route
    Default gateway: 192.168.1.1
  [OK] ip_connectivity
    Gateway 192.168.1.1 and 1.1.1.1 reachable
  [FAIL] dns
    DNS resolution failed (servers: 127.0.0.53, tests: archlinux.org: FAIL)
    -> Cannot resolve domain names - check DNS config
  [OK] manager_health
    NetworkManager running and healthy

Hypotheses:
  1. DNS resolution failure - resolver misconfigured (85% confidence)
     Fix available: restart_resolver
  2. systemd-resolved may need restart (75% confidence)
     Fix available: restart_resolver

Recommended Fix:
  Restart DNS Resolver - Restart systemd-resolved to fix DNS issues
  Risk: low
  To apply, confirm with: I CONFIRM (apply fix)

Overall Status: DEGRADED
```

### Tests Added

- `test_network_manager_detection` - Manager enum and service names
- `test_diagnosis_step_result` - Step result symbols
- `test_diagnosis_step_creation` - Step creation and pass/fail
- `test_network_evidence_creation` - Evidence struct initialization
- `test_fix_playbooks_exist` - All playbooks are defined
- `test_fix_playbook_confirmation` - Confirmation phrases required
- `test_diagnosis_status` - Status enum strings
- `test_hypothesis_generation_empty` - No hypotheses when all pass
- `test_parse_interface_line` - Ethernet interface parsing
- `test_parse_interface_line_wifi` - WiFi interface parsing
- `test_networking_doctor_case` - Case file JSON serialization
- `test_fix_risk_levels` - Risk level assignments
- `test_determine_status` - Status determination logic

---

## v0.0.37 - Recipe Engine v1 (Reusable Fixes + Safe Auto-Drafts)

**Release Date:** 2025-12-03

### Summary

Recipes now have full lifecycle management: Active, Draft, and Archived states. Recipes created from sessions with < 80% reliability are automatically saved as drafts (never auto-suggested). Drafts can be promoted to active after a successful validated run. Recipe events are tracked in case files for full audit trail.

### Key Features

**Recipe Status Lifecycle:**
- `RecipeStatus` enum: `Active`, `Draft`, `Archived`
- Active: Can be auto-suggested and executed
- Draft: Usable but never auto-suggested, awaits promotion
- Archived: Tombstone state for deleted recipes, not usable

**Creation Rules (>= 80% reliability threshold):**
- Reliability >= 80%: Recipe created as Active
- Reliability < 80%: Recipe created as Draft
- Drafts can be promoted after successful validated run
- `promote()` method on Recipe struct

**Recipe Schema Enhancements:**
- `intent_tags: Vec<String>` - Semantic tags for better matching
- `evidence_required: Vec<String>` - Evidence to collect before execution
- `post_checks: Vec<PostCheck>` - Verification after execution
- `origin_case_id: Option<String>` - Links to creating case
- `notes: String` - Free-form notes
- `confirmation_phrase: Option<String>` - Replaces `confirmation_required`

**Post-Check Types:**
- `ServiceRunning { name }` - Check if service is running
- `FileExists { path }` - Check if file exists
- `CommandSucceeds { command }` - Check if command exits 0
- `OutputContains { command, expected }` - Check command output

**Recipe Matching Improvements:**
- Match scoring uses `intent_tags` for better relevance
- Draft recipes return 0.0 match score (excluded from auto-suggest)
- Active recipes ranked by confidence and success_count

**Case File Recipe Events:**
- `RecipeEvent` struct with event types:
  - `Matched` - Recipe matched for this case
  - `Executed` - Recipe was applied
  - `Succeeded` - Recipe execution succeeded
  - `Failed` - Recipe execution failed
  - `Created` - Recipe was created from this case
  - `Promoted` - Recipe was promoted from draft
- Case files now have `recipe_events: Vec<RecipeEvent>`

**Recipe Introspection Methods:**
- `is_usable()` - Active or Draft (not Archived)
- `can_auto_suggest()` - Active with confidence >= 0.5
- `promote()` - Promote Draft to Active

### Files Changed

- **MODIFIED:** `crates/anna_common/src/recipes.rs` - RecipeStatus enum, PostCheck types, new fields, lifecycle methods
- **MODIFIED:** `crates/anna_common/src/transcript.rs` - RecipeEvent, RecipeEventType, case file integration
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Exported new types (RecipeStatus, RecipePostCheck, RecipePostCheckType, RecipeEvent, RecipeEventType)
- **MODIFIED:** `Cargo.toml` - Version bump to 0.0.37
- **MODIFIED:** `CLAUDE.md` - Version bump to 0.0.37

### Tests Added

**Recipe Tests:**
- `test_recipe_status_default` - Active by default
- `test_recipe_status_draft` - Draft behavior
- `test_recipe_status_archived` - Archived behavior
- `test_recipe_promote` - Promote draft to active
- `test_recipe_promote_already_active` - No-op for active
- `test_recipe_promote_archived_fails` - Cannot promote archived
- `test_recipe_draft_no_match_score` - Drafts don't match
- `test_post_check_types` - All PostCheckType variants
- `test_recipe_intent_tags` - Intent tags in matching
- `test_recipe_status_from_reliability_high` - >= 80% = Active
- `test_recipe_status_from_reliability_low` - < 80% = Draft
- `test_recipe_format_detail_with_status` - Status in output

**Recipe Event Tests:**
- `test_recipe_event_matched` - Matched event creation
- `test_recipe_event_executed` - Executed event creation
- `test_recipe_event_succeeded` - Succeeded event creation
- `test_recipe_event_failed` - Failed event with error message
- `test_recipe_event_created` - Created event with notes
- `test_recipe_event_promoted` - Promoted event creation
- `test_case_file_with_recipe_events` - Case file integration
- `test_case_file_recipe_created` - Recipe creation tracking
- `test_recipe_event_types_serialization` - JSON roundtrip

---

## v0.0.36 - Knowledge Packs v1 (Offline Q&A with Strict Citations)

**Release Date:** 2025-12-03

### Summary

Anna can now answer "how do I..." questions using locally indexed documentation - man pages and package docs from `/usr/share/doc`. All factual claims require K-citations (K1, K2, K3...) that reference specific documentation excerpts. No network access, no hallucination.

### Key Features

**Knowledge Packs Storage:**
- Location: `/var/lib/anna/knowledge_packs/`
- SQLite FTS5 index for fast full-text search
- Pack types: `manpages`, `package_docs`
- Trust level tracking: `official` (system docs), `local`, `user`

**Citation System:**
- K-citations: [K1], [K2], [K3]... for knowledge references
- Each citation links to: title, pack, source path, trust level, excerpt
- Junior rejects uncited factual claims
- "How do I..." questions search knowledge first

**Status Visibility:**
- `annactl status` [KNOWLEDGE] section shows:
  - Pack count and document count
  - Index size and last indexed time
  - Breakdown by source type
  - Top packs by query count

**Auto-Build on Daemon Start:**
- Knowledge packs build automatically on first run
- Limited to 500 manpages and 500 package docs for fast startup
- Full index available for subsequent queries

**Case File Integration:**
- New `knowledge_refs` field in case files
- Records: evidence_id, title, pack_id, pack_name, source_path, trust, excerpt
- Full audit trail of which docs were used for each answer

### Files Changed

- **MODIFIED:** `crates/anna_common/src/knowledge_packs.rs` - Updated header, added citation tests
- **MODIFIED:** `crates/anna_common/src/transcript.rs` - Added KnowledgeRef struct, knowledge_refs field
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Exported CaseKnowledgeRef
- **MODIFIED:** `crates/annactl/src/commands/status.rs` - Re-enabled [KNOWLEDGE] section
- **MODIFIED:** `crates/annad/src/main.rs` - Auto-build knowledge packs on start

### Example Q&A with Citations

```
[you] to [anna]: how do I connect via SSH?

[anna] to [junior]: Let me search the knowledge packs...

[junior] to [anna]: Found relevant documentation:
  [K1] ssh - OpenSSH remote login client (man:ssh, official)
  [K2] openssh - README (/usr/share/doc/openssh/README, official)

[anna] to [you]:
To connect via SSH, use:

    ssh user@hostname [K1]

Configuration files are in /etc/ssh/ for system-wide settings
and ~/.ssh/ for user settings [K2].

Reliability: 85% (cited from official documentation)
```

### Tests Added

- `test_case_file_with_knowledge_refs` - Single knowledge reference
- `test_case_file_multiple_knowledge_refs` - Multiple references
- `test_citation_format_for_qa` - Citation format verification
- `test_search_how_to_question` - "How do I..." query handling

Tests: 644 passed

### Migration Notes

- Existing case files remain compatible (knowledge_refs defaults to empty)
- Knowledge packs build automatically on first daemon start
- No configuration changes required

---

## v0.0.35 - Ollama Role Selection (Policy-Driven Model Management)

**Release Date:** 2025-12-03

### Summary

Anna now independently selects models for the Translator and Junior roles based on hardware capabilities and a policy-driven configuration. The system detects hardware (CPU, RAM, GPU, VRAM), benchmarks available models, and selects the best fit for each role's requirements.

### Key Features

**Policy-Driven Model Selection:**
- Configuration file: `/etc/anna/policy/models.toml`
- Separate requirements for Translator (fast, low latency) and Junior (higher reasoning)
- Scoring weights: latency (0.3), throughput (0.2), quality (0.4), memory_fit (0.1)
- Candidate models in preference order with automatic fallback

**Hardware Detection:**
- Detects: CPU cores, total RAM, GPU vendor (nvidia/amd/intel), VRAM
- Hardware tier classification: Low (<8GB RAM), Medium (8-16GB), High (>16GB or dedicated GPU)
- Hardware hash for change detection and rebenchmarking triggers

**Model Readiness UX:**
- `annactl status` [MODELS] section shows:
  - Hardware tier (Low/Medium/High)
  - Translator model and status
  - Junior model and status (or fallback mode)
  - Download progress with ETA when pulling models
  - Readiness summary (full capability / partial / not ready)

**Fallback Behavior:**
- Translator fallback: deterministic classifier when no model available
- Junior fallback: skip mode with reliability capped at 60%
- Clear visibility in status output: "reliability capped at 60%"

**Case Files with Model Info:**
- New `models` field in case files tracks which models were used
- Records: translator model, junior model (if any), fallback states, hardware tier
- Supports debugging and learning from model performance

### Files Changed

- **NEW:** `crates/anna_common/src/model_policy.rs` - Complete policy system (500+ lines)
  - `ModelsPolicy` with role-specific policies
  - `RolePolicy` for Translator and Junior requirements
  - `ScoringWeights` for model selection scoring
  - `DownloadProgress` and `DownloadStatus` for tracking
  - `ModelReadinessState` for case file serialization
  - Default policy with candidate models for each role
- **MODIFIED:** `crates/anna_common/src/model_selection.rs` - Added role field to DownloadProgress
- **MODIFIED:** `crates/anna_common/src/transcript.rs` - Added CaseModelInfo struct and models field
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Exported model_policy and CaseModelInfo
- **MODIFIED:** `crates/annactl/src/commands/status.rs` - Enhanced [MODELS] section
- **MODIFIED:** `crates/annad/src/llm_bootstrap.rs` - Track role for each model download

### Default Policy

```toml
[translator]
max_latency_ms = 2000
min_quality_tier = "low"
max_tokens = 512
candidates = ["qwen2.5:0.5b", "qwen2.5:1.5b", "llama3.2:1b", "phi3:mini", "gemma2:2b"]
fallback = "deterministic"

[junior]
max_latency_ms = 5000
min_quality_tier = "medium"
max_tokens = 1024
candidates = ["qwen2.5:1.5b-instruct", "qwen2.5:3b-instruct", "llama3.2:3b-instruct"]
fallback = "skip"
no_junior_max_reliability = 60
```

### Migration Notes

- Existing case files remain compatible (models field is optional)
- Policy file is auto-generated on first run if missing
- No configuration changes required for default behavior

---

## v0.0.34 - Fix-It Mode (Bounded Troubleshooting Loops)

**Release Date:** 2025-12-03

### Summary

Anna can now actively troubleshoot problems using a bounded state machine approach. When you report an issue like "WiFi keeps disconnecting" or "my service won't start", Anna enters Fix-It mode: collecting evidence, forming hypotheses, testing them, and proposing fixes with full rollback support.

### Key Features

**Fix-It Mode State Machine:**
- States: `understand` → `evidence` → `hypothesize` → `test` → `plan_fix` → `apply_fix` → `verify` → `close`
- Maximum 2 hypothesis cycles before declaring "stuck" with evidence of what was tried
- Every state transition is logged in the transcript and case file
- Stuck state reports: "what I tried" + "what evidence is missing"

**Problem Category Detection:**
- Automatic detection: Networking, Audio, Performance, SystemdService, Storage, Graphics, Boot
- Each category has a predefined tool bundle for baseline evidence collection
- Example: Networking bundle includes hw_snapshot, NetworkManager status, journal warnings

**Tool Bundles (Policy-Driven):**
```
Networking: hw_snapshot_summary, service_status(NetworkManager), journal_warnings(NetworkManager, wpa_supplicant)
Audio: hw_snapshot_summary, service_status(pipewire/pulseaudio), journal_warnings(pipewire)
Performance: hw_snapshot_summary, top_resource_processes, slowness_hypotheses, what_changed
SystemdService: sw_snapshot_summary, journal_warnings
Storage: disk_usage, hw_snapshot_summary
```

**Change Sets (Mutation Batches):**
- Group 1-5 mutations into a single confirmation
- Each change specifies: what, why, risk, rollback action, post-check
- Single confirmation phrase: `I CONFIRM (apply fix)`
- Automatic rollback in reverse order if any post-check fails

**Fix Timeline Tracking:**
- New `fix_timeline.json` in case files for troubleshooting sessions
- Records: problem statement, category, all hypotheses, selected fix, state transitions
- Full audit trail for debugging and learning

**Intent Classification:**
- New `IntentType::FixIt` for troubleshooting requests
- Detection patterns: "fix my X", "X is broken", "keeps disconnecting", "won't start", etc.
- Automatic routing to Fix-It mode state machine

### Files Changed

- **NEW:** `crates/anna_common/src/fixit.rs` - Complete Fix-It mode system (400 lines)
  - `FixItState` enum with state machine
  - `FixItSession` for tracking troubleshooting progress
  - `ProblemCategory` with tool bundle mapping
  - `Hypothesis` and `HypothesisTestResult` for evidence-based reasoning
  - `ChangeSet` and `ChangeItem` for mutation batches
  - `FixTimeline` for audit trail
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added fixit module exports
- **MODIFIED:** `crates/anna_common/src/transcript.rs` - Added fix_timeline field and save
- **MODIFIED:** `crates/annactl/src/pipeline.rs` - Added IntentType::FixIt and detection
- **MODIFIED:** `Cargo.toml` - Version 0.0.34
- **MODIFIED:** `CLAUDE.md` - Version 0.0.34
- **MODIFIED:** `TODO.md` - Version 0.0.34
- **MODIFIED:** `README.md` - Version 0.0.34 with Fix-It description

### Tests Added

- `test_is_fixit_request` - Detection of troubleshooting requests
- `test_problem_category_detection` - Category classification from problem descriptions
- `test_fixit_session_creation` - Session initialization with category
- `test_fixit_session_transitions` - State machine transitions
- `test_hypothesis_cycle_limit` - Max 2 cycles enforcement
- `test_change_set_limit` - Max 5 mutations per batch
- `test_tool_bundles` - Correct tools for each category
- `test_change_set_confirmation_format` - User-facing confirmation display

### Example Fix-It Transcript

```
[you] to [anna]:
  WiFi keeps disconnecting

[anna] to [translator]:
  Classify: "WiFi keeps disconnecting"

[translator] to [anna]:
  INTENT: fix_it
  CATEGORY: networking
  TOOLS: hw_snapshot_summary, service_status(NetworkManager), journal_warnings(NetworkManager, 30)

[state] understand -> evidence
  reason: Starting evidence collection for networking issue

[anna] to [annad]:
  Execute tool bundle: networking

[annad] to [anna]:
  [E1] hw_snapshot_summary: Intel WiFi 6 AX200, driver iwlwifi
  [E2] service_status(NetworkManager): active (running)
  [E3] journal_warnings: "wlp0s20f3: deauthenticating by local choice (Reason: 3=DEAUTH_LEAVING)"

[state] evidence -> hypothesize
  reason: Evidence collected, forming hypotheses

[anna] to [junior]:
  Hypotheses based on [E1], [E2], [E3]:
  H1: Power management causing disconnects (confidence: 75%, evidence: [E3])
  H2: Driver bug in iwlwifi (confidence: 40%, evidence: [E1])

[junior] to [anna]:
  SCORE: 72
  CRITIQUE: H1 is well-supported by deauth reason. Suggest testing power management first.

[state] hypothesize -> plan_fix
  reason: H1 selected, planning fix

[anna] to [you]:
  ╭─────────────────────────────────────────────────────────────────╮
  │ Change Set: 18e4a7f2-3c1d (1 changes)
  ├─────────────────────────────────────────────────────────────────┤
  │ 1. Disable WiFi power management
  │    Why: Prevent power-save triggered disconnects
  │    Risk: low
  │    Rollback: Re-enable power management
  ├─────────────────────────────────────────────────────────────────┤
  │ To apply, type: I CONFIRM (apply fix)
  ╰─────────────────────────────────────────────────────────────────╯
```

### Rollback Behavior Example

```
[state] apply_fix -> verify
  reason: Change applied, verifying

[annad] to [anna]:
  Post-check FAILED: Network still disconnecting after 30 seconds

[state] verify -> apply_fix
  reason: Post-check failed, rolling back

[anna] to [annad]:
  Rollback: Re-enable power management

[anna] to [you]:
  Fix attempt failed. Rolled back change.
  Hypothesis H1 (power management) was not the cause.
  Cycle 1/2 complete. Trying next hypothesis...

[state] apply_fix -> hypothesize
  reason: Rollback complete, trying H2
```

---

## v0.0.33 - Human-First Transcript + Local Case Files

**Release Date:** 2025-12-03

### Summary

Anna now includes a human-first debug transcript system and local case file storage for troubleshooting. Debug levels now control verbosity, not readability - transcripts are always human-readable. Development mode defaults to maximum verbosity. Case files are stored locally per request and can be queried via natural language.

### Key Features

**Human-First Transcript Renderer:**
- `TranscriptBuilder` produces human-readable output at all debug levels
- Format: `[you] to [anna]:` style with proper text wrapping
- Debug level 0: User-facing messages only
- Debug level 1: Includes internal Anna/Translator dialogue
- Debug level 2: Full verbosity with Junior/Senior dialogue
- Evidence citations displayed inline with messages
- Automatic text wrapping to terminal width

**Local Case Files:**
- Case files stored per request at `/var/lib/anna/cases/YYYY/MM/DD/<request_id>/`
- Each case contains:
  - `summary.txt` - Human-readable case summary
  - `transcript.log` - Full debug transcript
  - `evidence.json` - Evidence entries with tool calls
  - `policy_refs.json` - Policy references cited
  - `timing.json` - Timing breakdown per phase
  - `result.json` - Outcome and reliability score
- Atomic file writes with secret redaction
- Automatic pruning (30 days retention, 1GB max)

**Case Retrieval via Natural Language:**
- "Show me the last case summary" - Displays most recent case
- "Show me what happened in the last failure" - Shows last failed case
- "List today's cases" - Shows today's cases
- "List recent cases" - Shows recent case summaries
- New tools: `last_case_summary`, `last_failure_summary`, `list_today_cases`, `list_recent_cases`
- Intent classifier detects case-related queries

**Dev Mode Defaults:**
- `dev_mode = true` in config during development
- `debug_level = 2` default for maximum verbosity
- `UiConfig.is_dev_mode()` and `effective_debug_level()` helpers
- [CASES] section in status shows dev mode indicator

**Status Additions:**
- New [CASES] section showing:
  - Dev mode status
  - Recent case count with success/failure breakdown
  - Case storage usage
  - Last failure summary
  - Latest case list (in non-compact mode)

**Helper Improvements:**
- `ethtool` now always installed (useful for USB/Thunderbolt adapters, wifi stats)
- Relevance check removed - network diagnostics always available

### Files Changed

- **NEW:** `crates/anna_common/src/transcript.rs` - Complete transcript and case file system (680+ lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - Added transcript module exports
- **MODIFIED:** `crates/anna_common/src/config.rs` - Added dev_mode, updated debug_level default to 2
- **MODIFIED:** `crates/anna_common/src/tools.rs` - Added case retrieval tools
- **MODIFIED:** `crates/anna_common/src/tool_executor.rs` - Added case tool executors
- **MODIFIED:** `crates/anna_common/src/helpers.rs` - Made ethtool always relevant
- **MODIFIED:** `crates/annactl/src/pipeline.rs` - Added case query detection
- **MODIFIED:** `crates/annactl/src/commands/status.rs` - Added [CASES] section
- **MODIFIED:** `Cargo.toml` - Version 0.0.33
- **MODIFIED:** `CLAUDE.md` - Version 0.0.33
- **MODIFIED:** `TODO.md` - Version 0.0.33
- **MODIFIED:** `README.md` - Version 0.0.33

### Tests Added

- `test_transcript_builder` - TranscriptBuilder message handling
- `test_transcript_debug_levels` - Debug level filtering
- `test_case_file_evidence` - Evidence entry management
- `test_case_file_outcome` - Outcome and error handling
- `test_case_file_timing` - Timing information
- `test_redact_transcript` - Secret redaction in case files
- `test_case_outcome_partial` - Partial completion handling

---

## v0.0.32 - CI Hardening + "No Regressions" Gate

**Release Date:** 2025-12-03

### Summary

Anna now has comprehensive CI/CD infrastructure that enforces quality gates on Arch Linux, the only supported platform. All builds, tests, and releases run in Arch Linux containers with strict validation of version consistency, documentation, and code quality.

### Key Features

**GitHub Actions CI (Arch-Only):**
- Build matrix: debug + release profiles in `archlinux:latest` container
- Unit tests and integration tests
- Clippy lints (advisory)
- Rustfmt check (advisory)
- Security audit via cargo-audit (advisory)
- Smoke tests for CLI verification
- Repo hygiene checks
- Policy/redaction security tests
- Final "CI Pass" gate requiring all critical jobs

**Smoke Tests:**
- `annactl --version` must succeed and show valid version
- `annactl --help` must not show legacy commands (sw, hw)
- `annactl status` must handle missing daemon gracefully
- `annactl <request>` must produce Reliability score even in fallback mode

**Release Pipeline Guardrails:**
- Triggered on `v*.*.*` tags only
- Validates version consistency:
  - Cargo.toml version matches tag
  - README.md updated for version
  - CLAUDE.md updated for version
  - RELEASE_NOTES.md has entry for version
  - TODO.md version matches
- Builds in Arch Linux container
- Runs full test suite
- Generates SHA256 checksums
- Extracts release notes automatically
- Creates GitHub release

**Repo Hygiene Checks:**
- No legacy commands (annactl sw, annactl hw, --json) in docs
- Version consistency across all documentation files
- Root-level file allowlist enforcement
- RELEASE_NOTES.md must have current version entry

**Documentation Updates:**
- Explicit "Arch Linux only" support statement
- CI badge in README
- CI/CD section explaining pipeline
- CLAUDE.md updated with CI rules:
  - No green, no merge
  - No release without updated docs
  - No regressions allowed

### Files Changed

- **NEW:** `.github/workflows/ci.yml` - Comprehensive CI workflow
- **MODIFIED:** `.github/workflows/release.yml` - Added guardrails
- **MODIFIED:** `README.md` - v0.0.32, CI badge, Arch-only statement, CI/CD section
- **MODIFIED:** `CLAUDE.md` - v0.0.32, CI rules, platform support
- **MODIFIED:** `Cargo.toml` - version 0.0.32
- **MODIFIED:** `TODO.md` - version 0.0.32

### CI Jobs Summary

| Job | Purpose | Blocking? |
|-----|---------|-----------|
| build (debug) | Compile in debug mode | Yes |
| build (release) | Compile in release mode | Yes |
| test | Unit + integration tests | Yes |
| clippy | Lint warnings | No (advisory) |
| fmt | Format check | No (advisory) |
| audit | Security vulnerabilities | No (advisory) |
| smoke | CLI verification | Yes |
| hygiene | Repo cleanliness | Yes |
| security-tests | Redaction/policy tests | Yes |
| ci-pass | Final gate | Yes |

### Release Validation Checks

1. Tag version matches Cargo.toml
2. README.md mentions version
3. CLAUDE.md mentions version
4. RELEASE_NOTES.md has version entry
5. TODO.md has current version
6. All tests pass
7. Binaries build successfully

---

## v0.0.31 - Reliability Engineering Integration

**Release Date:** 2025-12-03

### Summary

Anna now tracks her own reliability metrics in real-time with proper integration into the request pipeline. Metrics (request success/failure, tool success/failure, LLM timeouts, cache hits) are recorded during every request. Error budgets trigger alerts when thresholds are exceeded. Self-diagnostics reports can be generated via natural language requests.

### Key Features

**Pipeline Metrics Integration:**
- Metrics recording wired into `process()` function in pipeline.rs
- RequestStart/Success recorded at beginning/end of each request
- TranslatorStart/Success/Timeout recorded for Translator LLM calls
- ToolStart/Success/Failure recorded for each tool execution
- JuniorStart/Success/Timeout recorded for Junior LLM calls
- Latency samples recorded for translator, tools, junior, and e2e
- CacheHit/CacheMiss recorded based on cache status
- Metrics automatically pruned and saved after each request

**Natural Language Support for Reliability Tools:**
- Translator system prompt updated with reliability tools (v0.0.31)
- New tools available: self_diagnostics, metrics_summary, error_budgets
- Source planning rules for "diagnostics report", "metrics", "error budget"
- Deterministic fallback handles reliability keywords
- Tool plan generator creates plans for reliability evidence needs

**Tool Format Examples:**
```
TOOLS: self_diagnostics
TOOLS: metrics_summary(days=7), error_budgets
```

**User Queries Supported:**
- "Generate a self-diagnostics report"
- "Show me the error budgets"
- "What are my reliability metrics?"
- "Generate a bug report"

### Files Changed

- **MODIFIED:** `crates/annactl/src/pipeline.rs`
  - Added `MetricsStore, MetricType` imports
  - Added metrics recording in `process()` function
  - Updated TRANSLATOR_SYSTEM_PROMPT with reliability tools
  - Added reliability keywords to `classify_intent_deterministic()`
  - Added reliability tools to `generate_tool_plan_from_evidence_needs()`

- **MODIFIED:** `Cargo.toml` - version 0.0.31
- **MODIFIED:** `CLAUDE.md` - version 0.0.31
- **MODIFIED:** `README.md` - v0.0.31 documentation
- **MODIFIED:** `TODO.md` - version 0.0.31

### Metrics Recording Points

```rust
// Start of request
metrics.record(MetricType::RequestStart);

// Translator LLM
metrics.record(MetricType::TranslatorStart);
metrics.record(MetricType::TranslatorSuccess); // or TranslatorTimeout

// Tools
for each tool:
  metrics.record(MetricType::ToolStart);
  metrics.record(MetricType::ToolSuccess); // or ToolFailure

// Junior LLM
metrics.record(MetricType::JuniorStart);
metrics.record(MetricType::JuniorSuccess); // or JuniorTimeout

// End of request
metrics.record(MetricType::RequestSuccess);
metrics.record_latency("e2e", total_ms);
metrics.record(MetricType::CacheHit); // or CacheMiss
metrics.save();
```

### Tests

- All 45 existing tests pass (25 pipeline unit tests + 20 CLI tests)
- 14 reliability module tests pass
- No regressions in existing functionality

---

## v0.0.30 - Helper Auto-Installation on Daemon Start

**Release Date:** 2025-12-03

### Summary

Anna now automatically installs missing helpers on daemon startup and maintains them via periodic health checks. If a user removes an Anna-installed helper, it will be reinstalled on the next health check (every 10 minutes).

### Key Features

- `install_missing_helpers()` called on daemon startup
- Periodic helper health check every 10 minutes (HELPER_CHECK_INTERVAL_SECS = 600)
- Auto-reinstall of missing helpers during health checks
- Helper tracking maintained in manifest

---

## v0.0.29 - Auto-Update Artifact Name Fix

**Release Date:** 2025-12-03

### Summary

Fixed auto-update artifact name matching to handle architecture suffixes. Updates now properly detect and install new versions from GitHub releases with names like `annad-0.0.29-x86_64-unknown-linux-gnu`.

### Key Features

- `fetch_github_releases()` uses `starts_with("annad-")` pattern
- `atomic_install()` extracts base name from versioned artifacts

---

## v0.0.28 - System-Aware Helper Filtering & Reliability Improvements

**Release Date:** 2025-12-03

### Summary

Anna now shows only helpers that are relevant to your specific hardware. No more seeing ethtool if you have no ethernet, no nvme-cli if you have no NVMe drives. Improved Ollama detection reliability and cleaner status display.

### Key Features

**System-Aware Helper Filtering (v0.0.28):**
- `RelevanceCheck` enum for hardware-based filtering
- Checks: `HasEthernet`, `HasWiFi`, `HasNvme`, `HasSata`, `Always`
- `get_relevant_helper_definitions()` filters helpers by system hardware
- Hardware detection via `/sys/class/net`, `/sys/class/nvme`, `/sys/block`, `/sys/class/ata_device`
- Only shows helpers that are useful for YOUR specific machine

**Helper Relevance Mapping:**
- `ethtool` - Only shown if ethernet interfaces exist
- `iw` - Only shown if WiFi interfaces exist
- `nvme-cli` - Only shown if NVMe devices exist
- `smartmontools` - Only shown if SATA devices exist
- `hdparm` - Only shown if SATA devices exist
- `lm_sensors`, `usbutils`, `pciutils`, `ollama` - Always shown

**Improved Ollama Detection:**
- Fixed `get_helper_status_list()` to use `check_helper_presence()` with `provides_command` check
- Ollama correctly detected via `which ollama`, not just `pacman -Qi`
- Unified detection logic across all helper functions

**Cleaner Status Display:**
- Removed confusing INSTALL REVIEW section from `annactl status`
- Helpers display shows only relevant helpers
- Install script output updated to remove legacy sw/hw commands
- Now shows: `annactl status`, `annactl "question"`, `annactl` (REPL)

**README.md Updated:**
- Comprehensive documentation for v0.0.28
- CLI surface documentation
- Helper auto-installation explained
- Recent changes section

### Files Changed

- **MODIFIED:** `crates/anna_common/src/helpers.rs`
  - Added `RelevanceCheck` enum
  - Added `is_helper_relevant()` function
  - Added hardware detection: `has_ethernet_interfaces()`, `has_wifi_interfaces()`, `has_nvme_devices()`, `has_sata_devices()`
  - Added `get_relevant_helper_definitions()`
  - Updated `get_helper_status_list()` to use relevant helpers only
  - Updated `refresh_helper_states()` to use relevant helpers only
  - Added `relevance_check` field to `HelperDefinition`

- **MODIFIED:** `crates/annactl/src/commands/status.rs`
  - Removed `print_installer_review_section()` function
  - Removed `InstallState` import

- **MODIFIED:** `scripts/install.sh`
  - Updated completion message to show correct CLI surface
  - Removed legacy sw/hw command references

- **MODIFIED:** `README.md`
  - Complete rewrite for v0.0.28

### Hardware Detection Logic

```rust
// Ethernet: Check /sys/class/net for eth*, en*, em* interfaces
// WiFi: Check for /sys/class/net/<name>/wireless or wlan*/wlp* names
// NVMe: Check /sys/class/nvme has entries
// SATA: Check /sys/class/ata_device or sd* devices with "ata" in path
```

### Example Status (System with WiFi only, NVMe only)

Before v0.0.28:
```
[HELPERS]
  ethtool       missing (Anna will install when needed)
  smartmontools missing (Anna will install when needed)
  nvme-cli      missing (Anna will install when needed)
  ...8 more helpers...
```

After v0.0.28:
```
[HELPERS]
  iw            present (user-installed)
  nvme-cli      present (Anna-installed)
  lm_sensors    present (user-installed)
  ...only relevant helpers shown...
```

---

## v0.0.22 - Reliability Engineering (Metrics, Error Budgets, and Self-Diagnostics)

**Release Date:** 2025-12-03

### Summary

Anna now tracks her own reliability like an SRE-managed service. Request success rates, tool failures, mutation rollbacks, and LLM timeouts are continuously recorded with configurable error budgets that trigger alerts when burned. A new `self_diagnostics()` tool generates comprehensive health reports with evidence IDs for every claim.

### Key Features

**Metrics Collection:**
- Structured metrics stored locally in `/var/lib/anna/internal/metrics.json`
- Tracks 16 metric types: requests, tools, mutations, LLM timeouts, cache hits/misses
- Rolling 7-day retention with daily aggregation
- Latency recording with p50/p95 percentile calculation

**Error Budgets (SRE-Style):**
- Configurable thresholds in `config.toml`:
  - Request failures: max 1% per day (default)
  - Tool failures: max 2% per day (default)
  - Mutation rollbacks: max 0.5% per day (default)
  - LLM timeouts: max 3% per day (default)
- Budget states: Ok, Warning (50% burned), Critical (80% burned), Exhausted (100%+)
- Automatic alerts generated when budgets are burned

**Self-Diagnostics Tool:**
- `self_diagnostics()` - read-only tool generating comprehensive health reports
- Sections with individual evidence IDs:
  - Version information
  - Install review status
  - Update state (channels, phases)
  - Model readiness (Ollama, selected models)
  - Policy status (capabilities, blocked, risk files)
  - Storage usage
  - Error budget consumption
  - Recent errors (redacted)
  - Active alerts
- Overall status derived from worst section status
- Redaction applied to error messages (secrets never exposed)

**Additional Tools:**
- `metrics_summary(days)` - reliability metrics for specified period
- `error_budgets()` - current budget consumption status

**Status Display ([RELIABILITY] section):**
```
[RELIABILITY]
  Status:           healthy

  Error Budgets (today):
    request_failures:  0.0% / 1.0% [OK]
    tool_failures:     0.0% / 2.0% [OK]
    mutation_rollback: 0.0% / 0.5% [OK]
    llm_timeouts:      0.0% / 3.0% [OK]

  Request Success Rate (7d): 100.0% (42/42)
  Latency (7d): p50=1234ms, p95=2456ms
```

### Configuration (config.toml)

```toml
[reliability]
# Error budget thresholds (percentages)
[reliability.error_budgets]
request_failure_percent = 1.0
tool_failure_percent = 2.0
mutation_rollback_percent = 0.5
llm_timeout_percent = 3.0

# Metrics settings
[reliability.metrics]
retention_days = 7
```

### New Types

- `MetricType` - 16 metric categories (RequestSuccess, ToolFailure, etc.)
- `DailyMetrics` - per-day aggregated counts and latencies
- `MetricsStore` - persistent metrics storage with schema versioning
- `ErrorBudgets` - configurable budget thresholds
- `BudgetStatus` - current consumption state per category
- `BudgetState` - Ok, Warning, Critical, Exhausted
- `BudgetAlert` - generated alert with severity and message
- `DiagnosticsReport` - full health report with sections
- `DiagnosticsSection` - individual section with evidence ID

### Files Changed

- **NEW:** `crates/anna_common/src/reliability.rs` (~1300 lines)
- **MODIFIED:** `crates/anna_common/src/lib.rs` - module + exports
- **MODIFIED:** `crates/anna_common/src/config.rs` - ReliabilityConfig
- **MODIFIED:** `crates/anna_common/src/tools.rs` - 3 new tools
- **MODIFIED:** `crates/anna_common/src/tool_executor.rs` - tool implementations
- **MODIFIED:** `crates/annactl/src/commands/status.rs` - [RELIABILITY] section

### Tests

14 unit tests covering:
- Metric type string conversions
- Daily metrics increment/get
- Latency percentile calculations (p50, p95)
- Budget state calculation (Ok, Warning, Critical, Exhausted)
- Budget status calculation from metrics
- Budget alert generation on threshold breach
- Warning threshold detection
- Error budgets default values
- Ops log entry creation
- Diagnostics status strings
- Metrics store defaults
- Average latency calculation

---

## v0.0.21 - Performance and Latency Sprint

**Release Date:** 2025-12-03

### Summary

Anna now feels snappier with time-to-first-output (TTFO) optimizations, intelligent caching for repeated queries, and strict token budgets per role. The goal: always on, always fast, always transparent.

### Key Features

**Time-to-First-Output (TTFO) < 150ms:**
- Immediate header line and request display
- "I'm starting analysis and gathering evidence..." indicator
- No waiting for LLM before showing progress
- Streaming updates as work progresses

**Token Budgets Per Role:**
- `translator.max_tokens = 256` with `translator.max_ms = 1500`
- `junior.max_tokens = 384` with `junior.max_ms = 2500`
- Graceful degradation if exceeded (recorded, reliability adjusted)
- Configurable via `config.toml [performance]` section

**Read-Only Tool Result Caching:**
- Cache keyed by: tool name + args + snapshot version hash
- Default TTL: 5 minutes (configurable)
- Storage: `/var/lib/anna/internal/cache/tools/`
- Repeated queries like "what changed last 14 days" are instant

**LLM Response Caching (Safe):**
- Cache Translator plans and Junior critiques when:
  - Same request text (hashed)
  - Same evidence hashes
  - Same policy version
  - Same model version
- Redaction occurs before caching (secrets never stored)
- Default TTL: 10 minutes (configurable)
- Storage: `/var/lib/anna/internal/cache/llm/`

**Performance Statistics:**
- Latency samples tracked per request (total, translator, tools, junior)
- Cache hit rate calculated and displayed
- Top cached tools tracked
- Budget violations recorded

**Status Display ([PERFORMANCE] section):**
```
[PERFORMANCE]
  Samples:    42 (last 24h)
  Avg total:  1234ms
  Translator: 456ms avg
  Junior:     678ms avg
  Cache hit:  73% (31 hits, 11 misses)

  Top cached tools:
    hw_snapshot_summary (12 hits)
    sw_snapshot_summary (8 hits)
    recent_installs (6 hits)

  Cache storage:
    Tool cache:  15 entries (24 KB)
    LLM cache:   8 translator, 6 junior (48 KB)
```

### Configuration (config.toml)

```toml
[performance]
# Token budgets
[performance.budgets]
translator_max_tokens = 256
translator_max_ms = 1500
junior_max_tokens = 384
junior_max_ms = 2500
log_overruns = true

# Cache settings
[performance.cache]
tool_cache_enabled = true
tool_cache_ttl_secs = 300
llm_cache_enabled = true
llm_cache_ttl_secs = 600
max_entries = 1000
```

### New Types

- `TokenBudget`: Max tokens and time per role
- `BudgetSettings`: Configuration for all role budgets
- `BudgetViolation`: Recorded when budgets exceeded
- `ToolCacheKey`, `ToolCacheEntry`, `ToolCache`: Tool result caching
- `LlmCacheKey`, `LlmCacheEntry`, `LlmCache`: LLM response caching
- `PerfStats`, `LatencySample`: Performance tracking
- `PerformanceConfig`, `PerformanceBudgets`, `PerformanceCacheConfig`: Config types

### Cache Invalidation

Caches are automatically invalidated when:
- Snapshot data changes (tool cache)
- Policy files change (LLM cache)
- Model version changes (LLM cache)
- TTL expires (both caches)

### Files Changed

- NEW: `crates/anna_common/src/performance.rs` (~600 lines)
- MODIFIED: `crates/anna_common/src/lib.rs` (module + exports)
- MODIFIED: `crates/anna_common/src/config.rs` (PerformanceConfig)
- MODIFIED: `crates/annactl/src/pipeline.rs` (TTFO, latency tracking)
- MODIFIED: `crates/annactl/src/commands/status.rs` ([PERFORMANCE] section)

### Tests

- 10 unit tests for token budgets and cache key determinism
- Cache key determinism tests (args order, snapshot hash, policy version)
- Budget exceeded detection
- Performance stats calculations

---

## v0.0.20 - Ask Me Anything Mode (Source-Labeled Answers)

**Release Date:** 2025-12-03

### Summary

Anna now answers questions like a real senior Linux admin with clear source attribution. Every factual claim in responses is labeled with its source: system evidence [E#], knowledge documentation [K#], or explicit reasoning (Reasoning). The Translator plans the right source mix based on question type, and Junior penalizes unlabeled claims.

### Key Features

**Source Labeling System:**
- `[E#]` for system evidence (measurements, snapshots, tool output)
- `[K#]` for knowledge pack documentation (man pages, package docs)
- `(Reasoning)` for general inference not from direct evidence
- All factual claims must be labeled - unlabeled claims reduce reliability score

**Question Type Classification:**
- `HowTo`: "How do I...?" → knowledge packs first, then system tools
- `SystemStatus`: "What is happening on my machine?" → system evidence first
- `Mixed`: Both how-to and system state → query both sources
- `General`: Documentation or reasoning

**New Read-Only Tools:**
- `answer_context()`: Target user, distro, kernel, available knowledge packs
- `source_plan(request)`: Plans which sources to query with rationale
- `qa_stats()`: Today's Q&A statistics

**Junior Source Label Enforcement:**
- New penalty: `-10` for factual claim without [E#], [K#], or (Reasoning) label
- UNCITED_CLAIMS output now checks for missing source labels
- Examples in prompt show good vs bad source labeling

**Status Display ([Q&A TODAY] section):**
```
[Q&A TODAY]
  Answers:    5
  Avg reliability: 82%
  Citations:  K:8 E:12 R:3
  Top sources: system_evidence: 12, knowledge_docs: 8
```

**MissingEvidenceReport for "I don't know":**
- Explicit list of what information is missing
- Suggested read-only tools to gather evidence
- Suggested knowledge queries
- No guessing - propose what could be checked

### Example Transcripts

**How-to Question (Knowledge Citations):**
```
[you] to [anna]: How do I enable syntax highlighting in Vim?

[translator] to [anna]:
  INTENT: question
  TOOLS: knowledge_search(query=vim syntax highlighting)
  RATIONALE: How-to question: searching knowledge packs for documentation

[anna] to [you]:
  To enable syntax highlighting in Vim [K1]:
  1. Add `syntax on` to your ~/.vimrc file [K1]
  2. For specific filetypes, use `filetype plugin on` [K1]

  (Reasoning) This is the standard approach for most Vim installations.

  Reliability: 88%
```

**System Question (Evidence Citations):**
```
[you] to [anna]: What did I install in the last 2 weeks that might slow my machine?

[translator] to [anna]:
  INTENT: system_query
  TOOLS: recent_installs(days=14), what_changed(days=14), slowness_hypotheses(days=14)
  RATIONALE: System status question: gathering package changes and performance data

[anna] to [you]:
  In the last 14 days, you installed 8 packages [E1]:
  - electron28 (large runtime) [E1]
  - docker-compose (background service) [E2]

  Slowness analysis [E3] suggests docker may be consuming resources.
  Current CPU: 45% [E4], Memory: 68% [E4]

  Reliability: 85%
```

### Files Changed

- **NEW:** `crates/anna_common/src/source_labels.rs` (~500 lines)
- `crates/anna_common/src/lib.rs` - Module export and version bump
- `crates/anna_common/src/tools.rs` - Added answer_context, source_plan, qa_stats tools
- `crates/anna_common/src/tool_executor.rs` - Tool execution handlers
- `crates/annactl/src/pipeline.rs` - Updated Translator and Junior prompts
- `crates/annactl/src/commands/status.rs` - [Q&A TODAY] section
- `Cargo.toml` - Version 0.0.20

### Tests

- 8 unit tests in source_labels.rs
- Question type classification
- Source plan creation
- Citation detection and counting
- Q&A statistics tracking

---

## v0.0.19 - Offline Documentation Engine (Knowledge Packs)

**Release Date:** 2025-12-03

### Summary

Anna now has a local knowledge base for answering general questions without network access. Knowledge packs index man pages, package documentation, project docs, and user notes with SQLite FTS5 for fast full-text search. Answers include evidence citations (K1, K2...) linking to source documents.

### Key Features

**Knowledge Pack System (`/var/lib/anna/knowledge_packs/`):**
- Pack sources: manpages, package_docs, project_docs, user_notes, archwiki_cache, local_markdown
- Trust levels: Official (system docs), Local (project docs), User (user notes)
- Retention policies: Permanent, RefreshOnUpdate, Manual
- Per-pack metadata: document count, index size, timestamps

**SQLite FTS5 Index:**
- Full-text search with relevance ranking
- Automatic triggers to keep FTS index synchronized
- Evidence ID generation (K1, K2, K3...) for citation
- Excerpt extraction with keyword highlighting
- Secrets hygiene applied to all excerpts

**Knowledge Pack Schema:**
```rust
pub struct KnowledgePack {
    pub id: String,
    pub name: String,
    pub source: PackSource,
    pub trust: TrustLevel,
    pub retention: RetentionPolicy,
    pub source_paths: Vec<String>,
    pub created_at: u64,
    pub last_indexed_at: u64,
    pub document_count: usize,
    pub index_size_bytes: u64,
    pub description: String,
    pub enabled: bool,
}

pub struct SearchResult {
    pub doc_id: i64,
    pub evidence_id: String,  // K1, K2, etc.
    pub title: String,
    pub pack_id: String,
    pub pack_name: String,
    pub source_path: String,
    pub trust: TrustLevel,
    pub excerpt: String,  // redacted
    pub score: f64,
    pub matched_keywords: Vec<String>,
}
```

**Default Pack Ingestion:**
- `ingest_manpages()`: Index man pages via apropos/man commands
- `ingest_package_docs()`: Index /usr/share/doc/* markdown and text files
- `ingest_project_docs()`: Index Anna's own documentation (README, CLAUDE.md, etc.)
- `ingest_user_note()`: Add user-provided documentation

**New Read-Only Tools:**
- `knowledge_search(query, top_k)`: Search indexed documentation, returns excerpts with evidence IDs
- `knowledge_stats()`: Get pack counts, document counts, index size, last indexed time

**Status Display ([KNOWLEDGE] section):**
```
[KNOWLEDGE]
  Packs:      3
  Documents:  1247
  Index size: 2.4 MB
  Last index: 5m ago

  By source:
    manpages: 1
    package_docs: 1
    project_docs: 1

  Top packs:
    System Man Pages (42 queries)
    Package Documentation (18 queries)
```

**Security:**
- Secrets hygiene (redact_evidence) applied to all excerpts
- Restricted paths blocked from indexing
- Trust level tracking for provenance

### Files Changed

- **NEW:** `crates/anna_common/src/knowledge_packs.rs` (~900 lines)
- `crates/anna_common/src/lib.rs` - Module export and version bump
- `crates/anna_common/src/tools.rs` - Added knowledge_search, knowledge_stats tools
- `crates/anna_common/src/tool_executor.rs` - Tool execution handlers
- `crates/annactl/src/commands/status.rs` - [KNOWLEDGE] section
- `Cargo.toml` - Version 0.0.19

### Tests

- 10+ unit tests in knowledge_packs.rs
- Pack creation and document indexing
- FTS search with relevance scoring
- Evidence ID generation
- Excerpt extraction and redaction

---

## v0.0.18 - Secrets Hygiene

**Release Date:** 2025-12-03

### Summary

Anna now automatically redacts secrets from all output to prevent credential leaks. Passwords, API keys, tokens, private keys, and other sensitive data are replaced with `[REDACTED:TYPE]` placeholders throughout the transcript, evidence, and LLM prompts. Evidence from restricted paths (like ~/.ssh) is blocked entirely.

### Key Features

**Centralized Redaction Module (redaction.rs):**
- 22 secret types with compiled regex patterns
- Type-specific placeholders: `[REDACTED:PASSWORD]`, `[REDACTED:API_KEY]`, `[REDACTED:PRIVATE_KEY]`, etc.
- Lazy static compilation for performance

**Patterns Detected:**
- Passwords: `password=`, `--password`, `pwd=`
- API Keys: `api_key=`, `x-api-key:`, etc.
- Bearer tokens: `Bearer xxx`, `Authorization: Bearer`
- JWT tokens: `eyJ...` format
- Private keys: `-----BEGIN PRIVATE KEY-----`
- SSH keys: `-----BEGIN OPENSSH PRIVATE KEY-----`
- PEM blocks: RSA, EC, PGP private keys
- AWS credentials: `AKIA...`, `aws_secret_access_key`
- Azure/GCP credentials
- Git credentials: `https://user:pass@...`
- Database URLs: `postgres://user:pass@...`
- Cookies: `Set-Cookie:`, `session_id=`
- Generic secrets: `_token=`, `_secret=`, `_key=`

**Evidence Restriction Policy:**
Paths that are NEVER excerpted (content replaced with policy message):
- `~/.ssh/**`
- `~/.gnupg/**`
- `/etc/shadow`, `/etc/gshadow`
- `/proc/*/environ`
- `~/.password-store/**`
- Browser credential files (key*.db, Login Data)
- Keyrings and credential managers

**Junior Leak Detection (Rules 20-24):**
```
SECRETS LEAK PREVENTION (v0.0.18):
20. Responses MUST NOT reveal: passwords, tokens, API keys, etc.
21. Evidence from restricted paths MUST show [REDACTED:TYPE]
22. If secrets detected: force redaction, cite redaction ID, downscore
23. Restricted paths NEVER excerpted
24. Examples: BAD vs GOOD patterns

PENALTIES:
- Secret revealed in response: -50 (SECURITY LEAK)
- Unredacted restricted path: -40 (RESTRICTED PATH VIOLATION)
- Missing redaction for secret: -30 (INCOMPLETE REDACTION)
```

**Redaction in All Outputs:**
- `dialogue()` and `dialogue_always()` apply redaction
- Evidence summaries redacted before LLM prompt
- Draft answers redacted before Junior verification
- Final responses redacted for display

### New Types

```rust
/// Types of secrets that can be redacted
pub enum SecretType {
    Password, ApiKey, BearerToken, AuthHeader, PrivateKey, PemBlock,
    SshKey, Cookie, AwsCredential, AzureCredential, GcpCredential,
    GitCredential, NetrcEntry, EnvSecret, JwtToken, DatabaseUrl,
    ConnectionString, OAuthToken, WebhookSecret, EncryptionKey,
    Certificate, GenericSecret,
}

/// Result of a redaction operation
pub struct RedactionResult {
    pub text: String,
    pub redaction_count: usize,
    pub secret_types_found: Vec<SecretType>,
    pub was_redacted: bool,
}

/// Result of checking for leaks
pub struct LeakCheckResult {
    pub has_leaks: bool,
    pub leaked_types: Vec<SecretType>,
    pub penalty: i32,
    pub suggestions: Vec<String>,
}
```

### New Functions

```rust
// Main redaction
pub fn redact(text: &str) -> String;
pub fn redact_secrets(text: &str) -> RedactionResult;
pub fn contains_secrets(text: &str) -> bool;

// Context-specific redaction
pub fn redact_transcript(text: &str) -> String;
pub fn redact_evidence(content: &str, path: Option<&str>) -> Result<String, String>;
pub fn redact_audit_details(details: &serde_json::Value) -> serde_json::Value;

// Environment variable handling
pub fn redact_env_value(name: &str, value: &str) -> Cow<str>;
pub fn redact_env_map(vars: &[(String, String)]) -> Vec<(String, String)>;

// Path restriction
pub fn is_path_restricted(path: &str) -> bool;
pub fn get_restriction_message(path: &str) -> String;

// Leak detection
pub fn check_for_leaks(text: &str) -> LeakCheckResult;
pub fn calculate_leak_penalty(types: &[SecretType]) -> i32;
```

### Tests Added

- 22 unit tests for redaction patterns
- Path restriction tests with wildcard matching
- Leak detection and penalty calculation tests
- Environment variable redaction tests
- Audit details redaction tests

### Files Changed

- **NEW:** `crates/anna_common/src/redaction.rs` - Centralized redaction module
- `crates/anna_common/src/lib.rs` - Module export and version bump
- `crates/annactl/src/pipeline.rs` - Redaction integration in dialogue/evidence
- `Cargo.toml` - Version 0.0.18
- `CLAUDE.md` - Version 0.0.18
- `TODO.md` - Mark 0.0.18 completed

---

## v0.0.17 - Multi-User Correctness

**Release Date:** 2025-12-03

### Summary

Stop pretending there is only one user on the machine. Anna now correctly identifies and operates on the target user for user-scoped tasks while keeping root-only operations in annad. This prevents root-owned files appearing in user home directories and ensures correct ownership for backups.

### Key Features

**Target User Selection (Strict Precedence):**
1. REPL session chosen user (if set)
2. `SUDO_USER` environment variable (for sudo invocations)
3. Non-root invoking user (getuid)
4. Primary interactive user (most recent login, or clarification prompt if ambiguous)

**Transcript Message:**
```
[anna] to [you]:
  I will treat Barbara as the target user for user-scoped changes,
  because you invoked annactl via sudo. [E-user-12345]
```

**Safe Home Directory Detection:**
- Canonical home from /etc/passwd lookup
- NEVER guess `/home/<username>`
- Evidence ID for home directory determination
- Functions: `get_user_home()`, `is_path_in_user_home()`, `expand_home_path()`

**User-Scoped File Operations:**
- `write_file_as_user()`: Creates files owned by target user
- `backup_file_as_user()`: Backups owned by target user
- `create_dir_as_user()`: Directories with correct ownership
- `fix_file_ownership()`: Repair incorrect ownership
- Uses `install` command with `-o` and `-g` flags for atomic ownership

**UserHomePolicy in capabilities.toml:**
```toml
[mutation_tools.file_edit.user_home]
enabled = true
max_file_size_bytes = 1048576

allowed_subpaths = [
    ".config/**",
    ".local/share/**",
    ".bashrc",
    ".zshrc",
    ".vimrc",
    ".gitconfig",
    # ... more dotfiles
]

blocked_subpaths = [
    ".ssh/**",
    ".gnupg/**",
    ".password-store/**",
    ".mozilla/**/key*.db",
    ".mozilla/**/logins.json",
    # ... browser credentials
]
```

**Clarification Prompt for Ambiguous Users:**
```
[anna] to [you]:
  Which user should I target?
    1) alice (Alice Smith)
    2) bob
  Select [1-2]:
```

### New Types

```rust
/// Information about a Unix user
pub struct UserInfo {
    pub username: String,
    pub uid: u32,
    pub gid: u32,
    pub home: PathBuf,
    pub shell: String,
    pub gecos: String,
}

/// How the target user was determined
pub enum UserSelectionSource {
    ReplSession,
    SudoUser,
    InvokingUser,
    PrimaryInteractive,
    UserChoice,
    FallbackRoot,
}

/// Result of target user selection
pub struct TargetUserSelection {
    pub user: UserInfo,
    pub source: UserSelectionSource,
    pub evidence_id: String,
    pub explanation: String,
    pub required_clarification: bool,
    pub other_candidates: Vec<UserInfo>,
}

/// Selection result: determined or needs clarification
pub enum SelectionResult {
    Determined(TargetUserSelection),
    NeedsClarification(AmbiguousUserSelection),
}

/// Policy for user home directory edits
pub struct UserHomePolicy {
    pub enabled: bool,
    pub allowed_subpaths: Vec<String>,
    pub blocked_subpaths: Vec<String>,
    pub max_file_size_bytes: u64,
}
```

### Example Transcript: "Enable syntax highlighting"

```
[anna] to [you]:
  I will treat barbara as the target user for user-scoped changes,
  because you invoked annactl via sudo. [E-user-34521]

[you] to [anna]:
  Enable syntax highlighting

[anna] to [translator]:
  Classify this request:
  "Enable syntax highlighting"

[translator] to [anna]:
  Intent: action_request
  Targets: .vimrc, vim, syntax
  Risk: medium
  Tools: user_config_edit
  Confidence: 82%

[anna] to [annad]:
  Please prepare a file edit for /home/barbara/.vimrc to enable syntax highlighting.
  Target user: barbara (uid=1000, gid=1000)

[annad] to [anna]:
  Preflight checks passed: [E-pre-99821]
  - Path /home/barbara/.vimrc is allowed by user_home policy
  - File is text, size 423 bytes
  - Backup will be created at /var/lib/anna/rollback/files/... (owned by barbara:barbara)

  Dry-run diff preview: [E-diff-99822]
  --- /home/barbara/.vimrc (current)
  +++ /home/barbara/.vimrc (proposed)
  @@ lines 1-5 @@
  + syntax on
  + set hlsearch
    " existing content...
  Changes: +2 added, -0 removed

[anna] to [you]:
  I can enable syntax highlighting by adding to ~/.vimrc:
    syntax on
    set hlsearch

  This is a medium-risk change to your shell configuration.
  Type "I CONFIRM (medium risk)" to proceed:

[you] to [anna]:
  I CONFIRM (medium risk)

[anna] to [annad]:
  Execute the file edit for barbara.

[annad] to [anna]:
  Edit applied successfully. [E-mut-99823]
  - File written as barbara:barbara (uid=1000, gid=1000)
  - Backup at /var/lib/anna/rollback/files/1701612345_mut_123/vimrc
  - Post-check: syntax on present, file readable

[anna] to [you]:
  Done! Syntax highlighting is now enabled in vim.
  The change was made to /home/barbara/.vimrc
  Backup available at: /var/lib/anna/rollback/files/1701612345_mut_123/vimrc

  Reliability: 94% [E-user-34521, E-pre-99821, E-diff-99822, E-mut-99823]
```

### Testing

- 15 unit tests for target user system (selection, precedence, clarification)
- 10 unit tests for user home policy (allowed/blocked paths, glob matching)
- Integration with existing mutation safety tests

### Files Changed

- `crates/anna_common/src/target_user.rs` (NEW - ~900 lines)
- `crates/anna_common/src/policy.rs` (UserHomePolicy added)
- `crates/anna_common/src/lib.rs` (exports)
- `crates/annactl/src/pipeline.rs` (target user selection integration)
- `Cargo.toml` (version bump, libc dependency)

---

## v0.0.16 - Better Mutation Safety

**Release Date:** 2025-12-03

### Summary

Senior-engineer-level safety for all mutations: preflight checks verify preconditions, dry-run diffs preview changes, post-checks verify expected state, and automatic rollback restores system on failure. Mutation state machine tracks lifecycle from planned through verified_ok or rolled_back.

### Key Features

**Mutation State Machine:**
- `MutationState` enum: `Planned` -> `PreflightOk` -> `Confirmed` -> `Applied` -> `VerifiedOk` | `RolledBack` | `Failed`
- Complete lifecycle tracking for audit trail
- State transitions logged with evidence IDs

**Preflight Checks (`PreflightResult`):**
- File edits: path allowed, file exists/creatable, is text, size under limit, permissions OK, hash recorded, backup available
- Systemd ops: unit exists, current state captured, operation allowed by policy
- Package ops: Arch Linux check, packages exist and not blocked, disk space check
- All checks generate evidence IDs for traceability

**Dry-Run Diff Preview (`DiffPreview`):**
- Line-based diff with context, additions, removals, modifications
- `DiffLine` enum: `Context`, `Added`, `Removed`, `Modified`
- Truncated output (max 20 lines) with overflow indicator
- Shows backup path and rollback command
- Human-readable format: `+1 added, -0 removed, ~2 modified`

**Post-Check Verification (`PostCheckResult`):**
- File edits: verify file exists, readable, contains expected content, hash changed
- Systemd ops: verify active/enabled state matches expectation, no immediate failure (500ms check)
- Package ops: verify package installed/removed
- Evidence IDs for post-state documentation

**Automatic Rollback:**
- `RollbackResult` with success, message, evidence_id, restored_state
- File edits: restore from backup
- Systemd ops: restore prior active/enabled state
- Logged and cited in audit trail
- Reliability score downgrade on rollback

**SafeMutationExecutor:**
- `preflight_file_edit()`, `preflight_systemd()`, `preflight_package()`
- `dry_run_file_edit()` for diff preview without modification
- `postcheck_file_edit()`, `postcheck_systemd()`, `postcheck_package()`
- `rollback_file_edit()`, `rollback_systemd()`
- Evidence ID generation (PRE#####, POST#####, RB#####)

**Junior Enforcement (v0.0.16):**
- New rules 14-19 for mutation safety
- Penalties: -25 for missing preflight, -20 for missing diff, -25 for missing post-check, -30 for missing rollback

### Tests Added

- 21 unit tests in `mutation_safety.rs`:
  - State display, preflight checks, diff generation
  - Edit operations, rollback results
  - Helper functions (arch detection, disk space)

### Files Changed

- `crates/anna_common/src/mutation_safety.rs`: New module (~1300 lines)
- `crates/anna_common/src/lib.rs`: Added module export
- `crates/annactl/src/pipeline.rs`: Updated Junior prompt with mutation safety rules

### Example Transcript: Config Change with Diff Preview

```
[you] to [anna]:
  Add log_level=debug to /etc/myapp/config.toml

[anna] to [translator]:
  Classify this request...

[translator] to [anna]:
  Intent: action_request
  Targets: /etc/myapp/config.toml
  Risk: medium

[anna] to [annad]:
  Run preflight checks for file edit

[annad] to [anna]:
  Preflight checks for edit_file_lines:
    [OK] path_allowed: Path /etc/myapp/config.toml is allowed [POL00012]
    [OK] file_exists: File exists
    [OK] is_text_file: File is text (no binary content)
    [OK] file_size: 256 bytes (limit: 1048576 bytes)
    [OK] permissions: File is readable and writable
    [OK] hash_recorded: Current hash: a1b2c3d4e5f6...
    [OK] backup_available: Backup directory writable
    [OK] operations_valid: 1 edit operation(s) validated

[anna] to [you]:
  Preflight passed. Here's what will change:

  Changes to /etc/myapp/config.toml:
    +1 added, -0 removed, ~0 modified

  +   3 | log_level=debug

  Backup: /var/lib/anna/rollback/config.toml_REQ...
  Rollback: cp /var/lib/anna/rollback/config.toml_REQ... /etc/myapp/config.toml

  To proceed, type exactly: I CONFIRM (medium risk)
```

### Example Transcript: Post-Check Failure with Rollback

```
[anna] to [annad]:
  Execute file edit (confirmed)

[annad] to [anna]:
  File edited. Running post-checks...

  Post-checks for edit_file_lines:
    [OK] file_exists: exists
    [OK] file_readable: readable
    [FAIL] content_check: expected 'log_level=debug', got 'not found'

  Post-check FAILED. Initiating automatic rollback...

[annad] to [anna]:
  Rollback completed [RB00001]:
  Restored /etc/myapp/config.toml from backup
  Reason: post-check failed - expected content not found

[anna] to [you]:
  The change was applied but post-verification failed.
  I've automatically rolled back to the previous state.

  Reliability: 45% (downgraded due to rollback)
```

---

## v0.0.15 - Governance UX Polish

**Release Date:** 2025-12-03

### Summary

Debug level configuration for controlling output verbosity. Unified formatting module for consistent terminal output. Enhanced `annactl status` as single source of truth with comprehensive sections. No regressions - all 500+ tests pass.

### Key Features

**Debug Levels (`config.toml`):**
- Configure in `/etc/anna/config.toml` with `ui.debug_level = 0 | 1 | 2`
- Level 0 (minimal): Only [you]->[anna] and final [anna]->[you], plus confirmations
- Level 1 (normal/default): Dialogues condensed, tool calls summarized, evidence IDs included
- Level 2 (full): Full dialogues between all players, tool execution summaries, Junior critique in full

**UI Configuration (`UiConfig`):**
- `debug_level`: Output verbosity (0=minimal, 1=normal, 2=full)
- `colors_enabled`: Terminal color output (default: true)
- `max_width`: Text wrapping width (0=auto-detect terminal width)
- Helper methods: `is_minimal()`, `is_normal_debug()`, `is_full_debug()`, `effective_width()`

**Unified Formatting (`display_format.rs`):**
- `colors` module: `section_header()`, `label()`, `success()`, `warning()`, `error()`, `evidence_id()`, `reliability()`
- `SectionFormatter`: Consistent status section headers and key-value formatting
- `DialogueFormatter`: Debug level filtering for pipeline output
- Format helpers: `format_bytes()`, `format_timestamp()`, `format_duration_ms()`, `format_percent()`, `format_eta()`
- Text helpers: `wrap_text()`, `indent()`, `format_list()`, `format_summary()`

**Enhanced Status Display (`annactl status`):**
- [VERSION]: Current version and build info
- [INSTALLER REVIEW]: Installation health and component status
- [UPDATES]: Update channel and status
- [MODELS]: LLM model status (Translator, Junior)
- [POLICY]: Policy status, schema version, blocked counts
- [HELPERS]: Installed helper packages
- [ALERTS]: Active alerts by severity
- [LEARNING]: Session memory and recipe counts
- [RECENT ACTIONS]: Audit log summary with last 3 actions
- [STORAGE]: Disk usage for Anna directories

**Pipeline Updates:**
- `dialogue()` function respects debug level configuration
- `dialogue_always()` for confirmations that always show
- Condensed message display at debug level 0

### Tests Added

- 19 tests in `display_format.rs` for formatting utilities
- 5 tests in `config.rs` for UiConfig
- All existing tests pass (500+ total)

### Files Changed

- `crates/anna_common/src/config.rs`: Added UiConfig struct
- `crates/anna_common/src/display_format.rs`: Enhanced formatting module
- `crates/annactl/src/commands/status.rs`: Added new sections
- `crates/annactl/src/pipeline.rs`: Debug level filtering

---

## v0.0.14 - Policy Engine + Security Posture

**Release Date:** 2025-12-03

### Summary

Policy-driven allowlists with no hardcoded deny rules. All major allow/deny decisions flow from editable TOML files in `/etc/anna/policy/`. Structured audit logging with secret redaction. Junior enforcement for policy compliance. Installer review validates policy sanity.

### Key Features

**Policy Engine (`policy.rs` - ~1400 lines):**
- Four policy files in `/etc/anna/policy/`:
  - `capabilities.toml`: Read-only and mutation tool settings
  - `risk.toml`: Risk thresholds, confirmations, reliability requirements
  - `blocked.toml`: Blocked packages, services, paths, commands
  - `helpers.toml`: Helper dependency management and policy
- Hot-reload support via `reload_policy()`
- Global policy cache with RwLock for thread safety
- Policy evidence IDs (POL##### format)

**Policy Checks:**
- `is_path_allowed()`: Check if path can be edited
- `is_package_allowed()`: Check if package can be installed/removed
- `is_service_allowed()`: Check if service can be modified
- `is_systemd_operation_allowed()`: Check systemd operations
- `PolicyCheckResult` with allowed, reason, evidence_id, policy_rule fields

**Blocked Categories (Default):**
- Kernel packages: `linux`, `linux-*`, `kernel*`
- Bootloader packages: `grub`, `systemd-boot`, `refind`, `syslinux`
- Init packages: `systemd`, `openrc`, `runit`
- Critical services: `systemd-*`, `dbus.service`, `NetworkManager.service`
- Protected paths: `/boot/*`, `/etc/shadow`, `/etc/passwd`, `/etc/sudoers`

**Audit Logging (`audit_log.rs` - ~400 lines):**
- Structured JSONL audit trail at `/var/lib/anna/audit/audit.jsonl`
- Entry types: ReadOnlyTool, MutationTool, PolicyCheck, Confirmation, ActionBlocked, Rollback, SecurityEvent
- Secret sanitization: passwords, API keys, tokens, Bearer headers
- Environment variable redaction for sensitive keys
- Log rotation at 10MB with archive directory
- Evidence ID linkage in all entries

**Mutation Tools Integration:**
- `validate_mutation_path()` uses policy for path validation
- `validate_package_policy()` checks blocked packages
- `validate_service_policy()` checks critical services
- `PolicyBlocked` error variant with evidence_id and policy_rule
- Symlink traversal protection (follows symlinks for policy checks)

**Junior Enforcement (v0.0.14):**
- Policy citation requirement for risky operations
- New rules 9-13 in Junior system prompt:
  - Risky operations MUST cite policy evidence [POL#####]
  - Refusals MUST explain which policy rule applied
  - Policy bypass suggestions = DANGEROUS = max penalty
- Penalties: -20 for risky operation without policy citation, -50 for policy bypass

**Installer Review (`installer_review.rs`):**
- `check_policy_sanity()` validates policy files
- Auto-repair creates default policy files if missing
- Policy file parsing and validation
- Evidence IDs for repair tracking

**Configuration Defaults:**
```toml
# /etc/anna/policy/capabilities.toml
[read_only_tools]
enabled = true
max_evidence_bytes = 1048576

[mutation_tools]
enabled = true

[mutation_tools.file_edit]
enabled = true
allowed_paths = ["/etc/", "/home/", "/root/", "/var/lib/anna/", "/tmp/"]
blocked_paths = ["/etc/shadow", "/etc/passwd", "/etc/sudoers"]
max_file_size_bytes = 1048576
text_only = true

[mutation_tools.systemd]
enabled = true
allowed_operations = ["status", "restart", "reload", "enable", "disable", "start", "stop"]
blocked_units = []
protected_units = ["sshd.service", "networkd.service", "systemd-resolved.service"]

[mutation_tools.packages]
enabled = true
max_packages_per_operation = 10
blocked_categories = ["kernel", "bootloader", "init"]
```

### Files Changed

- `crates/anna_common/src/policy.rs` (NEW - ~1400 lines)
- `crates/anna_common/src/audit_log.rs` (NEW - ~400 lines)
- `crates/anna_common/src/mutation_tools.rs` - Policy integration, PolicyBlocked error
- `crates/anna_common/src/installer_review.rs` - Policy sanity checks
- `crates/anna_common/src/lib.rs` - Module declarations and exports
- `crates/annactl/src/pipeline.rs` - Junior policy enforcement rules

### Tests

24 new unit tests covering:
- Policy evidence ID generation
- Pattern matching for packages/services
- Path policy checks (allowed, blocked, boot)
- Package policy checks (allowed, blocked categories, patterns)
- Service policy checks (critical services)
- Systemd operation validation
- Policy validation and defaults
- Confirmation phrase parsing
- Audit entry creation and serialization
- Secret sanitization (passwords, API keys, Bearer tokens)
- Environment variable redaction

### Bug Fixes

- Fixed `parse_tool_plan()` to handle commas inside parentheses correctly
- Fixed `execute_tool_plan()` double-counting evidence IDs
- Updated version test to use pattern matching instead of hardcoded version

---

## v0.0.13 - Conversation Memory + Recipe Evolution

**Release Date:** 2025-12-03

### Summary

Local-first conversation memory and recipe evolution system. Anna remembers past sessions, creates recipes for successful patterns, and allows user introspection via natural language. Privacy-first with summaries by default (no raw transcripts unless configured).

### Key Features

**Session Memory (`/var/lib/anna/memory/`):**
- Compact session records for every REPL/one-shot request
- Fields: request_id, request_text, translator_plan_summary, tools_used, evidence_ids, final_answer_summary, reliability_score, recipe_actions, timestamp
- Privacy default: summaries only (`store_raw` config option for full transcripts)
- Keyword-based search indexing
- Append-only JSONL storage with atomic writes
- Archive/tombstone pattern for "forget" operations
- Evidence IDs: MEM##### format

**Recipe System (`/var/lib/anna/recipes/`):**
- Named intent patterns with conditions
- Tool plan templates (read-only and/or mutation)
- Safety classification and required confirmations
- Precondition validation checks
- Rollback templates for mutations
- Provenance tracking (creator, timestamps)
- Confidence scoring and success/failure counters
- Evidence IDs: RCP##### format

**Recipe Creation Rules:**
- Created when: successful request AND Junior reliability >= 80% AND tools used repeatably
- Below 80% reliability creates "experimental" draft
- Configurable via `memory.min_reliability_for_recipe`

**Recipe Matching:**
- Keyword-based scoring (BM25-style for now)
- Intent type and target matching
- Negative keyword exclusion
- Top matching recipes provided to Translator

**User Introspection (Natural Language):**
- "What have you learned recently?"
- "List recipes" / "Show all recipes"
- "Show recipe for X" / "How do you handle X?"
- "Forget what you learned about X" (requires confirmation)
- "Delete recipe X" (requires confirmation)
- "Show my recent questions"
- "Search memory for X"

**Forget/Delete Operations:**
- Medium risk classification
- Requires explicit confirmation: "I CONFIRM (forget)"
- Reversible via archive (not hard delete)
- All operations logged

**Status Display (`annactl status`):**
- New [LEARNING] section
- Shows: recipe count, last learned time, sessions count
- Top 3 most used recipes with confidence scores
- Memory settings display (store_raw, max_sessions, min_reliability)

**Junior Enforcement (v0.0.13):**
- New learning claim detection in Junior system prompt
- Claims about "learned", "remembered", "knows", "has recipes for" must cite MEM/RCP IDs
- Automatic penalty (-25 per uncited learning claim) in fallback scoring
- Detection of fabricated learning claims in final response

**Configuration (`/etc/anna/config.toml`):**
```toml
[memory]
enabled = true              # Enable memory/learning
store_raw = false          # Store raw transcripts (privacy)
max_sessions = 10000       # Max sessions in index
min_reliability_for_recipe = 80  # Minimum % to create recipe
max_recipes = 500          # Max recipes to store
```

### Files Changed

- `crates/anna_common/src/memory.rs` (NEW - ~650 lines)
- `crates/anna_common/src/recipes.rs` (NEW - ~850 lines)
- `crates/anna_common/src/introspection.rs` (NEW - ~630 lines)
- `crates/anna_common/src/config.rs` - Added MemoryConfig
- `crates/anna_common/src/lib.rs` - Module declarations and exports
- `crates/annactl/src/commands/status.rs` - [LEARNING] section
- `crates/annactl/src/pipeline.rs` - Junior learning claim enforcement

### Tests

28 new unit tests covering:
- Session record creation and serialization
- Memory index management
- Keyword extraction
- Recipe creation and matching
- Recipe score calculation
- Introspection intent detection
- Evidence ID generation
- Learning claim detection

---

## v0.0.12 - Proactive Anomaly Detection

**Release Date:** 2025-12-03

### Summary

Proactive anomaly detection engine with alert queue, what_changed correlation tool, slowness_hypotheses analysis, and alert surfacing in REPL and one-shot modes. Complete with evidence IDs for traceability.

### Key Features

**Anomaly Detection Engine (`anomaly_engine.rs`):**
- Periodic anomaly detection (every 5 minutes when integrated with daemon)
- Baseline window (14 days) vs recent window (2 days) comparison
- Configurable thresholds for all metrics
- Evidence ID generation (ANO##### format)

**Supported Anomaly Signals:**
- `BootTimeRegression` - Boot time increased significantly
- `CpuLoadIncrease` - CPU load trend increasing
- `MemoryPressure` - High memory usage or swap activity
- `DiskSpaceLow` - Disk space below threshold
- `SystemCrash` - System crash detected
- `ServiceCrash` - Individual service crash
- `ServiceFailed` - Service in failed state
- `JournalWarningsIncrease` - Warning rate increase
- `JournalErrorsIncrease` - Error rate increase
- `DiskIoLatency` - Disk I/O latency increasing

**Alert Queue (`/var/lib/anna/internal/alerts.json`):**
- Deduplication by signal type
- Severity levels: Info, Warning, Critical
- Severity upgrades on repeated occurrences
- Acknowledgment support
- Persistence across restarts

**New Read-Only Tools:**
- `active_alerts` - Returns current alerts with evidence IDs
- `what_changed(days)` - Packages installed/removed, services enabled/disabled, config changes
- `slowness_hypotheses(days)` - Ranked hypotheses combining changes, anomalies, and resource usage

**Alert Surfacing:**
- REPL welcome: Shows active alerts on startup
- One-shot: Footer with alert summary
- `annactl status [ALERTS]` section: Detailed alert display with evidence IDs

**Evidence Integration:**
- All anomalies have unique evidence IDs
- Hypotheses cite supporting evidence
- Junior enforces citation requirements

### Files Changed

- `crates/anna_common/src/anomaly_engine.rs` (NEW - 1600+ lines)
- `crates/anna_common/src/tools.rs` - Added 3 new tools
- `crates/anna_common/src/tool_executor.rs` - Tool implementations
- `crates/anna_common/src/lib.rs` - Module exports
- `crates/annactl/src/main.rs` - Alert surfacing
- `crates/annactl/src/commands/status.rs` - Enhanced [ALERTS] section
- `crates/annactl/src/commands/mod.rs` - Version update

### Tests

16 new unit tests covering:
- Severity ordering and comparison
- Signal deduplication keys
- Alert queue operations (add, acknowledge, dedup)
- What_changed result formatting
- Slowness hypothesis builders
- Anomaly engine defaults
- Time window calculations

---

## v0.0.11 - Safe Auto-Update System

**Release Date:** 2025-12-03

### Summary

Complete safe auto-update system with update channels (stable/canary), integrity verification, atomic installation, zero-downtime restart, and automatic rollback on failure. Full state visibility in status display.

### Key Features

**Update Channels:**
- `stable` (default): Only stable tagged releases (no -alpha, -beta, -rc, -canary suffixes)
- `canary`: Accept all releases including pre-releases

**Configuration (`/etc/anna/config.toml`):**
```toml
[update]
mode = "auto"           # auto or manual
channel = "stable"      # stable or canary
interval_seconds = 600  # check every 10 minutes
min_disk_space_bytes = 104857600  # 100 MB minimum
```

**Safe Update Workflow:**
1. Check GitHub releases API for new version matching channel
2. Download artifacts to staging directory (`/var/lib/anna/internal/update_stage/<version>/`)
3. Verify integrity (SHA256 checksums)
4. Backup current binaries (`/var/lib/anna/internal/update_backup/`)
5. Atomic installation via rename
6. Signal systemd restart
7. Post-restart validation
8. Cleanup staging and old backups

**Rollback Support:**
- Previous binaries kept in backup directory
- Automatic rollback on restart failure
- Manual rollback possible via backup files
- Rollback state shown in `annactl status`

**Guardrails:**
- Never updates during active mutation operations
- Checks disk space before download (100 MB minimum)
- Verifies installer review health before update
- Rate-limits on consecutive failures with exponential backoff
- No partial installations (atomic or nothing)

**Status Display ([UPDATES] section):**
```
[UPDATES]
  Mode:       auto (stable)
  Interval:   10m
  Last check: 2025-12-03 14:30:00
  Result:     up to date
  Next check: in 8m
```

During update:
```
[UPDATES]
  Mode:       auto (stable)
  Interval:   10m
  Progress:   downloading (45%, ETA: 30s)
  Updating:   0.0.10 -> 0.0.11
```

After update:
```
[UPDATES]
  Mode:       auto (stable)
  Interval:   10m
  Last update: 0.0.10 -> 0.0.11 (5m ago)
```

### Technical Details

**UpdatePhase enum:**
- `Idle`, `Checking`, `Downloading`, `Verifying`, `Staging`, `Installing`, `Restarting`, `Validating`, `Completed`, `Failed`, `RollingBack`

**UpdateState fields (v0.0.11):**
- `channel`: Update channel (stable/canary)
- `update_phase`: Current phase name
- `update_progress_percent`: Progress 0-100
- `update_eta_seconds`: Estimated time remaining
- `updating_to_version`: Target version
- `last_update_at`: Timestamp of last successful update
- `previous_version`: For rollback display

**IntegrityStatus:**
- `StrongVerified`: Verified against release checksum
- `WeakComputed`: Self-computed checksum (no release checksum available)
- `Failed`: Checksum mismatch
- `NotVerified`: Skipped verification

### Files Changed

- `crates/anna_common/src/update_system.rs` (new - 600+ lines)
- `crates/anna_common/src/config.rs` (enhanced UpdateConfig, UpdateState, UpdateResult)
- `crates/anna_common/src/lib.rs` (exports)
- `crates/annactl/src/commands/status.rs` (update progress display)
- `crates/annactl/src/main.rs` (version update)

### Tests

```rust
// Version comparison
assert!(is_newer_version("0.0.11", "0.0.10"));
assert!(!is_newer_version("0.0.9", "0.0.10"));

// Channel matching
assert!(stable.matches_tag("v0.0.10"));
assert!(!stable.matches_tag("v0.0.11-alpha"));
assert!(canary.matches_tag("v0.0.11-alpha"));

// Update phase display
assert_eq!(UpdatePhase::Downloading { progress_percent: 50, eta_seconds: Some(30) }
    .format_display(), "downloading... 50% (ETA: 30s)");
```

---

## v0.0.10 - Reset + Uninstall + Installer Review

**Release Date:** 2025-12-03

### Summary

Factory reset and clean uninstall commands with provenance-aware helper removal. Installer review system verifies installation health and auto-repairs common issues. Confirmation phrases required for destructive operations.

### Key Features

**Reset Command (`annactl reset`):**
- Factory reset returns Anna to fresh install state
- Confirmation phrase: "I CONFIRM (reset)"
- --dry-run flag shows what would be deleted
- --force flag skips confirmation prompt
- Clears all data directories, config, and helper tracking
- Recreates directory structure with correct permissions
- Runs installer review at end and reports health status

**Uninstall Command (`annactl uninstall`):**
- Complete Anna removal from system
- Confirmation phrase: "I CONFIRM (uninstall)"
- Provenance-aware helper removal (only anna-installed)
- Asks user about helper removal unless --keep-helpers
- Removes: systemd unit, binaries, data, config
- --dry-run and --force flags supported

**Install State (`install_state.rs`):**
- Tracks installation artifacts for accurate uninstall
- BinaryInfo: path, checksum, version, last_verified
- UnitInfo: path, exec_start, enabled state
- DirectoryInfo: path, expected permissions, ownership
- Review history with last 10 reviews
- Stored at `/var/lib/anna/install_state.json`

**Installer Review (`installer_review.rs`):**
- Verifies installation correctness
- Checks: binary presence, systemd correctness, directories/permissions, config, update scheduler, Ollama health, helper inventory
- Auto-repair for common issues (without user confirmation):
  - Recreate missing internal directories
  - Fix Anna-owned permissions
  - Re-enable annad service if misconfigured
- Evidence IDs for repair tracking (format: IRxxxx)
- ReviewResult: Healthy, Repaired, NeedsAttention, Failed

**Status Display ([INSTALL REVIEW] section):**
```
[INSTALL REVIEW]
  Status:     healthy
  Last run:   5 minute(s) ago
  Duration:   42 ms
```

### Technical Details

- Install state schema version: 1
- Auto-repair rules: read-only or low-risk internal fixes only
- Review checks ordered: critical (binaries) to informational (helpers)
- Confirmation gates use exact phrase matching

### CLI Changes

```bash
annactl reset              # Factory reset (requires root)
annactl reset --dry-run    # Show what would be deleted
annactl reset --force      # Skip confirmation

annactl uninstall          # Complete removal (requires root)
annactl uninstall --dry-run
annactl uninstall --force
annactl uninstall --keep-helpers
```

### Files Changed

- `crates/anna_common/src/install_state.rs` (new)
- `crates/anna_common/src/installer_review.rs` (new)
- `crates/annactl/src/commands/reset.rs` (updated)
- `crates/annactl/src/commands/uninstall.rs` (new)
- `crates/annactl/src/commands/status.rs` (updated)
- `crates/annactl/src/main.rs` (updated)

---

## v0.0.9 - Package Management + Helper Tracking

**Release Date:** 2025-12-03

### Summary

Package management (controlled) with helper tracking and provenance. Tracks all packages Anna relies on, distinguishing between anna-installed and user-installed packages. Only anna-installed packages can be removed via Anna.

### Key Features

**Helper Tracking System (`helpers.rs`):**
- Tracks ALL helpers Anna relies on for telemetry/diagnostics/execution
- Two dimensions: present/missing + installed_by (anna/user/unknown)
- HelperDefinition, HelperState, HelpersManifest types
- InstalledBy enum with Display implementation
- Persistent storage in `/var/lib/anna/helpers.json`
- First-seen and anna-install timestamps
- Provenance tracked per-machine, not globally

**Helper Definitions (9 helpers):**
- `lm_sensors`: Temperature and fan monitoring
- `smartmontools`: SATA/SAS disk health (SMART)
- `nvme-cli`: NVMe SSD health monitoring
- `ethtool`: Network interface diagnostics
- `iw`: WiFi signal and stats
- `usbutils`: USB device enumeration
- `pciutils`: PCI device enumeration
- `hdparm`: SATA disk parameters
- `ollama`: Local LLM inference

**Package Management Mutation Tools:**
- `package_install`: Install package via pacman (tracks as anna-installed)
- `package_remove`: Remove package (only anna-installed packages)
- 8 mutation tools total (6 from v0.0.8 + 2 new)

**Status Display ([HELPERS] section):**
```
[HELPERS]
  Summary:    7 present, 2 missing (1 by Anna)

  ethtool (present, installed by user)
  lm_sensors (present, installed by Anna)
  ollama (present, installed by Anna)
  smartmontools (missing, unknown origin)
```

**StatusSnapshot Extensions:**
- `helpers_total`: Total helpers tracked
- `helpers_present`: Helpers currently installed
- `helpers_missing`: Helpers not installed
- `helpers_anna_installed`: Helpers installed by Anna
- `helpers`: Vec<HelperStatusEntry> for detailed display

**Transaction Logging:**
- MutationType::PackageInstall, MutationType::PackageRemove
- MutationDetails::Package with package, version, reason, operation
- log_package_operation() in RollbackManager

**Provenance Rules:**
- If helper present before Anna tracked it → installed_by=user
- If Anna installs helper → installed_by=anna
- Only helpers with installed_by=anna are removal_eligible
- package_remove rejects non-anna packages with clear error

### New Error Types

- `PackageAlreadyInstalled(String)`
- `PackageNotInstalled(String)`
- `PackageNotRemovable { package, reason }`
- `PackageInstallFailed { package, reason }`
- `PackageRemoveFailed { package, reason }`

### New Files

- `crates/anna_common/src/helpers.rs` - Helper tracking system

### Modified Files

- `crates/anna_common/src/mutation_tools.rs` - Added package_install, package_remove tools
- `crates/anna_common/src/mutation_executor.rs` - Added package execution functions
- `crates/anna_common/src/rollback.rs` - Added PackageInstall/Remove types and logging
- `crates/anna_common/src/daemon_state.rs` - Added helpers fields to StatusSnapshot
- `crates/anna_common/src/lib.rs` - Added helpers module and exports
- `crates/annactl/src/commands/status.rs` - Added [HELPERS] section

### New Tests

**helpers.rs:**
- test_helper_state_new_missing
- test_helper_state_new_user_installed
- test_helper_state_mark_anna_installed
- test_helper_state_already_present_not_anna
- test_helper_definitions
- test_manifest_removal_eligible
- test_installed_by_display
- test_format_status

### Breaking Changes

- None - backward compatible with v0.0.8
- New fields in StatusSnapshot are optional with `#[serde(default)]`

---

## v0.0.8 - First Safe Mutations

**Release Date:** 2025-12-03

### Summary

First safe mutations (medium-risk only) with automatic rollback and confirmation gates. Introduces mutation tool catalog with allowlist enforcement, file backup system, and Junior reliability threshold for execution approval.

### Key Features

**Mutation Tool Catalog (`mutation_tools.rs`):**
- Allowlist-enforced mutation tools (6 tools total)
- `edit_file_lines`: Text file edits under /etc/** and $HOME/**
- `systemd_restart`: Service restart
- `systemd_reload`: Service configuration reload
- `systemd_enable_now`: Enable and start service
- `systemd_disable_now`: Disable and stop service
- `systemd_daemon_reload`: Reload systemd daemon
- MutationRisk enum (Medium, High)
- File size limit: MAX_EDIT_FILE_SIZE = 1 MiB
- Path validation: is_path_allowed(), validate_mutation_path()

**Rollback System (`rollback.rs`):**
- RollbackManager with timestamped backups
- Backup location: /var/lib/anna/rollback/files/
- Logs location: /var/lib/anna/rollback/logs/
- File hashing (SHA256) for integrity verification
- Diff summary generation for file edits
- Structured JSON logs per mutation request
- JSONL append log (mutations.jsonl) for audit trail
- Rollback instructions with exact restore commands

**Confirmation Gate:**
- Exact phrase requirement: "I CONFIRM (medium risk)"
- Validation via validate_confirmation()
- User confirmation displayed in dialogue transcript
- Action NOT executed if phrase doesn't match exactly

**Junior Verification Threshold:**
- Minimum 70% reliability required for execution
- MutationPlan.junior_approved flag
- MutationPlan.junior_reliability score
- is_approved_for_execution() check

**Mutation Executor (`mutation_executor.rs`):**
- execute_mutation(): Single mutation execution
- execute_mutation_plan(): Batch execution
- Request ID generation (UUID v4)
- Evidence ID propagation from tool results
- Structured MutationResult with human_summary and rollback_info

**Pipeline Updates (`pipeline.rs`):**
- ActionPlan extended with mutation_plan and is_medium_risk_executable
- generate_action_plan() creates MutationPlan for service operations
- handle_mutation_execution() for confirmation and execution flow
- ensure_service_suffix() helper for .service extension

### Not Allowed in v0.0.8

The following are explicitly NOT executable:
- Package installs/removals (pacman, yay)
- Filesystem operations (mount, umount, mkfs)
- Destructive operations (rm, rmdir)
- Permission changes (chmod, chown)
- User management (useradd, userdel, usermod)
- Firewall rules (iptables, ufw)
- Kernel parameters (sysctl)

### Mutation Execution Flow

```
[you] to [anna]: restart nginx
[anna] to [translator]: Classify this request...
[translator] to [anna]: Intent: action_request, Targets: nginx
                        Risk: medium
[anna] generates action plan with MutationPlan:
  What will change: Services to restart: nginx.service
  Why: Service restart requested
  Risk: medium
  Rollback: Service state can be restored by restarting again
[anna] to [junior]: Verify this mutation plan...
[junior] to [anna]: Reliability: 85%
[anna] to [you]: I can execute this action for you.

                 What will change:
                   Services to restart: nginx.service

                 Risk level: medium

                 To proceed, type exactly: I CONFIRM (medium risk)
[you] to [anna]: I CONFIRM (medium risk)
[anna] to [annad]: User confirmed. Please execute:
                   - systemd_restart (nginx.service)
[annad] executes: systemctl restart nginx.service
SUCCESS systemd_restart: Restarted nginx.service
  Rollback: sudo systemctl restart nginx.service
[annad] to [anna]: All operations completed successfully.
                   1 mutation(s) executed.
                   Logs saved to: /var/lib/anna/rollback/logs/
```

### Rollback Log Example

```json
{
  "request_id": "a1b2c3d4-...",
  "timestamp": 1701619200,
  "tool_name": "systemd_restart",
  "evidence_ids": ["E1", "E2"],
  "mutation_type": "systemd",
  "target": "nginx.service",
  "details": {
    "Systemd": {
      "operation": "restart",
      "service": "nginx.service",
      "previous_state": { "active": true, "enabled": true }
    }
  },
  "success": true,
  "error": null
}
```

### New Tests

**mutation_tools.rs:**
- test_path_allowed_etc
- test_path_allowed_home
- test_path_not_allowed
- test_confirmation_valid
- test_confirmation_missing
- test_confirmation_wrong
- test_mutation_catalog_has_expected_tools
- test_mutation_catalog_rejects_unknown
- test_mutation_plan_approval
- test_risk_display

**rollback.rs:**
- test_diff_summary_generation
- test_backup_file
- test_file_hash
- test_rollback_info_generation

### Breaking Changes

- None - backward compatible with v0.0.7
- New fields in ActionPlan are optional (mutation_plan, is_medium_risk_executable)

---

## v0.0.7 - Read-Only Tooling & Evidence Citations

**Release Date:** 2025-12-03

### Summary

Read-only tool catalog with allowlist enforcement, Evidence IDs for citations, and human-readable natural language transcripts. Junior now enforces no-guessing with uncited claim detection. Translator outputs tool plans for evidence gathering.

### Key Features

**Read-Only Tool Catalog (`tools.rs`):**
- Internal tool allowlist with security classification
- 10 read-only tools: status_snapshot, sw_snapshot_summary, hw_snapshot_summary, recent_installs, journal_warnings, boot_time_trend, top_resource_processes, package_info, service_status, disk_usage
- ToolDef with human-readable descriptions and latency hints
- ToolSecurity enum (ReadOnly, LowRisk, MediumRisk, HighRisk)
- parse_tool_plan() for parsing Translator output

**Tool Executor (`tool_executor.rs`):**
- execute_tool() for individual tool execution
- execute_tool_plan() for batch execution with EvidenceCollector
- Structured ToolResult with human_summary
- Unknown tool handling with graceful errors

**Evidence IDs and Citations:**
- EvidenceCollector assigns sequential IDs (E1, E2, ...)
- Evidence IDs in all tool results and dialogue
- Citations expected in final responses: [E1], [E2]
- Evidence legend in final response

**Junior No-Guessing Enforcement:**
- UNCITED_CLAIMS output field for speculation detection
- Strict scoring: uncited claims = -15 per claim
- "Unknown" preferred over guessing
- Label inference explicitly: "Based on [E2], likely..."

**Translator Tool Plans:**
- TOOLS output field in Translator response
- Tool plan parsing from comma-separated calls
- RATIONALE field for tool selection reasoning
- Deterministic fallback generates tool plans from evidence_needs

**Natural Language Transcripts:**
```
[anna] to [annad]: Please gather evidence using: hw_snapshot_summary
[annad] to [anna]: [E1] hw_snapshot_summary: CPU: AMD Ryzen 7 5800X (8 cores)
                                            Memory: 32GB total, 16GB available (found)
```

### Pipeline Flow (v0.0.7)

```
[you] to [anna]: what CPU do I have?
[anna] to [translator]: Classify this request...
[translator] to [anna]: Intent: system_query, Targets: cpu
                        Risk: read_only
                        Tools: hw_snapshot_summary
[anna] to [annad]: Please gather evidence using: hw_snapshot_summary
[annad] to [anna]: [E1] hw_snapshot_summary: CPU: AMD Ryzen 7 5800X (found)
[anna] to [junior]: Verify this draft response:
                    Based on gathered evidence:
                    [E1] CPU: AMD Ryzen 7 5800X
[junior] to [anna]: Reliability: 95%, Critique: All claims cite evidence
                    Uncited claims: none
[anna] to [you]: Based on gathered evidence:
                 [E1] CPU: AMD Ryzen 7 5800X
                 ---
                 Evidence sources:
                   [E1]: hw_snapshot_summary (OK)
Reliability: 95%
```

### New Tests (7 tests added)

- test_deterministic_generates_tool_plan
- test_deterministic_service_query_generates_service_tool
- test_parse_junior_response_with_uncited_claims
- test_parse_junior_response_no_uncited_claims
- test_tool_catalog_creation
- test_evidence_collector
- test_fallback_scoring_v2_with_tool_results

### Breaking Changes

- None - backward compatible with v0.0.6
- Legacy evidence retrieval still works when tool_plan is None

---

## v0.0.6 - Real Translator LLM

**Release Date:** 2025-12-03

### Summary

Real LLM-powered Translator for intent classification with clarification loop support. Evidence-first pipeline with real snapshot integration and 8KB excerpt cap. Action plan generation for mutation requests (no execution yet).

### Key Features

**Real Translator LLM (`pipeline.rs`):**
- Real LLM-backed intent classification replacing deterministic mock
- Structured output parsing: intent, targets, risk, evidence_needs, clarification
- System prompt with strict output format
- Fallback to deterministic classification when LLM unavailable

**Clarify-or-Proceed Loop:**
- Multiple-choice clarification prompts
- Default option selection
- Single-turn clarification (no infinite loops)
- CLARIFICATION field in Translator output: `question|option1|option2|option3|default:N`

**Evidence-First Pipeline:**
- Real snapshot integration from annad
- Evidence excerpting with 8KB hard cap
- `[EXCERPT truncated]` indication when data exceeds limit
- Evidence sources: hw_snapshot, sw_snapshot, status, journalctl

**Action Plan Generation:**
- Action plans for action_request intent
- Steps, affected files/services/packages
- Risk classification propagation
- Rollback outline
- Confirmation phrase (no execution - confirmation-gated)

**Translator System Prompt:**
```
OUTPUT FORMAT (follow exactly, one field per line):
INTENT: [question|system_query|action_request|unknown]
TARGETS: [comma-separated list or "none"]
RISK: [read_only|low|medium|high]
EVIDENCE_NEEDS: [hw_snapshot, sw_snapshot, status, journalctl, or "none"]
CLARIFICATION: [empty OR "question|opt1|opt2|opt3|default:N"]
```

### Pipeline Flow (v0.0.6)

```
[you] to [anna]: install nginx
[anna] to [translator]: Please classify this request...
[translator thinking via qwen2.5:0.5b...]
[translator] to [anna]: Intent: action_request, Targets: nginx, Risk: medium
                        Evidence: sw_snapshot
                        Clarification: none
[anna] to [annad]: Retrieve evidence for: nginx
[annad] to [anna]: snapshot:sw: [package data, 8KB max excerpt]
[anna] generates action plan:
  Steps: 1. Run pacman -S nginx  2. Enable nginx.service  3. Start nginx.service
  Affected: packages: nginx, services: nginx.service
  Rollback: pacman -Rns nginx
  Confirmation: Type "I understand and accept the risk" to proceed
[anna] to [junior]: Please verify this action plan...
[junior] to [anna]: Reliability: 75%, Critique: Plan looks correct...
                    MUTATION_WARNING: This will install a package.
[anna] to [you]: Action plan ready. Type confirmation phrase to execute.
Reliability: 75%
```

### Tests

- 15 pipeline unit tests (parsing, clarification, evidence, action plans)
- 20 CLI integration tests (all passing)
- Test coverage for deterministic fallback
- Evidence excerpting edge cases

### Breaking Changes

- None - backward compatible with v0.0.5

---

## v0.0.5 - Role-Based Model Selection + Benchmarking

**Release Date:** 2025-12-03

### Summary

Role-based LLM model selection with hardware-aware configuration and built-in benchmarking. Translator (fast) and Junior (reliable) now have distinct model pools selected based on system capabilities. Bootstrap process with progress tracking.

### Key Features

**Hardware Detection (`model_selection.rs`):**
- CPU detection (cores, model name)
- RAM detection (total, available)
- GPU/VRAM detection via nvidia-smi
- Hardware tier classification:
  - Low: <8GB RAM
  - Medium: 8-16GB RAM
  - High: >16GB RAM

**Role-Based Model Selection:**
- LlmRole enum: Translator, Junior
- Translator candidates (smallest/fastest first): qwen2.5:0.5b, qwen2.5:1.5b, phi3:mini, gemma2:2b, llama3.2:1b
- Junior candidates (most reliable first): llama3.2:3b-instruct, qwen2.5:3b-instruct, mistral:7b-instruct, gemma2:9b
- Priority-based selection respecting hardware tier

**Benchmark Suites:**
- Translator: 30 prompts testing intent classification (read-only vs mutation, targets, etc.)
- Junior: 15 cases testing verification quality (evidence handling, honesty scoring, etc.)
- Per-case latency and pattern matching evaluation

**Ollama Model Pull with Progress:**
- Streaming pull progress via `/api/pull`
- Real-time progress percentage
- Download speed (MB/s)
- ETA calculation
- Progress exposed in status snapshot

**Bootstrap State Machine:**
- `detecting_ollama`: Checking Ollama availability
- `installing_ollama`: Installing Ollama (future)
- `pulling_models`: Downloading required models
- `benchmarking`: Running model benchmarks (future)
- `ready`: Models ready for use
- `error`: Bootstrap failed

**Status Snapshot Updates (schema v3):**
- `llm_bootstrap_phase`: Current phase
- `llm_translator_model`: Selected translator
- `llm_junior_model`: Selected junior
- `llm_downloading_model`: Model being pulled
- `llm_download_percent`: Progress percentage
- `llm_download_speed`: Download speed
- `llm_download_eta_secs`: ETA in seconds
- `llm_error`: Error message if any
- `llm_hardware_tier`: Detected tier

**annactl Progress Display:**
- Shows bootstrap progress when models not ready
- Progress bar for model downloads
- ETA display
- Graceful fallback with reduced reliability score (-10 points)
- Explicit reason shown when LLM unavailable

### Configuration

New LLM settings in `/etc/anna/config.toml`:
```toml
[llm]
enabled = true
ollama_url = "http://127.0.0.1:11434"

[llm.translator]
model = ""  # Empty for auto-select
timeout_ms = 30000
enabled = true

[llm.junior]
model = ""  # Empty for auto-select
timeout_ms = 60000
enabled = true

# Custom candidate pools (optional)
translator_candidates = ["qwen2.5:0.5b", "qwen2.5:1.5b"]
junior_candidates = ["llama3.2:3b-instruct", "mistral:7b-instruct"]
```

### Tests

- 21 model_selection unit tests
- Hardware tier boundary tests
- Model priority selection tests
- Fallback behavior tests
- Bootstrap state tests

### Breaking Changes

- Config structure changed: `junior.*` deprecated, use `llm.junior.*`
- Status snapshot schema bumped to v3

---

## v0.0.4 - Real Junior Verifier

**Release Date:** 2024-12-03

### Summary

Junior becomes a real LLM-powered verifier via local Ollama. Translator remains deterministic. No Senior implementation yet - keeping complexity low while measuring real value.

### Key Features

**Junior LLM Integration:**
- Real verification via Ollama local LLM
- Auto-selects best model (prefers qwen2.5:1.5b, llama3.2:1b, etc.)
- Structured output parsing (SCORE, CRITIQUE, SUGGESTIONS, MUTATION_WARNING)
- Fallback to deterministic scoring when Ollama unavailable
- Spinner while Junior thinks

**Ollama Client (`ollama.rs`):**
- HTTP client for local Ollama API
- Health check, model listing, generation
- Timeout and retry handling
- Model auto-selection based on availability

**Junior Config:**
- `junior.enabled` (default: true)
- `junior.model` (default: auto-select)
- `junior.timeout_ms` (default: 60000)
- `junior.ollama_url` (default: http://127.0.0.1:11434)

### Pipeline Flow (with real LLM)

```
[you] to [anna]: what CPU do I have?
[anna] to [translator]: Please classify this request...
[translator] to [anna]: Intent: system_query, Targets: cpu, Risk: read-only, Confidence: 85%
[anna] to [annad]: Retrieve evidence for: cpu
[annad] to [anna]: snapshot:hw.cpu: [CPU data]
[anna] to [junior]: Please verify this draft response...
[junior thinking via qwen2.5:1.5b...]
[junior] to [anna]: Reliability: 80%
                    Critique: The response mentions evidence but doesn't parse it
                    Suggestions: Add specific CPU model and core count
[anna] to [you]: Based on system data from: snapshot:hw.cpu...
Reliability: 80%
```

### Junior System Prompt

Junior is instructed to:
- NEVER invent machine facts
- Downscore missing evidence
- Prefer "unknown" over guessing
- Keep output short and structured
- Warn about mutations for action requests

### Graceful Degradation

When Ollama is not available:
- REPL shows warning with install instructions
- Pipeline falls back to deterministic scoring (v0.0.3 logic)
- Exit code 0 - no crashes

### Tests

- 9 unit tests for pipeline (Translator, Junior parsing, fallback scoring)
- 20 CLI integration tests
- 4 new v0.0.4 tests (Critique, Suggestions, mutation warning, graceful degradation)

### Model Selection Order

1. qwen2.5:1.5b (fastest, good for verification)
2. qwen2.5:3b
3. llama3.2:1b
4. llama3.2:3b
5. phi3:mini
6. gemma2:2b
7. mistral:7b
8. First available model

---

## v0.0.3 - Request Pipeline Skeleton

**Release Date:** 2024-12-03

### Summary

Implements the full multi-party dialogue transcript with deterministic mocks for intent classification, evidence retrieval, and Junior scoring. No LLM integration yet - all behavior is keyword-based and deterministic.

### Pipeline Flow

```
[you] to [anna]: what CPU do I have?
[anna] to [translator]: Please classify this request...
[translator] to [anna]: Intent: system_query, Targets: cpu, Risk: read-only, Confidence: 85%
[anna] to [annad]: Retrieve evidence for: cpu
[annad] to [anna]: snapshot:hw.cpu: [CPU data would come from snapshot]
[anna] to [junior]: Please verify and score this response.
[junior] to [anna]: Reliability: 100%, Breakdown: +40 evidence, +30 confident, +20 observational+cited, +10 read-only
[anna] to [you]: Based on system data from: snapshot:hw.cpu...
Reliability: 100%
```

### Changes

**Pipeline Module (`pipeline.rs`):**
- DialogueActor enum: You, Anna, Translator, Junior, Annad
- `dialogue()` function with format: `[actor] to [target]: message`
- IntentType enum: question, system_query, action_request, unknown
- RiskLevel enum: read-only, low-risk, medium-risk, high-risk
- Intent struct with keywords, targets, risk, confidence
- Evidence struct with source, data, timestamp

**Translator Mock:**
- Keyword-based intent classification
- Target detection (cpu, memory, disk, network, docker, nginx, etc.)
- Action keyword detection (install, remove, restart, etc.)
- Confidence scoring based on keyword matches

**Evidence Retrieval Mock:**
- Maps targets to snapshot sources (hw.cpu, hw.memory, sw.services.*)
- Returns mock evidence with timestamps
- System queries trigger annad dialogue

**Junior Scoring:**
- +40: evidence exists
- +30: confident classification (>70%)
- +20: observational + cited (read-only with evidence)
- +10: read-only operation
- Breakdown shown in output

**Tests:**
- test_annactl_pipeline_shows_translator
- test_annactl_pipeline_shows_junior
- test_annactl_pipeline_shows_annad_for_system_query
- test_annactl_pipeline_intent_classification
- test_annactl_pipeline_target_detection
- test_annactl_pipeline_reliability_breakdown
- test_annactl_pipeline_action_risk_level

### Internal Notes

- All responses are mocked (no LLM integration)
- Evidence retrieval is simulated (no actual snapshot reads)
- Risk classification is keyword-based
- Pipeline is ready for LLM integration in 0.1.x

---

## v0.0.2 - Strict CLI Surface

**Release Date:** 2024-12-03

### Summary

Enforces the strict CLI surface. All legacy commands (sw, hw, JSON flags) are removed from public dispatch and now route through natural language processing.

### Supported Entrypoints

```bash
annactl                  # REPL mode (interactive)
annactl <request>        # One-shot natural language request
annactl status           # Self-status
annactl --version        # Version (also: -V)
annactl --help           # Help (also: -h)
```

**That's the entire public surface.**

### Changes

**CLI Surface:**
- Removed `sw` command from public surface
- Removed `hw` command from public surface
- Removed all JSON flags (--json, --full) from public surface
- Legacy commands now route as natural language requests (no custom error message)
- Added --help/-h flags for explicit help display

**REPL Mode:**
- Implemented basic REPL loop
- Exit commands: exit, quit, bye, q
- Help command shows REPL-specific help
- Status command works in REPL

**Dialogue Format:**
- Natural language requests show `[you] to [anna]:` format
- Responses show `[anna] to [you]:` format
- Reliability score displayed (stub: 0% until LLM integration)

**Tests:**
- Added test for --help showing strict surface only
- Added test for status command exit 0
- Added test for --version format
- Added test for legacy command routing (sw, hw)
- Added test for natural language request format

### Breaking Changes

- `annactl sw` no longer shows software overview (routes as request)
- `annactl hw` no longer shows hardware overview (routes as request)
- `annactl` (no args) now enters REPL instead of showing help
- Use `annactl --help` or `annactl -h` for help

### Internal

- Snapshot architecture preserved (internal capabilities only)
- Status command unchanged
- Version output format unchanged: `annactl vX.Y.Z`

---

## v0.0.1 - Specification Lock-In

**Release Date:** 2024-12-03

### Summary

Complete specification reset. Anna transitions from a "snapshot reader with fixed commands" to a "natural language virtual sysadmin" architecture.

### Changes

**Governance:**
- Established immutable operating contract (CLAUDE.md)
- Created implementation roadmap (TODO.md)
- Set up release notes workflow
- Version reset to 0.0.1 (staying in 0.x.x until production)

**Documentation:**
- README.md rewritten for natural language assistant vision
- CLAUDE.md created with full engineering contract
- TODO.md created with phased implementation roadmap
- RELEASE_NOTES.md created for change tracking

**Architecture Decision:**
- Preserve existing snapshot-based telemetry foundation
- Build natural language layer on top
- Strict CLI surface: `annactl`, `annactl <request>`, `annactl status`, `annactl --version`
- All old commands (sw, hw, JSON flags) become internal capabilities only

**Spec Highlights:**
- 4-player model: User, Anna, Translator, Junior, Senior
- Debug mode always on (visible dialogue)
- Reliability scores on all answers (0-100%)
- Safety classification: read-only, low-risk, medium-risk, high-risk
- Rollback mandate for all mutations
- Recipe learning system
- XP and gamification (levels 0-100, nerdy titles)
- Auto-update every 10 minutes
- Auto Ollama setup

### Breaking Changes

- Version number reset from 7.42.5 to 0.0.1
- Old CLI commands will be removed in 0.0.2
- New CLI surface is strict and minimal

### Migration Path

Existing snapshot infrastructure is preserved. Natural language capabilities will be added incrementally without breaking current performance.

---

## Previous Versions

Prior to v0.0.1, Anna was a snapshot-based telemetry daemon with fixed CLI commands. See git history for v7.x releases.
