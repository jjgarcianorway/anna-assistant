//! QueryClass implementation methods (v0.0.172).

use super::QueryClass;

impl QueryClass {
    /// Parse from string (for corpus tests)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "system_triage" => Some(Self::SystemTriage),
            "cpu_info" => Some(Self::CpuInfo),
            "cpu_cores" => Some(Self::CpuCores),
            "cpu_temp" => Some(Self::CpuTemp),
            "ram_info" => Some(Self::RamInfo),
            "gpu_info" => Some(Self::GpuInfo),
            "hardware_audio" => Some(Self::HardwareAudio),
            "top_memory_processes" => Some(Self::TopMemoryProcesses),
            "top_cpu_processes" => Some(Self::TopCpuProcesses),
            "disk_space" => Some(Self::DiskSpace),
            "network_interfaces" => Some(Self::NetworkInterfaces),
            "help" => Some(Self::Help),
            "system_slow" => Some(Self::SystemSlow),
            "memory_usage" => Some(Self::MemoryUsage),
            "memory_free" => Some(Self::MemoryFree),
            "disk_usage" => Some(Self::DiskUsage),
            "largest_folders" => Some(Self::LargestFolders),
            "service_status" => Some(Self::ServiceStatus),
            "system_health_summary" => Some(Self::SystemHealthSummary),
            "boot_time_status" => Some(Self::BootTimeStatus),
            "boot_blame" => Some(Self::BootBlame),
            "installed_packages_overview" => Some(Self::InstalledPackagesOverview),
            "package_count" => Some(Self::PackageCount),
            "installed_tool_check" => Some(Self::InstalledToolCheck),
            "app_alternatives" => Some(Self::AppAlternatives),
            "configure_editor" => Some(Self::ConfigureEditor),
            "meta_small_talk" => Some(Self::MetaSmallTalk),
            "kernel_version" => Some(Self::KernelVersion),
            "config_file_location" => Some(Self::ConfigFileLocation),
            "install_package" => Some(Self::InstallPackage),
            "manage_service" => Some(Self::ManageService),
            "configure_shell" => Some(Self::ConfigureShell),
            "configure_git" => Some(Self::ConfigureGit),
            "ssh_key_management" => Some(Self::SshKeyManagement),
            "ticket_history" => Some(Self::TicketHistory),
            "staff_roster" => Some(Self::StaffRoster),
            "package_updates" => Some(Self::PackageUpdates),
            "swap_info" => Some(Self::SwapInfo),
            "timezone_info" => Some(Self::TimezoneInfo),
            "system_uptime" => Some(Self::SystemUptime),
            "logged_in_users" => Some(Self::LoggedInUsers),
            "battery_status" => Some(Self::BatteryStatus),
            "system_load" => Some(Self::SystemLoad),
            "last_boot" => Some(Self::LastBoot),
            "hostname" => Some(Self::Hostname),
            "os_info" => Some(Self::OsInfo),
            "network_connectivity" => Some(Self::NetworkConnectivity),
            "mounted_filesystems" => Some(Self::MountedFilesystems),
            "usb_devices" => Some(Self::UsbDevices),
            "listening_ports" => Some(Self::ListeningPorts),
            "running_services" => Some(Self::RunningServices),
            "current_user" => Some(Self::CurrentUser),
            "system_architecture" => Some(Self::SystemArchitecture),
            "environment_vars" => Some(Self::EnvironmentVars),
            "process_tree" => Some(Self::ProcessTree),
            "dns_servers" => Some(Self::DnsServers),
            "default_gateway" => Some(Self::DefaultGateway),
            "open_files" => Some(Self::OpenFiles),
            "system_locale" => Some(Self::SystemLocale),
            "block_devices" => Some(Self::BlockDevices),
            "installed_kernels" => Some(Self::InstalledKernels),
            "cpu_frequency" => Some(Self::CpuFrequency),
            "memory_slots" => Some(Self::MemorySlots),
            "zfs_status" => Some(Self::ZfsStatus),
            "boot_loader" => Some(Self::BootLoader),
            "firewall_status" => Some(Self::FirewallStatus),
            "systemd_units" => Some(Self::SystemdUnits),
            "crontabs" => Some(Self::Crontabs),
            "ssh_connections" => Some(Self::SshConnections),
            "docker_containers" => Some(Self::DockerContainers),
            "docker_images" => Some(Self::DockerImages),
            "systemd_timers" => Some(Self::SystemdTimers),
            "last_logins" => Some(Self::LastLogins),
            "failed_logins" => Some(Self::FailedLogins),
            "systemd_journal" => Some(Self::SystemdJournal),
            "network_namespaces" => Some(Self::NetworkNamespaces),
            "available_shells" => Some(Self::AvailableShells),
            "sudoers_info" => Some(Self::SudoersInfo),
            "installed_desktops" => Some(Self::InstalledDesktops),
            "virtualization_info" => Some(Self::VirtualizationInfo),
            "selinux_status" => Some(Self::SelinuxStatus),
            "apparmor_status" => Some(Self::AppArmorStatus),
            "systemd_slices" => Some(Self::SystemdSlices),
            "coredump_list" => Some(Self::CoredumpList),
            "kernel_modules" => Some(Self::KernelModules),
            "systemd_targets" => Some(Self::SystemdTargets),
            "ip_routes" => Some(Self::IpRoutes),
            "arp_table" => Some(Self::ArpTable),
            "iptables_rules" => Some(Self::IptablesRules),
            "pci_devices" => Some(Self::PciDevices),
            "dmesg_errors" => Some(Self::DmesgErrors),
            "systemd_sockets" => Some(Self::SystemdSockets),
            "tmp_files" => Some(Self::TmpFiles),
            "user_groups" => Some(Self::UserGroups),
            "lvm_status" => Some(Self::LvmStatus),
            "raid_status" => Some(Self::RaidStatus),
            "ntp_status" => Some(Self::NtpStatus),
            "sensors_temp" => Some(Self::SensorsTemp),
            "gpu_memory" => Some(Self::GpuMemory),
            "xorg_log" => Some(Self::XorgLog),
            "bluetooth_devices" => Some(Self::BluetoothDevices),
            "wireless_networks" => Some(Self::WirelessNetworks),
            "printer_status" => Some(Self::PrinterStatus),
            "audio_devices" => Some(Self::AudioDevices),
            "systemd_paths" => Some(Self::SystemdPaths),
            "systemctl_mask" => Some(Self::SystemctlMask),
            "hosts_file" => Some(Self::HostsFile),
            "fstab_entries" => Some(Self::FstabEntries),
            "sysctl_settings" => Some(Self::SysctlSettings),
            "loginctl_sessions" => Some(Self::LoginctlSessions),
            "environment_variables" => Some(Self::EnvironmentVariables),
            "systemd_scopes" => Some(Self::SystemdScopes),
            "kernel_cmdline" => Some(Self::KernelCmdline),
            "module_params" => Some(Self::ModuleParams),
            "network_bonding" => Some(Self::NetworkBonding),
            "swap_files" => Some(Self::SwapFiles),
            "cpu_governor" => Some(Self::CpuGovernor),
            "systemd_mounts" => Some(Self::SystemdMounts),
            "loaded_firmware" => Some(Self::LoadedFirmware),
            "network_stats" => Some(Self::NetworkStats),
            "system_update" => Some(Self::SystemUpdate),
            "device_type" => Some(Self::DeviceType),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// Check if this class is RAG-first (answered from knowledge store)
    pub fn is_rag_first(&self) -> bool {
        matches!(
            self,
            Self::BootTimeStatus | Self::InstalledPackagesOverview | Self::AppAlternatives
        )
    }

