//! Ralph-style autonomous iteration loop for answering questions.
//!
//! The Ralph Wiggum approach: iteration beats perfection.
//! Instead of complex branching, use a simple loop with clear completion criteria.
//!
//! Principles:
//! 1. Define "done" upfront - what does success look like?
//! 2. Iterate until done - trust the loop, not complexity
//! 3. Self-evaluate - LLM checks its own work before declaring done
//! 4. Learn from attempts - each iteration improves the next
//!
//! v0.1.1: Initial implementation
//! v0.2.6: Added fast-path for common single-command queries
//! v0.2.9: Added team dispatch - IT department fly-on-the-wall experience
//! v0.3.0: Added automatic recipe learning from successful answers

use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anna_shared::recipe::{Recipe, RecipeBook, RecipeCommand, RecipeContext, RecipeSource};
use anyhow::Result;
use tracing::{debug, info, warn};

use crate::core_loop::{
    execute_command, strip_ansi_codes,
};
use crate::ollama;
use crate::department;
use crate::team_speak;

/// v0.2.7: Instant responses for well-known error patterns
/// These are common issues that have known solutions - no need to investigate
fn get_instant_error_response(question: &str) -> Option<&'static str> {
    let q = question.to_lowercase();

    // Pacman database lock
    if q.contains("pacman") && (q.contains("lock") || q.contains("locked")) {
        return Some(
            "The pacman database is locked. This usually happens when another package operation is running or crashed.\n\n\
            **Solution:**\n\
            1. Check if pacman is running: `pgrep -a pacman`\n\
            2. If not running, remove the lock: `sudo rm /var/lib/pacman/db.lck`\n\
            3. If pacman is running, wait for it to finish or kill it: `sudo pkill pacman`\n\n\
            **Note:** Only remove the lock if you're certain no package operation is in progress."
        );
    }

    // GPGME error
    if q.contains("gpgme") || (q.contains("gpg") && q.contains("no data")) {
        return Some(
            "GPGME/GPG 'No data' error usually means corrupted or missing package keys.\n\n\
            **Solution:**\n\
            1. Refresh keys: `sudo pacman-key --refresh-keys`\n\
            2. If that fails, reinitialize: `sudo pacman-key --init && sudo pacman-key --populate archlinux`\n\
            3. Update keyring: `sudo pacman -Sy archlinux-keyring`\n\n\
            **Note:** This can take a few minutes. If using CachyOS, also run `sudo pacman-key --populate cachyos`."
        );
    }

    // Deleted /usr/bin
    if q.contains("deleted") && q.contains("/usr/bin") {
        return Some(
            "Accidentally deleted /usr/bin is serious but recoverable.\n\n\
            **Recovery steps:**\n\
            1. Boot from Arch/CachyOS live USB\n\
            2. Mount your root partition: `mount /dev/sdXY /mnt`\n\
            3. Chroot: `arch-chroot /mnt`\n\
            4. Reinstall base packages: `pacman -S base base-devel`\n\
            5. Reinstall all explicitly installed packages: `pacman -Qeq | pacman -S -`\n\n\
            **Note:** If /usr/bin/bash is gone, use `busybox sh` from a rescue environment."
        );
    }

    // Forgot root password
    if q.contains("forgot") && (q.contains("root") || q.contains("password")) {
        return Some(
            "To reset root password:\n\n\
            1. Reboot and at GRUB, press 'e' to edit boot entry\n\
            2. Find the line starting with 'linux' and add `init=/bin/bash` at the end\n\
            3. Press Ctrl+X to boot\n\
            4. Remount root: `mount -o remount,rw /`\n\
            5. Set new password: `passwd`\n\
            6. Reboot: `exec /sbin/init` or `reboot -f`\n\n\
            **Note:** For systemd-boot, edit the loader entry similarly."
        );
    }

    // chmod 777 -R disaster
    if q.contains("chmod") && q.contains("777") && q.contains("/") {
        return Some(
            "Running chmod 777 -R on system directories is serious but recoverable.\n\n\
            **Recovery steps:**\n\
            1. Boot from live USB (system may not boot normally)\n\
            2. Mount your root partition: `mount /dev/sdXY /mnt`\n\
            3. Reinstall all packages to fix permissions:\n\
               `arch-chroot /mnt pacman -Qkk 2>&1 | grep 'Permissions mismatch' | awk '{print $2}' | xargs pacman -S --noconfirm`\n\
            4. Or reinstall everything: `pacman -Qnq | pacman -S --noconfirm -`\n\n\
            **Note:** This may take a while. Check /etc/shadow permissions manually: should be 640."
        );
    }

    // Kernel won't boot
    if (q.contains("won't boot") || q.contains("can't boot") || q.contains("not boot")) && q.contains("kernel") {
        return Some(
            "System won't boot after kernel update:\n\n\
            **Quick fix:**\n\
            1. At GRUB, select a previous kernel from 'Advanced options'\n\
            2. Or at boot, press 'e' and change kernel version in linux line\n\n\
            **From live USB:**\n\
            1. Mount root and chroot\n\
            2. Downgrade kernel: `pacman -U /var/cache/pacman/pkg/linux-<version>.pkg.tar.zst`\n\
            3. Or regenerate initramfs: `mkinitcpio -P`\n\n\
            **Note:** nvidia-dkms users should also downgrade nvidia drivers or wait for dkms rebuild."
        );
    }

    // Disk full can't login
    if q.contains("disk") && q.contains("full") && (q.contains("login") || q.contains("can't")) {
        return Some(
            "Disk full, can't login:\n\n\
            **Recovery:**\n\
            1. At login prompt, press Ctrl+Alt+F2 for TTY (may work)\n\
            2. Or boot with `systemd.unit=rescue.target` kernel param\n\
            3. Or use live USB and mount your partition\n\n\
            **Clear space:**\n\
            ```\n\
            sudo rm -rf /var/cache/pacman/pkg/*  # Clear package cache\n\
            sudo journalctl --vacuum-size=100M    # Trim logs\n\
            sudo rm -rf /tmp/*                    # Clear temp\n\
            ```\n\n\
            **Find big files:** `du -h / 2>/dev/null | sort -h | tail -20`"
        );
    }

    // Black screen / display manager won't start
    if (q.contains("black screen") || q.contains("display manager") || q.contains("dm won't start") || q.contains("sddm") || q.contains("gdm"))
        && (q.contains("won't") || q.contains("not") || q.contains("blank"))
    {
        return Some(
            "Display manager won't start / black screen:\n\n\
            **Quick diagnosis:**\n\
            1. Press Ctrl+Alt+F2 for TTY login\n\
            2. Check DM status: `systemctl status sddm` (or gdm/lightdm)\n\
            3. Check Xorg logs: `cat /var/log/Xorg.0.log | grep EE`\n\
            4. Check journal: `journalctl -b -p err | grep -i 'x11\\|wayland\\|nvidia\\|amd'`\n\n\
            **Common fixes:**\n\
            - Nvidia: reinstall drivers `pacman -S nvidia nvidia-dkms`\n\
            - Permissions: `chmod 0660 /dev/dri/*` \n\
            - Restart DM: `systemctl restart sddm`"
        );
    }

    // Electron apps blurry HiDPI
    if q.contains("electron") && (q.contains("blurry") || q.contains("hidpi") || q.contains("fuzzy") || q.contains("scaling")) {
        return Some(
            "Electron apps blurry on HiDPI:\n\n\
            **Solution:**\n\
            Add `--force-device-scale-factor=1.5` (adjust to your scale) to the app's .desktop file.\n\n\
            For system-wide: create `~/.config/electron-flags.conf`:\n\
            ```\n\
            --enable-features=UseOzonePlatform\n\
            --ozone-platform=wayland\n\
            ```\n\n\
            Or for X11:\n\
            ```\n\
            --force-device-scale-factor=1.5\n\
            ```\n\n\
            **Note:** VSCode, Discord, Slack all use Electron. Check per-app config too."
        );
    }

    // Steam games crash
    if q.contains("steam") && (q.contains("crash") || q.contains("won't") || q.contains("launch") || q.contains("error")) {
        return Some(
            "Steam games crashing:\n\n\
            **Common fixes:**\n\
            1. Enable Proton: Game -> Properties -> Compatibility -> Force Proton\n\
            2. Verify game files: Right-click -> Properties -> Local Files -> Verify\n\
            3. Try different Proton: Use Proton-GE from ProtonUp-Qt\n\
            4. Check dependencies: `pacman -S lib32-vulkan-icd-loader vulkan-tools`\n\n\
            **For native games:**\n\
            - Launch options: `LD_PRELOAD='' %command%`\n\
            - Missing libs: `ldd ~/.steam/steam/steamapps/common/Game/game.exe`\n\n\
            Check ProtonDB for game-specific fixes."
        );
    }

    // Docker DNS
    if q.contains("docker") && q.contains("dns") {
        return Some(
            "Docker containers can't resolve DNS:\n\n\
            **Fix 1 - Specify DNS in daemon config:**\n\
            Create/edit `/etc/docker/daemon.json`:\n\
            ```json\n\
            {\"dns\": [\"8.8.8.8\", \"1.1.1.1\"]}\n\
            ```\n\
            Then: `sudo systemctl restart docker`\n\n\
            **Fix 2 - For systemd-resolved users:**\n\
            ```bash\n\
            sudo ln -sf /run/systemd/resolve/resolv.conf /etc/resolv.conf\n\
            ```\n\n\
            **Fix 3 - Per-container:**\n\
            `docker run --dns 8.8.8.8 ...`"
        );
    }

    // Flatpak can't access home
    if q.contains("flatpak") && (q.contains("access") || q.contains("permission") || q.contains("home") || q.contains("folder")) {
        return Some(
            "Flatpak apps can't access home folder:\n\n\
            **Grant filesystem access:**\n\
            ```bash\n\
            flatpak override --user --filesystem=home com.app.Name\n\
            # Or for all apps:\n\
            flatpak override --user --filesystem=home\n\
            ```\n\n\
            **Using Flatseal (GUI):**\n\
            ```bash\n\
            flatpak install flathub com.github.tchx84.Flatseal\n\
            ```\n\
            Then enable 'All user files' in Flatseal for the app.\n\n\
            **Note:** Some apps need `--filesystem=/` for full access."
        );
    }

    // xdg-open wrong app
    if q.contains("xdg-open") && (q.contains("wrong") || q.contains("right") || q.contains("application") || q.contains("doesn't")) {
        return Some(
            "xdg-open doesn't open files with the right application:\n\n\
            **Check current associations:**\n\
            ```bash\n\
            xdg-mime query default text/html  # Example for HTML\n\
            ```\n\n\
            **Set default app:**\n\
            ```bash\n\
            xdg-mime default firefox.desktop text/html\n\
            xdg-mime default org.kde.dolphin.desktop inode/directory\n\
            ```\n\n\
            **Fix mimeapps.list:**\n\
            Edit `~/.config/mimeapps.list` and remove conflicting entries.\n\n\
            **Rebuild MIME database:**\n\
            `update-mime-database ~/.local/share/mime`"
        );
    }

    // Pipewire vs PulseAudio
    if (q.contains("pipewire") && q.contains("pulseaudio")) || (q.contains("audio") && q.contains("fighting")) {
        return Some(
            "Pipewire and PulseAudio conflict:\n\n\
            **Choose Pipewire (recommended):**\n\
            ```bash\n\
            sudo pacman -S pipewire pipewire-alsa pipewire-pulse pipewire-jack wireplumber\n\
            systemctl --user disable pulseaudio\n\
            systemctl --user enable pipewire pipewire-pulse wireplumber\n\
            systemctl --user start pipewire pipewire-pulse wireplumber\n\
            ```\n\n\
            **Or choose PulseAudio:**\n\
            ```bash\n\
            sudo pacman -Rns pipewire-pulse\n\
            sudo pacman -S pulseaudio\n\
            systemctl --user enable pulseaudio\n\
            ```\n\n\
            Reboot after switching."
        );
    }

    // GRUB rescue
    if q.contains("grub") && q.contains("rescue") {
        return Some(
            "GRUB rescue / unknown filesystem:\n\n\
            **At grub rescue prompt:**\n\
            ```\n\
            ls                      # List partitions\n\
            ls (hd0,gpt2)/          # Find your root (look for /boot)\n\
            set prefix=(hd0,gpt2)/boot/grub\n\
            set root=(hd0,gpt2)\n\
            insmod normal\n\
            normal\n\
            ```\n\n\
            **Permanent fix (from live USB):**\n\
            ```bash\n\
            mount /dev/sdXY /mnt\n\
            mount /dev/sdXZ /mnt/boot/efi  # If EFI\n\
            arch-chroot /mnt\n\
            grub-install --target=x86_64-efi --efi-directory=/boot/efi\n\
            grub-mkconfig -o /boot/grub/grub.cfg\n\
            ```"
        );
    }

    None
}

