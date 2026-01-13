//! Fast-path pattern matching for common queries.
//! Each function checks for patterns in its category and returns (command, template).

/// System info fast-path patterns.
pub fn get_system_fast_path(q: &str) -> Option<(&'static str, &'static str)> {
    if q.contains("kernel") && (q.contains("version") || q.contains("running")) {
        return Some(("uname -r", "You are running kernel {output}"));
    }
    if q.contains("uptime") || (q.contains("how long") && q.contains("up")) {
        return Some(("uptime -p", "System uptime: {output}"));
    }
    if q.contains("shell") && (q.contains("what") || q.contains("which") || q.contains("using")) {
        return Some(("echo $SHELL", "Your shell is {output}"));
    }
    if (q.contains("desktop") || q.contains("window manager") || q.contains("de") || q.contains("wm"))
        && (q.contains("what") || q.contains("which") || q.contains("running") || q.contains("using"))
    {
        return Some(("echo $XDG_CURRENT_DESKTOP", "You are running {output}"));
    }
    if (q.contains("wayland") || q.contains("x11") || q.contains("display server") || q.contains("xorg"))
        && (q.contains("what") || q.contains("which") || q.contains("using"))
    {
        return Some(("echo $XDG_SESSION_TYPE", "Display server: {output}"));
    }
    if q.contains("hostname") && (q.contains("what") || q.contains("my")) {
        return Some(("hostname", "Hostname: {output}"));
    }
    if (q.contains("username") || q.contains("user") || q.contains("uid"))
        && (q.contains("what") || q.contains("my") || q.contains("current"))
    {
        return Some(("id", "{output}"));
    }
    if q.contains("groups") && (q.contains("what") || q.contains("member") || q.contains("my")) {
        return Some(("groups", "Your groups: {output}"));
    }
    if q.contains("timezone") && (q.contains("what") || q.contains("configured") || q.contains("my")) {
        return Some(("timedatectl | grep 'Time zone'", "{output}"));
    }
    if q.contains("locale") && (q.contains("what") || q.contains("my") || q.contains("system")) {
        return Some(("locale | head -5", "{output}"));
    }
    if (q.contains("distro") || q.contains("distribution") || q.contains("os ") || q.contains("operating system"))
        && (q.contains("what") || q.contains("which") || q.contains("running"))
    {
        return Some(("cat /etc/os-release | grep PRETTY_NAME | cut -d= -f2 | tr -d '\"'", "OS: {output}"));
    }
    if (q.contains("boot") || q.contains("startup")) && (q.contains("time") || q.contains("how long") || q.contains("fast")) {
        return Some(("systemd-analyze | head -1", "{output}"));
    }
    if q.contains("user") && (q.contains("who am i") || q.contains("logged") || q.contains("current")) {
        return Some(("whoami", "You are logged in as: {output}"));
    }
    if q.contains("reboot") && (q.contains("last") || q.contains("when")) {
        return Some(("who -b | awk '{print $3, $4}'", "Last reboot: {output}"));
    }
    if q.contains("load") && (q.contains("average") || q.contains("system")) {
        return Some(("uptime | awk -F'load average:' '{print $2}'", "Load average:{output}"));
    }
    if q.contains("process") && (q.contains("how many") || q.contains("running") || q.contains("count")) {
        return Some(("ps aux | wc -l", "Running processes: {output}"));
    }
    if q.contains("default") && (q.contains("target") || q.contains("runlevel")) {
        return Some(("systemctl get-default", "Default target: {output}"));
    }
    if q.contains("x11") || q.contains("wayland") || (q.contains("session") && q.contains("type")) {
        return Some(("echo $XDG_SESSION_TYPE", "Session: {output}"));
    }
    if q.contains("domain") || q.contains("fqdn") {
        return Some(("hostname -f 2>/dev/null || hostname", "FQDN: {output}"));
    }
    None
}

