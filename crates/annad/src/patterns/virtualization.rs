//! Virtualization patterns for KVM, QEMU, libvirt.
//! v0.0.974: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a virtualization-related DeepUnderstanding
fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::Factual,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

type VirtPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match virtualization-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_kvm(q)
        .or_else(|| match_libvirt(q))
        .or_else(|| match_qemu(q))
        .or_else(|| match_vms(q))
        .or_else(|| match_virt_troubleshoot(q))
}

/// KVM patterns
fn match_kvm(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[VirtPattern] = &[
        // KVM support
        (&["kvm", "support"], "check KVM support", "virtualization",
         &["grep -E 'vmx|svm' /proc/cpuinfo | head -1", "lscpu | grep Virtualization"]),
        (&["kvm", "enabled"], "check if KVM is enabled", "virtualization",
         &["lsmod | grep kvm", "cat /sys/module/kvm*/parameters/nested 2>/dev/null"]),
        (&["kvm", "installed"], "check if KVM is installed", "virtualization",
         &["which kvm qemu-system-x86_64 2>/dev/null", "lsmod | grep kvm"]),
        // KVM module
        (&["kvm", "module"], "show KVM kernel modules", "virtualization",
         &["lsmod | grep -E 'kvm|vhost'"]),
        (&["kvm", "status"], "show KVM status", "virtualization",
         &["lsmod | grep kvm", "systemctl status libvirtd 2>/dev/null"]),
        // Nested virtualization
        (&["nested", "virtualization"], "check nested virtualization", "virtualization",
         &["cat /sys/module/kvm_intel/parameters/nested 2>/dev/null || cat /sys/module/kvm_amd/parameters/nested 2>/dev/null"]),
        // CPU virtualization
        (&["cpu", "virtualization"], "check CPU virtualization support", "virtualization",
         &["lscpu | grep -i virtualization", "grep -E 'vmx|svm' /proc/cpuinfo | head -1"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Libvirt patterns
fn match_libvirt(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[VirtPattern] = &[
        // Libvirt service
        (&["libvirt", "status"], "show libvirt status", "virtualization",
         &["systemctl status libvirtd"]),
        (&["libvirtd", "status"], "show libvirtd status", "virtualization",
         &["systemctl status libvirtd"]),
        (&["libvirt", "running"], "check if libvirt is running", "virtualization",
         &["systemctl is-active libvirtd"]),
        // Libvirt version
        (&["libvirt", "version"], "show libvirt version", "virtualization",
         &["virsh version", "libvirtd --version"]),
        // Libvirt networks
        (&["libvirt", "networks"], "list libvirt networks", "virtualization",
         &["virsh net-list --all"]),
        (&["virtual", "networks"], "list virtual networks", "virtualization",
         &["virsh net-list --all"]),
        // Libvirt pools
        (&["libvirt", "pools"], "list libvirt storage pools", "virtualization",
         &["virsh pool-list --all"]),
        (&["storage", "pools"], "list storage pools", "virtualization",
         &["virsh pool-list --all"]),
        // Libvirt default network
        (&["default", "network"], "show default network", "virtualization",
         &["virsh net-info default 2>/dev/null", "virsh net-list"]),
        // Virsh
        (&["virsh", "version"], "show virsh version", "virtualization",
         &["virsh version"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// QEMU patterns
fn match_qemu(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[VirtPattern] = &[
        // QEMU version
        (&["qemu", "version"], "show QEMU version", "virtualization",
         &["qemu-system-x86_64 --version 2>/dev/null | head -1"]),
        (&["qemu", "installed"], "check if QEMU is installed", "virtualization",
         &["which qemu-system-x86_64 qemu-img 2>/dev/null"]),
        // QEMU images
        (&["qemu", "images"], "list QEMU images", "virtualization",
         &["ls -la /var/lib/libvirt/images/ 2>/dev/null"]),
        // QEMU-img
        (&["qemu", "img"], "show qemu-img info", "virtualization",
         &["qemu-img --version", "echo 'Use: qemu-img info <image>'"]),
        // QEMU running
        (&["qemu", "running"], "show running QEMU processes", "virtualization",
         &["ps aux | grep qemu | grep -v grep"]),
        (&["qemu", "processes"], "show QEMU processes", "virtualization",
         &["ps aux | grep qemu | grep -v grep"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// VM management patterns
fn match_vms(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[VirtPattern] = &[
        // List VMs
        (&["list", "vms"], "list virtual machines", "virtualization",
         &["virsh list --all"]),
        (&["virtual", "machines"], "list virtual machines", "virtualization",
         &["virsh list --all"]),
        (&["running", "vms"], "show running VMs", "virtualization",
         &["virsh list"]),
        // VM info
        (&["vm", "info"], "show VM info", "virtualization",
         &["echo 'Use: virsh dominfo <vm-name>'"]),
        (&["vm", "details"], "show VM details", "virtualization",
         &["echo 'Use: virsh dominfo <vm-name>'", "virsh list --all"]),
        // VM state
        (&["vm", "state"], "show VM state", "virtualization",
         &["virsh list --all"]),
        // VM resources
        (&["vm", "resources"], "show VM resources", "virtualization",
         &["echo 'Use: virsh dominfo <vm-name> for CPU/memory'"]),
        // VM disks
        (&["vm", "disks"], "show VM disks", "virtualization",
         &["ls -la /var/lib/libvirt/images/", "virsh pool-list --all"]),
        // Virt-manager
        (&["virt", "manager"], "check virt-manager", "virtualization",
         &["which virt-manager 2>/dev/null && echo 'virt-manager installed' || echo 'virt-manager not installed'"]),
        // VM snapshots
        (&["vm", "snapshots"], "list VM snapshots", "virtualization",
         &["echo 'Use: virsh snapshot-list <vm-name>'"]),
        (&["snapshots"], "list VM snapshots", "virtualization",
         &["echo 'Use: virsh snapshot-list <vm-name>'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Virtualization troubleshooting patterns
fn match_virt_troubleshoot(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[VirtPattern] = &[
        // Virtualization logs
        (&["libvirt", "logs"], "show libvirt logs", "virtualization",
         &["journalctl -u libvirtd -n 30"]),
        (&["virtualization", "logs"], "show virtualization logs", "virtualization",
         &["journalctl -u libvirtd -n 30", "dmesg | grep -i kvm | tail -20"]),
        // IOMMU
        (&["iommu", "status"], "check IOMMU status", "virtualization",
         &["dmesg | grep -i iommu | head -10", "cat /proc/cmdline | grep -o 'iommu=[^ ]*'"]),
        (&["iommu", "groups"], "show IOMMU groups", "virtualization",
         &["find /sys/kernel/iommu_groups/ -type l 2>/dev/null | head -20"]),
        // Passthrough
        (&["gpu", "passthrough"], "check GPU passthrough", "virtualization",
         &["lspci -nnk | grep -A3 VGA", "dmesg | grep -i vfio | tail -10"]),
        (&["pci", "passthrough"], "check PCI passthrough", "virtualization",
         &["dmesg | grep -i vfio | tail -10", "lspci -nnk | head -30"]),
        // Virt capabilities
        (&["virt", "capabilities"], "show virtualization capabilities", "virtualization",
         &["virsh capabilities | head -50"]),
        // Libvirt errors
        (&["libvirt", "errors"], "show libvirt errors", "virtualization",
         &["journalctl -u libvirtd -p err -n 20"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kvm() {
        assert!(match_patterns("kvm support").is_some());
        assert!(match_patterns("kvm enabled").is_some());
        assert!(match_patterns("kvm module").is_some());
    }

    #[test]
    fn test_libvirt() {
        assert!(match_patterns("libvirt status").is_some());
        assert!(match_patterns("libvirt networks").is_some());
        assert!(match_patterns("libvirt pools").is_some());
    }

    #[test]
    fn test_qemu() {
        assert!(match_patterns("qemu version").is_some());
        assert!(match_patterns("qemu running").is_some());
    }

    #[test]
    fn test_vms() {
        assert!(match_patterns("list vms").is_some());
        assert!(match_patterns("virtual machines").is_some());
        assert!(match_patterns("running vms").is_some());
    }

    #[test]
    fn test_virt_troubleshoot() {
        assert!(match_patterns("libvirt logs").is_some());
        assert!(match_patterns("iommu status").is_some());
        assert!(match_patterns("gpu passthrough").is_some());
    }
}