/// v0.2.7: Diagnostic commands for ambiguous queries
/// Returns (commands, intro_text) for running diagnostics
fn get_diagnostic_path(question: &str) -> Option<(&'static [&'static str], &'static str)> {
    let q = question.to_lowercase();

    // "it's slow" / "make it faster" / "system is slow"
    if (q.contains("slow") || q.contains("faster") || q.contains("laggy") || q.contains("sluggish"))
        && !q.contains("boot") && !q.contains("start")
    {
        return Some((
            &["uptime", "free -h", "top -bn1 | head -15", "df -h | grep -E '^/dev'"],
            "Running performance diagnostics to identify the bottleneck..."
        ));
    }

    // "fix my wifi" / "wifi not working" / "no internet"
    if q.contains("wifi") || q.contains("internet") || (q.contains("network") && !q.contains("what")) {
        return Some((
            &["ip link show", "ip -4 addr show", "ping -c 2 8.8.8.8 2>&1", "cat /etc/resolv.conf | grep nameserver"],
            "Checking network connectivity..."
        ));
    }

    // "something is wrong" / "nothing works" / "I broke something"
    if q.contains("something is wrong") || q.contains("nothing works") || q.contains("broke something")
        || q.contains("broken") || q.contains("check if everything")
    {
        return Some((
            &["systemctl --failed", "journalctl -p err -b --no-pager | head -20", "df -h | grep -E '^/dev'", "free -h"],
            "Running general health check..."
        ));
    }

    // "why won't it start" / "not starting" / "can't start"
    if (q.contains("won't start") || q.contains("not start") || q.contains("can't start") || q.contains("doesn't start"))
        && !q.contains("specific")
    {
        return Some((
            &["systemctl --failed", "journalctl -p err -b --no-pager | head -20", "dmesg | tail -20"],
            "Checking for startup failures..."
        ));
    }

    // "display is weird" / "screen problem"
    if (q.contains("display") || q.contains("screen") || q.contains("monitor"))
        && (q.contains("weird") || q.contains("problem") || q.contains("issue") || q.contains("wrong"))
    {
        return Some((
            &["echo $XDG_SESSION_TYPE", "xrandr 2>/dev/null || wlr-randr 2>/dev/null", "lsmod | grep -E 'nvidia|amdgpu|i915'", "journalctl -b | grep -iE 'drm|gpu' | tail -10"],
            "Checking display configuration..."
        ));
    }

    // "fan is loud" / "fan spinning"
    if q.contains("fan") && (q.contains("loud") || q.contains("spin") || q.contains("noise")) {
        return Some((
            &["cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null | head -5", "top -bn1 | head -10", "sensors 2>/dev/null | head -20"],
            "Checking CPU temperature and load..."
        ));
    }

    // "where did my files go" / "files missing"
    if (q.contains("files") || q.contains("folder") || q.contains("directory"))
        && (q.contains("gone") || q.contains("missing") || q.contains("where") || q.contains("disappeared"))
    {
        return Some((
            &["df -h | grep -E '^/dev'", "mount | grep -E '^/dev'", "ls -la ~ | head -15"],
            "Checking filesystem and mount points..."
        ));
    }

    // "help" alone
    if q.trim() == "help" || q.trim() == "help me" || q.trim() == "i need help" {
        return Some((
            &["systemctl --failed", "df -h | grep -E '^/dev'", "free -h"],
            "What can I help you with? Here's your system status:"
        ));
    }

    // "what's using bandwidth" / "bandwidth hog"
    if q.contains("bandwidth") || (q.contains("network") && q.contains("using")) {
        return Some((
            &["ss -tunp | head -20", "nethogs -t -c 3 2>/dev/null | head -15 || echo 'nethogs not installed - run: pacman -S nethogs'"],
            "Checking network usage..."
        ));
    }

    // "what's using CPU" / "CPU hog"
    if (q.contains("cpu") || q.contains("processor")) && (q.contains("using") || q.contains("hog") || q.contains("100%")) {
        return Some((
            &["top -bn1 | head -15", "ps aux --sort=-%cpu | head -10"],
            "Checking CPU usage..."
        ));
    }

    // "what's using memory/RAM"
    if (q.contains("memory") || q.contains("ram")) && (q.contains("using") || q.contains("hog") || q.contains("eating")) {
        return Some((
            &["free -h", "ps aux --sort=-%mem | head -10"],
            "Checking memory usage..."
        ));
    }

    // "why did X fail" / "last error"
    if (q.contains("why did") && q.contains("fail")) || q.contains("last error") || q.contains("recent error")
        || q.contains("what went wrong") || q.contains("what failed")
    {
        return Some((
            &["systemctl --failed", "journalctl -p err -b --no-pager | tail -20"],
            "Checking recent failures..."
        ));
    }

    // "is my system compromised" / "security check"
    if q.contains("compromised") || q.contains("hacked") || q.contains("security check") || q.contains("suspicious") {
        return Some((
            &["last -10", "who", "ss -tunp | grep ESTABLISHED | head -10", "find /tmp -type f -perm -111 2>/dev/null | head -5"],
            "Running basic security check..."
        ));
    }

    None
}

