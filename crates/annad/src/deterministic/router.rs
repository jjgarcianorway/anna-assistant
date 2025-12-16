//! Query class router for deterministic answers (v0.0.805).

use anna_shared::rpc::{ProbeResult, RuntimeContext};

use super::{
    answer_cpu_cores, answer_cpu_temp, answer_disk_usage, answer_hardware_audio,
    answer_installed_tool_check, answer_memory_free, answer_memory_usage, answer_package_count,
    answer_service_status, answer_system_health_summary, DeterministicResult,
};
use crate::det;
use crate::det_extended;
use crate::probe_answers;
use crate::router::{classify_query, QueryClass};

/// Try to produce a deterministic answer from available data
pub fn try_answer(
    query: &str,
    context: &RuntimeContext,
    probe_results: &[ProbeResult],
) -> Option<DeterministicResult> {
    let query_class = classify_query(query);
    let route_class = query_class.to_string();

    match query_class {
        // FAST PATH: SystemTriage - errors/warnings only, no specialist
        QueryClass::SystemTriage => crate::triage_answer::generate_triage_answer(probe_results),
        QueryClass::CpuInfo => probe_answers::answer_cpu_info(&context.hardware, probe_results)
            .map(|mut r| {
                r.route_class = route_class;
                r
            }),
        // v0.0.45: CpuCores - uses lscpu probe
        QueryClass::CpuCores => answer_cpu_cores(probe_results, &route_class),
        // v0.0.45: CpuTemp - uses sensors probe
        QueryClass::CpuTemp => answer_cpu_temp(probe_results, &route_class),
        QueryClass::RamInfo => probe_answers::answer_ram_info(&context.hardware, probe_results)
            .map(|mut r| {
                r.route_class = route_class;
                r
            }),
        QueryClass::GpuInfo => probe_answers::answer_gpu_info(&context.hardware).map(|mut r| {
            r.route_class = route_class;
            r
        }),
        // v0.0.45: HardwareAudio - uses lspci_audio probe
        QueryClass::HardwareAudio => answer_hardware_audio(probe_results, &route_class),
        QueryClass::TopMemoryProcesses => {
            probe_answers::answer_top_memory(probe_results).map(|mut r| {
                r.route_class = route_class;
                r
            })
        }
        QueryClass::TopCpuProcesses => probe_answers::answer_top_cpu(probe_results).map(|mut r| {
            r.route_class = route_class;
            r
        }),
        QueryClass::DiskSpace => probe_answers::answer_disk_space(probe_results).map(|mut r| {
            r.route_class = route_class;
            r
        }),
        QueryClass::NetworkInterfaces => probe_answers::answer_network_interfaces(probe_results)
            .map(|mut r| {
                r.route_class = route_class;
                r
            }),
        QueryClass::Help => Some(super::answer_help(&route_class)),
        QueryClass::SystemSlow => probe_answers::answer_system_slow(probe_results).map(|mut r| {
            r.route_class = route_class;
            r
        }),
        QueryClass::MemoryUsage => answer_memory_usage(probe_results, &route_class),
        // v0.0.45: MemoryFree - uses free probe (same as MemoryUsage)
        QueryClass::MemoryFree => answer_memory_free(probe_results, &route_class),
        QueryClass::DiskUsage => answer_disk_usage(probe_results, &route_class),
        QueryClass::ServiceStatus => answer_service_status(probe_results, &route_class),
        QueryClass::SystemHealthSummary => {
            answer_system_health_summary(probe_results, &route_class)
        }
        // RAG-first classes - handled by rag_answerer, not here
        QueryClass::BootTimeStatus
        | QueryClass::InstalledPackagesOverview
        | QueryClass::AppAlternatives => None,
        // v0.0.799: BootBlame - "why is my boot slow?"
        QueryClass::BootBlame => det::answer_boot_blame(probe_results, &route_class),
        // v0.0.45: PackageCount - uses pacman_count probe
        QueryClass::PackageCount => answer_package_count(probe_results, &route_class),
        // v0.0.45: InstalledToolCheck - uses command_v probe
        QueryClass::InstalledToolCheck => answer_installed_tool_check(probe_results, &route_class),
        // v0.45.5: ConfigureEditor - needs clarification, cannot be answered deterministically
        QueryClass::ConfigureEditor => None,
        // v0.0.77: MetaSmallTalk - deterministic static response
        QueryClass::MetaSmallTalk => Some(det::answer_meta_small_talk(query, &route_class)),
        // v0.0.77: KernelVersion - deterministic from uname probe
        QueryClass::KernelVersion => det::answer_kernel_version(probe_results, &route_class),
        // v0.0.77: ConfigFileLocation - deterministic from known paths
        QueryClass::ConfigFileLocation => det::answer_config_file_location(query, &route_class),
        // v0.0.99: InstallPackage - needs user confirmation, handled in rpc_handler
        QueryClass::InstallPackage => None,
        // v0.0.99: ManageService - needs user confirmation, handled in rpc_handler
        QueryClass::ManageService => None,
        // v0.0.101: ConfigureShell - recipe-based, handled in recipe_fast_path
        QueryClass::ConfigureShell => None,
        // v0.0.101: ConfigureGit - recipe-based, handled in recipe_fast_path
        QueryClass::ConfigureGit => None,
        // v0.0.104: SshKeyManagement - recipe-based, handled in recipe_fast_path
        QueryClass::SshKeyManagement => None,
        // v0.0.111: TicketHistory - deterministic from internal stats
        QueryClass::TicketHistory => Some(det::answer_ticket_history(&route_class)),
        // v0.0.111: StaffRoster - deterministic from roster data
        QueryClass::StaffRoster => Some(det::answer_staff_roster(&route_class)),
        // v0.0.122: PackageUpdates - deterministic from checkupdates
        QueryClass::PackageUpdates => det::answer_package_updates(probe_results, &route_class),
        // v0.0.122: SwapInfo - deterministic from free
        QueryClass::SwapInfo => det::answer_swap_info(probe_results, &route_class),
        // v0.0.122: TimezoneInfo - deterministic from timedatectl
        QueryClass::TimezoneInfo => det::answer_timezone_info(probe_results, &route_class),
        // v0.0.122: SystemUptime - deterministic from uptime
        QueryClass::SystemUptime => det::answer_system_uptime(probe_results, &route_class),
        // v0.0.123: LoggedInUsers - deterministic from who command
        QueryClass::LoggedInUsers => det::answer_logged_in_users(probe_results, &route_class),
        // v0.0.123: BatteryStatus - deterministic from upower/acpi
        QueryClass::BatteryStatus => det::answer_battery_status(probe_results, &route_class),
        // v0.0.123: SystemLoad - deterministic from /proc/loadavg
        QueryClass::SystemLoad => det::answer_system_load(probe_results, &route_class),
        // v0.0.123: LastBoot - deterministic from who -b
        QueryClass::LastBoot => det::answer_last_boot(probe_results, &route_class),
        // v0.0.124: Hostname - deterministic from hostname command
        QueryClass::Hostname => det::answer_hostname(probe_results, &route_class),
        // v0.0.124: OsInfo - deterministic from /etc/os-release
        QueryClass::OsInfo => det::answer_os_info(probe_results, &route_class),
        // v0.0.124: NetworkConnectivity - deterministic from ping
        QueryClass::NetworkConnectivity => {
            det::answer_network_connectivity(probe_results, &route_class)
        }
        // v0.0.124: MountedFilesystems - deterministic from findmnt
        QueryClass::MountedFilesystems => {
            det::answer_mounted_filesystems(probe_results, &route_class)
        }
        // v0.0.124: UsbDevices - deterministic from lsusb
        QueryClass::UsbDevices => det::answer_usb_devices(probe_results, &route_class),
        // v0.0.125: ListeningPorts - deterministic from ss
        QueryClass::ListeningPorts => det::answer_listening_ports(probe_results, &route_class),
        // v0.0.125: RunningServices - deterministic from systemctl
        QueryClass::RunningServices => det::answer_running_services(probe_results, &route_class),
        // v0.0.125: CurrentUser - deterministic from id
        QueryClass::CurrentUser => det::answer_current_user(probe_results, &route_class),
        // v0.0.125: SystemArchitecture - deterministic from uname -m
        QueryClass::SystemArchitecture => {
            det::answer_system_architecture(probe_results, &route_class)
        }
        // v0.0.125: EnvironmentVars - deterministic from env
        QueryClass::EnvironmentVars => det::answer_environment_vars(probe_results, &route_class),
        // v0.0.126: ProcessTree - deterministic from pstree
        QueryClass::ProcessTree => det_extended::answer_process_tree(probe_results, &route_class),
        // v0.0.126: DnsServers - deterministic from /etc/resolv.conf
        QueryClass::DnsServers => det_extended::answer_dns_servers(probe_results, &route_class),
        // v0.0.126: DefaultGateway - deterministic from ip route
        QueryClass::DefaultGateway => {
            det_extended::answer_default_gateway(probe_results, &route_class)
        }
        // v0.0.126: OpenFiles - deterministic from lsof
        QueryClass::OpenFiles => det_extended::answer_open_files(probe_results, &route_class),
        // v0.0.126: SystemLocale - deterministic from locale
        QueryClass::SystemLocale => det_extended::answer_system_locale(probe_results, &route_class),
        // v0.0.127: BlockDevices - deterministic from lsblk
        QueryClass::BlockDevices => det_extended::answer_block_devices(probe_results, &route_class),
        // v0.0.127: InstalledKernels - deterministic from package manager
        QueryClass::InstalledKernels => {
            det_extended::answer_installed_kernels(probe_results, &route_class)
        }
        // v0.0.127: CpuFrequency - deterministic from cpufreq
        QueryClass::CpuFrequency => det_extended::answer_cpu_frequency(probe_results, &route_class),
        // v0.0.127: MemorySlots - deterministic from dmidecode
        QueryClass::MemorySlots => det_extended::answer_memory_slots(probe_results, &route_class),
        // v0.0.127: ZfsStatus - deterministic from zpool
        QueryClass::ZfsStatus => det_extended::answer_zfs_status(probe_results, &route_class),
        // v0.0.128: BootLoader - deterministic from bootctl/grub
        QueryClass::BootLoader => det_extended::answer_boot_loader(probe_results, &route_class),
        // v0.0.128: FirewallStatus - deterministic from iptables/nftables
        QueryClass::FirewallStatus => {
            det_extended::answer_firewall_status(probe_results, &route_class)
        }
        // v0.0.128: SystemdUnits - deterministic from systemctl
        QueryClass::SystemdUnits => det_extended::answer_systemd_units(probe_results, &route_class),
        // v0.0.128: Crontabs - deterministic from crontab
        QueryClass::Crontabs => det_extended::answer_crontabs(probe_results, &route_class),
        // v0.0.128: SshConnections - deterministic from who/ss
        QueryClass::SshConnections => {
            det_extended::answer_ssh_connections(probe_results, &route_class)
        }
        // v0.0.129: DockerContainers - deterministic from docker ps
        QueryClass::DockerContainers => {
            det_extended::answer_docker_containers(probe_results, &route_class)
        }
        // v0.0.129: DockerImages - deterministic from docker images
        QueryClass::DockerImages => det_extended::answer_docker_images(probe_results, &route_class),
        // v0.0.129: SystemdTimers - deterministic from systemctl list-timers
        QueryClass::SystemdTimers => {
            det_extended::answer_systemd_timers(probe_results, &route_class)
        }
        // v0.0.129: LastLogins - deterministic from last
        QueryClass::LastLogins => det_extended::answer_last_logins(probe_results, &route_class),
        // v0.0.129: FailedLogins - deterministic from lastb/journalctl
        QueryClass::FailedLogins => det_extended::answer_failed_logins(probe_results, &route_class),
        // v0.0.130: SystemdJournal - deterministic from journalctl
        QueryClass::SystemdJournal => {
            det_extended::answer_systemd_journal(probe_results, &route_class)
        }
        // v0.0.130: NetworkNamespaces - deterministic from ip netns
        QueryClass::NetworkNamespaces => {
            det_extended::answer_network_namespaces(probe_results, &route_class)
        }
        // v0.0.130: AvailableShells - deterministic from /etc/shells
        QueryClass::AvailableShells => {
            det_extended::answer_available_shells(probe_results, &route_class)
        }
        // v0.0.130: SudoersInfo - deterministic from sudo -l
        QueryClass::SudoersInfo => det_extended::answer_sudoers_info(probe_results, &route_class),
        // v0.0.130: InstalledDesktops - deterministic from package query
        QueryClass::InstalledDesktops => {
            det_extended::answer_installed_desktops(probe_results, &route_class)
        }
        // v0.0.131: VirtualizationInfo - deterministic from systemd-detect-virt
        QueryClass::VirtualizationInfo => {
            det_extended::answer_virtualization_info(probe_results, &route_class)
        }
        // v0.0.131: SelinuxStatus - deterministic from sestatus
        QueryClass::SelinuxStatus => {
            det_extended::answer_selinux_status(probe_results, &route_class)
        }
        // v0.0.131: AppArmorStatus - deterministic from aa-status
        QueryClass::AppArmorStatus => {
            det_extended::answer_apparmor_status(probe_results, &route_class)
        }
        // v0.0.131: SystemdSlices - deterministic from systemd-cgls
        QueryClass::SystemdSlices => {
            det_extended::answer_systemd_slices(probe_results, &route_class)
        }
        // v0.0.131: CoredumpList - deterministic from coredumpctl
        QueryClass::CoredumpList => det_extended::answer_coredump_list(probe_results, &route_class),
        // v0.0.132: KernelModules - deterministic from lsmod
        QueryClass::KernelModules => {
            det_extended::answer_kernel_modules(probe_results, &route_class)
        }
        // v0.0.132: SystemdTargets - deterministic from systemctl
        QueryClass::SystemdTargets => {
            det_extended::answer_systemd_targets(probe_results, &route_class)
        }
        // v0.0.132: IpRoutes - deterministic from ip route
        QueryClass::IpRoutes => det_extended::answer_ip_routes(probe_results, &route_class),
        // v0.0.132: ArpTable - deterministic from ip neigh
        QueryClass::ArpTable => det_extended::answer_arp_table(probe_results, &route_class),
        // v0.0.132: IptablesRules - deterministic from iptables
        QueryClass::IptablesRules => {
            det_extended::answer_iptables_rules(probe_results, &route_class)
        }
        // v0.0.133: PciDevices - deterministic from lspci
        QueryClass::PciDevices => det_extended::answer_pci_devices(probe_results, &route_class),
        // v0.0.133: DmesgErrors - deterministic from dmesg
        QueryClass::DmesgErrors => det_extended::answer_dmesg_errors(probe_results, &route_class),
        // v0.0.133: SystemdSockets - deterministic from systemctl
        QueryClass::SystemdSockets => {
            det_extended::answer_systemd_sockets(probe_results, &route_class)
        }
        // v0.0.133: TmpFiles - deterministic from ls /tmp
        QueryClass::TmpFiles => det_extended::answer_tmp_files(probe_results, &route_class),
        // v0.0.133: UserGroups - deterministic from groups
        QueryClass::UserGroups => det_extended::answer_user_groups(probe_results, &route_class),
        // v0.0.134: LvmStatus - deterministic from lvs/vgs
        QueryClass::LvmStatus => det_extended::answer_lvm_status(probe_results, &route_class),
        // v0.0.134: RaidStatus - deterministic from mdstat
        QueryClass::RaidStatus => det_extended::answer_raid_status(probe_results, &route_class),
        // v0.0.134: NtpStatus - deterministic from timedatectl
        QueryClass::NtpStatus => det_extended::answer_ntp_status(probe_results, &route_class),
        // v0.0.134: SensorsTemp - deterministic from sensors
        QueryClass::SensorsTemp => det_extended::answer_sensors_temp(probe_results, &route_class),
        // v0.0.134: GpuMemory - deterministic from nvidia-smi
        QueryClass::GpuMemory => det_extended::answer_gpu_memory(probe_results, &route_class),
        // v0.0.134: XorgLog - deterministic from Xorg.log
        QueryClass::XorgLog => det_extended::answer_xorg_log(probe_results, &route_class),
        // v0.0.135: BluetoothDevices - deterministic from bluetoothctl
        QueryClass::BluetoothDevices => {
            det_extended::answer_bluetooth_devices(probe_results, &route_class)
        }
        // v0.0.135: WirelessNetworks - deterministic from nmcli
        QueryClass::WirelessNetworks => {
            det_extended::answer_wireless_networks(probe_results, &route_class)
        }
        // v0.0.135: PrinterStatus - deterministic from lpstat
        QueryClass::PrinterStatus => {
            det_extended::answer_printer_status(probe_results, &route_class)
        }
        // v0.0.135: AudioDevices - deterministic from pactl
        QueryClass::AudioDevices => det_extended::answer_audio_devices(probe_results, &route_class),
        // v0.0.135: SystemdPaths - deterministic from systemctl
        QueryClass::SystemdPaths => det_extended::answer_systemd_paths(probe_results, &route_class),
        // v0.0.136: SystemctlMask - deterministic from systemctl list-unit-files
        QueryClass::SystemctlMask => {
            det_extended::answer_systemctl_mask(probe_results, &route_class)
        }
        // v0.0.136: HostsFile - deterministic from /etc/hosts
        QueryClass::HostsFile => det_extended::answer_hosts_file(probe_results, &route_class),
        // v0.0.136: FstabEntries - deterministic from /etc/fstab
        QueryClass::FstabEntries => det_extended::answer_fstab_entries(probe_results, &route_class),
        // v0.0.136: SysctlSettings - deterministic from sysctl
        QueryClass::SysctlSettings => {
            det_extended::answer_sysctl_settings(probe_results, &route_class)
        }
        // v0.0.136: LoginctlSessions - deterministic from loginctl
        QueryClass::LoginctlSessions => {
            det_extended::answer_loginctl_sessions(probe_results, &route_class)
        }
        // v0.0.139: EnvironmentVariables - deterministic from printenv
        QueryClass::EnvironmentVariables => {
            det_extended::answer_environment_variables(probe_results, &route_class)
        }
        // v0.0.139: SystemdScopes - deterministic from systemctl list-units
        QueryClass::SystemdScopes => {
            det_extended::answer_systemd_scopes(probe_results, &route_class)
        }
        // v0.0.139: KernelCmdline - deterministic from /proc/cmdline
        QueryClass::KernelCmdline => {
            det_extended::answer_kernel_cmdline(probe_results, &route_class)
        }
        // v0.0.139: ModuleParams - deterministic from modinfo
        QueryClass::ModuleParams => det_extended::answer_module_params(probe_results, &route_class),
        // v0.0.139: NetworkBonding - deterministic from /proc/net/bonding
        QueryClass::NetworkBonding => {
            det_extended::answer_network_bonding(probe_results, &route_class)
        }
        // v0.0.141: SwapFiles - deterministic from /proc/swaps
        QueryClass::SwapFiles => det_extended::answer_swap_files(probe_results, &route_class),
        // v0.0.141: CpuGovernor - deterministic from cpufreq
        QueryClass::CpuGovernor => det_extended::answer_cpu_governor(probe_results, &route_class),
        // v0.0.141: SystemdMounts - deterministic from systemctl
        QueryClass::SystemdMounts => {
            det_extended::answer_systemd_mounts(probe_results, &route_class)
        }
        // v0.0.141: LoadedFirmware - deterministic from dmesg
        QueryClass::LoadedFirmware => {
            det_extended::answer_loaded_firmware(probe_results, &route_class)
        }
        // v0.0.141: NetworkStats - deterministic from /proc/net/dev
        QueryClass::NetworkStats => det_extended::answer_network_stats(probe_results, &route_class),
        // v0.0.309: DesktopWallpaper - handled in llm_request.rs
        QueryClass::DesktopWallpaper => None,
        // v0.0.311: SystemUpdate - handled in llm_request.rs
        QueryClass::SystemUpdate => None,
        // v0.0.390: LargestFolders - "top folders taking space"
        // v0.0.809: Reverted - du scan is inherently slow, let LLM explain
        QueryClass::LargestFolders => None,
        // v0.0.801: DeviceType - laptop vs desktop via hostnamectl
        QueryClass::DeviceType => det::answer_device_type(probe_results, &route_class),
        // v0.0.802: WebcamStatus - webcam/camera detection
        QueryClass::WebcamStatus => det::answer_webcam_status(probe_results, &route_class),
        // v0.0.805: ScreenResolution - screen/display/monitor info
        QueryClass::ScreenResolution => det::answer_screen_resolution(probe_results, &route_class),
        QueryClass::Unknown => None,
    }
}
