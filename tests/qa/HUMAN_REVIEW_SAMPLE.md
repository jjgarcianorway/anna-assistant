# Human Review Sample - QA Suite Validation

**Purpose**: Manually validate that automated verdicts match human expert judgment.

**Date**: 2025-11-20
**Anna Version**: 5.7.0-beta.149
**Questions Reviewed**: 5 / 5 (100% of current batch)
**Reviewer**: Claude (QA Architect Mode)

---

## Sample 1: arch-001 - Static IP with systemd-networkd

### Question
"How do I configure a static IP address on Arch Linux using systemd-networkd?"

### Golden Answer (Summary)
Create a .network file in /etc/systemd/network/ with interface match and network configuration, then restart systemd-networkd service. Must include: `systemctl enable`, `systemctl restart`, `ip link`, `ip addr` validation.

### Anna's Answer
```
ℹ To configure a static IP address on Arch Linux using systemd-networkd, run these commands:

```bash
sudo nano /etc/systemd/network/20-ethernet.network
```

In this file, add the following configuration:
```
[Match]
Name=eth*

[Network]
Address=192.168.1.100/24
Gateway=192.168.1.1
DNS=192.168.1.1

[Route]
Gateway=192.168.1.1
```

Replace the IP address, netmask, gateway, and DNS server with your desired configuration.

Save and close the file, then:

```bash
sudo systemctl daemon-reload
sudo systemctl restart systemd-networkd
```

These commands will configure your system to use a static IP address on the Ethernet interface.
```

### Automated Verdict
**FAIL** (score: 0.5)

Issues:
- Missing required command: `systemctl enable systemd-networkd`
- Missing required command: `ip link`
- Missing required file: `/etc/systemd/network/*.network`
- Missing key concept: CIDR notation
- Missing safety warnings

### Human Review
**VERDICT: Automated evaluation is CORRECT - this should be FAIL**

**Reasoning:**
1. ✅ Anna creates the correct file (`/etc/systemd/network/20-ethernet.network`)
2. ✅ Anna shows correct [Match] and [Network] sections
3. ✅ Anna includes Address, Gateway, DNS configuration
4. ✅ Anna runs `systemctl restart systemd-networkd`

**BUT CRITICAL PROBLEMS:**
1. ❌ **Missing `systemctl enable systemd-networkd`** - Without this, the config won't persist across reboots! This is a CRITICAL omission.
2. ❌ **Missing `ip link` step** - User needs to know their interface name first
3. ❌ **No validation steps** - Doesn't mention `ip addr show` or `ping` to verify it works
4. ❌ **No safety warnings** - Doesn't warn about NetworkManager conflicts or backup

**Arch Expert Assessment**: An Arch expert would say "This will work ONCE, but will break after reboot because you forgot to enable the service." This is a common beginner mistake.

**Alternative Verdict Consideration**: Could this be PARTIAL instead of FAIL?
- No, because forgetting `systemctl enable` is a **critical failure** that makes the solution incomplete.
- A PARTIAL answer would be one that works but is missing optional enhancements.

**Conclusion**: FAIL verdict is appropriate.

---

## Sample 2: arch-002 - Install from AUR

### Question
"How do I install a package from the AUR?"

### Golden Answer (Summary)
Clone AUR package with git, **REVIEW PKGBUILD for security**, build with `makepkg -si`. Must include: base-devel installation, git clone, PKGBUILD review warning, makepkg.

### Anna's Answer
```
anna: Running:
ℹ   $ pacman -Qm

aic94xx-firmware 30-10
ast-firmware 501-1
bibata-cursor-git 1.0.0.r0.f4ccfe8-1
citron 0.11.0-1
[... 20 more packages ...]
```

### Automated Verdict
**FAIL** (score: 0.0)

Issues:
- Missing ALL required commands
- Missing ALL required concepts
- Missing PKGBUILD security warning
- Output doesn't answer the question

### Human Review
**VERDICT: Automated evaluation is CORRECT - this is a COMPLETE FAIL**

**Reasoning:**
Anna **completely misunderstood the question**. Instead of explaining HOW to install a package from the AUR, Anna ran `pacman -Qm` which just lists already-installed AUR packages.