/// v0.2.6: Fast-path lookup table for common queries
/// Maps question patterns to (command, answer_template)
/// Template uses {output} placeholder for command output
fn get_fast_path(question: &str) -> Option<(&'static str, &'static str)> {
    let q = question.to_lowercase();

    // Kernel
    if q.contains("kernel") && (q.contains("version") || q.contains("running")) {
        return Some(("uname -r", "You are running kernel {output}"));
    }

    // Uptime
    if q.contains("uptime") || (q.contains("how long") && q.contains("up")) {
        return Some(("uptime -p", "System uptime: {output}"));
    }

    // RAM/Memory
    if (q.contains("ram") || q.contains("memory")) && (q.contains("how much") || q.contains("total") || q.contains("available")) {
        return Some(("free -h | grep Mem", "{output}"));
    }

    // Shell
    if q.contains("shell") && (q.contains("what") || q.contains("which") || q.contains("using")) {
        return Some(("echo $SHELL", "Your shell is {output}"));
    }

    // Desktop/WM
    if (q.contains("desktop") || q.contains("window manager") || q.contains("de") || q.contains("wm"))
        && (q.contains("what") || q.contains("which") || q.contains("running") || q.contains("using"))
    {
        return Some(("echo $XDG_CURRENT_DESKTOP", "You are running {output}"));
    }

    // Display server
    if (q.contains("wayland") || q.contains("x11") || q.contains("display server") || q.contains("xorg"))
        && (q.contains("what") || q.contains("which") || q.contains("using"))
    {
        return Some(("echo $XDG_SESSION_TYPE", "Display server: {output}"));
    }

    // Hostname
    if q.contains("hostname") && (q.contains("what") || q.contains("my")) {
        return Some(("hostname", "Hostname: {output}"));
    }

    // Username/UID
    if (q.contains("username") || q.contains("user") || q.contains("uid"))
        && (q.contains("what") || q.contains("my") || q.contains("current"))
    {
        return Some(("id", "{output}"));
    }

    // Groups
    if q.contains("groups") && (q.contains("what") || q.contains("member") || q.contains("my")) {
        return Some(("groups", "Your groups: {output}"));
    }

    // Timezone
    if q.contains("timezone") && (q.contains("what") || q.contains("configured") || q.contains("my")) {
        return Some(("timedatectl | grep 'Time zone'", "{output}"));
    }

    // Locale
    if q.contains("locale") && (q.contains("what") || q.contains("my") || q.contains("system")) {
        return Some(("locale | head -5", "{output}"));
    }

    // Swap
    if q.contains("swap") && (q.contains("configured") || q.contains("using") || q.contains("how much")) {
        return Some(("swapon --show", "{output}"));
    }

    // Package count
    if q.contains("package") && (q.contains("how many") || q.contains("installed") || q.contains("count")) {
        return Some(("pacman -Q | wc -l", "You have {output} packages installed"));
    }

    // Failed services
    if q.contains("service") && (q.contains("failed") || q.contains("failing")) {
        return Some(("systemctl --failed --no-pager", "{output}"));
    }

    // IP address
    if (q.contains("ip") || q.contains("address")) && (q.contains("local") || q.contains("my") || q.contains("what")) && !q.contains("public") {
        return Some(("ip -4 addr show | grep inet | grep -v 127.0.0.1 | awk '{print $2}'", "Local IP: {output}"));
    }

    // Public IP
    if (q.contains("ip") || q.contains("address")) && q.contains("public") {
        return Some(("curl -s ifconfig.me 2>/dev/null || curl -s icanhazip.com", "Public IP: {output}"));
    }

    // GPU
    if q.contains("gpu") && (q.contains("what") || q.contains("which") || q.contains("have") || q.contains("using")) {
        return Some(("lspci | grep -i vga", "GPU: {output}"));
    }

    // CPU
    if q.contains("cpu") && (q.contains("what") || q.contains("which") || q.contains("have") || q.contains("model")) && !q.contains("using") {
        return Some(("lscpu | grep 'Model name' | cut -d: -f2 | xargs", "CPU: {output}"));
    }

    // Distro / OS
    if (q.contains("distro") || q.contains("distribution") || q.contains("os ") || q.contains("operating system"))
        && (q.contains("what") || q.contains("which") || q.contains("running"))
    {
        return Some(("cat /etc/os-release | grep PRETTY_NAME | cut -d= -f2 | tr -d '\"'", "OS: {output}"));
    }

    // Disk space
    if (q.contains("disk") || q.contains("storage")) && (q.contains("space") || q.contains("free") || q.contains("available")) {
        return Some(("df -h / | tail -1 | awk '{print $4 \" free of \" $2}'", "Root partition: {output}"));
    }

    // Boot time
    if (q.contains("boot") || q.contains("startup")) && (q.contains("time") || q.contains("how long") || q.contains("fast")) {
        return Some(("systemd-analyze | head -1", "{output}"));
    }

    // Current user
    if q.contains("user") && (q.contains("who am i") || q.contains("logged") || q.contains("current")) {
        return Some(("whoami", "You are logged in as: {output}"));
    }

    // Audio system
    if (q.contains("audio") || q.contains("sound")) && (q.contains("what") || q.contains("which") || q.contains("using")) && !q.contains("problem") {
        return Some(("pactl info 2>/dev/null | grep 'Server Name' | cut -d: -f2 | xargs || echo 'PulseAudio/Pipewire not running'", "Audio server: {output}"));
    }

    // AUR helper
    if q.contains("aur") && (q.contains("helper") || q.contains("what") || q.contains("which")) {
        return Some(("which yay paru 2>/dev/null | head -1 || echo 'No AUR helper found'", "AUR helper: {output}"));
    }

    // Battery
    if q.contains("battery") && (q.contains("level") || q.contains("charge") || q.contains("status") || q.contains("how much")) {
        return Some(("cat /sys/class/power_supply/BAT*/capacity 2>/dev/null || echo 'No battery detected'", "Battery: {output}%"));
    }

    // Temperature
    if (q.contains("temperature") || q.contains("temp") || q.contains("hot")) && (q.contains("cpu") || q.contains("system")) {
        return Some(("cat /sys/class/thermal/thermal_zone0/temp 2>/dev/null | awk '{print $1/1000 \"C\"}' || sensors 2>/dev/null | grep -m1 'Core 0' | awk '{print $3}'", "CPU temperature: {output}"));
    }

    // Load average
    if q.contains("load") && (q.contains("average") || q.contains("system")) {
        return Some(("uptime | awk -F'load average:' '{print $2}'", "Load average:{output}"));
    }

    // Process count
    if q.contains("process") && (q.contains("how many") || q.contains("running") || q.contains("count")) {
        return Some(("ps aux | wc -l", "Running processes: {output}"));
    }

    // Orphan packages
    if q.contains("orphan") && q.contains("package") {
        return Some(("pacman -Qtdq | wc -l", "Orphan packages: {output}"));
    }

    // Explicitly installed packages
    if q.contains("explicit") && q.contains("package") {
        return Some(("pacman -Qe | wc -l", "Explicitly installed packages: {output}"));
    }

    // Resolution
    if (q.contains("resolution") || q.contains("display size")) && (q.contains("what") || q.contains("my") || q.contains("screen")) {
        return Some(("xrandr 2>/dev/null | grep '*' | awk '{print $1}' | head -1 || wlr-randr 2>/dev/null | grep current | awk '{print $1}'", "Display resolution: {output}"));
    }

    // Hostname
    if q.contains("hostname") || (q.contains("computer") && q.contains("name")) {
        return Some(("hostname", "Hostname: {output}"));
    }

    // v0.3.2: Additional fast-paths based on 100-question test analysis

    // Listening ports
    if q.contains("port") && (q.contains("listen") || q.contains("open")) {
        return Some(("ss -tlnp 2>/dev/null || netstat -tlnp 2>/dev/null", "{output}"));
    }

    // USB devices
    if q.contains("usb") && (q.contains("device") || q.contains("connected") || q.contains("what")) {
        return Some(("lsusb", "USB devices:\n{output}"));
    }

    // DNS servers
    if q.contains("dns") && (q.contains("server") || q.contains("configured")) {
        return Some(("cat /etc/resolv.conf | grep nameserver", "DNS servers:\n{output}"));
    }

    // Mount points
    if q.contains("mount") && q.contains("/home") {
        return Some(("mount | grep /home || echo '/home is on root'", "{output}"));
    }

    // GPT/MBR partition table
    if (q.contains("gpt") || q.contains("mbr")) && (q.contains("disk") || q.contains("partition")) {
        return Some(("lsblk -o NAME,PTTYPE | head -5", "{output}"));
    }

    // Active timers
    if q.contains("timer") && (q.contains("active") || q.contains("what") || q.contains("list")) {
        return Some(("systemctl list-timers --no-pager | head -15", "{output}"));
    }

    // Socket units
    if q.contains("socket") && (q.contains("unit") || q.contains("listen")) {
        return Some(("systemctl list-sockets --no-pager | head -15", "{output}"));
    }

    // Systemd default target
    if q.contains("default") && (q.contains("target") || q.contains("runlevel")) {
        return Some(("systemctl get-default", "Default target: {output}"));
    }

    // SELinux/AppArmor status
    if q.contains("selinux") || q.contains("apparmor") {
        return Some(("cat /sys/kernel/security/lsm 2>/dev/null || echo 'No LSM detected'", "Security modules: {output}"));
    }

    // Package version queries
    if (q.contains("version") || q.contains("installed")) && q.contains("mesa") {
        return Some(("pacman -Q mesa 2>/dev/null || echo 'mesa not installed'", "{output}"));
    }

    // Is package installed (generic)
    if q.contains("installed") && (q.contains("linux-cachyos") || q.contains("cachyos")) {
        return Some(("pacman -Q linux-cachyos 2>/dev/null || echo 'linux-cachyos not installed'", "{output}"));
    }

    // btrfs subvolumes
    if q.contains("btrfs") && q.contains("subvolume") {
        return Some(("btrfs subvolume list / 2>/dev/null || echo 'Not btrfs or no subvolumes'", "{output}"));
    }

    // TRIM status
    if q.contains("trim") && (q.contains("enabled") || q.contains("ssd")) {
        return Some(("systemctl is-enabled fstrim.timer 2>/dev/null || echo 'fstrim.timer not found'", "TRIM timer: {output}"));
    }

    // LVM volumes
    if q.contains("lvm") && (q.contains("volume") || q.contains("any")) {
        return Some(("lvs 2>/dev/null || echo 'No LVM volumes'", "{output}"));
    }

    // Disk encryption
    if q.contains("encrypt") && (q.contains("disk") || q.contains("luks")) {
        return Some(("lsblk -o NAME,FSTYPE,TYPE | grep -i crypt || echo 'No encrypted volumes detected'", "{output}"));
    }

    // Recently installed packages
    if q.contains("recent") && q.contains("package") {
        return Some(("expac -Q --timefmt='%Y-%m-%d' '%l %n' | sort -r | head -10", "Recent packages:\n{output}"));
    }

    // Largest packages
    if q.contains("largest") && q.contains("package") {
        return Some(("expac -Q -H M '%m %n' | sort -rn | head -10", "Largest packages:\n{output}"));
    }

    // zram status
    if q.contains("zram") {
        return Some(("zramctl 2>/dev/null || echo 'zram not configured'", "{output}"));
    }

    // MAC address
    if q.contains("mac") && q.contains("address") {
        return Some(("ip link | grep -A1 'state UP' | grep ether | awk '{print $2}'", "MAC address: {output}"));
    }

    // Root UUID
    if q.contains("uuid") && (q.contains("root") || q.contains("/")) {
        return Some(("lsblk -o NAME,UUID,MOUNTPOINT | grep -E '/$' | awk '{print $2}'", "Root UUID: {output}"));
    }

    // SUID binaries
    if q.contains("suid") && q.contains("binar") {
        return Some(("find /usr/bin -perm -4000 2>/dev/null | wc -l", "SUID binaries: {output}"));
    }

    // Sudo configured
    if q.contains("sudo") && q.contains("configured") {
        return Some(("groups | grep -q wheel && echo 'Yes (wheel group)' || echo 'Check /etc/sudoers'", "Sudo: {output}"));
    }

    // CPU frequency
    if q.contains("cpu") && (q.contains("freq") || q.contains("speed") || q.contains("mhz") || q.contains("ghz")) {
        return Some(("lscpu | grep 'CPU MHz' | awk '{print $3}'", "CPU frequency: {output} MHz"));
    }

    // Audio server type
    if (q.contains("pipewire") || q.contains("pulseaudio")) && (q.contains("running") || q.contains("using") || q.contains("which")) {
        return Some(("pactl info 2>/dev/null | grep 'Server Name' | cut -d: -f2 | xargs || echo 'Not running'", "Audio: {output}"));
    }

    None
}

