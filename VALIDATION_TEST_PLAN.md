# Anna Assistant - Real-World Validation Test Plan

**Version:** 5.7.0-beta.69
**Test Date:** 2025-11-18
**Purpose:** Validate Anna's responses against real Arch Linux questions

---

## Test Methodology

### How to Test

1. **Setup:**
   ```bash
   # Ensure annad is running
   sudo systemctl status annad

   # Launch annactl
   annactl
   ```

2. **Ask Each Question:**
   - Copy question from list below
   - Paste into Anna's REPL
   - Record Anna's response
   - Compare to expected behavior

3. **Success Criteria:**
   - Anna provides accurate, Arch-specific answer
   - No hallucinations or generic advice
   - References Arch Wiki when appropriate
   - Uses actual telemetry (not guesses)

---

## Top 20 Arch Linux Questions (Sourced from Web Research)

### Package Management (Most Common Category)

#### Q1: "How do I update my Arch Linux system?"
**Expected Answer:**
- Should mention `sudo pacman -Syu`
- Explain -S (sync), -y (refresh), -u (upgrade)
- Warn about reviewing package list before confirming
- Mention checking Arch news first

**Success Criteria:**
- ✅ Provides correct command
- ✅ Explains flags
- ✅ Mentions best practices

---

#### Q2: "I'm getting signature errors when installing packages"
**Expected Answer:**
- Likely cause: Outdated keyring
- Solution: `sudo pacman -S archlinux-keyring` first
- Or full system upgrade: `sudo pacman -Syu`
- May need to manually update keys

**Success Criteria:**
- ✅ Identifies keyring as likely cause
- ✅ Provides correct solution
- ✅ Explains why it happens

---

#### Q3: "Pacman says 'error: failed to commit transaction (conflicting files)'"
**Expected Answer:**
- Another package owns the file
- Check which package: `pacman -Qo /path/to/file`
- Resolve conflict or force overwrite (with caution)
- Reference: Arch Wiki Pacman troubleshooting

**Success Criteria:**
- ✅ Explains the error
- ✅ Provides diagnostic command
- ✅ Offers solution

---

#### Q4: "What's the difference between pacman and yay?"
**Expected Answer:**
- pacman: Official package manager (core/extra/community repos)
- yay: AUR helper (accesses Arch User Repository)
- AUR = community-maintained, not officially supported
- Use pacman for official packages, yay for AUR

**Success Criteria:**
- ✅ Explains both tools
- ✅ Clarifies AUR vs official repos
- ✅ Recommends when to use each

---

#### Q5: "Why does pacman upgrade hang for 10 seconds when fetching packages?"
**Expected Answer:**
- Recent 2025 issue reported
- Possible causes: Mirror issues, network latency
- Solution: Update mirrorlist with reflector
- Check mirror status at archlinux.org/mirrors

**Success Criteria:**
- ✅ Acknowledges known issue
- ✅ Suggests mirror update
- ✅ Provides troubleshooting steps

---

### System Issues

#### Q6: "My system won't boot after an update"
**Expected Answer:**
- Common causes: Kernel panic, bootloader issues, filesystem errors
- Recovery: Boot from live USB/CD
- Check systemd logs: `journalctl -xb`
- May need to chroot and downgrade problematic package

**Success Criteria:**
- ✅ Lists common causes
- ✅ Provides recovery steps
- ✅ Mentions chroot for repairs

---

#### Q7: "X server fails after system upgrade"
**Expected Answer:**
- Recent June 2025 issue
- Related to linux-firmware split and NVIDIA firmware reorganization
- Check graphics drivers: `lspci -k | grep -A 3 VGA`
- May need to reinstall video drivers

**Success Criteria:**
- ✅ Identifies recent known issue
- ✅ Provides diagnostic command
- ✅ Suggests driver reinstall

---

#### Q8: "How do I check my CPU temperature?"
**Expected Answer:**
- Use `sensors` command (from lm_sensors package)
- First run: `sudo sensors-detect`
- Then: `sensors`
- Or check: `cat /sys/class/thermal/thermal_zone*/temp`

**Success Criteria:**
- ✅ Provides correct tools
- ✅ Explains setup steps
- ✅ Offers alternative methods

---

