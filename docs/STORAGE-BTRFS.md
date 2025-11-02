# Anna Btrfs Storage Intelligence

**Version:** 1.0
**Added in:** Anna v0.12.5
**Status:** Production

## Overview

Anna provides comprehensive Btrfs filesystem intelligence, including automated snapshots, health monitoring, and system optimization recommendations. All features respect the "advisory by default" principle - no automatic modifications without explicit user consent.

## Features

### 1. Storage Profile (`annactl storage btrfs show`)

Displays comprehensive Btrfs filesystem information:
- **Layout:** Subvolumes, mount points, snapshots directory
- **Mount Options:** Compression, autodefrag, SSD mode, space_cache
- **Tools:** Detection of snapper, timeshift, grub-btrfs, pacman hooks
- **Bootloader:** GRUB/systemd-boot detection and snapshot boot entries
- **Health:** Free space, last scrub date, balance status, qgroups

**Output Modes:**
- **TUI:** Beautiful terminal output with colors, emoji, proper width handling
- **JSON:** Machine-readable for scripting (`--json`)
- **Wide:** Show all subvolumes (`--wide`)
- **Explain:** Detailed explanations (`--explain snapshots|compression|scrub|balance`)

**Example:**
```bash
$ annactl storage btrfs show

╭─ Anna Storage Profile (Btrfs) ────────────╮

✓ Btrfs filesystem detected

Layout
──────────────────────────────────────────────
Root FS:       btrfs
Default:       @
Snapshots:     /.snapshots
Separate /home: yes
Separate /var:  yes

Mount Options
──────────────────────────────────────────────
Compression:   zstd:3
Autodefrag:    disabled
NoAtime:       yes
SSD:           yes
Space cache:   v2

Tools
──────────────────────────────────────────────
Snapper:       installed
Pacman hook:   installed
GRUB-btrfs:    installed

Health
──────────────────────────────────────────────
Device:        /dev/nvme0n1p2 (476.9 GB total, 68.2% free)
Free space:    68.2%
Last scrub:    12 days ago
Balance:       not needed
Qgroups:       enabled
```

### 2. Automatic Snapshots

**Script:** `scripts/btrfs/autosnap-pre.sh`
**Hook:** `packaging/arch/hooks/90-btrfs-autosnap.hook`

Creates read-only snapshots before pacman package operations (install/upgrade/remove).

**Installation:**
```bash
sudo cp packaging/arch/hooks/90-btrfs-autosnap.hook /etc/pacman.d/hooks/
sudo cp scripts/btrfs/autosnap-pre.sh /usr/local/bin/
sudo chmod +x /usr/local/bin/autosnap-pre.sh
```

**Behavior:**
- Triggers on **all** package operations
- Creates snapshots in `/.snapshots/pacman-TIMESTAMP`
- Snapshots are **read-only** for safety
- Automatically prunes old snapshots (keeps last 10)
- Silent during pacman operations, verbose when run manually
- Respects `NO_COLOR` environment variable

**Configuration:** Edit `/usr/local/bin/autosnap-pre.sh`
```bash
SNAPSHOT_ROOT="/.snapshots"       # Where to store snapshots
MAX_SNAPSHOTS=10                  # Auto-prune threshold
```

### 3. Snapshot Pruning

**Script:** `scripts/btrfs/prune.sh`

Cleans up old snapshots based on retention policy (count + age).

**Usage:**
```bash
# Dry-run (preview what would be deleted)
./scripts/btrfs/prune.sh --dry-run

# Keep last 5 snapshots
./scripts/btrfs/prune.sh --keep 5

# Delete snapshots older than 7 days
./scripts/btrfs/prune.sh --days 7

# Both policies (keeps last 10 AND only if <30 days old)
./scripts/btrfs/prune.sh --keep 10 --days 30

# Prune specific prefix
./scripts/btrfs/prune.sh --prefix "manual-*" --keep 3

# Prune all snapshots
./scripts/btrfs/prune.sh --all --keep 5
```

