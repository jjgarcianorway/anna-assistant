# Anna Assistant - Roadmap

**Current Version:** 5.7.0-beta.39

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

## Milestone 1 - Deep Caretaker 🚧 (In Progress - ~70% Complete)

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
- [x] CPU throttling events ✅ (beta.37)
- [x] CPU power states ✅ (beta.37)
- [x] GPU vendor, model, driver ✅ (beta.28)
- [x] GPU VRAM ✅ (partial)
- [x] GPU CUDA/OpenCL support detection ✅ (beta.39)
- [x] GPU temperatures ✅ (beta.29)
- [x] GPU throttling events ✅ (beta.38)
- [x] Memory total/available RAM ✅ (beta.29)
- [x] Swap config/type/usage ✅ (beta.29)
- [x] OOM events ✅ (beta.29)
- [x] Memory pressure (PSI) ✅ (beta.29)
- [x] Storage device types (SSD vs HDD) ✅ (beta.30)
- [x] SMART status ✅ (beta.30)
- [x] Storage health degradation ✅ (beta.30)
- [x] Storage I/O errors ✅ (beta.30)
- [x] Storage throughput/latency ✅ (beta.30)
- [x] Partition alignment ✅ (beta.30)
- [x] Filesystem types ✅ (beta.26)
- [x] TRIM status ✅ (beta.26)
- [x] Network interfaces (active) ✅ (beta.31)
- [x] DHCP vs static config ✅ (beta.31)
- [x] IPv4/IPv6 status ✅ (beta.31)
- [x] DNS servers ✅ (beta.27)
- [x] DNSSEC status ✅ (beta.31)
- [x] Network latency ✅ (beta.31)
- [x] Packet loss ✅ (beta.31)
- [x] Route table ✅ (beta.31)
- [x] Active firewall rules ✅ (beta.31)
- [x] NVMe temperatures ✅ (beta.29)
- [x] Fan speeds ✅ (beta.29)
- [x] Voltage readings ✅ (beta.29)
- [x] Voltage anomalies ✅ (beta.39)
- [x] AC vs battery status ✅ (beta.29)
- [x] Battery health ✅ (beta.29)
- [x] Battery charge cycles ✅ (beta.29)
- [x] Power draw ✅ (beta.29)
- [x] TLP/power-profiles-daemon status ✅ (beta.29)

#### Software Detection
- [x] Kernel version ✅
- [x] Installed kernels (LTS vs mainline) ✅ (beta.32)
- [x] Loaded kernel modules ✅ (beta.32)
- [x] Broken kernel modules ✅ (beta.32)
- [x] DKMS failures ✅ (beta.32)
- [x] Boot system (UEFI vs BIOS) ✅ (beta.25)
- [x] Secure Boot status ✅ (beta.25)
- [x] Bootloader (systemd-boot vs GRUB) ✅ (beta.25)
- [x] Boot entries sanity check ✅ (beta.32)
- [x] Boot errors/warnings ✅ (beta.32)
- [x] systemd failed units ✅ (beta.27)
- [x] systemd slow-starting units ✅ (beta.27)
- [x] systemd daemon crashes ✅ (beta.36)
- [x] systemd critical timers ✅ (beta.27)
- [x] System load averages ✅ (beta.36)
- [x] mkinitcpio/dracut config ✅ (beta.34)
- [x] Missing initramfs hooks/modules ✅ (beta.34)
- [x] Initramfs compression type ✅ (beta.34)
- [x] pacman configuration ✅ (beta.28)
- [x] Mirror speed/age ✅ (beta.28)
- [x] pacman database corruption ✅ (beta.33)
- [x] Partial upgrades ✅ (beta.33)
- [x] Orphaned packages ✅ (beta.36)
- [x] Unowned files ✅ (beta.33)
- [x] Conflicting files ✅ (beta.33)
- [x] Firewall type/status ✅ (beta.28)
- [x] SSH configuration ✅ (beta.28)
- [x] SELinux/AppArmor status ✅ (beta.35)
- [x] Polkit configuration ✅ (beta.35)
- [x] Sudoers configuration ✅ (beta.35)
- [x] Kernel lockdown mode ✅ (beta.35)
- [x] Btrfs subvolumes/snapshots ✅ (beta.26)
- [x] Btrfs balance status ✅ (beta.26)
- [x] Ext4 fsck status/errors ✅ (beta.40)
- [x] XFS log/errors ✅ (beta.40)
- [x] ZFS pools/scrubs ✅ (beta.40)
- [x] Backup tools installed ✅ (beta.41)
- [x] Last backup timestamp ✅ (beta.41)
- [x] Backup integrity errors ✅ (beta.41)
- [x] Missing snapshots ✅ (beta.41)
- [x] Docker/Podman service status ✅ (beta.28)
- [x] Broken containers ✅ (beta.42)
- [x] High CPU containers ✅ (beta.42)
- [x] Missing cgroup limits ✅ (beta.42)
- [x] KVM capability ✅ (beta.28)
- [x] Nested virtualization ✅ (beta.42)
- [x] QEMU performance flags ✅ (beta.42)
- [x] Display server (X11/Wayland) ✅ (beta.28)
- [x] Display driver issues ✅ (beta.43)
- [x] Resolution/refresh rate ✅ (beta.43)
- [x] Multi-monitor issues ✅ (beta.43)

#### User Behavior Detection
- [x] Command execution patterns ✅ (beta.44)
- [x] Resource usage patterns ✅ (beta.44)
- [x] Disk usage patterns ✅ (beta.44)
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
| 5.7.0-beta.39 | 2025-11-15 | Milestone 1 | GPU compute: CUDA/OpenCL/ROCm/oneAPI detection; voltage monitoring & anomalies (~705 lines) |
| 5.7.0-beta.38 | 2025-11-15 | Milestone 1 | GPU throttling: NVIDIA/AMD/Intel thermal/power limits, multi-GPU support (~420 lines) |
| 5.7.0-beta.37 | 2025-11-15 | Milestone 1 | CPU throttling & power states: throttle events, thermal logs, C-states (~320 lines) |
| 5.7.0-beta.36 | 2025-11-15 | Milestone 1 | System health: load averages, daemon crashes, uptime; orphaned packages (~640 lines) |
| 5.7.0-beta.35 | 2025-11-15 | Milestone 1 | Security features: SELinux, AppArmor, Polkit, sudo, kernel lockdown (~650 lines) |
| 5.7.0-beta.34 | 2025-11-15 | Milestone 1 | Initramfs config: mkinitcpio/dracut, hooks, modules, compression (~590 lines) |
| 5.7.0-beta.33 | 2025-11-15 | Milestone 1 | Package health: database corruption, file conflicts, partial upgrades (~590 lines) |
| 5.7.0-beta.32 | 2025-11-15 | Milestone 1 | Kernel & boot detection: modules, DKMS, boot entries, boot health (~1050 lines) |
| 5.7.0-beta.31 | 2025-11-15 | Milestone 1 | Network monitoring: interfaces, latency, packet loss, routes, firewall (~750 lines) |
| 5.7.0-beta.30 | 2025-11-15 | Milestone 1 | Storage health & performance: SMART, I/O errors, partition alignment (~570 lines) |
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
