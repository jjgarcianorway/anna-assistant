# Anna Assistant - Testing Strategy V1

**Document Version**: PLANNING_STEP_1
**Anna Version**: v5.7.0-beta.277
**Date**: 2025-11-23
**Purpose**: Safe, staged testing plan for real-world validation

---

## 1. Objectives

### Primary Goal

**Validate that Anna behaves correctly on a real Arch Linux system** without breaking anything or requiring heroic recovery.

### Non-Goals

- ❌ NOT trying to stress-test until failure
- ❌ NOT trying to discover every possible bug
- ❌ NOT trying to test on production-critical systems

### Principles

1. **Incremental**: Start simple, add complexity gradually
2. **Reversible**: Every test scenario has a clear recovery path
3. **Observable**: Know what to watch (logs, outputs, behavior)
4. **Safe**: No irreversible actions, no data loss
5. **Documented**: Record what was tested, what worked, what failed

---

## 2. Testing Levels

### Level 0: Read-Only Checks (SAFE)

**Preconditions**: None (always safe)

**Actions Allowed**:
- Run `annactl status`
- Run `annactl status --json`
- Ask natural language questions in TUI
- View diagnostic outputs
- Read logs (`journalctl -u annad`)

**What to Observe**:
- Does `annad` daemon start cleanly?
- Is telemetry collected correctly?
- Do diagnostic insights make sense?
- Does TUI render correctly in your terminal?
- Are proactive issues (if any) accurate?

**Risk**: None (purely read-only)

**Duration**: 15-30 minutes

---

### Level 1: Light Stress (SAFE on test systems)

**Preconditions**:
- Use a non-production Arch system (VM, spare laptop, etc.)
- OR use a production system but only with non-critical services

**Actions Allowed**:
- Stop/start a single non-critical systemd service
- Create a small test file to increase disk usage slightly
- Run a simple CPU stress test (e.g., `stress-ng --cpu 1 --timeout 30s`)
- Plug/unplug a USB Ethernet adapter (if available)

**What to Observe**:
- Does Anna detect the stopped service?
- Does Anna report disk usage changes?
- Does Anna notice CPU load?
- Does Anna re-rank network interfaces when USB Ethernet is plugged in?
- Are remediation suggestions correct?

**Risk**: Low (easily reversible, non-critical services)

**Duration**: 1-2 hours

---

### Level 2: Combined Scenarios (MODERATE RISK)

**Preconditions**:
- **Test system ONLY** (VM highly recommended)
- Backups available
- Know how to recover system manually

**Actions Allowed**:
- Stop multiple services simultaneously
- Fill a test partition to 90-95%
- Create packet loss with `tc` (traffic control)
- Simulate memory pressure with `stress-ng`
- Combine 2-3 issues at once (service + disk, network + CPU)

**What to Observe**:
- Does Anna correlate multiple issues correctly?
- Does proactive engine identify relationships?
- Are remediation suggestions still accurate?
- Does TUI remain stable under load?

**Risk**: Medium (system may become sluggish, but recoverable)

**Duration**: 2-4 hours

---

### Level 3: Edge Cases (HIGH RISK)

**Preconditions**:
- **Dedicated test system ONLY**
- **NOT on production hardware**
- VM snapshot taken before testing
- Comfortable with full system reinstall if needed

**Actions Allowed**:
- Extreme disk fill (99%+)
- OOM killer triggers
- Kernel upgrades mid-session
- Flapping interfaces (rapid plug/unplug)
- Corrupted log files
- Bad USB dongles with misreported speeds

**What to Observe**:
- Does Anna handle extreme conditions gracefully?
- Does daemon crash or hang?
- Can you recover system to working state?

**Risk**: High (may require system reinstall)

**Duration**: 4+ hours

**Recommendation**: Skip this level until Level 0-2 pass cleanly.

---

## 3. Scenario Catalog

### A) Health and Diagnostics (Level 0-1)

#### A1: Baseline Health Check (Level 0)
**Objective**: Verify Anna reports correctly on a healthy system

**Steps**:
1. Ensure system is healthy (no failed services, disk <80%, normal load)
2. Run `annactl status`
3. Check output for:
   - Overall health: "healthy"
   - No critical issues
   - Proactive health score: 90-100