**Features:**
- Dual retention policy (count-based + age-based)
- Filter by snapshot name prefix (default: `pacman-*`)
- Dry-run mode to preview deletions
- TUI output with colors and progress
- Safe: only deletes read-only snapshots via `btrfs subvolume delete`

### 4. Bootloader Integration

#### GRUB (Recommended)

Install `grub-btrfs` for automatic snapshot menu generation:
```bash
sudo pacman -S grub-btrfs
sudo systemctl enable --now grub-btrfsd.service
sudo grub-mkconfig -o /boot/grub/grub.cfg
```

Snapshots will appear in GRUB menu under "Arch Linux snapshots".

#### systemd-boot

Use Anna's generator script to create boot entries:

**Script:** `scripts/btrfs/sdboot-gen.sh`

```bash
# Dry-run
./scripts/btrfs/sdboot-gen.sh --dry-run

# Generate entries for last 5 snapshots
./scripts/btrfs/sdboot-gen.sh --limit 5

# Clean old entries and regenerate
./scripts/btrfs/sdboot-gen.sh --clean --limit 3
```

**Features:**
- Generates `.conf` entries in `/boot/loader/entries/`
- Uses `rootflags=subvolid=X` to boot into snapshot
- Automatically detects kernel and initramfs from existing entry
- Only creates entries for read-only snapshots
- Safe: does not modify existing boot entries

**Generated entry example:**
```
title   Arch Linux (snapshot 2025-11-02_153045)
linux   /vmlinuz-linux
initrd  /initramfs-linux.img
options rootflags=subvolid=1234 root=UUID=xxx rw quiet
```

### 5. Advisor Rules

Anna's advisor includes 10 Btrfs-specific rules (see [ADVISOR-ARCH.md](./ADVISOR-ARCH.md#storage-rules-btrfs)):

1. **btrfs-layout-missing-snapshots** - No snapshot tool configured
2. **pacman-autosnap-missing** - No automatic snapshot hooks
3. **grub-btrfs-missing-on-grub** - GRUB without snapshot menu
4. **sd-boot-snapshots-missing** - systemd-boot without snapshot entries
5. **scrub-overdue** - Scrub not run in >30 days
6. **low-free-space** - <10% free space
7. **compression-suboptimal** - Not using zstd compression
8. **qgroups-disabled** - Cannot track snapshot space usage
9. **copy-on-write-exceptions** - Info about CoW for databases/VMs
10. **balance-required** - Metadata chunks full

**View recommendations:**
```bash
annactl advisor arch
annactl advisor arch --json | jq '.[] | select(.category == "storage")'
annactl advisor arch --explain scrub-overdue
```

## Architecture

### Data Collection

**Module:** `src/annad/src/storage_btrfs.rs` - `BtrfsCollector`

Collects data in parallel with 2-second per-tool timeout:
- Filesystem detection (`findmnt`, `stat`)
- Subvolume layout (`btrfs subvolume list`)
- Mount options (`/proc/mounts`)
- Tool detection (`which snapper`, package checks)
- Health metrics (`btrfs filesystem df`, `btrfs scrub status`)
- Bootloader detection (`efibootmgr`, `/boot/grub/grub.cfg`)

**Privacy:** All data collected locally, no network calls.

### RPC Endpoint

**Method:** `storage_profile`
**Returns:** `BtrfsProfile` JSON

Used by `annactl storage btrfs show` command.

### Advisory Engine

**Module:** `src/annad/src/advisor_v13.rs` - `ArchAdvisor::run()`

Storage rules only execute when `BtrfsProfile.detected == true`.

## Best Practices

### Recommended Layout

```
/ (subvol @)
├── @home → /home
├── @var → /var
├── @snapshots → /.snapshots
│   ├── pacman-2025-11-02_120000 (ro)
│   ├── pacman-2025-11-01_180000 (ro)
│   └── ...
```

### Mount Options (Optimal)