    /// Check if this class is a fast-path (skip translator, no specialist)
    pub fn is_fast_path(&self) -> bool {
        matches!(
            self,
            Self::SystemTriage
                | Self::Help
                | Self::MetaSmallTalk
                | Self::TicketHistory
                | Self::StaffRoster
                | Self::PackageUpdates
                | Self::SwapInfo
                | Self::TimezoneInfo
                | Self::SystemUptime
                | Self::LoggedInUsers
                | Self::BatteryStatus
                | Self::SystemLoad
                | Self::LastBoot
                | Self::Hostname
                | Self::OsInfo
                | Self::NetworkConnectivity
                | Self::MountedFilesystems
                | Self::UsbDevices
                | Self::ListeningPorts
                | Self::RunningServices
                | Self::CurrentUser
                | Self::SystemArchitecture
                | Self::EnvironmentVars
                | Self::ProcessTree
                | Self::DnsServers
                | Self::DefaultGateway
                | Self::OpenFiles
                | Self::SystemLocale
                | Self::BlockDevices
                | Self::InstalledKernels
                | Self::CpuFrequency
                | Self::MemorySlots
                | Self::ZfsStatus
                | Self::BootLoader
                | Self::FirewallStatus
                | Self::SystemdUnits
                | Self::Crontabs
                | Self::SshConnections
                | Self::DockerContainers
                | Self::DockerImages
                | Self::SystemdTimers
                | Self::LastLogins
                | Self::FailedLogins
                | Self::SystemdJournal
                | Self::NetworkNamespaces
                | Self::AvailableShells
                | Self::SudoersInfo
                | Self::InstalledDesktops
                | Self::VirtualizationInfo
                | Self::SelinuxStatus
                | Self::AppArmorStatus
                | Self::SystemdSlices
                | Self::CoredumpList
                | Self::KernelModules
                | Self::SystemdTargets
                | Self::IpRoutes
                | Self::ArpTable
                | Self::IptablesRules
                | Self::PciDevices
                | Self::DmesgErrors
                | Self::SystemdSockets
                | Self::TmpFiles
                | Self::UserGroups
                | Self::LvmStatus
                | Self::RaidStatus
                | Self::NtpStatus
                | Self::SensorsTemp
                | Self::GpuMemory
                | Self::XorgLog
                | Self::BluetoothDevices
                | Self::WirelessNetworks
                | Self::PrinterStatus
                | Self::AudioDevices
                | Self::SystemdPaths
                | Self::SystemctlMask
                | Self::HostsFile
                | Self::FstabEntries
                | Self::SysctlSettings
                | Self::LoginctlSessions
                | Self::EnvironmentVariables
                | Self::SystemdScopes
                | Self::KernelCmdline
                | Self::ModuleParams
                | Self::NetworkBonding
                | Self::SwapFiles
                | Self::CpuGovernor
                | Self::SystemdMounts
                | Self::LoadedFirmware
                | Self::NetworkStats
                | Self::BootBlame // v0.0.799
                | Self::DeviceType // v0.0.801
        )
    }

    /// Check if this class needs clarification before proceeding
    pub fn needs_clarification(&self) -> bool {
        matches!(self, Self::ConfigureEditor)
    }

    /// Check if this class requires confirmation before action
    pub fn needs_confirmation(&self) -> bool {
        matches!(
            self,
            Self::InstallPackage
                | Self::ManageService
                | Self::ConfigureEditor
                | Self::ConfigureShell
                | Self::ConfigureGit
                | Self::SystemUpdate
        )
    }

    /// Check if this class is recipe-first (answered from recipes, skip LLM)
    pub fn is_recipe_first(&self) -> bool {
        matches!(self, Self::ConfigureShell | Self::ConfigureGit)
    }

    /// Get the fact key needed for clarification
    pub fn clarification_fact_key(&self) -> Option<&'static str> {
        match self {
            Self::ConfigureEditor => Some("preferred_editor"),
            _ => None,
        }
    }
}