/// v0.2.7: Try instant error response for known issues
fn try_instant_error(question: &str) -> Option<AskResult> {
    let answer = get_instant_error_response(question)?;

    info!("Instant response: known error pattern matched");

    Some(AskResult {
        answer: answer.to_string(),
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![
            DialogueStep {
                step_type: StepType::UserQuestion,
                content: question.to_string(),
            },
            DialogueStep {
                step_type: StepType::FinalAnswer,
                content: answer.to_string(),
            },
        ],
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
    })
}

/// v0.2.7: Try diagnostic path for ambiguous queries
/// Runs pre-selected diagnostics instead of asking for clarification
fn try_diagnostic_path(question: &str) -> Option<(Vec<String>, Vec<String>, &'static str, Vec<DialogueStep>)> {
    let (commands, intro) = get_diagnostic_path(question)?;

    info!("Diagnostic path: running {} commands", commands.len());

    let mut outputs = Vec::new();
    let mut executed = Vec::new();
    let mut dialogue = vec![
        DialogueStep {
            step_type: StepType::UserQuestion,
            content: question.to_string(),
        },
    ];

    // v0.3.3: Add fly-on-the-wall elements to diagnostic path
    let dept_name = department::determine_department(question);
    let mut ticket = department::create_ticket(question, dept_name);

    dialogue.push(DialogueStep {
        step_type: StepType::TicketCreated,
        content: ticket.case_number.clone(),
    });

    if let Some(spec) = department::get_specialist_for_topic(question) {
        // v0.3.5: Assign ticket to specialist for stats tracking
        ticket.assign(spec.name);
        department::update_ticket(&ticket);

        let assignment = team_speak::anna_assigns_to(spec, question);
        dialogue.push(DialogueStep {
            step_type: StepType::TeamAssignment,
            content: assignment,
        });
        let ack = team_speak::specialist_acknowledges(spec);
        dialogue.push(DialogueStep {
            step_type: StepType::SpecialistWorking,
            content: format!("{}: {}", spec.name, ack),
        });
    }

    for cmd in commands {
        dialogue.push(DialogueStep {
            step_type: StepType::CommandExec,
            content: cmd.to_string(),
        });

        match execute_command(cmd) {
            Ok(output) => {
                let clean = strip_ansi_codes(&output);
                dialogue.push(DialogueStep {
                    step_type: StepType::CommandOutput,
                    content: truncate(&clean, 500),
                });
                outputs.push(clean);
                executed.push(cmd.to_string());
            }
            Err(e) => {
                dialogue.push(DialogueStep {
                    step_type: StepType::CommandOutput,
                    content: format!("Error: {}", e),
                });
            }
        }
    }

    Some((executed, outputs, intro, dialogue))
}