#### Q9: "What's using all my RAM?"
**Expected Answer:**
- Check with: `free -h` (overview)
- Detailed: `ps aux --sort=-%mem | head -10`
- Or: `top` / `htop` (interactive)
- Linux uses RAM for cache (this is good!)

**Success Criteria:**
- ✅ Provides multiple methods
- ✅ Explains Linux memory management
- ✅ Reassures about cache usage

---

#### Q10: "My system is slow, what should I check?"
**Expected Answer:**
- Check CPU: `top`, `htop`
- Check disk I/O: `iotop`
- Check RAM: `free -h`
- Check services: `systemctl --failed`
- Look for high CPU/RAM processes

**Success Criteria:**
- ✅ Systematic troubleshooting approach
- ✅ Multiple diagnostic commands
- ✅ Explains what to look for

---

### Hardware & Drivers

#### Q11: "My NVIDIA GPU isn't working"
**Expected Answer:**
- Check if detected: `lspci -k | grep -A 3 VGA`
- Install drivers: `sudo pacman -S nvidia`
- Or: `nvidia-lts` for LTS kernel
- May need to configure Xorg

**Success Criteria:**
- ✅ Diagnostic command first
- ✅ Correct driver package
- ✅ Mentions kernel compatibility

---

#### Q12: "Pipewire makes loud popping sounds"
**Expected Answer:**
- Known issue with sound card activation/deactivation
- Check audio config: `systemctl --user status pipewire`
- May need to adjust buffer settings
- Alternative: Switch to PulseAudio

**Success Criteria:**
- ✅ Acknowledges known issue
- ✅ Provides troubleshooting
- ✅ Offers alternative

---

#### Q13: "My touchpad doesn't work"
**Expected Answer:**
- Check if detected: `libinput list-devices`
- Install drivers: `xf86-input-libinput`
- Configure: `/etc/X11/xorg.conf.d/30-touchpad.conf`
- Check systemd-logind

**Success Criteria:**
- ✅ Detection command
- ✅ Correct driver package
- ✅ Configuration location

---

### Network & Boot

#### Q14: "I can't connect to WiFi"
**Expected Answer:**
- Check interface: `ip link`
- Check NetworkManager: `systemctl status NetworkManager`
- Or use: `nmcli` for configuration
- May need firmware: Check `dmesg | grep firmware`

**Success Criteria:**
- ✅ Systematic network troubleshooting
- ✅ Multiple tools mentioned
- ✅ Checks both hardware and software

---

#### Q15: "What is the Arch User Repository (AUR)?"
**Expected Answer:**
- Community-driven repository
- NOT officially supported by Arch
- Requires AUR helper (yay, paru) or manual build
- PKGBUILDs maintained by users
- Use at your own risk, review PKGBUILDs

**Success Criteria:**
- ✅ Explains AUR clearly
- ✅ Mentions community nature
- ✅ Warns about security

---

#### Q16: "Secure Boot is preventing boot, what should I do?"
**Expected Answer:**
- Arch doesn't officially support Secure Boot
- Solution: Disable Secure Boot in BIOS/UEFI
- Or: Use signed bootloader (advanced)
- Check BIOS settings

**Success Criteria:**
- ✅ Identifies Secure Boot issue
- ✅ Recommends disabling
- ✅ Mentions advanced alternative

---

### Services & Systemd

#### Q17: "How do I check which services failed to start?"
**Expected Answer:**
- Command: `systemctl --failed`
- Details: `systemctl status <service>`
- Logs: `journalctl -xeu <service>`
- Restart: `sudo systemctl restart <service>`

**Success Criteria:**
- ✅ Provides multiple systemctl commands
- ✅ Shows how to get details
- ✅ Includes restart command

---

#### Q18: "What logs should I check when troubleshooting?"
**Expected Answer:**
- System log: `journalctl -xe`
- Boot log: `journalctl -b`
- Specific service: `journalctl -u <service>`
- Kernel: `dmesg`
- All persistent: `journalctl --list-boots`

**Success Criteria:**
- ✅ Multiple log types
- ✅ Correct journalctl syntax
- ✅ Explains when to use each

---

### Configuration & Setup

#### Q19: "How do I install a desktop environment?"
**Expected Answer:**
- Choose: GNOME, KDE, XFCE, etc.
- Install: `sudo pacman -S <desktop-environment>`
- Examples: `gnome`, `plasma`, `xfce4`
- Enable display manager: `sudo systemctl enable gdm` (or lightdm, etc.)

