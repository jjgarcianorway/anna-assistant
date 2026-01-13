//! Instant responses for well-known error patterns.
//! These have known solutions - no investigation needed.

use anna_shared::rpc::{AskResult, Citation, DialogueStep, StepType};
use tracing::info;

/// Instant response with wiki citation source
pub struct InstantResponse {
    pub answer: &'static str,
    pub wiki_source: &'static str,
    pub wiki_url: Option<&'static str>,
}

/// Get instant response for common error patterns
pub fn get_instant_error_response(question: &str) -> Option<InstantResponse> {
    let q = question.to_lowercase();

    // Pacman database lock
    if q.contains("pacman") && (q.contains("lock") || q.contains("locked")) {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: Pacman",
            wiki_url: Some("https://wiki.archlinux.org/title/Pacman"),
            answer:
            "The pacman database is locked. This usually happens when another package operation is running or crashed.\n\n\
            **Solution:**\n\
            1. Check if pacman is running: `pgrep -a pacman`\n\
            2. If not running, remove the lock: `sudo rm /var/lib/pacman/db.lck`\n\
            3. If pacman is running, wait for it to finish or kill it: `sudo pkill pacman`\n\n\
            **Note:** Only remove the lock if you're certain no package operation is in progress."
        });
    }

    // GPGME error
    if q.contains("gpgme") || (q.contains("gpg") && q.contains("no data")) {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: Pacman/Package signing",
            wiki_url: Some("https://wiki.archlinux.org/title/Pacman/Package_signing"),
            answer: "GPGME/GPG 'No data' error usually means corrupted or missing package keys.\n\n\
            **Solution:**\n\
            1. Refresh keys: `sudo pacman-key --refresh-keys`\n\
            2. If that fails, reinitialize: `sudo pacman-key --init && sudo pacman-key --populate archlinux`\n\
            3. Update keyring: `sudo pacman -Sy archlinux-keyring`\n\n\
            **Note:** This can take a few minutes. If using CachyOS, also run `sudo pacman-key --populate cachyos`."
        });
    }

    // Deleted /usr/bin
    if q.contains("deleted") && q.contains("/usr/bin") {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: Pacman/Tips and tricks",
            wiki_url: Some("https://wiki.archlinux.org/title/Pacman/Tips_and_tricks#Reinstalling_all_packages"),
            answer: "Accidentally deleted /usr/bin is serious but recoverable.\n\n\
            **Recovery steps:**\n\
            1. Boot from Arch/CachyOS live USB\n\
            2. Mount your root partition: `mount /dev/sdXY /mnt`\n\
            3. Chroot: `arch-chroot /mnt`\n\
            4. Reinstall base packages: `pacman -S base base-devel`\n\
            5. Reinstall all explicitly installed packages: `pacman -Qeq | pacman -S -`\n\n\
            **Note:** If /usr/bin/bash is gone, use `busybox sh` from a rescue environment."
        });
    }

    // Forgot root password
    if q.contains("forgot") && (q.contains("root") || q.contains("password")) {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: Reset lost root password",
            wiki_url: Some("https://wiki.archlinux.org/title/Reset_lost_root_password"),
            answer: "To reset root password:\n\n\
            1. Reboot and at GRUB, press 'e' to edit boot entry\n\
            2. Find the line starting with 'linux' and add `init=/bin/bash` at the end\n\
            3. Press Ctrl+X to boot\n\
            4. Remount root: `mount -o remount,rw /`\n\
            5. Set new password: `passwd`\n\
            6. Reboot: `exec /sbin/init` or `reboot -f`\n\n\
            **Note:** For systemd-boot, edit the loader entry similarly."
        });
    }

    // chmod 777 -R disaster
    if q.contains("chmod") && q.contains("777") && q.contains("/") {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: File permissions and attributes",
            wiki_url: Some("https://wiki.archlinux.org/title/File_permissions_and_attributes"),
            answer: "Running chmod 777 -R on system directories is serious but recoverable.\n\n\
            **Recovery steps:**\n\
            1. Boot from live USB (system may not boot normally)\n\
            2. Mount your root partition: `mount /dev/sdXY /mnt`\n\
            3. Reinstall all packages to fix permissions:\n\
               `arch-chroot /mnt pacman -Qkk 2>&1 | grep 'Permissions mismatch' | awk '{print $2}' | xargs pacman -S --noconfirm`\n\
            4. Or reinstall everything: `pacman -Qnq | pacman -S --noconfirm -`\n\n\
            **Note:** This may take a while. Check /etc/shadow permissions manually: should be 640."
        });
    }

    // Kernel won't boot
    if (q.contains("won't boot") || q.contains("can't boot") || q.contains("not boot")) && q.contains("kernel") {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: Kernel",
            wiki_url: Some("https://wiki.archlinux.org/title/Kernel#Compilation"),
            answer: "System won't boot after kernel update:\n\n\
            **Quick fix:**\n\
            1. At GRUB, select a previous kernel from 'Advanced options'\n\
            2. Or at boot, press 'e' and change kernel version in linux line\n\n\
            **From live USB:**\n\
            1. Mount root and chroot\n\
            2. Downgrade kernel: `pacman -U /var/cache/pacman/pkg/linux-<version>.pkg.tar.zst`\n\
            3. Or regenerate initramfs: `mkinitcpio -P`\n\n\
            **Note:** nvidia-dkms users should also downgrade nvidia drivers or wait for dkms rebuild."
        });
    }

    // Disk full can't login
    if q.contains("disk") && q.contains("full") && (q.contains("login") || q.contains("can't")) {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: Pacman#Cleaning the package cache",
            wiki_url: Some("https://wiki.archlinux.org/title/Pacman#Cleaning_the_package_cache"),
            answer: "Disk full, can't login:\n\n\
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
        });
    }

    // Black screen / display manager won't start
    if (q.contains("black screen") || q.contains("display manager") || q.contains("dm won't start") || q.contains("sddm") || q.contains("gdm"))
        && (q.contains("won't") || q.contains("not") || q.contains("blank"))
    {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: Display manager",
            wiki_url: Some("https://wiki.archlinux.org/title/Display_manager"),
            answer: "Display manager won't start / black screen:\n\n\
            **Quick diagnosis:**\n\
            1. Press Ctrl+Alt+F2 for TTY login\n\
            2. Check DM status: `systemctl status sddm` (or gdm/lightdm)\n\
            3. Check Xorg logs: `cat /var/log/Xorg.0.log | grep EE`\n\
            4. Check journal: `journalctl -b -p err | grep -i 'x11\\|wayland\\|nvidia\\|amd'`\n\n\
            **Common fixes:**\n\
            - Nvidia: reinstall drivers `pacman -S nvidia nvidia-dkms`\n\
            - Permissions: `chmod 0660 /dev/dri/*` \n\
            - Restart DM: `systemctl restart sddm`"
        });
    }

    // Part 2: More error patterns
    get_instant_error_response_part2(&q)
}