/// Hardware fast-path patterns.
pub fn get_hardware_fast_path(q: &str) -> Option<(&'static str, &'static str)> {
    if (q.contains("ram") || q.contains("memory")) && (q.contains("how much") || q.contains("total") || q.contains("available")) {
        return Some(("free -h | grep Mem", "{output}"));
    }
    if q.contains("swap") && (q.contains("configured") || q.contains("using") || q.contains("how much")) {
        return Some(("swapon --show", "{output}"));
    }
    if q.contains("gpu") && (q.contains("what") || q.contains("which") || q.contains("have") || q.contains("using")) {
        return Some(("lspci | grep -i vga", "GPU: {output}"));
    }
    if q.contains("cpu") && (q.contains("what") || q.contains("which") || q.contains("have") || q.contains("model")) && !q.contains("using") {
        return Some(("lscpu | grep 'Model name' | cut -d: -f2 | xargs", "CPU: {output}"));
    }
    if (q.contains("disk") || q.contains("storage")) && (q.contains("space") || q.contains("free") || q.contains("available")) {
        return Some(("df -h / | tail -1 | awk '{print $4 \" free of \" $2}'", "Root partition: {output}"));
    }
    if q.contains("battery") && (q.contains("level") || q.contains("charge") || q.contains("status") || q.contains("how much")) {
        return Some(("cat /sys/class/power_supply/BAT*/capacity 2>/dev/null || echo 'No battery detected'", "Battery: {output}%"));
    }
    if (q.contains("temperature") || q.contains("temp") || q.contains("hot")) && (q.contains("cpu") || q.contains("system")) {
        return Some(("cat /sys/class/thermal/thermal_zone0/temp 2>/dev/null | awk '{print $1/1000 \"C\"}' || sensors 2>/dev/null | grep -m1 'Core 0' | awk '{print $3}'", "CPU temperature: {output}"));
    }
    if q.contains("usb") && (q.contains("device") || q.contains("connected") || q.contains("what")) {
        return Some(("lsusb", "USB devices:\n{output}"));
    }
    if (q.contains("resolution") || q.contains("display size")) && (q.contains("what") || q.contains("my") || q.contains("screen")) {
        return Some(("xrandr 2>/dev/null | grep '*' | awk '{print $1}' | head -1 || wlr-randr 2>/dev/null | grep current | awk '{print $1}'", "Display resolution: {output}"));
    }
    if q.contains("cpu") && q.contains("core") {
        return Some(("lscpu | grep -E 'Core\\(s\\) per socket|Socket\\(s\\)|Thread\\(s\\) per core' | head -3", "CPU topology: {output}"));
    }
    if q.contains("cpu") && q.contains("thread") {
        return Some(("nproc", "CPU threads (logical): {output}"));
    }
    if q.contains("cpu") && (q.contains("freq") || q.contains("speed") || q.contains("mhz") || q.contains("ghz")) {
        return Some(("lscpu | grep 'CPU MHz' | awk '{print $3}'", "CPU frequency: {output} MHz"));
    }
    if q.contains("ram") && q.contains("total") {
        return Some(("free -h | awk '/Mem:/ {print $2}'", "Total RAM: {output}"));
    }
    if q.contains("brightness") && (q.contains("screen") || q.contains("level") || q.contains("what")) {
        return Some(("cat /sys/class/backlight/*/brightness 2>/dev/null | head -1 || echo 'No backlight control'", "Brightness: {output}"));
    }
    if (q.contains("partition") || q.contains("disk")) && (q.contains("list") || q.contains("what") || q.contains("show")) {
        return Some(("lsblk -o NAME,SIZE,TYPE,MOUNTPOINT | head -20", "{output}"));
    }
    if q.contains("module") && (q.contains("loaded") || q.contains("kernel") || q.contains("how many")) {
        return Some(("lsmod | wc -l", "Kernel modules loaded: {output}"));
    }
    if q.contains("zram") {
        return Some(("zramctl 2>/dev/null || echo 'zram not configured'", "{output}"));
    }
    None
}