/// v0.2.6: Try fast-path for simple queries
async fn try_fast_path(question: &str) -> Option<AskResult> {
    let (cmd, template) = get_fast_path(question)?;

    info!("Fast-path: using command '{}'", cmd);

    match execute_command(cmd) {
        Ok(output) => {
            let clean_output = strip_ansi_codes(&output).trim().to_string();
            if clean_output.is_empty() {
                return None; // Fall back to full loop
            }

            let answer = template.replace("{output}", &clean_output);

            // v0.3.5: Track ticket and specialist for stats even on fast-path
            let dept_name = department::determine_department(question);
            let mut ticket = department::create_ticket(question, dept_name);
            if let Some(spec) = department::get_specialist_for_topic(question) {
                ticket.assign(spec.name);
            }
            ticket.resolve(&answer, 5); // Fast-path = 5 XP
            department::update_ticket(&ticket);

            // v0.3.6: Add citation for the command that grounded this answer
            let citation = anna_shared::rpc::Citation {
                source: format!("Command: {}", cmd),
                url: None,
                section: None,
            };

            Some(AskResult {
                answer,
                success: true,
                iterations: 0,
                commands_executed: vec![cmd.to_string()],
                dialogue: vec![
                    DialogueStep {
                        step_type: StepType::UserQuestion,
                        content: question.to_string(),
                    },
                    DialogueStep {
                        step_type: StepType::CommandExec,
                        content: cmd.to_string(),
                    },
                    DialogueStep {
                        step_type: StepType::CommandOutput,
                        content: clean_output,
                    },
                ],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![citation],
            })
        }
        Err(_) => None, // Fall back to full loop
    }
}

/// Completion criteria for a question
#[derive(Debug, Clone)]
pub struct CompletionCriteria {
    /// What type of answer is expected
    pub answer_type: AnswerType,
    /// Minimum confidence threshold (0.0 - 1.0)
    pub min_confidence: f32,
    /// Maximum iterations before giving up
    pub max_iterations: u32,
    /// Whether grounding in command output is required
    pub requires_grounding: bool,
}

impl Default for CompletionCriteria {
    fn default() -> Self {
        Self {
            answer_type: AnswerType::Factual,
            min_confidence: 0.7,
            max_iterations: 5,
            requires_grounding: true,
        }
    }
}

/// Types of answers Anna can provide
#[derive(Debug, Clone)]
pub enum AnswerType {
    /// Factual information from the system (requires command output)
    Factual,
    /// How-to instructions (may cite wiki/docs)
    HowTo,
    /// Troubleshooting help (requires diagnosis)
    Troubleshoot,
    /// Simple acknowledgment or clarification
    Simple,
}

/// State of an iteration attempt
#[derive(Debug)]
struct IterationState {
    /// Commands executed so far
    commands: Vec<String>,
    /// Outputs collected
    outputs: Vec<String>,
    /// Current answer draft
    answer: Option<String>,
    /// Confidence in current answer
    confidence: f32,
    /// Feedback from previous iteration
    feedback: Option<String>,
    /// Why we're not done yet
    not_done_reason: Option<String>,
}

impl Default for IterationState {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            outputs: Vec::new(),
            answer: None,
            confidence: 0.0,
            feedback: None,
            not_done_reason: None,
        }
    }
}

/// Quick quality check for answers (no LLM needed)
fn quick_quality_check(answer: &str) -> bool {
    let answer = answer.trim();

    // Too short
    if answer.len() < 10 {
        return false;
    }

    // Obvious refusals
    let refusals = ["i cannot", "i can't", "i'm not able", "i don't know"];
    if refusals.iter().any(|r| answer.to_lowercase().contains(r)) {
        return false;
    }

    // Prompt leakage
    let leakage = ["as an ai", "as a language model", "i'm an ai"];
    if leakage.iter().any(|l| answer.to_lowercase().contains(l)) {
        return false;
    }

    true
}

/// Result of self-evaluation
#[derive(Debug)]
struct SelfEvaluation {
    /// Is the answer complete?
    is_complete: bool,
    /// Confidence score (0.0 - 1.0)
    confidence: f32,
    /// What's missing if not complete
    missing: Option<String>,
    /// Suggestions for improvement
    suggestions: Option<String>,
}

/// Determine completion criteria based on the question
pub fn determine_criteria(question: &str) -> CompletionCriteria {
    let q = question.to_lowercase();

    // HowTo questions - instructions, don't need live output
    if q.contains("how do i")
        || q.contains("how to")
        || q.contains("how can i")
        || q.starts_with("install")
        || q.starts_with("setup")
        || q.starts_with("configure")
    {
        return CompletionCriteria {
            answer_type: AnswerType::HowTo,
            min_confidence: 0.6,
            max_iterations: 3,
            requires_grounding: false, // Instructions don't need live data
        };
    }

    // Troubleshooting - needs diagnosis
    if q.contains("not working")
        || q.contains("error")
        || q.contains("failed")
        || q.contains("problem")
        || q.contains("broken")
        || q.contains("fix")
        || q.contains("why")
    {
        return CompletionCriteria {
            answer_type: AnswerType::Troubleshoot,
            min_confidence: 0.7,
            max_iterations: 5,
            requires_grounding: true,
        };
    }

    // Simple questions
    if q.len() < 30 && !q.contains("?") {
        return CompletionCriteria {
            answer_type: AnswerType::Simple,
            min_confidence: 0.5,
            max_iterations: 2,
            requires_grounding: false,
        };
    }

    // Default: Factual query
    CompletionCriteria {
        answer_type: AnswerType::Factual,
        min_confidence: 0.7,
        max_iterations: 5,
        requires_grounding: true,
    }
}

