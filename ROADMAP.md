# Anna Assistant - Roadmap

**Current Version:** 5.7.0-beta.29

Anna is a local system caretaker for your Arch Linux machine. This roadmap outlines the key milestones in making Anna a capable, trustworthy assistant.

---

## Milestone 0 - Companion Shell ✅ (Complete)

**Goal:** Create a natural, conversational interface where you can talk to Anna.

**Status:** Complete

**What Works:**
- Natural language interface (REPL and one-shot queries)
- Intent detection for common tasks
- Multi-language support (English, Spanish, Norwegian, German, French, Portuguese)
- Terminal capability adaptation (color, Unicode, emoji)
- Consistent UI across all interactions
- Language Contract established

**Try it:**
```bash
annactl "how are you?"
annactl "what should I improve?"
annactl "use Spanish"
```

---

## Milestone 1 - Deep Caretaker 🚧 (In Progress - ~35% Complete)

**Goal:** Anna proactively watches your system and suggests improvements based on Arch Wiki knowledge.

**What We're Building:**
- Telemetry system that observes your machine
- Suggestion engine powered by Arch Wiki rules
- Top 2-5 actionable suggestions at a time
- Clear explanations with documentation links
- Safe, reversible actions with approval workflow

**When Complete, You'll Be Able To:**
```bash
# Get smart suggestions about your actual system
annactl "what's the state of my system?"

# Anna will tell you about:
# - Package cache that can be cleaned
# - Orphaned packages you can remove
# - Security updates needed
# - Performance optimizations available
# - Configuration improvements
```

**Key Principles:**
- All suggestions backed by Arch Wiki or official docs
- Never overwhelming (max 2-5 suggestions)
- Explain tradeoffs honestly
- Safe by default

### System Detection Progress

#### Hardware Detection
- [x] CPU model, cores, threads ✅ (beta.27)
- [x] CPU flags (SSE, AVX, AVX2, AVX-512) ✅ (beta.27)
- [x] CPU governor ✅ (beta.27)
- [x] CPU microcode package/version ✅ (beta.27)
- [x] CPU temperatures ✅ (beta.29)
- [ ] CPU throttling events
- [ ] CPU power states
- [x] GPU vendor, model, driver ✅ (beta.28)
- [x] GPU VRAM ✅ (partial)
- [ ] GPU CUDA/OpenCL support detection
- [x] GPU temperatures ✅ (beta.29)
- [ ] GPU throttling events
- [x] Memory total/available RAM ✅ (beta.29)
- [x] Swap config/type/usage ✅ (beta.29)
- [x] OOM events ✅ (beta.29)
- [x] Memory pressure (PSI) ✅ (beta.29)
- [ ] Storage device types (SSD vs HDD)
- [ ] SMART status
- [ ] Storage health degradation
- [ ] Storage I/O errors
- [ ] Storage throughput/latency
- [ ] Partition alignment
- [x] Filesystem types ✅ (beta.26)
- [x] TRIM status ✅ (beta.26)
- [ ] Network interfaces (active)
- [ ] DHCP vs static config
- [ ] IPv4/IPv6 status
- [x] DNS servers ✅ (beta.27)
- [ ] DNSSEC status
- [ ] Network latency
- [ ] Packet loss
- [ ] Route table
- [ ] Active firewall rules
- [x] NVMe temperatures ✅ (beta.29)
- [x] Fan speeds ✅ (beta.29)
- [x] Voltage readings ✅ (beta.29)
- [ ] Voltage anomalies
- [x] AC vs battery status ✅ (beta.29)
- [x] Battery health ✅ (beta.29)
- [x] Battery charge cycles ✅ (beta.29)
- [x] Power draw ✅ (beta.29)
- [x] TLP/power-profiles-daemon status ✅ (beta.29)

#### Software Detection
- [x] Kernel version ✅
- [ ] Installed kernels (LTS vs mainline)
- [ ] Loaded kernel modules
- [ ] Broken kernel modules
- [ ] DKMS failures
- [x] Boot system (UEFI vs BIOS) ✅ (beta.25)
- [x] Secure Boot status ✅ (beta.25)
- [x] Bootloader (systemd-boot vs GRUB) ✅ (beta.25)
- [ ] Boot entries sanity check
- [ ] Boot errors/warnings
- [x] systemd failed units ✅ (beta.27)
- [x] systemd slow-starting units ✅ (beta.27)
- [ ] systemd daemon crashes
- [x] systemd critical timers ✅ (beta.27)
- [ ] System load averages
- [ ] mkinitcpio/dracut config
- [ ] Missing initramfs hooks/modules
- [ ] Initramfs compression type
- [x] pacman configuration ✅ (beta.28)
- [x] Mirror speed/age ✅ (beta.28)
- [ ] pacman database corruption
- [ ] Partial upgrades
- [ ] Orphaned packages
- [ ] Unowned files
- [ ] Conflicting files
- [x] Firewall type/status ✅ (beta.28)
- [x] SSH configuration ✅ (beta.28)
- [ ] SELinux/AppArmor status
- [ ] Polkit configuration
- [ ] Sudoers configuration
- [ ] Kernel lockdown mode
- [x] Btrfs subvolumes/snapshots ✅ (beta.26)
- [x] Btrfs balance status ✅ (beta.26)
- [ ] Ext4 fsck status/errors
- [ ] XFS log/errors
- [ ] ZFS pools/scrubs
- [ ] Backup tools installed
- [ ] Last backup timestamp
- [ ] Backup integrity errors
- [ ] Missing snapshots
- [x] Docker/Podman service status ✅ (beta.28)
- [ ] Broken containers
- [ ] High CPU containers
- [ ] Missing cgroup limits
- [x] KVM capability ✅ (beta.28)
- [ ] Nested virtualization
- [ ] QEMU performance flags
- [x] Display server (X11/Wayland) ✅ (beta.28)
- [ ] Display driver issues
- [ ] Resolution/refresh rate
- [ ] Multi-monitor issues