/// Network fast-path patterns.
pub fn get_network_fast_path(q: &str) -> Option<(&'static str, &'static str)> {
    if (q.contains("ip") || q.contains("address")) && (q.contains("local") || q.contains("my") || q.contains("what")) && !q.contains("public") {
        return Some(("ip -4 addr show | grep inet | grep -v 127.0.0.1 | awk '{print $2}'", "Local IP: {output}"));
    }
    if (q.contains("ip") || q.contains("address")) && q.contains("public") {
        return Some(("curl -s ifconfig.me 2>/dev/null || curl -s icanhazip.com", "Public IP: {output}"));
    }
    if q.contains("dns") && (q.contains("server") || q.contains("configured")) {
        return Some(("cat /etc/resolv.conf | grep nameserver", "DNS servers:\n{output}"));
    }
    if q.contains("port") && (q.contains("listen") || q.contains("open")) {
        return Some(("ss -tlnp 2>/dev/null || netstat -tlnp 2>/dev/null", "{output}"));
    }
    if (q.contains("wifi") || q.contains("wireless")) && (q.contains("connected") || q.contains("ssid") || q.contains("network") || q.contains("what")) {
        return Some(("iwgetid -r 2>/dev/null || nmcli -t -f active,ssid dev wifi | grep '^yes' | cut -d: -f2 || echo 'Not connected'", "WiFi: {output}"));
    }
    if (q.contains("wifi") || q.contains("wireless")) && q.contains("signal") {
        return Some(("iwconfig 2>/dev/null | grep 'Signal level' | awk -F'=' '{print $3}' || echo 'No wireless'", "Signal: {output}"));
    }
    if q.contains("network") && (q.contains("interface") || q.contains("adapter")) {
        return Some(("ip -br link | head -10", "Network interfaces:\n{output}"));
    }
    if q.contains("mac") && q.contains("address") {
        return Some(("ip link | grep -A1 'state UP' | grep ether | awk '{print $2}'", "MAC address: {output}"));
    }
    if q.contains("vpn") && (q.contains("connected") || q.contains("running") || q.contains("active")) {
        return Some(("nmcli -t -f NAME,TYPE,DEVICE con show --active | grep vpn || ip link | grep -E 'tun|wg' || echo 'No VPN detected'", "{output}"));
    }
    if q.contains("firewall") && (q.contains("enabled") || q.contains("running") || q.contains("status") || q.contains("active")) {
        return Some(("systemctl is-active ufw 2>/dev/null || systemctl is-active firewalld 2>/dev/null || iptables -L 2>/dev/null | head -5 || echo 'No firewall detected'", "Firewall: {output}"));
    }
    None
}

/// Package management fast-path patterns.
pub fn get_package_fast_path(q: &str) -> Option<(&'static str, &'static str)> {
    if q.contains("package") && (q.contains("how many") || q.contains("installed") || q.contains("count")) {
        return Some(("pacman -Q | wc -l", "You have {output} packages installed"));
    }
    if q.contains("orphan") && q.contains("package") {
        return Some(("pacman -Qtdq | wc -l", "Orphan packages: {output}"));
    }
    if q.contains("explicit") && q.contains("package") {
        return Some(("pacman -Qe | wc -l", "Explicitly installed packages: {output}"));
    }
    if q.contains("aur") && (q.contains("helper") || q.contains("what") || q.contains("which")) {
        return Some(("which yay paru 2>/dev/null | head -1 || echo 'No AUR helper found'", "AUR helper: {output}"));
    }
    if q.contains("recent") && q.contains("package") {
        return Some(("expac -Q --timefmt='%Y-%m-%d' '%l %n' | sort -r | head -10", "Recent packages:\n{output}"));
    }
    if q.contains("largest") && q.contains("package") {
        return Some(("expac -Q -H M '%m %n' | sort -rn | head -10", "Largest packages:\n{output}"));
    }
    if (q.contains("version") || q.contains("installed")) && q.contains("mesa") {
        return Some(("pacman -Q mesa 2>/dev/null || echo 'mesa not installed'", "{output}"));
    }
    if q.contains("installed") && (q.contains("linux-cachyos") || q.contains("cachyos")) {
        return Some(("pacman -Q linux-cachyos 2>/dev/null || echo 'linux-cachyos not installed'", "{output}"));
    }
    if q.contains("mirror") && (q.contains("arch") || q.contains("pacman") || q.contains("using")) {
        return Some(("head -1 /etc/pacman.d/mirrorlist | grep -v '#' || grep -m1 '^Server' /etc/pacman.d/mirrorlist", "Mirror: {output}"));
    }
    if q.contains("pacman") && (q.contains("sync") || q.contains("update") || q.contains("last")) && !q.contains("lock") {
        return Some(("stat -c %y /var/lib/pacman/sync/*.db | head -1 | cut -d. -f1", "Last sync: {output}"));
    }
    None
}