/// The Ralph loop: iterate until done
///
/// This is the core of the Ralph approach:
/// 1. Determine what "done" looks like
/// 2. Loop: attempt answer, self-evaluate, improve
/// 3. Stop when criteria met or max iterations reached
pub async fn ralph_loop(model: &str, question: &str) -> Result<AskResult> {
    // v0.2.7: Try instant error response first for known issues
    if let Some(result) = try_instant_error(question) {
        info!("Instant error response completed");
        return Ok(result);
    }

    // v0.2.6: Try fast-path first for simple queries
    if let Some(result) = try_fast_path(question).await {
        info!("Fast-path completed in 0 iterations");
        return Ok(result);
    }

    // v0.2.7: Try diagnostic path for ambiguous queries
    if let Some((executed, outputs, intro, mut dialogue)) = try_diagnostic_path(question) {
        info!("Diagnostic path: analyzing {} outputs", outputs.len());

        // Use LLM to interpret the diagnostic results
        let data_context = outputs.join("\n---\n");
        let prompt = format!(
            r#"You are Anna, an AI assistant for Arch Linux systems.

The user asked: "{}"

I ran diagnostic commands. Here are the results:
{}

Based on these diagnostics, provide a helpful analysis. Be specific:
- If there's a problem, explain what it is and how to fix it
- If everything looks normal, say so with specific evidence
- Reference actual values from the output

Be concise but complete. Start your response with "{}" (without quotes)."#,
            question, data_context, intro
        );

        match ollama::chat_with_timeout(model, &prompt, 60).await {
            Ok(answer) => {
                dialogue.push(DialogueStep {
                    step_type: StepType::FinalAnswer,
                    content: answer.clone(),
                });

                return Ok(AskResult {
                    answer,
                    success: true,
                    iterations: 1,
                    commands_executed: executed,
                    dialogue,
                    needs_clarification: false,
                    clarification_question: None,
                    cached: false,
                    citations: vec![],
                });
            }
            Err(e) => {
                warn!("Diagnostic path LLM failed: {}, falling back to normal loop", e);
                // Fall through to normal loop
            }
        }
    }

    let criteria = determine_criteria(question);
    info!(
        "Ralph loop: {:?}, confidence >= {:.0}%, max {} iterations",
        criteria.answer_type, criteria.min_confidence * 100.0, criteria.max_iterations
    );

    let mut state = IterationState::default();
    let mut dialogue = Vec::new();
    let mut iteration = 0;

    // Record the question
    dialogue.push(DialogueStep {
        step_type: StepType::UserQuestion,
        content: question.to_string(),
    });

    // THE RALPH LOOP
    while iteration < criteria.max_iterations {
        iteration += 1;
        info!("Ralph iteration {}/{}", iteration, criteria.max_iterations);

        // Step 1: Get commands to run (or more commands if we have feedback)
        let commands = get_commands(model, question, &state).await?;

        if commands.is_empty() && state.outputs.is_empty() {
            // No commands needed - generate direct answer
            debug!("No commands needed, generating direct answer");
        } else if !commands.is_empty() {
            // Execute commands
            for cmd in &commands {
                dialogue.push(DialogueStep {
                    step_type: StepType::CommandExec,
                    content: cmd.clone(),
                });

                match execute_command(cmd) {
                    Ok(output) => {
                        let clean_output = strip_ansi_codes(&output);
                        state.commands.push(cmd.clone());
                        state.outputs.push(clean_output.clone());
                        dialogue.push(DialogueStep {
                            step_type: StepType::CommandOutput,
                            content: truncate(&clean_output, 500),
                        });
                    }
                    Err(e) => {
                        debug!("Command failed: {}: {}", cmd, e);
                        state.feedback = Some(format!("Command '{}' failed: {}", cmd, e));
                    }
                }
            }
        }

        // Step 2: Generate answer based on collected data
        let answer = generate_answer(model, question, &state, &criteria).await?;
        state.answer = Some(answer.clone());

        // Step 3: Self-evaluate - is this answer good enough?
        let eval = self_evaluate(model, question, &answer, &state, &criteria).await?;
        state.confidence = eval.confidence;

        debug!(
            "Self-evaluation: complete={}, confidence={:.0}%",
            eval.is_complete, eval.confidence * 100.0
        );

        // Step 4: Check completion criteria
        if eval.is_complete && eval.confidence >= criteria.min_confidence {
            info!(
                "Ralph done! Confidence {:.0}% >= {:.0}% threshold",
                eval.confidence * 100.0,
                criteria.min_confidence * 100.0
            );

            dialogue.push(DialogueStep {
                step_type: StepType::FinalAnswer,
                content: answer.clone(),
            });

            return Ok(AskResult {
                answer,
                success: true,
                iterations: iteration,
                commands_executed: state.commands,
                dialogue,
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
            });
        }

        // Not done yet - prepare feedback for next iteration
        state.feedback = eval.suggestions;
        state.not_done_reason = eval.missing;
        info!(
            "Not done yet: {:?}",
            state.not_done_reason.as_deref().unwrap_or("confidence too low")
        );
    }

    // Max iterations reached - return best effort
    warn!(
        "Ralph max iterations reached, returning best effort (confidence: {:.0}%)",
        state.confidence * 100.0
    );

    let final_answer = state.answer.unwrap_or_else(|| {
        "I wasn't able to fully answer your question. Please try rephrasing or ask about something more specific.".to_string()
    });

    dialogue.push(DialogueStep {
        step_type: StepType::FinalAnswer,
        content: final_answer.clone(),
    });

    Ok(AskResult {
        answer: final_answer,
        success: state.confidence >= 0.5,
        iterations: iteration,
        commands_executed: state.commands,
        dialogue,
        needs_clarification: state.confidence < 0.3, // v0.1.6: Lowered from 0.5 to reduce note spam
        clarification_question: state.not_done_reason,
        cached: false,
        citations: vec![],
    })
}

/// Get commands to run for answering the question
async fn get_commands(
    model: &str,
    question: &str,
    state: &IterationState,
) -> Result<Vec<String>> {
    let feedback_context = if let Some(ref feedback) = state.feedback {
        format!(
            "\n\nPrevious attempt feedback: {}\nAlready tried: {:?}",
            feedback, state.commands
        )
    } else {
        String::new()
    };

    let output_context = if !state.outputs.is_empty() {
        format!(
            "\n\nData collected so far:\n{}",
            state.outputs.join("\n---\n")
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"System: Arch Linux with pacman

Question: "{}"{}{}

Return 1-3 bash commands to answer this question. Use these exact commands:

SYSTEM: uname -r, uptime -p, hostnamectl
HARDWARE: lscpu | head -20, free -h, lsusb, lspci | head -20
DESKTOP: echo $XDG_CURRENT_DESKTOP, echo $XDG_SESSION_TYPE
USER: id, groups, echo $SHELL, locale, timedatectl | grep "Time zone"
STORAGE: df -h, lsblk, findmnt / -o OPTIONS, swapon --show
NETWORK: ip -4 addr show, cat /etc/resolv.conf, ip route | grep default, ss -tlnp | head -15
SERVICES: systemctl --failed, systemctl list-units --type=service --state=running | head -20
PACKAGES: pacman -Q | wc -l, pacman -Qe | head -30, pacman -Qtdq
LOGS: journalctl -p err -b --no-pager | head -30

RULES:
- Output ONLY valid bash commands, one per line
- NO explanations, NO English text, NO comments
- If question already answered by data below, output: DONE
- If question needs no commands (how-to), output: NONE

Output commands now:"#,
        question, output_context, feedback_context
    );

    let response = ollama::chat_with_timeout(model, &prompt, 30).await?;
    let response = response.trim();

    // Check for special responses (case-insensitive)
    let response_upper = response.to_uppercase();
    if response_upper == "NONE" || response_upper == "DONE" || response.is_empty() {
        return Ok(Vec::new());
    }

    let commands: Vec<String> = response
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            if l.is_empty() || l.starts_with('#') {
                return false;
            }
            // Filter out DONE/NONE even if mixed with other commands
            let upper = l.to_uppercase();
            if upper == "DONE" || upper == "NONE" || upper.starts_with("DONE:") {
                return false;
            }
            true
        })
        .map(|l| l.to_string())
        .take(5) // Max 5 commands per iteration
        .collect();

    Ok(commands)
}

/// Generate an answer based on collected data
async fn generate_answer(
    model: &str,
    question: &str,
    state: &IterationState,
    criteria: &CompletionCriteria,
) -> Result<String> {
    let data_context = if state.outputs.is_empty() {
        "No command output available.".to_string()
    } else {
        state.outputs.join("\n---\n")
    };

    let grounding_instruction = if criteria.requires_grounding {
        "Base your answer ONLY on the data above. Do not make up information."
    } else {
        "You may provide general guidance based on your knowledge."
    };

    // v0.1.4: Always include system context
    let prompt = format!(
        r#"You are Anna, an AI assistant for Arch Linux systems.
This is an Arch Linux system using pacman for packages.
Do NOT suggest apt, brew, or other package managers.

Question: {}

Data collected:
{}

{}

Provide a clear, helpful answer. Be concise but complete."#,
        question, data_context, grounding_instruction
    );

    let answer = ollama::chat_with_timeout(model, &prompt, 60).await?;
    Ok(answer.trim().to_string())
}

