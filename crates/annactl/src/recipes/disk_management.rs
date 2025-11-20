// Beta.175: Disk Management and Partitioning Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct DiskManagementRecipe;

#[derive(Debug, PartialEq)]
enum DiskManagementOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl DiskManagementOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            DiskManagementOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            DiskManagementOperation::ListTools
        } else {
            DiskManagementOperation::Install
        }
    }
}

impl DiskManagementRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("gparted") || input_lower.contains("gnome disk")
            || input_lower.contains("kde partition") || input_lower.contains("partitionmanager")
            || input_lower.contains("disk manag") || input_lower.contains("partition")
            && (input_lower.contains("install") || input_lower.contains("setup")
                || input_lower.contains("tool") || input_lower.contains("editor"));
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = DiskManagementOperation::detect(user_input);
        match operation {
            DiskManagementOperation::Install => Self::build_install_plan(telemetry),
            DiskManagementOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            DiskManagementOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("gparted") { "gparted" }
        else if input_lower.contains("gnome") { "gnome-disk-utility" }
        else if input_lower.contains("kde") || input_lower.contains("partition") { "partitionmanager" }
        else { "gparted" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "gparted" => ("GParted", "gparted", "GTK partition editor for creating, resizing, moving, and copying disk partitions"),
            "gnome-disk-utility" => ("GNOME Disks", "gnome-disk-utility", "Disk management utility for GNOME to view, modify, and configure disks"),
            "partitionmanager" => ("KDE Partition Manager", "partitionmanager", "KDE utility for managing disk devices, partitions, and file systems"),
            _ => ("GParted", "gparted", "Partition editor", ),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("disk_management.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = format!("sudo pacman -S --needed --noconfirm {}", package_name);

        let notes = format!(
            "{} installed. {}. Launch from app menu or run with sudo for full functionality. \
            ⚠️ WARNING: Disk partitioning can cause data loss if misused. Always backup important data before modifying partitions.",
            tool_name, description
        );

        Ok(ActionPlan {
            analysis: format!("Installing {} disk management tool", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool.replace("-", "_")),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
                    risk_level: RiskLevel::Medium,
                    rollback_id: Some(format!("remove-{}", tool.replace("-", "_"))),
                    requires_confirmation: true,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool.replace("-", "_")),
                    description: format!("Remove {}", tool_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("disk_management_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("disk_management.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking installed disk management tools".to_string(),
            goals: vec!["List installed disk utilities".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-disk-tools".to_string(),
                    description: "List disk management tools".to_string(),
                    command: "pacman -Q gparted gnome-disk-utility partitionmanager 2>/dev/null || echo 'No disk management tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed disk management and partitioning software".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("disk_management_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("disk_management.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available disk management tools".to_string(),
            goals: vec!["List available disk management software for Arch Linux".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available disk tools".to_string(),
                    command: r#"echo 'Disk Management and Partitioning Tools for Arch Linux:

GUI Partition Editors:
- GParted (official) - GTK-based partition editor, industry standard for disk management
- KDE Partition Manager (official) - KDE integrated partition manager with modern interface
- GNOME Disks (gnome-disk-utility) (official) - Simple disk management for GNOME desktop

Command-Line Tools:
- parted (official) - GNU partition manipulation program, text-based interactive
- fdisk (util-linux) (official) - Classic partition table manipulator for MBR
- gdisk (official) - GPT fdisk for GUID partition tables
- cfdisk (util-linux) (official) - Curses-based disk partition table manipulator

File System Tools:
- e2fsprogs (official) - ext2/ext3/ext4 file system utilities
- btrfs-progs (official) - Btrfs file system utilities
- xfsprogs (official) - XFS file system utilities
- dosfstools (official) - FAT/FAT32 file system utilities
- ntfs-3g (official) - NTFS file system driver with read/write support
- exfatprogs (official) - exFAT file system utilities

Disk Utilities:
- smartmontools (official) - S.M.A.R.T. monitoring tools for disk health
- hdparm (official) - Hard disk parameters utility
- sdparm (official) - SCSI device parameters utility
- nvme-cli (official) - NVMe management command-line interface

LVM Management:
- lvm2 (official) - Logical Volume Manager tools
- system-config-lvm (AUR) - GUI for LVM management

RAID Management:
- mdadm (official) - Tool for managing software RAID arrays
- dmraid (official) - Device-mapper RAID tool

Disk Imaging:
- dd (coreutils) (official) - Low-level disk copy and conversion
- ddrescue (official) - Data recovery tool for damaged disks
- Clonezilla (AUR) - Partition and disk imaging/cloning solution
- Partclone (official) - Partition cloning utility

Disk Encryption:
- cryptsetup (official) - LUKS encryption setup utility
- VeraCrypt (official) - Disk encryption software
- GNOME Disks (official) - Supports LUKS encryption in GUI

Disk Analysis:
- baobab (official) - GNOME disk usage analyzer (visual tree map)
- filelight (official) - KDE disk usage statistics (sunburst chart)
- ncdu (official) - NCurses disk usage analyzer
- duc (AUR) - High-performance disk usage analyzer

Benchmarking:
- fio (official) - Flexible I/O tester and benchmark
- hdparm (official) - Includes disk read benchmarking
- bonnie++ (official) - File system and disk benchmark
- iozone (AUR) - File system benchmark tool

Disk Wiping:
- shred (coreutils) (official) - Securely delete files and wipe disks
- wipe (official) - Secure file deletion utility
- nwipe (official) - Disk erasing tool (DBAN alternative)

Comparison by Use Case:

General Partition Management:
- GParted: Best all-around GUI, works on any desktop environment
- KDE Partition Manager: Best for KDE users, native integration
- GNOME Disks: Best for simple tasks on GNOME desktop

Command-Line Partitioning:
- parted: Best for scripting and automation
- gdisk: Best for GPT partition tables
- fdisk: Best for MBR partition tables (legacy)
- cfdisk: Best for interactive TUI partitioning

Disk Cloning/Backup:
- Clonezilla: Best for full disk/partition cloning
- dd: Best for low-level disk copying
- ddrescue: Best for recovering data from failing disks

Encryption:
- cryptsetup: Best for LUKS encryption setup
- VeraCrypt: Best for cross-platform encrypted volumes
- GNOME Disks: Best for simple LUKS encryption in GUI

Key Features:

GParted:
- Create, delete, resize, move, copy partitions
- Support for 20+ file systems (ext2/3/4, NTFS, FAT, XFS, Btrfs, etc.)
- Non-destructive partition operations
- Attempt data rescue from lost partitions
- Live USB available for offline operations

GNOME Disks:
- Simple, intuitive interface
- Format and partition drives
- Mount/unmount file systems
- S.M.A.R.T. data monitoring
- Create disk images
- Benchmark disk performance
- LUKS encryption support

KDE Partition Manager:
- Modern Qt interface
- Create, resize, delete, move partitions
- Backup and restore partition tables
- Support for LVM and RAID
- File system support similar to GParted

parted:
- Scriptable partition management
- Create and delete partitions
- Resize partitions (with file system support)
- Set partition flags
- GPT and MBR support

File System Support:

GParted Supported File Systems:
- ext2, ext3, ext4 (Linux native)
- btrfs (Linux CoW file system)
- XFS (High-performance Linux FS)
- NTFS (Windows)
- FAT16, FAT32, exFAT (Universal)
- HFS, HFS+ (macOS)
- ReiserFS, JFS, UFS, and more

Format/Create:
- Use mkfs.ext4, mkfs.btrfs, mkfs.xfs for Linux
- Use mkfs.ntfs or mkfs.fat for Windows compatibility
- Use mkfs.exfat for large removable drives

Mounting Options:
- Auto-mount: Use /etc/fstab entries
- Manual mount: Use mount command or file manager
- GNOME Disks: GUI mounting with options

Common Workflows:

Creating New Partition:
1. Launch GParted or KDE Partition Manager
2. Select target disk from dropdown
3. Unmount all partitions on disk
4. Create new partition table (GPT recommended)
5. Create partition with desired size
6. Apply changes
7. Format partition with file system

Resizing Partition:
1. Backup important data first
2. Unmount the partition to resize
3. Use GParted or parted to resize
4. For ext4: GParted handles file system resize
5. For NTFS: Use ntfsresize or GParted
6. Apply changes and verify

Disk Cloning:
1. Boot from Clonezilla Live USB
2. Select source and destination disks
3. Choose partition or disk clone mode
4. Optionally resize partitions during clone
5. Verify clone after completion

Alternatively with dd:
```bash
sudo dd if=/dev/sdX of=/dev/sdY bs=4M status=progress
```

LUKS Encryption Setup:
1. Use GNOME Disks "Format Partition" with encryption
2. Or command-line: cryptsetup luksFormat /dev/sdX
3. Open encrypted volume: cryptsetup open /dev/sdX myvolume
4. Format: mkfs.ext4 /dev/mapper/myvolume
5. Mount: mount /dev/mapper/myvolume /mnt

S.M.A.R.T. Monitoring:
```bash
sudo smartctl -a /dev/sdX  # Show all S.M.A.R.T. info
sudo smartctl -t short /dev/sdX  # Run short self-test
sudo smartctl -H /dev/sdX  # Health status
```

Or use GNOME Disks GUI for S.M.A.R.T. data visualization.

Safety Tips:

⚠️ CRITICAL WARNINGS:
- Always backup important data before partitioning
- Verify device names carefully (sdX vs sdY can destroy data)
- Unmount partitions before modifying them
- Don\'t interrupt operations in progress
- Test encryption/cloning procedures on non-critical data first

Best Practices:
- Use GPT partition tables for new disks (UEFI compatible)
- Align partitions to 1 MiB boundaries for SSD performance
- Leave some unallocated space on SSDs (over-provisioning)
- Use labels and UUIDs in /etc/fstab, not device names
- Regular S.M.A.R.T. monitoring for early failure detection

Troubleshooting:

GParted Won'\''t Resize NTFS:
- Boot Windows, disable hibernation and fast startup
- Run chkdsk /f from Windows to fix file system errors
- Defragment NTFS partition in Windows

Partition Won'\''t Unmount:
- Check with lsof or fuser what'\''s using it
- Close applications accessing the partition
- Kill processes if necessary: fuser -km /mount/point

GPT/MBR Confusion:
- Use gdisk for GPT partition tables
- Use fdisk for MBR partition tables
- Convert MBR to GPT: gdisk /dev/sdX (expert menu)

Accidentally Deleted Partition Table:
- DON'\''T write new partition table
- Use testdisk (official repo) to recover
- Or PhotoRec for file recovery

Performance Optimization:

SSD Optimization:
- Enable TRIM: fstrim -v / or fstrim.timer systemd service
- Use discard mount option in /etc/fstab
- Align partitions to 1 MiB boundaries
- Don'\''t fill SSD completely (leave 10-20% free)

HDD Optimization:
- Use ext4 with noatime mount option
- Enable write barriers for data integrity
- Consider using XFS for large files
- Defragmentation not needed for Linux file systems (except NTFS)

Benchmark Commands:
```bash
# Read speed test
sudo hdparm -tT /dev/sdX

# Comprehensive I/O test
fio --name=randwrite --ioengine=libaio --iodepth=16 --rw=randwrite --bs=4k --direct=1 --size=1G --numjobs=4 --runtime=60 --group_reporting
```

File System Recommendations:

General Purpose Linux:
- ext4: Most stable, widely supported, excellent for desktops
- btrfs: Advanced features (snapshots, compression), good for servers
- XFS: High performance for large files, used in enterprise

External Drives:
- exFAT: Best for removable drives > 32GB (Windows/Mac/Linux)
- FAT32: Best for drives ≤ 32GB, maximum compatibility
- NTFS: Use if primarily with Windows systems

Special Use Cases:
- Btrfs: Snapshots, RAID, compression (NAS, backups)
- ZFS (via zfs-linux AUR): Enterprise features, data integrity
- F2FS: Flash-friendly file system for SD cards and eMMC

LVM (Logical Volume Manager):

Benefits:
- Resize volumes on the fly without unmounting
- Span volumes across multiple disks
- Create snapshots for backups
- More flexible than traditional partitions

Basic LVM Setup:
1. Create physical volume: pvcreate /dev/sdX
2. Create volume group: vgcreate myvg /dev/sdX
3. Create logical volume: lvcreate -L 10G -n mylv myvg
4. Format: mkfs.ext4 /dev/myvg/mylv
5. Mount: mount /dev/myvg/mylv /mnt

RAID Configuration:

Software RAID with mdadm:
- RAID 0: Striping (performance, no redundancy)
- RAID 1: Mirroring (redundancy, 50% capacity)
- RAID 5: Striping with parity (balance, needs 3+ disks)
- RAID 6: Double parity (better redundancy, needs 4+ disks)
- RAID 10: Mirrored stripes (performance + redundancy)

Create RAID 1:
```bash
mdadm --create /dev/md0 --level=1 --raid-devices=2 /dev/sdX /dev/sdY
```

System Requirements:

GParted:
- Minimal CPU/RAM requirements
- Requires X11 (won'\''t work on Wayland-only)
- Need appropriate file system tools installed

GNOME Disks:
- GNOME desktop environment (optional)
- Works on Wayland and X11
- Minimal resource usage

KDE Partition Manager:
- KDE Plasma recommended but not required
- Qt libraries needed
- Similar resource usage to GParted

Documentation and Help:

GParted:
- Official documentation: gparted.org/documentation.php
- Man page: man gparted
- Forum: gparted.org/forum

parted:
- Man page: man parted
- GNU Parted manual: gnu.org/software/parted/manual
- Arch Wiki: wiki.archlinux.org/title/Parted

File Systems:
- Arch Wiki has excellent guides for each FS
- wiki.archlinux.org/title/File_systems'"#.to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Disk management and partitioning tools for Arch Linux - Use with extreme caution".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("disk_management_list_tools".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        assert!(DiskManagementRecipe::matches_request("install gparted"));
        assert!(DiskManagementRecipe::matches_request("install gnome disk utility"));
        assert!(DiskManagementRecipe::matches_request("setup kde partition manager"));
        assert!(DiskManagementRecipe::matches_request("install disk management tool"));
        assert!(DiskManagementRecipe::matches_request("install partition editor"));
        assert!(!DiskManagementRecipe::matches_request("what is gparted"));
    }

    #[test]
    fn test_install_plan_gparted() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install gparted".to_string());
        let plan = DiskManagementRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
        assert!(plan.notes_for_user.contains("GParted"));
        assert!(plan.command_plan[0].risk_level == RiskLevel::Medium);
        assert!(plan.command_plan[0].requires_confirmation);
    }

    #[test]
    fn test_install_plan_gnome_disks() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install gnome disk utility".to_string());
        let plan = DiskManagementRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
        assert!(plan.notes_for_user.contains("GNOME Disks"));
    }

    #[test]
    fn test_detect_tool() {
        assert_eq!(DiskManagementRecipe::detect_tool("install gparted"), "gparted");
        assert_eq!(DiskManagementRecipe::detect_tool("setup gnome disk"), "gnome-disk-utility");
        assert_eq!(DiskManagementRecipe::detect_tool("install kde partition"), "partitionmanager");
    }
}