/// Service and storage fast-path patterns.
pub fn get_service_fast_path(q: &str) -> Option<(&'static str, &'static str)> {
    if q.contains("service") && (q.contains("failed") || q.contains("failing")) {
        return Some(("systemctl --failed --no-pager", "{output}"));
    }
    if (q.contains("audio") || q.contains("sound")) && (q.contains("what") || q.contains("which") || q.contains("using")) && !q.contains("problem") {
        return Some(("pactl info 2>/dev/null | grep 'Server Name' | cut -d: -f2 | xargs || echo 'PulseAudio/Pipewire not running'", "Audio server: {output}"));
    }
    if q.contains("ssh") && (q.contains("running") || q.contains("enabled") || q.contains("status")) {
        return Some(("systemctl is-active sshd 2>/dev/null || systemctl is-active ssh 2>/dev/null || echo 'not running'", "SSH: {output}"));
    }
    if q.contains("bluetooth") && (q.contains("enabled") || q.contains("status") || q.contains("running") || q.contains("on")) {
        return Some(("systemctl is-active bluetooth 2>/dev/null && bluetoothctl show 2>/dev/null | grep -E 'Powered|Name' || echo 'bluetooth service not running'", "{output}"));
    }
    if q.contains("docker") && (q.contains("container") || q.contains("running") || q.contains("list")) {
        return Some(("docker ps --format 'table {{.Names}}\t{{.Status}}' 2>/dev/null || echo 'Docker not running'", "{output}"));
    }
    if q.contains("journal") && (q.contains("size") || q.contains("how big") || q.contains("space")) {
        return Some(("journalctl --disk-usage 2>/dev/null", "{output}"));
    }
    if q.contains("nvidia") && (q.contains("driver") || q.contains("version") || q.contains("installed")) {
        return Some(("nvidia-smi --query-gpu=driver_version --format=csv,noheader 2>/dev/null || pacman -Q nvidia 2>/dev/null || echo 'NVIDIA driver not found'", "NVIDIA: {output}"));
    }
    if (q.contains("pipewire") || q.contains("pulseaudio")) && (q.contains("running") || q.contains("using") || q.contains("which")) {
        return Some(("pactl info 2>/dev/null | grep 'Server Name' | cut -d: -f2 | xargs || echo 'Not running'", "Audio: {output}"));
    }
    if q.contains("timer") && (q.contains("active") || q.contains("what") || q.contains("list")) {
        return Some(("systemctl list-timers --no-pager | head -15", "{output}"));
    }
    if q.contains("socket") && (q.contains("unit") || q.contains("listen")) {
        return Some(("systemctl list-sockets --no-pager | head -15", "{output}"));
    }
    if q.contains("bootloader") || (q.contains("grub") && q.contains("using")) || (q.contains("systemd-boot") && q.contains("using")) {
        return Some(("[ -d /sys/firmware/efi ] && (bootctl status 2>/dev/null | head -5 || echo 'EFI system, bootctl not available') || echo 'BIOS/Legacy boot'", "{output}"));
    }
    if q.contains("selinux") || q.contains("apparmor") {
        return Some(("cat /sys/kernel/security/lsm 2>/dev/null || echo 'No LSM detected'", "Security modules: {output}"));
    }
    if q.contains("btrfs") && q.contains("subvolume") {
        return Some(("btrfs subvolume list / 2>/dev/null || echo 'Not btrfs or no subvolumes'", "{output}"));
    }
    if q.contains("trim") && (q.contains("enabled") || q.contains("ssd")) {
        return Some(("systemctl is-enabled fstrim.timer 2>/dev/null || echo 'fstrim.timer not found'", "TRIM timer: {output}"));
    }
    if q.contains("lvm") && (q.contains("volume") || q.contains("any")) {
        return Some(("lvs 2>/dev/null || echo 'No LVM volumes'", "{output}"));
    }
    if q.contains("encrypt") && (q.contains("disk") || q.contains("luks")) {
        return Some(("lsblk -o NAME,FSTYPE,TYPE | grep -i crypt || echo 'No encrypted volumes detected'", "{output}"));
    }
    if (q.contains("gpt") || q.contains("mbr")) && (q.contains("disk") || q.contains("partition")) {
        return Some(("lsblk -o NAME,PTTYPE | head -5", "{output}"));
    }
    if q.contains("mount") && q.contains("/home") {
        return Some(("mount | grep /home || echo '/home is on root'", "{output}"));
    }
    if q.contains("uuid") && (q.contains("root") || q.contains("/")) {
        return Some(("lsblk -o NAME,UUID,MOUNTPOINT | grep -E '/$' | awk '{print $2}'", "Root UUID: {output}"));
    }
    if q.contains("suid") && q.contains("binar") {
        return Some(("find /usr/bin -perm -4000 2>/dev/null | wc -l", "SUID binaries: {output}"));
    }
    if q.contains("sudo") && q.contains("configured") {
        return Some(("groups | grep -q wheel && echo 'Yes (wheel group)' || echo 'Check /etc/sudoers'", "Sudo: {output}"));
    }
    None
}