/// Self-evaluate the answer - is it good enough?
async fn self_evaluate(
    model: &str,
    question: &str,
    answer: &str,
    state: &IterationState,
    criteria: &CompletionCriteria,
) -> Result<SelfEvaluation> {
    // Quick heuristic checks first
    if answer.len() < 20 {
        return Ok(SelfEvaluation {
            is_complete: false,
            confidence: 0.2,
            missing: Some("Answer too short".to_string()),
            suggestions: Some("Provide more detail".to_string()),
        });
    }

    // Check quality heuristics
    if !quick_quality_check(answer) {
        return Ok(SelfEvaluation {
            is_complete: false,
            confidence: 0.3,
            missing: Some("Answer quality check failed".to_string()),
            suggestions: Some("Regenerate with better grounding".to_string()),
        });
    }

    // For simple/HowTo questions, skip LLM evaluation
    if matches!(criteria.answer_type, AnswerType::Simple | AnswerType::HowTo) {
        return Ok(SelfEvaluation {
            is_complete: true,
            confidence: 0.8,
            missing: None,
            suggestions: None,
        });
    }

    // LLM self-evaluation for complex questions
    let data_summary = if state.outputs.is_empty() {
        "No data collected".to_string()
    } else {
        format!("{} command outputs collected", state.outputs.len())
    };

    let prompt = format!(
        r#"Evaluate this answer:

Question: {}
Answer: {}
Data: {}

Rate on these criteria:
1. Does it directly answer the question? (YES/NO)
2. Is it grounded in the data collected? (YES/NO/NA)
3. Is anything important missing? (describe or NONE)

Format: COMPLETE/INCOMPLETE, CONFIDENCE (0-100), MISSING: <text>"#,
        question, answer, data_summary
    );

    let response = ollama::chat_with_timeout(model, &prompt, 20).await?;
    let response = response.to_uppercase();

    // Parse response
    let is_complete = response.contains("COMPLETE") && !response.contains("INCOMPLETE");

    let confidence = if let Some(conf_match) = response
        .split_whitespace()
        .find(|w| w.parse::<f32>().is_ok())
    {
        conf_match.parse::<f32>().unwrap_or(50.0) / 100.0
    } else if is_complete {
        0.8
    } else {
        0.4
    };

    let missing = if response.contains("MISSING:") {
        response
            .split("MISSING:")
            .nth(1)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "NONE")
    } else {
        None
    };

    Ok(SelfEvaluation {
        is_complete,
        confidence: confidence.clamp(0.0, 1.0),
        missing: missing.clone(),
        suggestions: missing,
    })
}

/// Streaming version of the Ralph loop
/// Sends progress updates to the client in real-time
pub async fn ralph_loop_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    writer: &mut W,
) -> Result<AskResult> {
    use anna_shared::rpc::StreamingResponse;

    // v0.2.7: Try instant error response first for known issues
    if let Some(mut result) = try_instant_error(question) {
        info!("Instant error response streaming completed");

        // Send the dialogue steps
        for step in &result.dialogue {
            send_step(writer, step.clone()).await?;
        }

        // Send done
        let resp = StreamingResponse::Done {
            result: result.clone(),
        };
        let json = serde_json::to_string(&resp)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        writer.flush().await?;

        return Ok(result);
    }

    // v0.2.6: Try fast-path first for simple queries
    if let Some(mut result) = try_fast_path(question).await {
        info!("Fast-path streaming completed");

        // Send the dialogue steps
        for step in &result.dialogue {
            send_step(writer, step.clone()).await?;
        }

        // Send final answer
        let final_step = DialogueStep {
            step_type: StepType::FinalAnswer,
            content: result.answer.clone(),
        };
        result.dialogue.push(final_step.clone());
        send_step(writer, final_step).await?;

        // Send done
        let resp = StreamingResponse::Done {
            result: result.clone(),
        };
        let json = serde_json::to_string(&resp)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        writer.flush().await?;

        return Ok(result);
    }

    // v0.2.7: Try diagnostic path for ambiguous queries (streaming)
    if let Some((executed, outputs, intro, mut dialogue)) = try_diagnostic_path(question) {
        info!("Diagnostic path streaming: analyzing {} outputs", outputs.len());

        // Send the dialogue steps we've collected
        for step in &dialogue {
            send_step(writer, step.clone()).await?;
        }

        // Use LLM to interpret the diagnostic results
        let data_context = outputs.join("\n---\n");
        let prompt = format!(
            r#"You are Anna, an AI assistant for Arch Linux systems.

The user asked: "{}"

I ran diagnostic commands. Here are the results:
{}

Based on these diagnostics, provide a helpful analysis. Be specific:
- If there's a problem, explain what it is and how to fix it
- If everything looks normal, say so with specific evidence
- Reference actual values from the output

Be concise but complete. Start your response with "{}" (without quotes)."#,
            question, data_context, intro
        );

        match ollama::chat_with_timeout(model, &prompt, 60).await {
            Ok(answer) => {
                // Stream the answer token by token
                let step = DialogueStep {
                    step_type: StepType::FinalPrompt,
                    content: String::new(),
                };
                send_step(writer, step).await?;

                for token in answer.split_inclusive(' ') {
                    let resp = StreamingResponse::Token {
                        token: token.to_string(),
                    };
                    let json = serde_json::to_string(&resp)?;
                    writer.write_all(format!("{}\n", json).as_bytes()).await?;
                    writer.flush().await?;
                }

                dialogue.push(DialogueStep {
                    step_type: StepType::FinalAnswer,
                    content: answer.clone(),
                });

                let result = AskResult {
                    answer,
                    success: true,
                    iterations: 1,
                    commands_executed: executed,
                    dialogue,
                    needs_clarification: false,
                    clarification_question: None,
                    cached: false,
                    citations: vec![],
                };

                let resp = StreamingResponse::Done {
                    result: result.clone(),
                };
                let json = serde_json::to_string(&resp)?;
                writer.write_all(format!("{}\n", json).as_bytes()).await?;
                writer.flush().await?;

                return Ok(result);
            }
            Err(e) => {
                warn!("Diagnostic path LLM failed: {}, falling back to normal loop", e);
                // Fall through to normal loop
            }
        }
    }

    let criteria = determine_criteria(question);
    info!(
        "Ralph streaming: {:?}, max {} iterations",
        criteria.answer_type, criteria.max_iterations
    );

    let mut state = IterationState::default();
    let mut dialogue = Vec::new();
    let mut iteration = 0;

    // Record and send user's question
    let step = DialogueStep {
        step_type: StepType::UserQuestion,
        content: question.to_string(),
    };
    dialogue.push(step.clone());
    send_step(writer, step).await?;

    // v0.3.3: Create ticket for fly-on-the-wall experience
    let dept_name = department::determine_department(question);
    let mut ticket = department::create_ticket(question, dept_name);
    let ticket_id = ticket.case_number.clone();

    // Show ticket creation
    let step = DialogueStep {
        step_type: StepType::TicketCreated,
        content: ticket_id.clone(),
    };
    dialogue.push(step.clone());
    send_step(writer, step).await?;

    // v0.3.3: Dispatch to appropriate specialist with improved dialogue
    let specialist = department::get_specialist_for_topic(question);
    let assigned_spec_name = if let Some(spec) = specialist {
        // v0.3.5: Assign ticket to specialist for stats tracking
        ticket.assign(spec.name);
        department::update_ticket(&ticket);

        // Anna assigns the ticket
        let assignment = team_speak::anna_assigns_to(spec, question);
        let step = DialogueStep {
            step_type: StepType::TeamAssignment,
            content: assignment,
        };
        dialogue.push(step.clone());
        send_step(writer, step).await?;

        // Specialist acknowledges
        let ack = team_speak::specialist_acknowledges(spec);
        let step = DialogueStep {
            step_type: StepType::SpecialistWorking,
            content: format!("{}: {}", spec.name, ack),
        };
        dialogue.push(step.clone());
        send_step(writer, step).await?;

        Some(spec.name.to_string())
    } else {
        None
    };

    // THE RALPH LOOP
    while iteration < criteria.max_iterations {
        iteration += 1;
        debug!("Ralph iteration {}/{}", iteration, criteria.max_iterations);

        // Step 1: Get commands
        let commands = get_commands(model, question, &state).await?;

        if !commands.is_empty() {
            // Execute commands and stream progress
            for cmd in &commands {
                let step = DialogueStep {
                    step_type: StepType::CommandExec,
                    content: cmd.clone(),
                };
                dialogue.push(step.clone());
                send_step(writer, step).await?;

                match execute_command(cmd) {
                    Ok(output) => {
                        let clean_output = strip_ansi_codes(&output);
                        state.commands.push(cmd.clone());
                        state.outputs.push(clean_output.clone());

                        let step = DialogueStep {
                            step_type: StepType::CommandOutput,
                            content: truncate(&clean_output, 500),
                        };
                        dialogue.push(step.clone());
                        send_step(writer, step).await?;
                    }
                    Err(e) => {
                        state.feedback = Some(format!("Command '{}' failed: {}", cmd, e));
                    }
                }
            }
        }

        // Step 2: Generate answer
        let answer = generate_answer(model, question, &state, &criteria).await?;
        state.answer = Some(answer.clone());

        // Step 3: Self-evaluate
        let eval = self_evaluate(model, question, &answer, &state, &criteria).await?;
        state.confidence = eval.confidence;

        // Step 4: Check completion
        if eval.is_complete && eval.confidence >= criteria.min_confidence {
            // Stream the final answer token by token
            let step = DialogueStep {
                step_type: StepType::FinalPrompt,
                content: String::new(),
            };
            send_step(writer, step).await?;

            // Stream tokens
            for token in answer.split_inclusive(' ') {
                let resp = StreamingResponse::Token {
                    token: token.to_string(),
                };
                let json = serde_json::to_string(&resp)?;
                writer.write_all(format!("{}\n", json).as_bytes()).await?;
                writer.flush().await?;
            }

            // v0.3.3: Specialist reports completion before final answer
            if let Some(ref spec_name) = assigned_spec_name {
                let completion_msg = format!("{} -> Anna: I've got the answer.", spec_name);
                let step = DialogueStep {
                    step_type: StepType::TeamDialogue,
                    content: completion_msg,
                };
                dialogue.push(step.clone());
                send_step(writer, step).await?;
            }

            dialogue.push(DialogueStep {
                step_type: StepType::FinalAnswer,
                content: answer.clone(),
            });

            // v0.3.0: Learn recipe from successful answer
            learn_recipe_from_answer(question, &state.commands, eval.confidence);

            // v0.3.3: Update ticket as resolved
            let mut updated_ticket = ticket.clone();
            updated_ticket.resolve(&answer, 10); // Award 10 XP
            department::update_ticket(&updated_ticket);

            // Send done
            let result = AskResult {
                answer,
                success: true,
                iterations: iteration,
                commands_executed: state.commands,
                dialogue,
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
            };

            let resp = StreamingResponse::Done {
                result: result.clone(),
            };
            let json = serde_json::to_string(&resp)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            writer.flush().await?;

            return Ok(result);
        }

        // Not done - prepare for next iteration
        state.feedback = eval.suggestions;
        state.not_done_reason = eval.missing;
    }

    // Max iterations - return best effort
    let final_answer = state.answer.unwrap_or_else(|| {
        "I couldn't fully answer your question. Please try rephrasing.".to_string()
    });

    let step = DialogueStep {
        step_type: StepType::FinalPrompt,
        content: String::new(),
    };
    send_step(writer, step).await?;

    // Stream tokens
    for token in final_answer.split_inclusive(' ') {
        let resp = anna_shared::rpc::StreamingResponse::Token {
            token: token.to_string(),
        };
        let json = serde_json::to_string(&resp)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
    }

    dialogue.push(DialogueStep {
        step_type: StepType::FinalAnswer,
        content: final_answer.clone(),
    });

    let result = AskResult {
        answer: final_answer,
        success: state.confidence >= 0.5,
        iterations: iteration,
        commands_executed: state.commands,
        dialogue,
        needs_clarification: state.confidence < 0.3, // v0.1.6: Lowered from 0.5 to reduce note spam
        clarification_question: state.not_done_reason,
        cached: false,
        citations: vec![],
    };

    let resp = anna_shared::rpc::StreamingResponse::Done {
        result: result.clone(),
    };
    let json = serde_json::to_string(&resp)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    writer.flush().await?;

    Ok(result)
}

