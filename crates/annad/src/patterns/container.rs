//! Container and virtualization patterns for Docker, Podman, VMs.
//! v0.0.957: Initial implementation.
//! v0.0.989: Added container logs, inspect, prune, kubernetes, swarm

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
        .or_else(|| match_container_advanced(q))
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

/// Advanced container patterns (logs, inspect, health, kubernetes, swarm)
fn match_container_advanced(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[ContainerPattern] = &[
        // Container logs
        (&["container", "logs"], "view container logs", "docker",
         &["echo 'docker logs <container>'", "echo 'docker logs -f <container> (follow)'"]),
        (&["docker", "logs"], "view Docker container logs", "docker",
         &["echo 'docker logs <container_id>'"]),
        // Container stats
        (&["container", "stats"], "show container statistics", "docker",
         &["docker stats --no-stream"]),
        (&["container", "resource"], "show container resource usage", "docker",
         &["docker stats --no-stream", "docker system df"]),
        // Running containers
        (&["running", "containers"], "list running containers", "docker",
         &["docker ps", "podman ps"]),
        // Container ports
        (&["container", "ports"], "show container port mappings", "docker",
         &["docker ps --format 'table {{.Names}}\\t{{.Ports}}'",
           "docker port <container>"]),
        // Docker inspect
        (&["docker", "inspect"], "inspect container details", "docker",
         &["echo 'docker inspect <container>'",
           "echo 'docker inspect --format=\"{{.State.Status}}\" <container>'"]),
        // Container environment
        (&["container", "environment"], "show container environment vars", "docker",
         &["echo 'docker inspect --format=\"{{.Config.Env}}\" <container>'"]),
        // Container filesystem
        (&["container", "filesystem"], "explore container filesystem", "docker",
         &["echo 'docker exec -it <container> sh'",
           "echo 'docker cp <container>:/path /local/path'"]),
        // Container health
        (&["container", "health"], "check container health", "docker",
         &["docker inspect --format='{{.State.Health.Status}}' <container>",
           "docker ps --filter health=unhealthy"]),
        // Docker prune
        (&["docker", "prune"], "docker prune cleanup info", "docker",
         &["echo 'Cleanup: docker system prune'",
           "echo 'More aggressive: docker system prune -a --volumes'",
           "docker system df"]),
        // Docker registry
        (&["docker", "registry"], "docker registry info", "docker",
         &["docker info | grep Registry", "echo 'Login: docker login <registry>'"]),
        // Docker build cache
        (&["docker", "build", "cache"], "show docker build cache", "docker",
         &["docker builder prune -a --force",
           "docker system df"]),
        // Container runtime
        (&["container", "runtime"], "show container runtime info", "docker",
         &["docker info | grep -i runtime", "containerd --version 2>/dev/null"]),
        // Docker swarm
        (&["docker", "swarm"], "show Docker Swarm status", "docker",
         &["docker info | grep Swarm", "docker node ls 2>/dev/null || echo 'Not in swarm mode'"]),
        (&["swarm", "status"], "Docker Swarm status", "docker",
         &["docker node ls 2>/dev/null || echo 'Swarm not initialized'"]),
        // Kubernetes
        (&["kubernetes", "pods"], "list Kubernetes pods", "kubernetes",
         &["kubectl get pods", "kubectl get pods -A"]),
        (&["kubectl", "pods"], "show kubectl pods", "kubernetes",
         &["kubectl get pods -A"]),
        (&["k8s", "pods"], "show K8s pods", "kubernetes",
         &["kubectl get pods"]),
        (&["kubernetes", "status"], "show Kubernetes cluster status", "kubernetes",
         &["kubectl cluster-info", "kubectl get nodes"]),
        // List containers (generic)
        (&["list", "containers"], "list all containers", "docker",
         &["docker ps -a 2>/dev/null || podman ps -a"]),
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