**Expected Behavior**:
```
[SUMMARY]
Overall system health: healthy
Proactive health score: 100/100

[DAILY SNAPSHOT]
Kernel: 6.17.8-arch1-1 (unchanged since last session)
Packages: 1234 (2 new since last session)
No reboots in last 24h

[DIAGNOSTICS]
No critical issues detected
```

**Recovery**: N/A (read-only)

---

#### A2: Single Service Failure (Level 1)
**Objective**: Test service failure detection

**Steps**:
1. Choose a non-critical service (e.g., `cups.service` if you don't use printing)
2. Stop it: `sudo systemctl stop cups.service`
3. Run `annactl status`
4. Ask in TUI: "are there any failed services?"

**Expected Behavior**:
- Anna reports `cups.service` as failed
- Diagnostic insight: "1 failed service detected"
- Remediation suggestion: `sudo systemctl start cups.service`
- Proactive health score drops slightly (95-98)

**Recovery**:
```bash
sudo systemctl start cups.service
```

---

#### A3: Noisy Log Injection (Level 1)
**Objective**: Test log health reporting

**Steps**:
1. Generate noisy logs: `logger -p user.err "TEST ERROR MESSAGE"`
2. Repeat 10-20 times
3. Run `annactl status`
4. Ask in TUI: "any critical logs?"

**Expected Behavior**:
- Anna may report increased error log activity
- Depends on diagnostic rule thresholds

**Recovery**: N/A (logs are harmless)

---

### B) Network (Level 1-2)

#### B1: USB Ethernet vs WiFi Priority (Level 1)
**Objective**: Test network priority ranking

**Requirements**: USB Ethernet adapter + WiFi available

**Steps**:
1. Start with WiFi connected (note speed: `annactl status`)
2. Plug in USB Ethernet adapter
3. Wait for network to come up (~5-10 seconds)
4. Run `annactl status` again
5. Ask in TUI: "which network interface should I use?"

**Expected Behavior**:
- Anna re-ranks interfaces
- USB Ethernet should rank higher (if faster link speed)
- Recommendation should match priority ranking

**Recovery**: Unplug USB Ethernet

---

#### B2: Packet Loss Simulation (Level 2)
**Objective**: Test packet loss detection

**Requirements**: Root access, `tc` (traffic control) tool

**Steps**:
1. Identify primary interface: `ip route | grep default`
2. Add packet loss: `sudo tc qdisc add dev <interface> root netem loss 10%`
3. Run `annactl status`
4. Ask in TUI: "is my network degraded?"

**Expected Behavior**:
- Anna detects packet loss (if telemetry captures it)
- May report degraded interface
- Recommendation to check driver/cable

**Recovery**:
```bash
sudo tc qdisc del dev <interface> root
```

---

#### B3: Interface Flapping (Level 2)
**Objective**: Test rapid interface state changes

**Requirements**: USB Ethernet or ability to toggle WiFi

**Steps**:
1. Plug in USB Ethernet
2. Wait 5 seconds
3. Unplug
4. Repeat 5-10 times over 1 minute
5. Run `annactl status`

**Expected Behavior**:
- Anna may not detect flapping (no historian yet)
- Should show current interface state correctly
- May show confusion if telemetry captured mid-transition

**Recovery**: Leave interface in stable state (plugged or unplugged)

---

### C) Disk (Level 1-2)

#### C1: Disk 90% Full (Level 1)
**Objective**: Test disk pressure detection at warning threshold

**Requirements**: Non-critical partition with free space

**Steps**:
1. Create test partition or use `/tmp` (if large enough)
2. Fill to 90%: `dd if=/dev/zero of=/tmp/bigfile bs=1M count=<size>`
3. Run `annactl status`
4. Ask in TUI: "is my disk full?"

**Expected Behavior**:
- Anna reports disk at 90% (warning level)
- Recommendation to clean up logs or old files
- Proactive health score drops (80-90)

**Recovery**:
```bash
rm /tmp/bigfile
```

---

#### C2: Disk 95% Full (Level 2)
**Objective**: Test disk pressure detection at critical threshold

**Requirements**: Test system only

**Steps**:
1. Fill partition to 95%: `dd if=/dev/zero of=/tmp/bigfile bs=1M count=<size>`
2. Run `annactl status`
3. Check if proactive engine correlates with other issues

**Expected Behavior**:
- Anna reports critical disk pressure
- Urgent remediation suggestions
- Proactive health score drops significantly (60-80)

**Recovery**:
```bash
rm /tmp/bigfile
```

---

#### C3: Runaway Log File (Level 2)
**Objective**: Test correlation of disk pressure with log growth

**Requirements**: Test system only

**Steps**:
1. Fill disk to 90% with logs: `dd if=/dev/zero of=/var/log/testlog bs=1M count=<size>`
2. Run `annactl status`
3. Check if proactive engine correlates disk pressure with log location

**Expected Behavior**:
- Anna reports disk pressure
- May identify `/var/log` as cause (depends on diagnostic rule granularity)
- Proactive engine may correlate if log issues also detected

**Recovery**:
```bash
sudo rm /var/log/testlog
```

---

### D) CPU and Memory (Level 1-2)

#### D1: CPU Stress Test (Level 1)
**Objective**: Test CPU load detection

**Requirements**: `stress-ng` installed

**Steps**:
1. Run stress test: `stress-ng --cpu 4 --timeout 60s`
2. While running, execute: `annactl status`
3. Ask in TUI: "is my CPU overloaded?"

**Expected Behavior**:
- Anna reports high CPU load
- May identify stress-ng as top process
- Recommendation may suggest checking runaway processes

**Recovery**: Wait for stress test to timeout (60s)

---

#### D2: Memory Pressure (Level 2)
**Objective**: Test memory usage detection

**Requirements**: `stress-ng` installed, test system

**Steps**:
1. Run memory stress: `stress-ng --vm 2 --vm-bytes 75% --timeout 60s`
2. While running, execute: `annactl status`
3. Check if Anna detects memory pressure

**Expected Behavior**:
- Anna reports high memory usage
- May identify stress-ng as memory hog
- Recommendation may suggest killing processes or adding swap

**Recovery**: Wait for stress test to timeout (60s)

---

#### D3: OOM Killer Scenario (Level 3)
**Objective**: Test extreme memory pressure handling

**Requirements**: VM with snapshot, comfortable with system instability

**Steps**:
1. Take VM snapshot
2. Run aggressive memory stress: `stress-ng --vm 4 --vm-bytes 95% --timeout 300s`
3. Monitor if OOM killer triggers
4. Run `annactl status` if system remains responsive

**Expected Behavior**:
- System may become unresponsive
- OOM killer may kill `stress-ng` or `annad`
- Anna may crash or hang

**Recovery**: Reboot VM, restore snapshot if needed

**Recommendation**: Skip until confident in Level 1-2 testing

---

### E) Proactive and Remediation (Level 2)

#### E1: Combined Service + Disk Issue (Level 2)
**Objective**: Test multi-issue correlation

**Requirements**: Test system

**Steps**:
1. Stop 2-3 non-critical services
2. Fill disk to 90%
3. Run `annactl status`
4. Check if proactive engine correlates issues

**Expected Behavior**:
- Anna reports both service failures and disk pressure
- Proactive engine may show correlation (if services failed due to disk)
- Health score drops significantly (60-80)

**Recovery**:
```bash
sudo systemctl start <service1> <service2>
rm /tmp/bigfile
```

---

#### E2: Service + Network Issue (Level 2)
**Objective**: Test network-related service correlation

**Requirements**: Test system, USB Ethernet or WiFi

**Steps**:
1. Stop NetworkManager: `sudo systemctl stop NetworkManager`
2. Unplug Ethernet (if applicable)
3. Run `annactl status`
4. Check if proactive engine correlates network and service

**Expected Behavior**:
- Anna reports NetworkManager stopped and network down
- Proactive engine may correlate (network down because service stopped)
- Remediation suggests starting NetworkManager

**Recovery**:
```bash
sudo systemctl start NetworkManager
# Plug in Ethernet if unplugged
```

---

#### E3: Natural Language Remediation Query (Level 1)
**Objective**: Test remediation command suggestions

**Requirements**: Failed service or disk pressure

**Steps**:
1. Create an issue (stopped service or disk 90% full)
2. In TUI, ask: "how do I fix this?"
3. Check if Anna provides actionable commands

**Expected Behavior**:
- Anna lists remediation commands
- Commands are deterministic and safe
- Commands are cited with source

**Recovery**: Execute suggested commands or manual recovery

---

## 4. 700-Question Suite Usage

### Purpose

The regression_nl_big.toml suite validates **NL routing accuracy**, not system behavior.

### When to Run

- After any changes to routing logic
- Before releasing a new beta
- To compare accuracy across versions

### How to Run

```bash
cargo test -p annactl regression_nl_big
```

### Expected Output

```
test result: ok. 609 passed; 91 failed; 0 ignored; 0 measured; 0 filtered out
```

**609/700 = 87.0% accuracy**

### Interpreting Results

**87% means**:
- 609 queries route as expected
- 91 queries do not (intentionally ambiguous or edge cases)

**Regression occurs if**:
- Pass rate drops below 87% after changes
- New router_bug classifications appear

**Improvement occurs if**:
- Pass rate increases above 87%
- router_bug count decreases

### Comparing Across Versions

Keep a log:
```
Beta.275: 608/700 (86.9%)
Beta.276: 609/700 (87.0%)
Beta.277: 609/700 (87.0%)
```

If a new beta drops to 600/700 (85.7%), investigate which queries regressed.

---

## 5. Logging and Evidence

### What Logs to Watch

**Daemon logs**:
```bash
journalctl -u annad -f
```
- Watch for errors, warnings, panics
- Check telemetry collection frequency
- Verify brain analysis pipeline runs

**System journal**:
```bash
journalctl -f
```
- Watch for service state changes
- Check for kernel errors
- Monitor network events

**Anna CLI output**:
- Save `annactl status` output to file: `annactl status > status_output.txt`
- Save JSON output for programmatic analysis: `annactl status --json > status.json`

**TUI screenshots**:
- Use terminal screenshot tool (e.g., `scrot`, `flameshot`)
- OR copy/paste TUI output to text file

### Keeping a Testing Log

Create a simple testing log file: `anna_testing_log.md`

**Format**:
```markdown
## 2025-11-23 - Level 0 Testing

### Test A1: Baseline Health Check
- **System**: Lenovo ThinkPad T480, Arch Linux 6.17.8
- **Anna Version**: v5.7.0-beta.277
- **Result**: ✅ PASS
- **Notes**: All healthy, proactive score 100/100

### Test A2: Single Service Failure
- **Service stopped**: cups.service
- **Result**: ✅ PASS
- **Notes**: Anna detected failed service, suggested correct remediation

### Test B1: USB Ethernet vs WiFi
- **Result**: ❌ FAIL
- **Issue**: Anna ranked WiFi higher than USB Ethernet (expected opposite)
- **Evidence**: See status_output_B1.txt
- **Next**: File bug, investigate priority scoring logic
```

### Evidence to Capture

**For bugs**:
- Exact Anna version (`annactl --version` or check README)
- System info (`uname -a`, `pacman -Q | wc -l`)
- Full `annactl status` output
- Relevant journal logs
- TUI screenshot if applicable
- Steps to reproduce

---

## 6. Safety Guidelines

### Operations to NEVER Do on Production Systems

❌ **DO NOT**:
- Fill disk to 99%+ on production system
- Trigger OOM killer on production system
- Stop critical services (sshd, systemd-logind, NetworkManager on remote system)
- Delete system files or logs
- Modify kernel parameters without backup plan
- Test on systems with irreplaceable data

### Recommended Test Environments

✅ **SAFE**:
- Spare laptop or desktop
- Virtual machine (VirtualBox, QEMU, VMware)
- Container (limited testing capabilities)
- Non-critical Arch installation

✅ **PREFERRED**:
- VM with snapshot capability
- Test machine with fresh Arch install
- Development laptop (not production workstation)

### Recovery Checklist

Before Level 2-3 testing:
- [ ] VM snapshot taken (if using VM)
- [ ] Backup of critical data
- [ ] Know how to boot into recovery mode
- [ ] Have Arch USB installer ready
- [ ] SSH access to system (if testing remotely)
- [ ] Console access (if testing headless)

### When to Stop Testing

**Stop immediately if**:
- System becomes unresponsive
- `annad` daemon crashes repeatedly
- Kernel panics occur
- File system errors appear
- Data corruption suspected

**Recover and investigate before continuing.**

---

## 7. Testing Schedule Recommendation

### Week 1: Level 0 Validation
- **Day 1**: Install Anna on test system, verify daemon starts
- **Day 2**: Run baseline health checks (A1)
- **Day 3**: Test NL routing with 10-20 queries in TUI
- **Day 4**: Run 700-question suite, verify 87% accuracy
- **Day 5**: Document findings, file any bugs

**Goal**: Confirm Anna installs cleanly and reports correctly on healthy system.

---

### Week 2: Level 1 Stress Testing
- **Day 1**: Service failure tests (A2)
- **Day 2**: Disk pressure tests (C1)
- **Day 3**: CPU/memory tests (D1)
- **Day 4**: Network priority test (B1, if USB Ethernet available)
- **Day 5**: Document findings, file any bugs

**Goal**: Validate single-issue detection and remediation suggestions.

---

### Week 3: Level 2 Combined Scenarios
- **Day 1**: Service + disk (E1)
- **Day 2**: Service + network (E2)
- **Day 3**: Packet loss simulation (B2)
- **Day 4**: Disk 95% full (C2)
- **Day 5**: Document findings, evaluate proactive engine accuracy

**Goal**: Validate multi-issue correlation and proactive analysis.

---

### Week 4: Edge Cases and Bugs
- **Day 1-3**: Fix bugs discovered in Weeks 1-3
- **Day 4**: Re-run failed tests to verify fixes
- **Day 5**: Write up testing summary, decide on Beta.278 readiness

**Goal**: Achieve confidence in existing functionality before adding features.

---

## 8. Success Criteria

### Level 0 Success
- ✅ `annad` daemon starts without errors
- ✅ `annactl status` reports correctly on healthy system
- ✅ TUI renders cleanly in your terminal
- ✅ 700-question suite passes at 87%

### Level 1 Success
- ✅ Anna detects single service failures
- ✅ Anna detects disk pressure at 90%
- ✅ Anna detects high CPU/memory usage
- ✅ Anna ranks network interfaces correctly (if USB Ethernet available)
- ✅ Remediation suggestions are accurate and safe

### Level 2 Success
- ✅ Anna correlates multiple issues (service + disk, service + network)
- ✅ Proactive engine shows reasonable correlations (no obvious false positives)
- ✅ TUI remains stable under combined stress
- ✅ Remediation suggestions remain accurate with multiple issues

### Level 3 Success
- ✅ Anna handles extreme conditions gracefully (no crashes)
- ✅ System recoverable after edge case testing
- ✅ No data corruption or system damage

---

## 9. Known Limitations to Accept

During testing, you may encounter:

**Expected Limitations** (not bugs):
- Anna cannot predict the future (no forecasting)
- Anna cannot detect security intrusions (no security module yet)
- Anna cannot track historical trends (no historian yet)
- Anna may misrank network interfaces (priority scoring is heuristic)
- Anna may not correlate all related issues (proactive engine is shallow)
- Some NL queries route to conversational instead of diagnostic (intentional)

**Potential Bugs** (file issues):
- Crashes or hangs
- Incorrect diagnostic insights
- Missing issues that should be detected
- False positive issues
- Inaccurate remediation suggestions
- TUI rendering glitches

---

## 10. Post-Testing Next Steps

### If Testing Goes Well (Minimal Bugs)

✅ Proceed with Beta.278 (Sysadmin Report v1)
✅ Continue feature development with confidence

### If Testing Reveals Major Bugs

⚠️ Pause feature development
⚠️ Create bug fix betas (Beta.278-280 for bug fixes)
⚠️ Re-test until confidence restored

### If Testing Reveals Fundamental Issues

🛑 Stop and reassess architecture
🛑 Consider refactoring before adding more features
🛑 Update roadmap based on findings

---

**End of Testing Strategy V1**

**Next**: Execute Level 0 testing on your Arch system and document results.

**Related Documents**:
- `PROJECT_STATUS_OVERVIEW.md` - Current state of Anna
- `BETA_277_NOTES.md` - Latest beta details
- `ARCHITECTURE_BETA_200.md` - System architecture
