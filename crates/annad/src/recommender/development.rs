//! Development recommendations

use super::{check_command_usage, is_package_installed};

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_git_enhancements() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if git is installed first
    let has_git = Command::new("which")
        .arg("git")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !has_git {
        return result;
    }

    // lazygit - terminal UI for git
    if !Command::new("which")
        .arg("lazygit")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-lazygit".to_string(),
            title: "Install lazygit - simple terminal UI for git".to_string(),
            reason: "lazygit is a beautiful terminal UI for git that makes staging, committing, branching, and merging super easy. Perfect if you prefer TUIs over typing git commands!".to_string(),
            action: "Install lazygit".to_string(),
            command: Some("sudo pacman -S --noconfirm lazygit".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Development Tools".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Git#Graphical_front-ends".to_string()],
            depends_on: vec!["git".to_string()],
            related_to: vec!["install-git-delta".to_string()],
            bundle: Some("git-tools".to_string()),
            satisfies: Vec::new(),
            popularity: 80,
            requires: Vec::new(),
        });
    }

    // git-delta - better git diff
    if !Command::new("which")
        .arg("delta")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-git-delta".to_string(),
            title: "Install git-delta - beautiful git diffs with syntax highlighting".to_string(),
            reason: "git-delta makes git diffs beautiful with syntax highlighting, line numbers, and side-by-side view. Much easier to read than default git diff!".to_string(),
            action: "Install git-delta".to_string(),
            command: Some("sudo pacman -S --noconfirm git-delta".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Development Tools".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Git#Diff_and_merge_tools".to_string()],
            depends_on: vec!["git".to_string()],
            related_to: vec!["install-lazygit".to_string()],
            bundle: Some("git-tools".to_string()),
            satisfies: Vec::new(),
            popularity: 75,
            requires: Vec::new(),
        });
    }

    result
}

