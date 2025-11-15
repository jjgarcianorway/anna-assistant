# Anna Facts Catalog

Complete list of system facts that `annad` should collect for intelligent, Arch-Wiki-backed recommendations.

**Privacy-first, local-only, with opt-in for sensitive data (history, usage).**

---

## System & Hardware

### CPU / Microcode / Power
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| CPU vendor/model | `/proc/cpuinfo`, `lscpu` | [Microcode](https://wiki.archlinux.org/title/Microcode) | ⚠️  Partial |
| Cores/threads | `lscpu` | [CPU frequency scaling](https://wiki.archlinux.org/title/CPU_frequency_scaling) | ⚠️  Partial |
| CPU flags (SSE/AVX) | `/proc/cpuinfo` | [Microcode](https://wiki.archlinux.org/title/Microcode) | ❌ TODO |
| Current governor | `cpupower frequency-info` | [CPU frequency scaling](https://wiki.archlinux.org/title/CPU_frequency_scaling) | ❌ TODO |
| Microcode package/version | `pacman -Q \| grep -E '(intel-ucode\|amd-ucode)'` | [Microcode](https://wiki.archlinux.org/title/Microcode) | ❌ TODO |

### Motherboard / BIOS/UEFI / Boot
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| UEFI vs BIOS | `bootctl status`, `efibootmgr -v` | [Arch boot process](https://wiki.archlinux.org/title/Arch_boot_process) | ✅ Done (beta.25) |
| Secure Boot status | `dmesg \| grep -i secure` | [Secure Boot](https://wiki.archlinux.org/title/Unified_Extensible_Firmware_Interface/Secure_Boot) | ✅ Done (beta.25) |
| Firmware versions | `dmidecode` | - | ❌ TODO |

### GPU / Graphics Stack
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| GPU model(s) | `lspci -k \| grep -A3 -E 'VGA\|3D'` | [GPU drivers](https://wiki.archlinux.org/title/Xorg#Driver_installation) | ⚠️  Partial |
| Driver in use | `lspci -k` | [NVIDIA](https://wiki.archlinux.org/title/NVIDIA), [AMDGPU](https://wiki.archlinux.org/title/AMDGPU) | ⚠️  Partial |
| Vulkan/OpenGL support | `glxinfo -B`, `vulkaninfo` | [Vulkan](https://wiki.archlinux.org/title/Vulkan) | ❌ TODO |
| PRIME offload | `nvidia-smi` (if NVIDIA) | [PRIME](https://wiki.archlinux.org/title/PRIME) | ❌ TODO |
| Wayland/Xorg | `echo $XDG_SESSION_TYPE`, `loginctl show-session` | [Wayland](https://wiki.archlinux.org/title/Wayland) | ❌ TODO |
| Compositor | `loginctl show-session` | - | ❌ TODO |

### Audio
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| PipeWire/Pulse/ALSA | `pactl info`, `pw-cli info all` | [PipeWire](https://wiki.archlinux.org/title/PipeWire) | ✅ Done (beta.25) |
| Audio devices/profiles | `aplay -l` | [ALSA](https://wiki.archlinux.org/title/Advanced_Linux_Sound_Architecture) | ✅ Done (beta.25) |
| JACK presence | `command -v jackd` | [JACK](https://wiki.archlinux.org/title/JACK_Audio_Connection_Kit) | ✅ Done (beta.25) |

### Storage / Filesystems / TRIM
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| Disks/partitions | `lsblk -f` | [File systems](https://wiki.archlinux.org/title/File_systems) | ✅ Done |
| FS types | `findmnt -rno TARGET,OPTIONS /` | [Btrfs](https://wiki.archlinux.org/title/Btrfs), [ext4](https://wiki.archlinux.org/title/Ext4) | ✅ Done |
| Mount options | `findmnt` | [fstab](https://wiki.archlinux.org/title/Fstab) | ❌ TODO |
| TRIM status | `systemctl status fstrim.timer` | [SSD](https://wiki.archlinux.org/title/Solid_state_drive) | ✅ Done (beta.26) |
| LUKS encryption | `cryptsetup status`, `lsblk -o FSTYPE` | [dm-crypt](https://wiki.archlinux.org/title/Dm-crypt) | ✅ Done (beta.26) |
| Btrfs subvolumes | `btrfs subvolume list -t /` | [Btrfs](https://wiki.archlinux.org/title/Btrfs#Subvolumes) | ✅ Done (beta.26) |
| Btrfs compression | `/etc/fstab` | [Btrfs](https://wiki.archlinux.org/title/Btrfs#Compression) | ✅ Done (beta.26) |

### Network
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| NICs | `ip -br a` | [Network configuration](https://wiki.archlinux.org/title/Network_configuration) | ✅ Done |
| Drivers | `ethtool -i <iface>` | - | ❌ TODO |
| DNS resolver | `resolvectl status` | [systemd-resolved](https://wiki.archlinux.org/title/Systemd-resolved) | ❌ TODO |
| NetworkManager/systemd-networkd | `nmcli general status`, `systemctl is-active systemd-networkd` | [NetworkManager](https://wiki.archlinux.org/title/NetworkManager) | ❌ TODO |
| Wi-Fi power save | `iw dev <iface> get power_save` | [Power management](https://wiki.archlinux.org/title/Power_management#Network_interfaces) | ❌ TODO |

### Sensors / Thermals / Battery (laptops)
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| Temps/fans | `sensors` | [lm_sensors](https://wiki.archlinux.org/title/Lm_sensors) | ❌ TODO |
| TLP status | `tlp-stat -s -b` | [TLP](https://wiki.archlinux.org/title/TLP) | ❌ TODO |
| Battery wear/capacity | `/sys/class/power_supply/BAT*/uevent` | [Laptop](https://wiki.archlinux.org/title/Laptop) | ❌ TODO |

### Virtualization / Containers
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| KVM/SVM support | `lscpu \| grep Virtualization` | [KVM](https://wiki.archlinux.org/title/KVM) | ❌ TODO |
| vfio bindings | `lsmod \| grep -E 'kvm\|vfio'` | [PCI passthrough](https://wiki.archlinux.org/title/PCI_passthrough_via_OVMF) | ❌ TODO |
| libvirt/qemu/podman/docker | `systemctl is-active libvirtd`, `podman --version` | [libvirt](https://wiki.archlinux.org/title/Libvirt), [Docker](https://wiki.archlinux.org/title/Docker) | ❌ TODO |

---

## Kernel, Boot & Init

### Kernel / Initramfs / Params
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| Kernel version/flavor | `uname -r` | [Kernel](https://wiki.archlinux.org/title/Kernel) | ✅ Done |
| mkinitcpio hooks | `/etc/mkinitcpio.conf` | [mkinitcpio](https://wiki.archlinux.org/title/Mkinitcpio) | ❌ TODO |
| Boot params | `/proc/cmdline` | [Kernel parameters](https://wiki.archlinux.org/title/Kernel_parameters) | ❌ TODO |
| Microcode order in initramfs | Check initramfs contents | [Microcode](https://wiki.archlinux.org/title/Microcode#mkinitcpio) | ❌ TODO |

### Bootloader
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| GRUB vs systemd-boot | `/etc/default/grub`, `/boot/loader/loader.conf` | [GRUB](https://wiki.archlinux.org/title/GRUB), [systemd-boot](https://wiki.archlinux.org/title/Systemd-boot) | ❌ TODO |
| Config presence/sanity | Check config files | - | ❌ TODO |

---

## Packages & Repos

### Package Manager State
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| pacman.conf options | `/etc/pacman.conf` | [pacman](https://wiki.archlinux.org/title/Pacman) | ❌ TODO |
| Mirrorlist freshness | `/etc/pacman.d/mirrorlist` (file age) | [Mirrors](https://wiki.archlinux.org/title/Mirrors) | ❌ TODO |
| AUR helper presence | `command -v yay paru` | [AUR helpers](https://wiki.archlinux.org/title/AUR_helpers) | ❌ TODO |

### Installed Packages
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| Explicit packages | `pacman -Qe` | [pacman/Tips and tricks](https://wiki.archlinux.org/title/Pacman/Tips_and_tricks) | ✅ Done |
| Orphans | `pacman -Qdt` | [pacman/Tips and tricks](https://wiki.archlinux.org/title/Pacman/Tips_and_tricks#Removing_unused_packages_(orphans)) | ✅ Done |
| Outdated packages | `checkupdates` | [System maintenance](https://wiki.archlinux.org/title/System_maintenance) | ❌ TODO |
| Installed kernels | `pacman -Qs linux` | [Kernel](https://wiki.archlinux.org/title/Kernel#Officially_supported_kernels) | ❌ TODO |
| Firmware blobs | `pacman -Qs firmware` | [Microcode](https://wiki.archlinux.org/title/Microcode) | ❌ TODO |

### Firmware & fwupd
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| fwupd availability | `fwupdmgr get-devices` | [fwupd](https://wiki.archlinux.org/title/Fwupd) | ❌ TODO |
| Updatable devices | `fwupdmgr get-updates` | [fwupd](https://wiki.archlinux.org/title/Fwupd) | ❌ TODO |

---

## Services & Daemons

### Systemd Services/Timers
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| Failed units | `systemctl --failed` | [systemd](https://wiki.archlinux.org/title/Systemd) | ❌ TODO |
| Essential timers status | `systemctl is-enabled fstrim.timer reflector.timer` | [systemd/Timers](https://wiki.archlinux.org/title/Systemd/Timers) | ❌ TODO |
| Journal size/rotation | `journalctl --disk-usage`, `/etc/systemd/journald.conf` | [systemd/Journal](https://wiki.archlinux.org/title/Systemd/Journal) | ❌ TODO |

---

## Security & Hardening

### Auth / sudo / firewall
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| wheel group sudo | `getent group wheel`, `/etc/sudoers` | [sudo](https://wiki.archlinux.org/title/Sudo) | ❌ TODO |
| ufw/nftables status | `systemctl is-active ufw`, `nft list ruleset` | [Uncomplicated Firewall](https://wiki.archlinux.org/title/Uncomplicated_Firewall) | ❌ TODO |
| sshd config | `/etc/ssh/sshd_config` | [OpenSSH](https://wiki.archlinux.org/title/OpenSSH) | ❌ TODO |
| Secure umask | `/etc/profile`, `/etc/bash.bashrc` | [Security](https://wiki.archlinux.org/title/Security#File_permissions) | ❌ TODO |

---

## Desktop / Session

### DE/WM / Display Manager
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| Xorg vs Wayland | `echo $XDG_SESSION_TYPE` | [Wayland](https://wiki.archlinux.org/title/Wayland) | ⚠️  Partial |
| DE (GNOME/KDE/Hyprland) | `loginctl show-session`, `echo $XDG_CURRENT_DESKTOP` | [Desktop environment](https://wiki.archlinux.org/title/Desktop_environment) | ⚠️  Partial |
| Display manager | `systemctl status gdm sddm lightdm` | [Display manager](https://wiki.archlinux.org/title/Display_manager) | ❌ TODO |

### Fonts & Locale
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| Locales generated | `/etc/locale.gen` | [Locale](https://wiki.archlinux.org/title/Locale) | ❌ TODO |
| Default locale | `/etc/locale.conf` | [Locale](https://wiki.archlinux.org/title/Locale) | ❌ TODO |
| Font packages | `fc-list \| wc -l` | [Fonts](https://wiki.archlinux.org/title/Fonts) | ❌ TODO |
| Emoji font | `fc-match emoji` | [Fonts#Emoji](https://wiki.archlinux.org/title/Fonts#Emoji_and_symbols) | ❌ TODO |

---

## User Environment & Behavior (opt-in)

### Shell / Editor / Terminal
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| Default shell | `$SHELL`, `/etc/shells` | [Command-line shell](https://wiki.archlinux.org/title/Command-line_shell) | ✅ Done |
| Shell configs exist | `~/.bashrc`, `~/.zshrc`, `~/.config/fish/config.fish` | [Bash](https://wiki.archlinux.org/title/Bash), [Zsh](https://wiki.archlinux.org/title/Zsh) | ❌ TODO |
| Prompt theme | `~/.config/starship.toml` | [Starship](https://starship.rs/) | ❌ TODO |
| Editor | `update-alternatives --display editor`, `$EDITOR` | [Vim](https://wiki.archlinux.org/title/Vim), [Neovim](https://wiki.archlinux.org/title/Neovim) | ❌ TODO |
| Terminal emulator | Process name, `$TERM` | [List of applications/Utilities#Terminal emulators](https://wiki.archlinux.org/title/List_of_applications/Utilities#Terminal_emulators) | ❌ TODO |

### Command Habits (opt-in, privacy-sensitive)
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| Command frequency | `~/.bash_history`, `~/.zsh_history` (opt-in) | - | ✅ Done |
| Long-running processes | `sa/acct` (if enabled) | - | ❌ TODO |
| Package command usage | - | - | ❌ TODO |

### Dotfiles Presence / Gaps
| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| ~/.inputrc | Check existence | [Readline](https://wiki.archlinux.org/title/Readline) | ❌ TODO |
| ~/.gitconfig | Check existence + user.name/email | [Git](https://wiki.archlinux.org/title/Git) | ❌ TODO |
| ~/.vimrc / ~/.config/nvim/init.lua | Check existence | [Vim](https://wiki.archlinux.org/title/Vim), [Neovim](https://wiki.archlinux.org/title/Neovim) | ❌ TODO |
| ~/.config/alacritty/alacritty.yml | Check existence | [Alacritty](https://github.com/alacritty/alacritty) | ❌ TODO |
| ~/.config/bat/config | Check existence | [bat](https://github.com/sharkdp/bat) | ❌ TODO |

---

## QoL & Beautification Targets

### Core Substitutions
| Tool | Replaces | Package | Config | Status |
|------|----------|---------|--------|--------|
| eza | ls | `eza` | Alias in shell rc | ⚠️  Detection only |
| bat | cat | `bat` | `~/.config/bat/config` | ⚠️  Detection only |
| ripgrep | grep | `ripgrep` | Alias | ⚠️  Detection only |
| fd | find | `fd` | - | ⚠️  Detection only |
| zoxide | cd | `zoxide` | Shell integration | ⚠️  Detection only |
| fzf | - | `fzf` | Shell keybindings | ⚠️  Detection only |
| starship | prompt | `starship` | `~/.config/starship.toml` | ⚠️  Detection only |

### Enablers
- **pacman**: Color, ParallelDownloads, ILoveCandy
- **grep**: color alias (`grep='grep --color=auto'`)
- **less**: color flags (LESS_TERMCAP_*)
- **vim/nvim**: syntax on, line numbers, LSP
- **Git**: sensible defaults (pull.rebase, init.defaultBranch, color.ui, delta pager)
- **Term UX**: Nerd Fonts, icons

---

## Maintenance & Health

| Fact | Source | Wiki Page | Status |
|------|--------|-----------|--------|
| Last update timestamp | `/var/lib/pacman/local/*/install` | [System maintenance](https://wiki.archlinux.org/title/System_maintenance) | ❌ TODO |
| Pending updates | `checkupdates` | [System maintenance](https://wiki.archlinux.org/title/System_maintenance) | ⚠️  Partial |
| Backups/Snapshots | `snapper list`, `btrbk list`, `timeshift --list` | [Snapper](https://wiki.archlinux.org/title/Snapper), [Btrbk](https://wiki.archlinux.org/title/Btrbk) | ❌ TODO |
| Disk SMART health | `smartctl -H /dev/sdX` (opt-in) | [S.M.A.R.T.](https://wiki.archlinux.org/title/S.M.A.R.T.) | ❌ TODO |
| Journal size | `journalctl --disk-usage` | [systemd/Journal](https://wiki.archlinux.org/title/Systemd/Journal) | ❌ TODO |

---

## Legend

- ✅ **Done**: Fully implemented
- ⚠️  **Partial**: Basic detection, needs enhancement
- ❌ **TODO**: Not yet implemented

---

## Implementation Priority

### Phase 1 (Critical - v1.0.0-rc.1)
1. CPU/Microcode detection
2. GPU driver analysis
3. Btrfs mount options & compression
4. Missing config detection (bat, starship, zoxide, git)
5. TRIM timer status
6. Orphan cleanup improvements

### Phase 2 (Important - v1.0.0)
1. Boot mode (UEFI/BIOS) & Secure Boot
2. Kernel parameters analysis
3. systemd failed units
4. Essential timers (fstrim, reflector)
5. pacman.conf optimization
6. AUR helper detection

### Phase 3 (Nice to Have - v1.1.0+)
1. PipeWire/audio stack
2. Virtualization support
3. TLP/power management (laptops)
4. Firmware updates (fwupd)
5. SMART disk health
6. Backup/snapshot status