fn get_instant_error_response_part2(q: &str) -> Option<InstantResponse> {
    // Electron apps blurry HiDPI
    if q.contains("electron") && (q.contains("blurry") || q.contains("hidpi") || q.contains("fuzzy") || q.contains("scaling")) {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: HiDPI#Electron",
            wiki_url: Some("https://wiki.archlinux.org/title/HiDPI#Electron"),
            answer: "Electron apps blurry on HiDPI:\n\n\
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
        });
    }

    // Steam games crash
    if q.contains("steam") && (q.contains("crash") || q.contains("won't") || q.contains("launch") || q.contains("error")) {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: Steam#Troubleshooting",
            wiki_url: Some("https://wiki.archlinux.org/title/Steam#Troubleshooting"),
            answer: "Steam games crashing:\n\n\
            **Common fixes:**\n\
            1. Enable Proton: Game -> Properties -> Compatibility -> Force Proton\n\
            2. Verify game files: Right-click -> Properties -> Local Files -> Verify\n\
            3. Try different Proton: Use Proton-GE from ProtonUp-Qt\n\
            4. Check dependencies: `pacman -S lib32-vulkan-icd-loader vulkan-tools`\n\n\
            **For native games:**\n\
            - Launch options: `LD_PRELOAD='' %command%`\n\
            - Missing libs: `ldd ~/.steam/steam/steamapps/common/Game/game.exe`\n\n\
            Check ProtonDB for game-specific fixes."
        });
    }

    // Docker DNS
    if q.contains("docker") && q.contains("dns") {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: Docker#DNS",
            wiki_url: Some("https://wiki.archlinux.org/title/Docker#DNS_issues"),
            answer: "Docker containers can't resolve DNS:\n\n\
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
        });
    }

    // Flatpak can't access home
    if q.contains("flatpak") && (q.contains("access") || q.contains("permission") || q.contains("home") || q.contains("folder")) {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: Flatpak#Permissions",
            wiki_url: Some("https://wiki.archlinux.org/title/Flatpak#Permissions"),
            answer: "Flatpak apps can't access home folder:\n\n\
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
        });
    }

    // xdg-open wrong app
    if q.contains("xdg-open") && (q.contains("wrong") || q.contains("right") || q.contains("application") || q.contains("doesn't")) {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: XDG MIME Applications",
            wiki_url: Some("https://wiki.archlinux.org/title/XDG_MIME_Applications"),
            answer: "xdg-open doesn't open files with the right application:\n\n\
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
        });
    }

    // Pipewire vs PulseAudio
    if (q.contains("pipewire") && q.contains("pulseaudio")) || (q.contains("audio") && q.contains("fighting")) {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: PipeWire",
            wiki_url: Some("https://wiki.archlinux.org/title/PipeWire"),
            answer: "Pipewire and PulseAudio conflict:\n\n\
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
        });
    }

    // GRUB rescue
    if q.contains("grub") && q.contains("rescue") {
        return Some(InstantResponse {
            wiki_source: "Arch Wiki: GRUB#Rescue",
            wiki_url: Some("https://wiki.archlinux.org/title/GRUB#Rescue"),
            answer: "GRUB rescue / unknown filesystem:\n\n\
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
        });
    }

    None
}

/// Try instant error response, returning full AskResult if matched
pub fn try_instant_error(question: &str) -> Option<AskResult> {
    let response = get_instant_error_response(question)?;

    info!("Instant response: known error pattern matched");

    let citation = Citation {
        source: response.wiki_source.to_string(),
        url: response.wiki_url.map(|s| s.to_string()),
        section: None,
    };

    Some(AskResult {
        answer: response.answer.to_string(),
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
                content: response.answer.to_string(),
            },
        ],
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![citation],
    })
}