pub(crate) fn check_docker_support(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if Docker is installed
    let has_docker = Command::new("pacman")
        .args(&["-Q", "docker"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if user has docker in command history (wants to use it)
    let uses_docker = facts
        .frequently_used_commands
        .iter()
        .any(|cmd| cmd.command.contains("docker"));

    // Check if user has container-related files
    let has_dockerfile = facts
        .common_file_types
        .iter()
        .any(|t| t.contains("docker") || t.contains("container"));

    if !has_docker && (uses_docker || has_dockerfile) {
        result.push(Advice {
            id: "docker-install".to_string(),
            title: "Install Docker for containerization".to_string(),
            reason: "You seem to be working with containers (Dockerfiles found or docker commands in history), but Docker isn't installed! Docker lets you run applications in isolated containers - perfect for development, testing, and deploying apps consistently across systems.".to_string(),
            action: "Install Docker".to_string(),
            command: Some("pacman -S --noconfirm docker".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Development Tools".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker".to_string()],
                        depends_on: Vec::new(),
                related_to: vec!["docker-compose-install".to_string(), "lazydocker-install".to_string()],
                bundle: Some("container-dev".to_string()),
            satisfies: Vec::new(),
                popularity: 75,
            requires: Vec::new(), // Docker is very popular for development
            });
    }

    if has_docker {
        // Check if Docker service is enabled
        let docker_enabled = Command::new("systemctl")
            .args(&["is-enabled", "docker"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !docker_enabled {
            result.push(Advice {
                id: "docker-enable-service".to_string(),
                title: "Enable Docker service".to_string(),
                reason: "You have Docker installed but the service isn't enabled! Docker needs its daemon running to work. Enabling it makes Docker start automatically on boot, so you don't have to start it manually every time.".to_string(),
                action: "Enable and start Docker service".to_string(),
                command: Some("systemctl enable --now docker".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker#Installation".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check if user is in docker group
        let current_user = std::env::var("SUDO_USER")
            .unwrap_or_else(|_| std::env::var("USER").unwrap_or_default());
        if !current_user.is_empty() {
            let in_docker_group = Command::new("groups")
                .arg(&current_user)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains("docker"))
                .unwrap_or(false);

            if !in_docker_group {
                result.push(Advice {
                    id: "docker-user-group".to_string(),
                    title: "Add your user to docker group".to_string(),
                    reason: format!("You're not in the 'docker' group, so you need to use 'sudo' for every Docker command! Adding yourself to the docker group lets you run Docker without sudo. Much more convenient for development! (You'll need to log out and back in for this to take effect)"),
                    action: format!("Add user '{}' to docker group", current_user),
                    command: Some(format!("usermod -aG docker {}", current_user)),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "Development Tools".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker#Installation".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }

        // Suggest Docker Compose
        let has_compose = Command::new("pacman")
            .args(&["-Q", "docker-compose"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_compose {
            result.push(Advice {
                id: "docker-compose".to_string(),
                title: "Install Docker Compose for multi-container apps".to_string(),
                reason: "You have Docker but not Docker Compose! Compose makes it easy to define and run multi-container applications with a simple YAML file. Instead of running multiple 'docker run' commands, you define everything in docker-compose.yml and start it all with one command. Essential for modern development!".to_string(),
                action: "Install Docker Compose".to_string(),
                command: Some("pacman -S --noconfirm docker-compose".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Docker#Docker_Compose".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("container-dev".to_string()),
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_virtualization_support(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if CPU supports virtualization
    let cpu_has_virt = facts.cpu_model.to_lowercase().contains("amd")
        || facts.cpu_model.to_lowercase().contains("intel");

    if !cpu_has_virt {
        return result; // No virtualization support
    }

    // Check if virtualization is enabled in BIOS
    let virt_enabled = std::path::Path::new("/dev/kvm").exists();

    if !virt_enabled {
        result.push(Advice {
            id: "virtualization-enable-bios".to_string(),
            title: "Enable virtualization in BIOS".to_string(),
            reason: "Your CPU supports virtualization (KVM), but /dev/kvm doesn't exist! This means virtualization is disabled in your BIOS/UEFI. You need to enable Intel VT-x (Intel) or AMD-V (AMD) in your BIOS settings to use virtual machines with hardware acceleration. Without it, VMs will be extremely slow!".to_string(),
            action: "Reboot and enable VT-x/AMD-V in BIOS".to_string(),
            command: Some("lscpu | grep -E 'Virtualization|VT-x|AMD-V'".to_string()), // Check CPU virtualization support
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/KVM#Checking_support".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        return result; // Don't suggest KVM tools if virtualization is disabled
    }

    // Check for QEMU
    let has_qemu = Command::new("pacman")
        .args(&["-Q", "qemu-full"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if user seems interested in virtualization
    let uses_virt = facts.frequently_used_commands.iter().any(|cmd| {
        cmd.command.contains("qemu") || cmd.command.contains("virt") || cmd.command.contains("kvm")
    });

    if !has_qemu && (uses_virt || virt_enabled) {
        result.push(Advice {
            id: "qemu-install".to_string(),
            title: "Install QEMU for virtual machines".to_string(),
            reason: "Your system supports hardware virtualization (KVM), but QEMU isn't installed! QEMU with KVM gives you near-native performance for running virtual machines. Perfect for testing different Linux distros, running Windows VMs, or development environments. With KVM, VMs run almost as fast as bare metal!".to_string(),
            action: "Install QEMU with full system emulation".to_string(),
            command: Some("pacman -S --noconfirm qemu-full".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/QEMU".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    if has_qemu {
        // Suggest virt-manager for GUI management
        let has_virt_manager = Command::new("pacman")
            .args(&["-Q", "virt-manager"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_virt_manager {
            result.push(Advice {
                id: "virt-manager".to_string(),
                title: "Install virt-manager for easy VM management".to_string(),
                reason: "You have QEMU but no graphical manager! virt-manager provides a beautiful GUI for creating and managing VMs. It's way easier than typing QEMU commands - just click to create VMs, attach ISOs, configure networks, etc. Think of it as VirtualBox but for KVM!".to_string(),
                action: "Install virt-manager".to_string(),
                command: Some("pacman -S --noconfirm virt-manager".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "System Configuration".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Virt-manager".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check if libvirt service is running
        let libvirt_running = Command::new("systemctl")
            .args(&["is-active", "libvirtd"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !libvirt_running {
            result.push(Advice {
                id: "libvirt-enable".to_string(),
                title: "Enable libvirt service for VM management".to_string(),
                reason: "You have QEMU installed but libvirtd service isn't running! Libvirt provides the management layer for VMs - it's what virt-manager and other tools use to control QEMU. Start it to manage your virtual machines properly.".to_string(),
                action: "Enable and start libvirtd".to_string(),
                command: Some("systemctl enable --now libvirtd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "System Configuration".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Libvirt#Installation".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_container_orchestration() -> Vec<Advice> {
    let mut result = Vec::new();

    let has_docker = is_package_installed("docker");

    // Check for Podman (Docker alternative)
    if !has_docker && !is_package_installed("podman") {
        result.push(
            Advice::new(
                "container-podman".to_string(),
                "Consider Podman as a Docker alternative".to_string(),
                "Podman is a daemonless container engine - it's Docker-compatible but doesn't need root! Same commands as Docker (alias docker=podman works!), but more secure (rootless by default), no daemon overhead, and generates Kubernetes YAML. Great for development and production!".to_string(),
                "Install Podman".to_string(),
                Some("sudo pacman -S --noconfirm podman".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/Podman".to_string(),
                ],
                "development".to_string(),
            )
            .with_popularity(60)
            .with_bundle("container-dev".to_string())
        );
    }

    // Check for lazydocker (Docker TUI)
    if has_docker && !is_package_installed("lazydocker") {
        result.push(
            Advice::new(
                "container-lazydocker".to_string(),
                "Install lazydocker for easy Docker management".to_string(),
                "lazydocker is a beautiful terminal UI for Docker! See all containers/images/volumes/networks at a glance, view logs, stats, inspect containers - all with keyboard shortcuts. Way better than memorizing Docker commands!".to_string(),
                "Install lazydocker".to_string(),
                Some("sudo pacman -S --noconfirm lazydocker".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://github.com/jesseduffield/lazydocker".to_string(),
                ],
                "development".to_string(),
            )
            .with_popularity(55)
            .with_bundle("container-dev".to_string())
        );
    }

    // Check for kubectl (Kubernetes CLI)
    if (has_docker || is_package_installed("podman")) && !is_package_installed("kubectl") {
        result.push(
            Advice::new(
                "container-kubectl".to_string(),
                "Install kubectl for Kubernetes management".to_string(),
                "kubectl is the Kubernetes command-line tool. Even if you're not running k8s clusters, it's useful for testing manifests, working with minikube, or managing cloud k8s. Essential for cloud-native development!".to_string(),
                "Install kubectl".to_string(),
                Some("sudo pacman -S --noconfirm kubectl".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/Kubectl".to_string(),
                ],
                "development".to_string(),
            )
            .with_popularity(50)
            .with_bundle("container-dev".to_string())
        );
    }

    // Check for k9s (Kubernetes TUI)
    if is_package_installed("kubectl") && !is_package_installed("k9s") {
        result.push(
            Advice::new(
                "container-k9s".to_string(),
                "Install k9s for Kubernetes cluster management".to_string(),
                "k9s is a powerful terminal UI for Kubernetes! Navigate pods/deployments/services with vim keys, view logs, shell into pods, port-forward - all without memorizing kubectl commands. Makes k8s actually enjoyable to use!".to_string(),
                "Install k9s".to_string(),
                Some("sudo pacman -S --noconfirm k9s".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://k9scli.io/".to_string(),
                ],
                "development".to_string(),
            )
            .with_popularity(45)
            .with_bundle("container-dev".to_string())
        );
    }

    // Check for dive (Docker image analyzer)
    if has_docker {
        result.push(
            Advice::new(
                "container-dive".to_string(),
                "Install dive to analyze Docker image layers".to_string(),
                "dive lets you explore Docker image layers to see what's wasting space! Find unnecessary files, optimize your Dockerfiles, reduce image sizes. Interactive TUI shows each layer's contents and file changes. Essential for optimizing containers!".to_string(),
                "Install dive".to_string(),
                Some("sudo pacman -S --noconfirm dive".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://github.com/wagoodman/dive".to_string(),
                ],
                "development".to_string(),
            )
            .with_popularity(50)
            .with_bundle("container-dev".to_string())
        );
    }

    result
}

pub(crate) fn check_golang_dev() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Go files
    let has_go_files = Path::new(&format!(
        "{}/.cache",
        std::env::var("HOME").unwrap_or_default()
    ))
    .exists()
        && Command::new("find")
            .args(&[
                &std::env::var("HOME").unwrap_or_default(),
                "-name",
                "*.go",
                "-type",
                "f",
            ])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

    let go_usage = check_command_usage(&["go"]);

    if has_go_files || go_usage > 5 {
        // Check for Go compiler
        let has_go = Command::new("pacman")
            .args(&["-Q", "go"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_go {
            result.push(Advice {
                id: "dev-go".to_string(),
                title: "Install Go compiler for Go development".to_string(),
                reason: format!("You have Go files or use 'go' commands ({} times)! Install the Go compiler to build and run Go programs. Go is fast, simple, and great for concurrent programming, web services, and system tools. 'go run main.go' and you're off!", go_usage),
                action: "Install Go".to_string(),
                command: Some("pacman -S --noconfirm go".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Go".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        } else {
            // Check for gopls (Go LSP)
            let has_gopls = Command::new("which")
                .arg("gopls")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_gopls {
                result.push(Advice {
                    id: "dev-gopls".to_string(),
                    title: "Install gopls for Go LSP support".to_string(),
                    reason: "You're developing in Go but missing gopls (Go Language Server)! It provides autocomplete, go-to-definition, refactoring, and error checking in your editor. Works with VSCode, Neovim, Emacs, any LSP-compatible editor. Makes Go development SO much better!".to_string(),
                    action: "Install gopls via go install".to_string(),
                    command: Some("go install golang.org/x/tools/gopls@latest".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "Development Tools".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Go#Language_Server".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }
    }

    result
}

pub(crate) fn check_java_dev() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Java files
    let has_java_files = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.java",
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let java_usage = check_command_usage(&["java", "javac", "mvn", "gradle"]);

    if has_java_files || java_usage > 5 {
        // Check for JDK
        let has_jdk = Command::new("pacman")
            .args(&["-Q", "jdk-openjdk"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_jdk {
            result.push(Advice {
                id: "dev-java-jdk".to_string(),
                title: "Install OpenJDK for Java development".to_string(),
                reason: format!("You have Java files or use Java commands ({} times)! OpenJDK is the open-source Java Development Kit - compile and run Java programs, build Android apps, develop enterprise software. 'javac Main.java && java Main' - Java everywhere!", java_usage),
                action: "Install OpenJDK".to_string(),
                command: Some("pacman -S --noconfirm jdk-openjdk".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Java".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for Maven
        let has_maven = Command::new("pacman")
            .args(&["-Q", "maven"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_maven {
            result.push(Advice {
                id: "dev-maven".to_string(),
                title: "Install Maven for Java project management".to_string(),
                reason: "Maven is the standard build tool for Java! It handles dependencies, builds projects, runs tests, packages JARs. Essential for any serious Java development. If you see a pom.xml file, you need Maven!".to_string(),
                action: "Install Maven".to_string(),
                command: Some("pacman -S --noconfirm maven".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Java#Maven".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_nodejs_dev() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for JavaScript/TypeScript files and package.json
    let has_package_json = Path::new(&format!(
        "{}/package.json",
        std::env::var("HOME").unwrap_or_default()
    ))
    .exists()
        || Command::new("find")
            .args(&[
                &std::env::var("HOME").unwrap_or_default(),
                "-name",
                "package.json",
                "-type",
                "f",
            ])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

    let node_usage = check_command_usage(&["node", "npm", "npx", "yarn"]);

    if has_package_json || node_usage > 5 {
        // Check for Node.js
        let has_nodejs = Command::new("pacman")
            .args(&["-Q", "nodejs"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_nodejs {
            result.push(Advice {
                id: "dev-nodejs".to_string(),
                title: "Install Node.js for JavaScript development".to_string(),
                reason: format!("You have Node.js projects or use node/npm commands ({} times)! Node.js runs JavaScript outside browsers - build web apps, CLIs, servers, desktop apps with Electron. Comes with npm for package management. JavaScript everywhere!", node_usage),
                action: "Install Node.js and npm".to_string(),
                command: Some("pacman -S --noconfirm nodejs npm".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Node.js".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("nodejs-dev".to_string()),
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for TypeScript
        let has_typescript = Command::new("npm")
            .args(&["list", "-g", "typescript"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_typescript {
            result.push(Advice {
                id: "dev-typescript".to_string(),
                title: "Install TypeScript for type-safe JavaScript".to_string(),
                reason: "TypeScript adds types to JavaScript - catch bugs before runtime, better IDE support, clearer code. Used by major frameworks (Angular, Vue 3, NestJS). If you're doing serious JavaScript development, TypeScript makes everything better!".to_string(),
                action: "Install TypeScript globally".to_string(),
                command: Some("npm install -g typescript".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Node.js#TypeScript".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("nodejs-dev".to_string()),
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_cpp_dev() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for C/C++ files
    let has_cpp_files = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-type",
            "f",
            "(",
            "-name",
            "*.c",
            "-o",
            "-name",
            "*.cpp",
            "-o",
            "-name",
            "*.h",
            "-o",
            "-name",
            "*.hpp",
            ")",
            "-print",
            "-quit",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let cpp_usage = check_command_usage(&["gcc", "g++", "clang", "make", "cmake"]);

    if has_cpp_files || cpp_usage > 5 {
        // Check for GCC
        let has_gcc = is_package_installed("gcc");

        if !has_gcc {
            result.push(
                Advice::new(
                    "dev-gcc".to_string(),
                    "Install GCC compiler for C/C++ development".to_string(),
                    format!("You have C/C++ files or use compilation commands ({} times)! GCC (GNU Compiler Collection) compiles C and C++ code into executables. Essential for systems programming, game development, and performance-critical applications.", cpp_usage),
                    "Install GCC and related build tools".to_string(),
                    Some("sudo pacman -S --noconfirm gcc".to_string()),
                    RiskLevel::Low,
                    Priority::Recommended,
                    vec![
                        "https://wiki.archlinux.org/title/GNU_Compiler_Collection".to_string(),
                    ],
                    "development".to_string(),
                )
                .with_popularity(70)
                .with_bundle("cpp-dev".to_string())
            );
        }

        // Check for Make
        if !is_package_installed("make") {
            result.push(
                Advice::new(
                    "dev-make".to_string(),
                    "Install Make for build automation".to_string(),
                    "Make is the standard build automation tool for C/C++ projects. It reads Makefiles to compile code efficiently - only rebuilding changed files. Essential for any C/C++ development!".to_string(),
                    "Install GNU Make".to_string(),
                    Some("sudo pacman -S --noconfirm make".to_string()),
                    RiskLevel::Low,
                    Priority::Recommended,
                    vec![
                        "https://wiki.archlinux.org/title/Makefile".to_string(),
                    ],
                    "development".to_string(),
                )
                .with_popularity(75)
                .with_bundle("cpp-dev".to_string())
            );
        }

        // Check for CMake
        if !is_package_installed("cmake") {
            result.push(
                Advice::new(
                    "dev-cmake".to_string(),
                    "Install CMake for modern C/C++ builds".to_string(),
                    "CMake is the modern build system for C/C++ projects - generates Makefiles, Ninja builds, or IDE projects. Used by major projects like LLVM, Qt, MySQL. If you see CMakeLists.txt, you need this!".to_string(),
                    "Install CMake".to_string(),
                    Some("sudo pacman -S --noconfirm cmake".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/CMake".to_string(),
                    ],
                    "development".to_string(),
                )
                .with_popularity(60)
                .with_bundle("cpp-dev".to_string())
            );
        }

        // Check for clangd (C/C++ LSP)
        if !is_package_installed("clang") {
            result.push(
                Advice::new(
                    "dev-clangd".to_string(),
                    "Install Clang and clangd for modern C/C++ development".to_string(),
                    "Clang is a modern C/C++ compiler with better error messages than GCC, and clangd provides LSP support (autocomplete, go-to-definition, refactoring) for your editor. Essential for modern C/C++ development!".to_string(),
                    "Install Clang compiler and tools".to_string(),
                    Some("sudo pacman -S --noconfirm clang".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/Clang".to_string(),
                    ],
                    "development".to_string(),
                )
                .with_popularity(55)
                .with_bundle("cpp-dev".to_string())
            );
        }
    }

    result
}

pub(crate) fn check_php_dev() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for PHP files
    let has_php_files = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.php",
            "-type",
            "f",
            "-print",
            "-quit",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let php_usage = check_command_usage(&["php", "composer"]);

    if has_php_files || php_usage > 5 {
        // Check for PHP
        if !is_package_installed("php") {
            result.push(
                Advice::new(
                    "dev-php".to_string(),
                    "Install PHP for web development".to_string(),
                    format!("You have PHP files or use PHP commands ({} times)! PHP powers WordPress, Laravel, Symfony - it's the backbone of millions of websites. Install PHP to run and develop PHP applications.", php_usage),
                    "Install PHP interpreter".to_string(),
                    Some("sudo pacman -S --noconfirm php".to_string()),
                    RiskLevel::Low,
                    Priority::Recommended,
                    vec![
                        "https://wiki.archlinux.org/title/PHP".to_string(),
                    ],
                    "development".to_string(),
                )
                .with_popularity(65)
            );
        }

        // Check for Composer
        if !is_package_installed("composer") {
            result.push(
                Advice::new(
                    "dev-composer".to_string(),
                    "Install Composer for PHP dependency management".to_string(),
                    "Composer is the standard dependency manager for PHP - like npm for Node.js. It manages libraries, autoloading, and packages from Packagist. Essential for modern PHP development (Laravel, Symfony, etc.)!".to_string(),
                    "Install Composer".to_string(),
                    Some("sudo pacman -S --noconfirm composer".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/PHP#Composer".to_string(),
                    ],
                    "development".to_string(),
                )
                .with_popularity(60)
            );
        }
    }

    result
}

pub(crate) fn check_ruby_dev() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Ruby files
    let has_ruby_files = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.rb",
            "-type",
            "f",
            "-print",
            "-quit",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let ruby_usage = check_command_usage(&["ruby", "gem", "bundle", "rails"]);

    if has_ruby_files || ruby_usage > 5 {
        // Check for Ruby
        if !is_package_installed("ruby") {
            result.push(
                Advice::new(
                    "dev-ruby".to_string(),
                    "Install Ruby for scripting and web development".to_string(),
                    format!("You have Ruby files or use Ruby commands ({} times)! Ruby is elegant, expressive, perfect for web development (Rails), DevOps tools (Chef, Puppet), and scripting. Install Ruby to run your Ruby programs.", ruby_usage),
                    "Install Ruby interpreter".to_string(),
                    Some("sudo pacman -S --noconfirm ruby".to_string()),
                    RiskLevel::Low,
                    Priority::Recommended,
                    vec![
                        "https://wiki.archlinux.org/title/Ruby".to_string(),
                    ],
                    "development".to_string(),
                )
                .with_popularity(55)
            );
        }

        // Check for Bundler
        if ruby_usage > 0
            && !Command::new("which")
                .arg("bundle")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        {
            result.push(
                Advice::new(
                    "dev-bundler".to_string(),
                    "Install Bundler for Ruby dependency management".to_string(),
                    "Bundler manages gem dependencies for Ruby projects. It ensures everyone on your team uses the same gem versions. If you see a Gemfile, you need Bundler!".to_string(),
                    "Install Bundler gem".to_string(),
                    Some("gem install bundler".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/Ruby#Bundler".to_string(),
                    ],
                    "development".to_string(),
                )
                .with_popularity(50)
            );
        }
    }

    result
}

pub(crate) fn check_git_advanced() -> Vec<Advice> {
    let mut result = Vec::new();

    let git_usage = check_command_usage(&["git"]);

    if git_usage > 20 {
        // Check for delta (better git diff)
        let has_delta = Command::new("which")
            .arg("delta")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_delta {
            result.push(Advice {
                id: "git-delta".to_string(),
                title: "Install delta for beautiful git diffs".to_string(),
                reason: format!("You use git {} times! Delta makes git diff beautiful - syntax highlighting, side-by-side diffs, line numbers, better merge conflict visualization. Configure with: git config --global core.pager delta. Your diffs will never be the same!", git_usage),
                action: "Install git-delta".to_string(),
                command: Some("pacman -S --noconfirm git-delta".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git#Diff_and_merge_tools".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for lazygit
        let has_lazygit = Command::new("pacman")
            .args(&["-Q", "lazygit"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_lazygit {
            result.push(Advice {
                id: "git-lazygit".to_string(),
                title: "Install lazygit for terminal UI git client".to_string(),
                reason: "lazygit is a gorgeous terminal UI for git! Stage files, create commits, manage branches, resolve conflicts - all with keyboard shortcuts. Way faster than typing git commands. Just run 'lazygit' in any repo. Git power users love it!".to_string(),
                action: "Install lazygit".to_string(),
                command: Some("pacman -S --noconfirm lazygit".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git#Graphical_tools".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_container_alternatives() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Docker
    let has_docker = Command::new("pacman")
        .args(&["-Q", "docker"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_docker {
        // Suggest Podman as alternative
        let has_podman = Command::new("pacman")
            .args(&["-Q", "podman"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_podman {
            result.push(Advice {
                id: "container-podman".to_string(),
                title: "Try Podman as Docker alternative".to_string(),
                reason: "Podman is Docker without the daemon! Rootless by default (more secure), drop-in replacement for Docker CLI. 'alias docker=podman' and you're good. No root daemon, better security, same containers. Great for developers who want Docker-compatible tools without Docker's architecture!".to_string(),
                action: "Install Podman".to_string(),
                command: Some("pacman -S --noconfirm podman".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Podman".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_python_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    let python_usage = check_command_usage(&["python", "python3", "pip"]);

    if python_usage > 10 {
        // Check for poetry
        let has_poetry = Command::new("which")
            .arg("poetry")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_poetry {
            result.push(Advice {
                id: "python-poetry".to_string(),
                title: "Install Poetry for Python dependency management".to_string(),
                reason: format!("You use Python {} times! Poetry is THE modern Python package manager. No more pip freeze, no more requirements.txt hell. Dependency resolution, virtual environments, publishing - all in one tool. 'poetry add requests' just works!", python_usage),
                action: "Install Poetry".to_string(),
                command: Some("pacman -S --noconfirm python-poetry".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python#Package_management".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("python-dev".to_string()),
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for virtualenv
        let has_virtualenv = Command::new("pacman")
            .args(&["-Q", "python-virtualenv"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_virtualenv {
            result.push(Advice {
                id: "python-virtualenv".to_string(),
                title: "Install virtualenv for isolated Python environments".to_string(),
                reason: "Virtual environments are essential for Python development! Isolate project dependencies, avoid conflicts, test different versions. Every Python developer needs this. 'python -m venv myenv' creates isolated environment!".to_string(),
                action: "Install python-virtualenv".to_string(),
                command: Some("pacman -S --noconfirm python-virtualenv".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python#Virtual_environment".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("python-dev".to_string()),
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for ipython
        let has_ipython = Command::new("pacman")
            .args(&["-Q", "ipython"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_ipython {
            result.push(Advice {
                id: "python-ipython".to_string(),
                title: "Install IPython for enhanced Python REPL".to_string(),
                reason: "IPython is Python REPL on steroids! Syntax highlighting, tab completion, magic commands, inline plots, history. Way better than plain 'python' prompt. Data scientists and developers love it. Try 'ipython' and never go back!".to_string(),
                action: "Install IPython".to_string(),
                command: Some("pacman -S --noconfirm ipython".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python#IPython".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("python-dev".to_string()),
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_rust_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    let rust_usage = check_command_usage(&["cargo", "rustc"]);

    if rust_usage > 10 {
        // Check for cargo-watch
        let has_cargo_watch = Command::new("which")
            .arg("cargo-watch")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_cargo_watch {
            result.push(Advice {
                id: "rust-cargo-watch".to_string(),
                title: "Install cargo-watch for automatic rebuilds".to_string(),
                reason: format!("You use Rust {} times! cargo-watch automatically rebuilds on file changes. 'cargo watch -x check -x test' runs checks and tests on every save. Essential for fast development iterations. No more manual cargo build!", rust_usage),
                action: "Install cargo-watch".to_string(),
                command: Some("pacman -S --noconfirm cargo-watch".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rust#Cargo".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("rust-dev".to_string()),
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for cargo-audit
        let has_cargo_audit = Command::new("which")
            .arg("cargo-audit")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_cargo_audit {
            result.push(Advice {
                id: "rust-cargo-audit".to_string(),
                title: "Install cargo-audit for security vulnerability scanning".to_string(),
                reason: "cargo-audit checks your Cargo.lock for known security vulnerabilities! Scans against RustSec database, finds CVEs in dependencies. 'cargo audit' shows security issues. Essential for production Rust code!".to_string(),
                action: "Install cargo-audit".to_string(),
                command: Some("cargo install cargo-audit".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rust#Security".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("rust-dev".to_string()),
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_python_enhancements() -> Vec<Advice> {
    let mut result = Vec::new();

    let python_usage = check_command_usage(&["python", "python3", "pip", "poetry", "virtualenv"]);

    if python_usage > 30 {
        // Check for pyenv (Python version manager)
        let has_pyenv = Command::new("which")
            .arg("pyenv")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_pyenv {
            result.push(Advice {
                id: "python-pyenv".to_string(),
                title: "Install pyenv for managing multiple Python versions".to_string(),
                reason: format!("You code in Python regularly ({}+ commands)! pyenv lets you install and switch between Python versions easily. Per-project Python versions, test across versions, use latest features. Like nvm for Node, but for Python. Essential for serious Python dev!", python_usage),
                action: "Install pyenv".to_string(),
                command: Some("yay -S --noconfirm pyenv".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Python#Multiple_versions".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("python-dev".to_string()),
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_git_workflow_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    let git_usage = check_command_usage(&["git commit", "git push", "git pull", "git log"]);

    if git_usage > 50 {
        // Check for lazygit (Git TUI)
        let has_lazygit = Command::new("which")
            .arg("lazygit")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_lazygit {
            result.push(Advice {
                id: "git-lazygit".to_string(),
                title: "Install lazygit for visual Git operations".to_string(),
                reason: format!("You're a Git power user ({}+ commands)! lazygit is a gorgeous TUI for Git. Visual staging, committing, branching, rebasing. See your repo status at a glance. Much faster than memorizing git commands. Vim-like keybindings, highly productive!", git_usage),
                action: "Install lazygit".to_string(),
                command: Some("pacman -S --noconfirm lazygit".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Git#Tips_and_tricks".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