```bash
# /etc/fstab
UUID=xxx  /  btrfs  rw,noatime,compress=zstd:3,space_cache=v2,subvol=@ 0 0
```

- **compress=zstd:3** - Best balance of ratio and speed
- **noatime** - Reduce write overhead
- **space_cache=v2** - Faster free space tracking
- **subvol=@** - Explicit subvolume mount

### Snapshot Retention

**Conservative:** Keep 10 pacman snapshots + manual snapshots indefinitely
**Moderate:** Keep 5 pacman snapshots, prune >7 days old
**Aggressive:** Keep 3 pacman snapshots, prune >3 days old

Set up cron job for automatic pruning:
```bash
# /etc/cron.weekly/btrfs-prune
#!/bin/sh
/usr/local/bin/btrfs-prune.sh --keep 5 --days 7
```

### Monthly Scrub

Enable system timer:
```bash
sudo systemctl enable --now btrfs-scrub@-.timer
```

Or manual cron:
```bash
# /etc/cron.monthly/btrfs-scrub
#!/bin/sh
btrfs scrub start -Bd /
```

## Troubleshooting

### Autosnap Hook Not Triggering

**Check hook installation:**
```bash
ls -lh /etc/pacman.d/hooks/90-btrfs-autosnap.hook
ls -lh /usr/local/bin/autosnap-pre.sh
```

**Test manually:**
```bash
sudo /usr/local/bin/autosnap-pre.sh
ls -lh /.snapshots/
```

**Check pacman hook syntax:**
```bash
# Hooks must have [Trigger] and [Action] sections
cat /etc/pacman.d/hooks/90-btrfs-autosnap.hook
```

### Snapshots Not Visible in GRUB

**Verify grub-btrfs installed:**
```bash
pacman -Q grub-btrfs
```

**Check service:**
```bash
sudo systemctl status grub-btrfsd
```

**Regenerate config:**
```bash
sudo grub-mkconfig -o /boot/grub/grub.cfg
```

### systemd-boot Entry Generation Fails

**Check prerequisites:**
```bash
# Must have systemd-boot
ls /boot/loader/entries/

# Must have template entry
ls /boot/loader/entries/arch*.conf

# Snapshots must be read-only
btrfs property get /.snapshots/pacman-* ro
```

**Test dry-run:**
```bash
./scripts/btrfs/sdboot-gen.sh --dry-run
```

### Low Free Space Despite Deleting Files

Btrfs allocates space in chunks. Free space shows "available for new chunks" not "unused in existing chunks".

**Solutions:**
1. Delete old snapshots
2. Run balance to reclaim chunks: `sudo btrfs balance start -dusage=50 /`
3. Clear package cache: `sudo pacman -Sc`

### Cannot Boot into Snapshot (systemd-boot)

**Verify entry syntax:**
```bash
cat /boot/loader/entries/anna-snapshot-*.conf
```

Must use `rootflags=subvolid=XXX`, not `rootflags=subvol=/path`.

**Verify subvolume ID:**
```bash
sudo btrfs subvolume show /.snapshots/pacman-TIMESTAMP | grep "Subvolume ID"
```

## References

- [Arch Wiki: Btrfs](https://wiki.archlinux.org/title/Btrfs)
- [Arch Wiki: Snapper](https://wiki.archlinux.org/title/Snapper)
- [Arch Wiki: systemd-boot](https://wiki.archlinux.org/title/Systemd-boot)
- [GitHub: grub-btrfs](https://github.com/Antynea/grub-btrfs)
- [GitHub: snap-pac](https://github.com/wesbarnett/snap-pac)
- [Btrfs Wiki](https://btrfs.wiki.kernel.org/)

## Future Enhancements

- Automatic balance scheduling based on usage patterns
- Snapshot diff visualization
- Space usage breakdown by subvolume (with qgroups)
- Integration with backup tools (btrbk, snapper-sync)
- Automated scrub error reporting
- Snapshot tagging and descriptions