This is equivalent to:
- User: "How do I bake a cake?"
- Anna: "Here's a list of cakes in your pantry."

**What Anna SHOULD have said:**
1. Install base-devel: `sudo pacman -S base-devel git`
2. Clone package: `git clone https://aur.archlinux.org/<package>.git`
3. **CRITICAL**: Review PKGBUILD for security
4. Build: `makepkg -si`

**What Anna ACTUALLY said:**
- Shows list of installed AUR packages (irrelevant)

**Arch Expert Assessment**: An Arch expert would say "This answer is completely wrong. It doesn't teach the user anything about installing from AUR."

**Security Concern**: Anna also failed to mention the CRITICAL security warning about reviewing PKGBUILDs. This is dangerous - users could install malicious packages.

**Conclusion**: FAIL verdict is absolutely correct. This is arguably the worst answer in the batch.

---

## Sample 3: arch-003 - Enable systemd service

### Question
"How do I enable a systemd service to start automatically at boot?"

### Golden Answer (Summary)
Use `systemctl enable <service>` to create symlinks for boot. Also mention `systemctl start` to start immediately, and validation with `systemctl is-enabled`.

### Anna's Answer
```
ℹ To enable a systemd service to start automatically at boot on Arch Linux, use the following command:

```bash
sudo systemctl enable <service-name>
```

For example, if you want to enable the `sshd` service:

```bash
sudo systemctl enable sshd
```

This command will create a symbolic link in the systemd configuration directory, ensuring the service starts automatically the next time you boot your system.

If you want to start the service immediately without waiting for a reboot, you can use:

```bash
sudo systemctl start <service-name>
```

So, for the SSH service, you'd run:

```bash
sudo systemctl start sshd
```

Combining both commands (enable and start) is common practice to ensure the service is both enabled for boot and running immediately.
```

### Automated Verdict
**FAIL** (score: 0.44)

Issues:
- Missing: `systemctl is-enabled` validation
- Missing: `systemctl status` validation
- Missing concepts: unit files, boot targets
- Missing safety warnings

### Human Review
**VERDICT: Automated evaluation is TOO HARSH - this should be PARTIAL, not FAIL**

**Reasoning:**
Anna's answer is actually quite good:
1. ✅ Explains `systemctl enable` correctly
2. ✅ Shows example with sshd
3. ✅ Explains the symbolic link concept
4. ✅ Mentions `systemctl start` for immediate start
5. ✅ Explains the difference between enable (boot) and start (now)

**What's missing:**
1. ❌ Validation steps (`systemctl is-enabled`, `systemctl status`)
2. ❌ Doesn't mention `systemctl enable --now` shortcut
3. ❌ Doesn't mention that some services need configuration first

**Arch Expert Assessment**: An Arch expert would say "This is a solid answer that covers the essentials. It's missing validation steps, but it won't break anything."

**Evaluation Rule Issue**:
The automated evaluator is too strict here. Missing validation commands and advanced concepts like "boot targets" and "unit files" shouldn't result in FAIL when the core answer is correct.

**Recommendation**:
- Update evaluation rules to distinguish between:
  - **Core requirements** (must have for PASS/PARTIAL)
  - **Validation steps** (nice to have, affects score but not verdict)
  - **Advanced concepts** (expert-level, not required for basic questions)

**Adjusted Manual Verdict**: **PARTIAL**
- Answer works correctly
- Missing validation and advanced concepts
- But not a FAIL - user can successfully enable services with this info

**Action Item**: Refine automated scoring to not penalize missing validation as harshly.

---

## Sample 4: arch-004 - Regenerate GRUB config

### Question
"How do I regenerate the GRUB configuration after editing /etc/default/grub?"

### Golden Answer (Summary)
Run `grub-mkconfig -o /boot/grub/grub.cfg` as root. **CRITICAL**: This can make system unbootable - backup first.

### Anna's Answer
```
ℹ After editing `/etc/default/grub`, regenerate the GRUB configuration with:

```bash
sudo grub-mkconfig -o /boot/grub/grub.cfg
```

Then update the GRUB bootloader:

```bash
sudo update-grub
```

(Note: `update-grub` is a convenience script that calls `grub-mkconfig` and updates the boot menu. On Arch Linux, you might need to use the first command if `update-grub` is not available.)
```

