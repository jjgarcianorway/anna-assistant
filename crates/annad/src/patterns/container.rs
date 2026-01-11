//! Container and virtualization patterns for Docker, Podman, VMs.
//! v0.0.957: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a container-related DeepUnderstanding
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

type ContainerPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match container-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_docker(q)
        .or_else(|| match_podman(q))
        .or_else(|| match_vms(q))
        .or_else(|| match_compose(q))
}

/// Docker patterns
fn match_docker(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ContainerPattern] = &[
        // Container listing
        (&["docker", "containers"], "list Docker containers", "docker",
         &["docker ps -a"]),
        (&["docker", "running"], "show running Docker containers", "docker",
         &["docker ps"]),
        (&["docker", "stopped"], "show stopped Docker containers", "docker",
         &["docker ps -a --filter 'status=exited'"]),
        // Images
        (&["docker", "images"], "list Docker images", "docker",
         &["docker images"]),
        (&["docker", "image", "size"], "show Docker image sizes", "docker",
         &["docker images --format 'table {{.Repository}}\\t{{.Tag}}\\t{{.Size}}'"]),
        (&["docker", "dangling"], "show dangling Docker images", "docker",
         &["docker images -f 'dangling=true'"]),
        // Resources
        (&["docker", "disk"], "show Docker disk usage", "docker",
         &["docker system df"]),
        (&["docker", "space"], "show Docker space usage", "docker",
         &["docker system df -v"]),
        (&["docker", "stats"], "show Docker container stats", "docker",
         &["docker stats --no-stream"]),
        // Networks
        (&["docker", "networks"], "list Docker networks", "docker",
         &["docker network ls"]),
        // Volumes
        (&["docker", "volumes"], "list Docker volumes", "docker",
         &["docker volume ls"]),
        (&["docker", "volume", "unused"], "show unused Docker volumes", "docker",
         &["docker volume ls -f 'dangling=true'"]),
        // Version and info
        (&["docker", "version"], "show Docker version", "docker",
         &["docker --version", "docker version"]),
        (&["docker", "info"], "show Docker system info", "docker",
         &["docker info"]),
        // Status
        (&["docker", "service"], "show Docker service status", "docker",
         &["systemctl status docker"]),
        (&["docker", "status"], "show Docker daemon status", "docker",
         &["systemctl status docker"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Podman patterns
fn match_podman(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ContainerPattern] = &[
        // Container listing
        (&["podman", "containers"], "list Podman containers", "podman",
         &["podman ps -a"]),
        (&["podman", "running"], "show running Podman containers", "podman",
         &["podman ps"]),
        (&["podman", "pods"], "list Podman pods", "podman",
         &["podman pod ls"]),
        // Images
        (&["podman", "images"], "list Podman images", "podman",
         &["podman images"]),
        // Resources
        (&["podman", "disk"], "show Podman disk usage", "podman",
         &["podman system df"]),
        (&["podman", "stats"], "show Podman container stats", "podman",
         &["podman stats --no-stream"]),
        // Networks
        (&["podman", "networks"], "list Podman networks", "podman",
         &["podman network ls"]),
        // Volumes
        (&["podman", "volumes"], "list Podman volumes", "podman",
         &["podman volume ls"]),
        // Version
        (&["podman", "version"], "show Podman version", "podman",
         &["podman --version"]),
        (&["podman", "info"], "show Podman system info", "podman",
         &["podman info"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// VM and virtualization patterns
fn match_vms(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ContainerPattern] = &[
        // libvirt/virsh
        (&["virsh", "list"], "list VMs using virsh", "virtualization",
         &["virsh list --all"]),
        (&["vms", "running"], "show running VMs", "virtualization",
         &["virsh list --state-running", "VBoxManage list runningvms"]),
        (&["vms", "list"], "list all VMs", "virtualization",
         &["virsh list --all", "VBoxManage list vms"]),
        (&["virtual", "machines"], "list virtual machines", "virtualization",
         &["virsh list --all", "VBoxManage list vms"]),
        // libvirt status
        (&["libvirtd", "status"], "show libvirt daemon status", "virtualization",
         &["systemctl status libvirtd"]),
        (&["libvirt", "running"], "check if libvirt is running", "virtualization",
         &["systemctl is-active libvirtd"]),
        // QEMU
        (&["qemu", "running"], "show running QEMU instances", "virtualization",
         &["pgrep -a qemu"]),
        (&["qemu", "version"], "show QEMU version", "virtualization",
         &["qemu-system-x86_64 --version"]),
        // VirtualBox
        (&["virtualbox", "vms"], "list VirtualBox VMs", "virtualization",
         &["VBoxManage list vms"]),
        (&["vbox", "running"], "show running VirtualBox VMs", "virtualization",
         &["VBoxManage list runningvms"]),
        // VM resources
        (&["vm", "disk"], "show VM disk images", "virtualization",
         &["virsh vol-list default", "ls -lh /var/lib/libvirt/images/"]),
        (&["vm", "networks"], "show VM networks", "virtualization",
         &["virsh net-list --all"]),
        // KVM
        (&["kvm", "enabled"], "check if KVM is enabled", "virtualization",
         &["lsmod | grep kvm", "cat /sys/module/kvm_intel/parameters/nested 2>/dev/null || cat /sys/module/kvm_amd/parameters/nested 2>/dev/null"]),
        (&["virtualization", "support"], "check virtualization support", "virtualization",
         &["grep -E 'vmx|svm' /proc/cpuinfo | head -1"]),
        (&["vt-x", "check"], "check Intel VT-x support", "virtualization",
         &["grep vmx /proc/cpuinfo | head -1"]),
        (&["amd-v", "check"], "check AMD-V support", "virtualization",
         &["grep svm /proc/cpuinfo | head -1"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Docker Compose / Podman Compose patterns
fn match_compose(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ContainerPattern] = &[
        // Docker Compose
        (&["docker", "compose", "status"], "show Docker Compose status", "compose",
         &["docker compose ps"]),
        (&["docker", "compose", "running"], "show running Compose services", "compose",
         &["docker compose ps"]),
        (&["compose", "services"], "list Compose services", "compose",
         &["docker compose ps", "docker-compose ps"]),
        (&["compose", "config"], "check Compose config", "compose",
         &["docker compose config"]),
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
    fn test_docker() {
        assert!(match_patterns("docker containers").is_some());
        assert!(match_patterns("docker images").is_some());
        assert!(match_patterns("docker disk usage").is_some());
        assert!(match_patterns("docker version").is_some());
    }

    #[test]
    fn test_podman() {
        assert!(match_patterns("podman containers").is_some());
        assert!(match_patterns("podman images").is_some());
        assert!(match_patterns("podman pods").is_some());
    }

    #[test]
    fn test_vms() {
        assert!(match_patterns("list vms").is_some());
        assert!(match_patterns("running vms").is_some());
        assert!(match_patterns("virtualization support").is_some());
        assert!(match_patterns("kvm enabled").is_some());
    }

    #[test]
    fn test_compose() {
        assert!(match_patterns("docker compose status").is_some());
        assert!(match_patterns("compose services").is_some());
    }
}