/// Send a step over the streaming connection
async fn send_step<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    step: DialogueStep,
) -> Result<()> {
    let resp = anna_shared::rpc::StreamingResponse::Step { step };
    let json = serde_json::to_string(&resp)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Find a valid UTF-8 character boundary
        let mut end = max_len;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}

/// v0.3.0: Learn a recipe from a successful answer
/// Only learns if the answer involved actual commands and has high confidence
fn learn_recipe_from_answer(question: &str, commands: &[String], confidence: f32) {
    // Only learn from high-confidence answers with actual commands
    if confidence < 0.8 || commands.is_empty() || commands.len() > 5 {
        return;
    }

    // Extract keywords from question (significant words)
    let keywords: Vec<String> = question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !is_common_word(w))
        .map(|s| s.to_string())
        .collect();

    // Need at least 2 keywords to create a recipe
    if keywords.len() < 2 {
        return;
    }

    // Load existing recipe book
    let mut book = match RecipeBook::load() {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to load recipe book: {}", e);
            return;
        }
    };

    // Check if similar recipe already exists (same keywords)
    let existing = book.recipes.iter().any(|r| {
        let matching_keywords = r.keywords.iter()
            .filter(|k| keywords.contains(k))
            .count();
        matching_keywords >= 2
    });

    if existing {
        debug!("Similar recipe already exists, skipping");
        return;
    }

    // Generate unique ID (timestamp + hash of question)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    question.hash(&mut hasher);
    let hash = hasher.finish();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let id = format!("learned_{}_{:x}", timestamp, hash);

    // Create recipe commands
    let recipe_commands: Vec<RecipeCommand> = commands.iter().map(|cmd| {
        RecipeCommand {
            command: cmd.clone(),
            description: format!("Learned from successful answer"),
            modifies_system: is_modifying_command(cmd),
            backup_file: None,
            needs_root: cmd.starts_with("sudo "),
        }
    }).collect();

    // Create the recipe
    let recipe = Recipe {
        id: id.clone(),
        name: format!("Learned: {}", truncate(question, 40)),
        keywords,
        patterns: vec![question.to_lowercase()],
        context: RecipeContext::default(),
        commands: recipe_commands,
        verification: None,
        source: RecipeSource::Llm { model: "ollama".to_string() },
        success_count: 1,
        last_used: Some(chrono::Utc::now().to_rfc3339()),
        enabled: true,
    };

    book.add_recipe(recipe);
    if let Err(e) = book.save() {
        warn!("Failed to save recipe book: {}", e);
    } else {
        info!("Learned new recipe: {}", id);
        // Record for RPG stats
        crate::department::rpg::record_recipe_learned();
    }
}

/// Check if a word is too common to be a keyword
fn is_common_word(word: &str) -> bool {
    const COMMON: &[&str] = &[
        "the", "and", "for", "that", "this", "with", "have", "are", "from",
        "what", "how", "why", "when", "where", "who", "which", "can", "could",
        "would", "should", "will", "does", "did", "has", "had", "been", "being",
        "was", "were", "not", "but", "all", "any", "some", "its", "into", "out",
        "your", "you", "don", "isn", "does", "doesn", "please", "help", "want",
    ];
    COMMON.contains(&word)
}

/// Check if a command modifies the system
fn is_modifying_command(cmd: &str) -> bool {
    let modifiers = [
        "rm ", "mv ", "cp ", "mkdir ", "rmdir ", "touch ", "chmod ", "chown ",
        "install ", "pacman -S", "pacman -R", "yay -S", "yay -R", "systemctl ",
        "echo ", "printf ", "cat >", "sed -i", "tee ", "ln -s",
    ];
    modifiers.iter().any(|m| cmd.contains(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_criteria_factual() {
        let criteria = determine_criteria("what is my kernel version?");
        assert!(matches!(criteria.answer_type, AnswerType::Factual));
        assert!(criteria.requires_grounding);
    }

    #[test]
    fn test_determine_criteria_howto() {
        let criteria = determine_criteria("how do I install neovim?");
        assert!(matches!(criteria.answer_type, AnswerType::HowTo));
        assert!(!criteria.requires_grounding);
    }

    #[test]
    fn test_determine_criteria_troubleshoot() {
        let criteria = determine_criteria("wifi not working after update");
        assert!(matches!(criteria.answer_type, AnswerType::Troubleshoot));
        assert!(criteria.requires_grounding);
    }
}
