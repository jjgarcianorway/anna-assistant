# System Personalization (v0.3.163)

## The Problem

Anna was speaking generically:
- "your system" → Should say "razorback" (actual hostname)
- "Arch Linux" → Should say "CachyOS" (actual distro)
- "network interface" → Should say "wlan0" (actual device)
- "you" → Should say "lhoqvso" (actual username)
- Assumed pacman → Should detect apt/dnf/zypper for other distros

## The Solution

**System Identity Module** (`system_identity.rs`)

Anna now discovers and caches:
1. **Real Names**
   - Hostname (e.g., "razorback")
   - Username (e.g., "lhoqvso")
   - Distro (e.g., "CachyOS", not "Arch Linux")

2. **Network Reality**
   - Actual device names (wlan0, enp3s0, not "interface")
   - Current WiFi SSID if connected
   - Device types (wireless/ethernet)
   - MAC addresses

3. **System Specifics**
   - Real package manager (pacman/apt/dnf/zypper/etc.)
   - Init system (systemd/openrc)
   - Desktop environment (GNOME/KDE/etc.)
   - Shell (bash/zsh/fish)

## Examples

### Before (Generic):
```
Anna: "I'll install htop on your system using pacman"
```

### After (Personalized):
```
Anna: "I'll install htop on razorback (CachyOS) using pacman"
```

### Before (Wrong Assumptions):
```
# On Ubuntu:
Anna: "sudo pacman -S htop"  ❌ WRONG
```

### After (Correct):
```
# On Ubuntu:
Anna: "sudo apt install -y htop"  ✓ CORRECT
```

### Before (Vague):
```
Anna: "Your wireless interface is up"
```

### After (Specific):
```
Anna: "wlan0 is up and connected to MyWiFiSSID"
```

## Integration

Every LLM call now receives system identity context:

```
SYSTEM IDENTITY:
Hostname: razorback
User: lhoqvso
Distro: CachyOS
Package Manager: pacman
Shell: zsh
Network Devices: wlan0 (wireless), enp3s0 (ethernet)
Current WiFi: MyHomeNetwork
Desktop: GNOME
```

The LLM uses this to speak naturally about THIS specific system.

## Distro Detection

Supports all major Linux distributions:

| Distro Family | Package Manager | Example Distros |
|---------------|----------------|-----------------|
| arch | pacman (+yay for AUR) | Arch, CachyOS, Manjaro |
| debian | apt | Ubuntu, Debian, Mint |
| fedora | dnf | Fedora, RHEL, CentOS |
| suse | zypper | openSUSE, SLES |
| gentoo | emerge | Gentoo |
| alpine | apk | Alpine Linux |

## Technical Details

### Discovery Process

1. **Hostname**: Read from `hostname` crate
2. **Username**: $USER or $USERNAME env var
3. **Distro**: Parse `/etc/os-release` PRETTY_NAME
4. **Network**: Parse `ip link show` for real device names
5. **SSID**: Try `iw dev` then fallback to `nmcli`
6. **Desktop**: Check $XDG_CURRENT_DESKTOP
7. **Init**: Check `/run/systemd/system` existence

### Caching

System identity is cached globally after first discovery:
```rust
lazy_static! {
    static ref SYSTEM_IDENTITY: RwLock<Option<SystemIdentity>> = RwLock::new(None);
}
```

Call `refresh_system_identity()` when network changes.

### Package Manager Auto-Detection

```rust
impl SystemIdentity {
    pub fn install_command(&self, package: &str) -> String {
        match self.distro_family.as_str() {
            "arch" => format!("sudo pacman -S --noconfirm {}", package),
            "debian" => format!("sudo apt install -y {}", package),
            "fedora" => format!("sudo dnf install -y {}", package),
            ...
        }
    }
}
```

Universal handler now uses this instead of assuming pacman.

## Greeting Example

```bash
$ annactl status

Hello lhoqvso! I'm Anna, running on razorback (CachyOS)
```

## Future Enhancements

1. **User Preferences**: Remember "I prefer vim over nano"
2. **Network History**: "You're usually on MyHomeNetwork, but now on PublicWiFi"
3. **Hardware Awareness**: "Your laptop is low on battery"
4. **Location Context**: "You're at home" vs "You're at coffee shop"
5. **Time Patterns**: "You usually compile at night"

## Impact

- More natural conversation ("razorback" feels like talking to YOUR system)
- Correct commands (no more Arch assumptions on Ubuntu)
- Proper context (knows wlan0 is wireless, enp3s0 is ethernet)
- Professional feel (Anna knows who she's talking to)

**Anna is no longer a generic assistant. She's the god living in YOUR computer, knowing YOUR system's real identity.**