#### User Behavior Detection
- [ ] Command execution patterns
- [ ] Resource usage patterns
- [ ] Disk usage patterns
- [ ] Networking behavior
- [ ] Application behavior
- [ ] Gaming/GPU usage patterns
- [ ] Development workflow patterns
- [ ] Security behavior patterns

#### LLM Contextualization
- [ ] System identity summary
- [ ] Stability indicators
- [ ] Performance indicators
- [ ] Risk indicators
- [ ] Inferred user goals

---

## Milestone 2 - Emergency Helper 🔮 (Planned)

**Goal:** Anna helps you recover from emergencies and critical situations.

**Planned Capabilities:**
- **Chroot recovery** - Guide you through fixing a broken system from live USB
- **OOM handling** - Help identify and mitigate out-of-memory situations
- **Kernel panic** - Collect info and suggest fixes for boot failures
- **SSH intrusion** - Detect suspicious SSH activity and help secure the system
- **Disk full** - Emergency cleanup when you can't even log in

**How It Will Work:**
```bash
# From a live USB after system won't boot:
annactl emergency chroot

# When system is crawling from memory issues:
annactl "why is my system so slow?"

# After seeing suspicious SSH logs:
annactl "check SSH security"
```

**Safety First:**
- Anna never runs dangerous commands automatically
- Always explains what will happen
- Shows you the commands before running them
- Asks for confirmation on destructive operations

---

## Long-Term Vision

### What Anna Will Never Be

Anna is designed with clear boundaries:

❌ **Not a fleet manager** - Anna focuses on ONE machine (yours)
❌ **Not a remote control plane** - Everything runs locally
❌ **Not a generic chatbot** - Anna knows Arch Linux, not weather or recipes
❌ **Not autonomous** - Anna asks before changing anything important

### What Anna Aspires To Be

✅ **Your knowledgeable companion** - Knows Arch Linux deeply
✅ **Honest and transparent** - Explains tradeoffs, admits uncertainty
✅ **Respectful of your time** - Gives you what you need, not overwhelming detail
✅ **Safe by default** - Never breaks your system
✅ **Privacy-focused** - All data stays local

---

## Contributing Ideas

Have an idea for Anna? Great!

**Before opening an issue:**
1. Check if it aligns with Anna's vision (see above)
2. Consider which milestone it fits into
3. Think about how it would work in conversation

**Good idea examples:**
- "Anna should detect when my battery health is declining"
- "Anna could warn about deprecated packages in my system"
- "Add support for [Language]"

**Out of scope examples:**
- "Anna should manage multiple servers"
- "Anna should have a web dashboard"
- "Anna should send alerts to Slack"

Open an issue with the `[feature-request]` tag and we'll discuss where it fits.

---

## Version History

| Version | Date | Milestone | Key Features |
|---------|------|-----------|--------------|
| 5.7.0-beta.29 | 2025-11-15 | Milestone 1 | Hardware monitoring: sensors, power, memory (~935 lines) |
| 5.7.0-beta.28 | 2025-11-15 | Milestone 1 | Graphics, security, virtualization, package mgmt (~1400 lines) |
| 5.7.0-beta.27 | 2025-11-15 | Milestone 1 | Systemd health, network config, CPU performance (~925 lines) |
| 5.7.0-beta.26 | 2025-11-15 | Milestone 1 | Filesystem features (TRIM, LUKS, Btrfs) |
| 5.7.0-beta.25 | 2025-11-14 | Milestone 1 | Boot info, audio system, desktop detection |
| 5.3.0-beta.1 | 2025-11-14 | Milestone 0 | Conversational interface, multi-language support, UI abstraction |
| 5.2.0-beta.1 | 2025-11 | Milestone 0 | Intent routing, personality system |
| 5.1.0-beta.1 | 2025-11 | Milestone 0 | REPL foundation |

---

**Questions about the roadmap?**

Open an issue with tag `[roadmap-question]` and we'll discuss.
