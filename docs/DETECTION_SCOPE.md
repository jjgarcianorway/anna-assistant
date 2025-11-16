# Anna Detection Scope

This file describes the complete detection surface Anna must monitor to give accurate, contextual answers. It is the source of truth for what the caretaker needs to sense across hardware, software, user behavior, and LLM execution.

---

## Hardware Detection
- **CPU**: model, cores, threads, AVX/AVX2/AVX512/SSE flags, temperatures, throttling events, power states
- **GPU**: vendor/model, driver in use, VRAM, CUDA/OpenCL availability, temperatures, throttling
- **Memory**: total/available RAM, swap config and type (file/partition/zram), swap usage, OOM events
- **Storage**: SSD/NVMe/HDD types, SMART status, health degradation, I/O errors, throughput/latency, partition alignment, filesystem per partition, trim status
- **Network**: active interfaces, DHCP/static config, IPv4/IPv6 status, DNS servers + DNSSEC, ping latency, packet loss, route table, active firewall rules
- **Sensors**: CPU/GPU/NVMe temps, fan speeds, voltage anomalies
- **Power**: AC/battery state, battery health and charge cycles, power draw, tlp or power-profiles-daemon mode

## Software Detection
- **Kernel**: current kernel, installed kernels, LTS vs mainline, loaded modules, broken modules, DKMS failures
- **Boot**: bootloader presence/health (systemd-boot or GRUB), boot entry sanity, boot errors from journal
- **System Services**: failed units, slow-starting units, daemon crashes, critical timers, system load averages
- **Initrd**: mkinitcpio or dracut config, missing hooks/modules, compression type
- **Package System**: pacman config, mirror speed, db corruption, partial upgrades, orphaned packages, unowned files, conflicting files
- **Security**: firewalld/nftables status, SSH config/keys, SELinux or AppArmor (if enabled), polkit rules, sudoers syntax, kernel lockdown, SecureBoot status
- **Filesystems**: btrfs subvolumes/snapshots/balance, ext4 fsck-needed/errors, XFS logs/errors, ZFS pool/scrub state
- **Backup State**: backup tools installed, last run timestamp, integrity errors, missing snapshots
- **Containers**: docker/podman service state, broken containers, high-CPU containers, missing cgroup limits
- **Virtualization**: KVM capability, nested virt, QEMU performance flags
- **Display**: X11/Wayland, drivers in use (nvidia proprietary/nouveau/amdgpu), resolution/refresh, multi-monitor inconsistencies

## User Behavior Detection (opt-in where sensitive)
- **Execution Patterns**: frequently used commands, installed-but-unused tools, repeated failures, repeated sudo attempts, CLI habits (editor, shells, package usage)
- **Resource Usage**: peak RAM, chronic swap usage, chronic high load, long-running processes, CPU-hog background services
- **Disk Usage Patterns**: home growth, cache accumulation, forgotten downloads, log bloat
- **Networking Behavior**: frequent drops, DNS failures, VPN usage patterns, public Wi-Fi warnings
- **Application Behavior**: repeated app crashes, electron memory spikes, browser freezes due to GPU misconfig
- **Gaming / GPU-heavy**: FPS drops, stuck GPU clocks, missing Vulkan layers
- **Development Workflow**: rust toolchains, python versions, node versions, broken virtual envs, dev containers, build failures
- **Security Behavior**: weak SSH keys, reused passwords (local-only detection), suspicious login attempts, open ports, world-writable files in home

## LLM Contextualization Requirements
- **System Identity**: hostname, OS version, kernel version, desktop environment, GPU stack, package count
- **Stability Indicators**: journald error rate, kernel warnings, segfault frequency, network stability, filesystem errors
- **Performance Indicators**: boot time, service bottlenecks, I/O bottlenecks, CPU/GPU bottlenecks
- **Risk Indicators**: outdated kernel, broken services, SMART warnings, near-full disks, OOM kill patterns
- **User Goals**: performance, stability, gaming, development, laptop battery optimization

## Summary Targets
- Detect hardware health, software integrity, system service stability, security posture, performance patterns, filesystem health, network reliability, and user behavior/workflow so the LLM can answer with full local context.