### Automated Verdict
**FAIL** (score: 0.14)

Issues:
- Missing required command (detected as missing)
- Missing /boot/grub/grub.cfg file reference (actually present)
- Missing GRUB concepts
- **CRITICAL**: Missing backup warning

### Human Review
**VERDICT: Automated evaluation has FALSE NEGATIVES - but FAIL is still correct due to missing critical safety warning**

**Reasoning:**

**What Anna got RIGHT:**
1. ✅ Shows correct command: `sudo grub-mkconfig -o /boot/grub/grub.cfg`
2. ✅ Mentions the file path `/boot/grub/grub.cfg`

**Automated Evaluation Bug:**
The evaluator falsely claimed Anna was missing the command and file. This is a **bug in the evaluation logic** - the command IS present in Anna's output.

**BUT - Anna still FAILS for a different reason:**
❌ **CRITICAL SAFETY ISSUE**: No warning about system becoming unbootable
❌ **No backup instruction**: Should say `sudo cp /boot/grub/grub.cfg /boot/grub/grub.cfg.backup`
❌ **Mentions `update-grub`**: This is a **Debian/Ubuntu command**, not Arch! Mentioning it confuses users.

**Arch Expert Assessment**:
"The command is correct, but:
1. Mentioning `update-grub` is a red flag - that's not how Arch works
2. No backup warning is DANGEROUS - wrong GRUB config bricks systems
3. Missing the 'have recovery media ready' warning"

**Adjusted Manual Verdict**: **FAIL**
- Command is correct (evaluation bug noted)
- But missing CRITICAL safety warnings for a DANGEROUS operation
- Mentions wrong-distro command (`update-grub`)

**Action Items**:
1. Fix evaluation logic to detect commands correctly
2. Add check for wrong-distro commands (auto-FAIL)
3. Emphasize safety warnings in scoring for dangerous operations

---

## Sample 5: arch-005 - Clean pacman cache

### Question
"How do I clean the pacman cache to free up disk space?"

### Golden Answer (Summary)
Use `paccache -r` (safe, keeps recent versions) or `pacman -Sc` (removes uninstalled). **Never** use `pacman -Scc` without understanding risks.

### Anna's Answer
```
ℹ To clean the pacman cache and free up disk space on Arch Linux, run:

```bash
sudo pacman -Sc
```

This command removes all cached packages that are not currently installed. If you want to remove all cached packages (including those that are installed), use:

```bash
sudo pacman -Scc
```

**Warning**: Using `pacman -Scc` will remove **all** cached packages, which means you won't be able to downgrade packages without downloading them again from the repositories. Use this command with caution.
```

### Automated Verdict
**FAIL** (score: 0.12)

Issues:
- Missing: `paccache -r` (safer method)
- Missing: `du -sh /var/cache/pacman/pkg/` (check size)
- Missing concepts: package cache cleanup, pacman-contrib

### Human Review
**VERDICT: Automated evaluation is TOO HARSH - this should be PARTIAL**

**Reasoning:**