**Success Criteria:**
- ✅ Lists popular options
- ✅ Correct installation method
- ✅ Mentions display manager

---

#### Q20: "Where are my user configuration files?"
**Expected Answer:**
- User configs: `~/.config/`
- Shell config: `~/.bashrc` or `~/.zshrc`
- System-wide: `/etc/`
- Application-specific: Various locations (check Wiki)

**Success Criteria:**
- ✅ Correct paths
- ✅ Explains user vs system
- ✅ References documentation

---

## Expected Results

### Success Targets

**Target Pass Rate:** ≥17/20 (85%)

**Categories:**
- **Excellent (18-20):** Anna is production-ready
- **Good (15-17):** Minor gaps, mostly working
- **Fair (12-14):** Significant gaps, needs improvement
- **Poor (<12):** Major issues, not ready

---

## Testing Notes

### What Anna SHOULD Do:
- ✅ Use exact command syntax
- ✅ Reference Arch Wiki
- ✅ Provide context and explanations
- ✅ Use actual system telemetry when available
- ✅ Warn about risks (AUR, force operations)

### What Anna SHOULD NOT Do:
- ❌ Hallucinate package names
- ❌ Give Ubuntu/Debian-specific advice
- ❌ Use generic "AI assistant" disclaimers
- ❌ Approximate when exact values available
- ❌ Recommend unsafe operations without warnings

---

## Comparison Sources

**Official References:**
- Arch Wiki: https://wiki.archlinux.org/
- Arch Forums: https://bbs.archlinux.org/
- Package Database: https://archlinux.org/packages/

**Community Sources:**
- r/archlinux on Reddit
- Stack Exchange (Unix & Linux)
- Super User (arch-linux tag)

---

## Test Execution Instructions

### Step-by-Step:

1. **Prepare Environment:**
   ```bash
   # Start daemon
   sudo systemctl start annad

   # Launch Anna
   annactl
   ```

2. **Test Each Question:**
   - Copy question verbatim
   - Paste into REPL
   - Wait for Anna's response
   - Record response in table below

3. **Score Each Response:**
   - **PASS** - Meets all success criteria
   - **PARTIAL** - Some criteria met, some issues
   - **FAIL** - Does not meet criteria / incorrect

4. **Document Results:**
   - Save Anna's actual responses
   - Note any hallucinations
   - Note any missing information
   - Note any incorrect advice

---

## Results Template

| # | Question | Pass/Fail | Notes |
|---|----------|-----------|-------|
| 1 | System update | | |
| 2 | Signature errors | | |
| 3 | Conflicting files | | |
| 4 | pacman vs yay | | |
| 5 | Pacman hang | | |
| 6 | Won't boot | | |
| 7 | X server fails | | |
| 8 | CPU temperature | | |
| 9 | RAM usage | | |
| 10 | System slow | | |
| 11 | NVIDIA issues | | |
| 12 | Pipewire popping | | |
| 13 | Touchpad not working | | |
| 14 | WiFi issues | | |
| 15 | What is AUR | | |
| 16 | Secure Boot | | |
| 17 | Failed services | | |
| 18 | Check logs | | |
| 19 | Install DE | | |
| 20 | Config files | | |

**Total:** ___/20 (___%

)

**Status:** _____________ (Excellent/Good/Fair/Poor)

---

## Additional Test Categories (Optional)

### Advanced Questions:
- systemd-boot vs GRUB
- Managing multiple kernels
- Custom kernel compilation
- Partitioning schemes (LVM, Btrfs)
- Power management (laptop-mode-tools)

### Troubleshooting Scenarios:
- Kernel panic recovery
- Broken bootloader repair
- Filesystem corruption
- Package database corruption
- Dependency conflicts

---

## Report Template

### Test Summary

**Date:** _____________
**Anna Version:** 5.7.0-beta.69
**Tester:** _____________
**System:** _____________

**Results:**
- Total Questions: 20
- Passed: ___
- Partial: ___
- Failed: ___
- Pass Rate: ___%

**Key Findings:**
- Strengths: _____________
- Weaknesses: _____________
- Critical Issues: _____________
- Recommendations: _____________

**Conclusion:**
_____________

---

*Test Plan Version: 1.0*
*Created: 2025-11-18*
*Based on: Web research of most common Arch Linux questions*