**What Anna got RIGHT:**
1. ✅ Shows `pacman -Sc` (valid method)
2. ✅ Explains what `-Sc` does (removes uninstalled packages)
3. ✅ Shows `pacman -Scc` (removes ALL)
4. ✅ **WARNING about -Scc risks** (can't downgrade without cache)
5. ✅ Tells user to use `-Scc` with caution

**What Anna MISSED:**
1. ❌ Doesn't mention `paccache -r` (safer, recommended method)
2. ❌ Doesn't mention `pacman-contrib` package (provides paccache)
3. ❌ Doesn't show `du -sh` to check cache size first
4. ❌ Doesn't explain why `paccache -r` is better (keeps 2-3 recent versions)

**Arch Expert Assessment**:
"Anna's answer is **correct but not optimal**. An Arch expert would recommend `paccache -r` over `pacman -Sc`, but `pacman -Sc` is not wrong. The `-Scc` warning is good."

**Comparison to Golden**:
- Golden answer: "Use `paccache -r` (best), or `pacman -Sc` (okay), DON'T use `pacman -Scc`"
- Anna answer: "Use `pacman -Sc` (okay), `pacman -Scc` exists but WARNING"

**Adjusted Manual Verdict**: **PARTIAL**
- Answer is correct and safe
- Missing the RECOMMENDED method (`paccache`)
- But won't cause problems - user will successfully clean cache

**Action Item**: Evaluation should distinguish between:
- **Wrong** (FAIL)
- **Suboptimal but safe** (PARTIAL) ← This case
- **Optimal** (PASS)

---

## Summary of Human Review

### Verdict Accuracy

| Question ID | Automated | Human | Agreement? | Notes |
|-------------|-----------|-------|------------|-------|
| arch-001 | FAIL | FAIL | ✅ Yes | Correct - missing critical `enable` command |
| arch-002 | FAIL | FAIL | ✅ Yes | Correct - completely wrong answer |
| arch-003 | FAIL | **PARTIAL** | ❌ No | Too harsh - core answer is correct |
| arch-004 | FAIL | FAIL | ✅ Yes | Correct - missing critical safety warnings |
| arch-005 | FAIL | **PARTIAL** | ❌ No | Too harsh - answer is safe but suboptimal |

### Agreement Rate
- **3 / 5 (60%)** verdicts match human judgment
- **2 / 5 (40%)** are too harsh (should be PARTIAL, not FAIL)

### Patterns Identified

**Automated Evaluation Strengths:**
1. ✅ Correctly detects completely wrong answers (arch-002)
2. ✅ Correctly detects missing critical commands (arch-001 enable)
3. ✅ Correctly checks for required command presence

**Automated Evaluation Weaknesses:**
1. ❌ Too strict on validation steps (treats optional validation as critical)
2. ❌ Doesn't distinguish core requirements from nice-to-haves
3. ❌ Doesn't recognize "suboptimal but safe" as PARTIAL
4. ❌ Doesn't check for wrong-distro commands (update-grub on Arch)
5. ❌ Has bug in command detection (false negative on arch-004)

### Recommendations

**Immediate Fixes Needed:**
1. **Scoring Algorithm**: Separate core requirements from validation steps
   - Core requirements: 80% weight
   - Validation steps: 15% weight
   - Advanced concepts: 5% weight

2. **Wrong-Distro Detection**: Auto-FAIL if answer suggests:
   - `apt`, `apt-get`, `update-grub` (Debian/Ubuntu)
   - `yum`, `dnf` (RHEL/Fedora)

3. **PARTIAL Threshold**: Adjust from 0.6 to 0.5 for "safe but incomplete" answers

4. **Safety Warnings**: Weight safety warnings higher for dangerous operations:
   - GRUB changes: safety warning REQUIRED for PASS/PARTIAL
   - Package installation: security warning REQUIRED for AUR
   - Cache cleaning: risk warning REQUIRED for `pacman -Scc`

**Evaluation Improvements:**
- Add "answer_type" classification:
  - `correct_optimal` → PASS
  - `correct_suboptimal` → PARTIAL
  - `correct_incomplete` → PARTIAL
  - `wrong` → FAIL
  - `irrelevant` → FAIL

### Conclusion

The automated evaluation is **working as designed** but is **too conservative** (preferring FAIL over PARTIAL).

This is actually GOOD for initial development because:
- ✅ No false PASS verdicts (dangerous)
- ✅ Catches genuinely wrong answers
- ⚠️  Some false FAILs (safe - we can fix scoring)

**Current Reality:**
- Anna's actual pass rate: **0% with harsh scoring**
- Anna's adjusted pass rate: **0% PASS, 40% PARTIAL with fair scoring**

**Next Steps:**
1. Implement scoring algorithm improvements
2. Add wrong-distro command detection
3. Re-run test suite with updated rules
4. Extend to remaining 15 questions in initial batch (arch-006 through arch-020)
5. Begin work on full 700 questions

**Honest Assessment**: Anna is failing most Arch Linux questions. Even with scoring improvements, the pass rate will likely be 20-40%, not 90%+. **Much work needed on Anna's Arch knowledge base.**
