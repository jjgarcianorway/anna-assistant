//! Gaming patterns - Steam, Wine, Proton, controllers, and gaming diagnostics
//! v0.0.950: Initial gaming patterns for Linux gaming
//! v0.0.989: Added gamepad, shader cache, esync/fsync, vkbasalt, gaming performance

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Pattern with keywords, description, topic, and command templates
type GamingPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

/// Match common gaming-related questions
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // Steam
    if let Some(u) = match_steam(q) {
        return Some(u);
    }
    // Wine/Proton
    if let Some(u) = match_wine_proton(q) {
        return Some(u);
    }
    // Controllers
    if let Some(u) = match_controllers(q) {
        return Some(u);
    }
    // Graphics/performance
    if let Some(u) = match_gaming_graphics(q) {
        return Some(u);
    }
    // Emulation
    if let Some(u) = match_emulation(q) {
        return Some(u);
    }
    None
}

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

/// Steam queries
fn match_steam(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[GamingPattern] = &[
        // Steam status
        (&["steam", "install"], "check Steam installation", "gaming",
            &["which steam 2>/dev/null", "pacman -Qs steam", "flatpak list | grep -i steam 2>/dev/null"]),
        (&["steam", "running"], "check if Steam is running", "gaming",
            &["pgrep -a steam", "ps aux | grep -i steam | grep -v grep"]),
        (&["steam", "version"], "check Steam version", "gaming",
            &["steam --version 2>/dev/null", "cat ~/.steam/steam/package/steam_client_*.vdf 2>/dev/null | head -5"]),
        // Steam games
        (&["steam", "game"], "list Steam games", "gaming",
            &["ls ~/.steam/steam/steamapps/common/ 2>/dev/null | head -20",
              "find ~/.steam/steam/steamapps -name '*.acf' -exec basename {} \\; 2>/dev/null | head -10"]),
        (&["steam", "library"], "show Steam library", "gaming",
            &["ls ~/.steam/steam/steamapps/common/ 2>/dev/null"]),
        // Steam logs
        (&["steam", "log"], "check Steam logs", "gaming",
            &["cat ~/.steam/steam/logs/console_log.txt 2>/dev/null | tail -30",
              "cat ~/.local/share/Steam/logs/console_log.txt 2>/dev/null | tail -30"]),
        (&["steam", "error"], "check Steam errors", "gaming",
            &["cat ~/.steam/steam/logs/console_log.txt 2>/dev/null | grep -i error | tail -20"]),
        // Steam runtime
        (&["steam", "runtime"], "check Steam runtime", "gaming",
            &["ls ~/.steam/steam/ubuntu12_32/steam-runtime 2>/dev/null | head -10",
              "echo $STEAM_RUNTIME_LIBRARY_PATH"]),
        // Proton in Steam
        (&["steam", "proton"], "list Proton versions in Steam", "gaming",
            &["ls ~/.steam/steam/steamapps/common/ 2>/dev/null | grep -i proton",
              "ls ~/.steam/steam/compatibilitytools.d/ 2>/dev/null"]),
        // Steam games list
        (&["steam", "games", "list"], "list Steam games", "gaming",
            &["ls ~/.steam/steam/steamapps/common/ 2>/dev/null",
              "find ~/.steam/steam/steamapps -name '*.acf' -exec basename {} \\;"]),
        // Steam library location
        (&["steam", "library", "location"], "show Steam library location", "gaming",
            &["cat ~/.steam/steam/steamapps/libraryfolders.vdf 2>/dev/null",
              "ls ~/.steam/steam/steamapps/common/"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Wine and Proton queries
fn match_wine_proton(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[GamingPattern] = &[
        // Wine
        (&["wine", "version"], "check Wine version", "gaming",
            &["wine --version 2>/dev/null", "wine64 --version 2>/dev/null"]),
        (&["wine", "install"], "check Wine installation", "gaming",
            &["which wine", "pacman -Qs wine"]),
        (&["wine", "config"], "Wine configuration", "gaming",
            &["winecfg &", "echo 'Run: winecfg to configure Wine'"]),
        (&["wine", "prefix"], "list Wine prefixes", "gaming",
            &["ls ~/.wine 2>/dev/null | head -20", "echo $WINEPREFIX"]),
        // Proton
        (&["proton", "version"], "check Proton versions", "gaming",
            &["ls ~/.steam/steam/steamapps/common/ 2>/dev/null | grep -i proton",
              "ls ~/.steam/steam/compatibilitytools.d/ 2>/dev/null"]),
        (&["proton", "ge"], "check Proton-GE versions", "gaming",
            &["ls ~/.steam/steam/compatibilitytools.d/ 2>/dev/null | grep -i ge",
              "ls ~/.steam/root/compatibilitytools.d/ 2>/dev/null | grep -i ge"]),
        (&["proton", "log"], "check Proton logs", "gaming",
            &["cat ~/.steam/steam/steamapps/compatdata/*/pfx/drive_c/users/steamuser/Temp/*.log 2>/dev/null | tail -50",
              "echo 'Enable PROTON_LOG=1 for logging'"]),
        // DXVK
        (&["dxvk", "version"], "check DXVK version", "gaming",
            &["pacman -Q dxvk 2>/dev/null", "ls /usr/share/dxvk 2>/dev/null"]),
        (&["dxvk", "install"], "check DXVK installation", "gaming",
            &["pacman -Qs dxvk", "echo 'Install: pacman -S dxvk-bin'"]),
        // VKD3D
        (&["vkd3d", "version"], "check VKD3D version", "gaming",
            &["pacman -Q vkd3d 2>/dev/null", "pacman -Qs vkd3d-proton"]),
        // Lutris
        (&["lutris", "game"], "list Lutris games", "gaming",
            &["lutris -l 2>/dev/null || echo 'Lutris not installed'",
              "ls ~/.local/share/lutris/runners 2>/dev/null"]),
        (&["lutris", "runner"], "list Lutris runners", "gaming",
            &["ls ~/.local/share/lutris/runners 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Controller/gamepad queries
fn match_controllers(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[GamingPattern] = &[
        // Controller detection
        (&["controller", "detect"], "detect game controllers", "gaming",
            &["cat /proc/bus/input/devices | grep -A5 -i gamepad",
              "ls /dev/input/js* 2>/dev/null || echo 'No joystick devices'",
              "lsusb | grep -i controller"]),
        (&["gamepad", "connect"], "check gamepad connection", "gaming",
            &["cat /proc/bus/input/devices | grep -A5 -i 'gamepad\\|joystick'",
              "evtest --list 2>/dev/null | grep -i game || echo 'Install: pacman -S evtest'"]),
        (&["joystick", "list"], "list joystick devices", "gaming",
            &["ls -la /dev/input/js* 2>/dev/null", "cat /proc/bus/input/devices | grep -B5 -A5 Joystick"]),
        // Xbox controller
        (&["xbox", "controller"], "check Xbox controller", "gaming",
            &["lsusb | grep -i xbox", "dmesg | grep -i xbox | tail -10",
              "cat /proc/bus/input/devices | grep -A5 -i xbox"]),
        (&["xone", "driver"], "check Xbox One controller driver", "gaming",
            &["lsmod | grep xone", "dmesg | grep -i xone | tail -10",
              "pacman -Qs xone 2>/dev/null"]),
        // PlayStation controller
        (&["playstation", "controller"], "check PlayStation controller", "gaming",
            &["lsusb | grep -i 'sony\\|dualshock\\|dualsense'",
              "dmesg | grep -i 'sony\\|playstation' | tail -10"]),
        (&["dualsense", "connect"], "check DualSense connection", "gaming",
            &["lsusb | grep -i sony", "bluetoothctl devices | grep -i controller 2>/dev/null"]),
        // Steam controller
        (&["steam", "controller"], "check Steam controller", "gaming",
            &["lsusb | grep -i valve", "cat /proc/bus/input/devices | grep -A5 -i valve"]),
        // Controller test
        (&["test", "controller"], "test game controller", "gaming",
            &["jstest /dev/input/js0 2>/dev/null || echo 'Install: pacman -S joyutils'",
              "evtest /dev/input/event* 2>/dev/null | head -30"]),
        (&["calibrate", "controller"], "calibrate controller", "gaming",
            &["jscal-store /dev/input/js0 2>/dev/null || echo 'Install: pacman -S joyutils'",
              "echo 'Use jstest-gtk for GUI calibration'"]),
        // Gamepad devices
        (&["gamepad", "device"], "show gamepad devices", "gaming",
            &["ls /dev/input/js* 2>/dev/null", "cat /proc/bus/input/devices | grep -B2 -A5 -i gamepad"]),
        (&["show", "gamepad"], "show gamepad devices", "gaming",
            &["ls /dev/input/js* 2>/dev/null", "evtest --list 2>/dev/null | grep -i game"]),
        // Controller configuration
        (&["controller", "config"], "controller configuration", "gaming",
            &["echo 'Use: antimicrox for button mapping'",
              "echo 'Steam: use Steam Input for controller config'",
              "ls /usr/share/applications/*antimicro* 2>/dev/null"]),
        // Gaming mouse
        (&["gaming", "mouse"], "gaming mouse setup", "gaming",
            &["xinput list 2>/dev/null | grep -i mouse",
              "echo 'Use: piper for Logitech mice, or libratbag'",
              "pacman -Qs piper libratbag 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Gaming graphics queries
fn match_gaming_graphics(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[GamingPattern] = &[
        // Vulkan
        (&["vulkan", "support"], "check Vulkan support", "gaming",
            &["vulkaninfo 2>/dev/null | head -30 || echo 'Install: pacman -S vulkan-tools'",
              "ls /usr/share/vulkan/icd.d/"]),
        (&["vulkan", "driver"], "check Vulkan drivers", "gaming",
            &["pacman -Qs vulkan", "vulkaninfo 2>/dev/null | grep -i 'driver\\|version' | head -10"]),
        (&["vulkan", "version"], "check Vulkan version", "gaming",
            &["vulkaninfo 2>/dev/null | grep -i 'vulkan\\|version' | head -5"]),
        // OpenGL
        (&["opengl", "version"], "check OpenGL version", "gaming",
            &["glxinfo 2>/dev/null | grep -i 'opengl version' || echo 'Install: pacman -S mesa-utils'",
              "glxinfo 2>/dev/null | grep -E 'OpenGL|renderer' | head -5"]),
        (&["opengl", "driver"], "check OpenGL driver", "gaming",
            &["glxinfo 2>/dev/null | grep -E 'renderer|vendor|version' | head -5"]),
        // Gaming performance
        (&["game", "fps"], "monitor game FPS", "gaming",
            &["echo 'Use: MANGOHUD=1 <game> for FPS overlay'",
              "pacman -Qs mangohud 2>/dev/null || echo 'Install: pacman -S mangohud'"]),
        (&["mangohud"], "check MangoHud", "gaming",
            &["pacman -Qs mangohud", "mangohud --version 2>/dev/null",
              "echo 'Usage: MANGOHUD=1 <game>'"]),
        // GameMode
        (&["gamemode", "status"], "check GameMode status", "gaming",
            &["gamemoded -s 2>/dev/null || echo 'GameMode not running'",
              "pacman -Qs gamemode"]),
        (&["gamemode", "install"], "check GameMode installation", "gaming",
            &["pacman -Qs gamemode", "which gamemoderun 2>/dev/null"]),
        // GPU for gaming
        (&["gaming", "gpu"], "check GPU for gaming", "gaming",
            &["lspci -k | grep -A2 -i vga", "glxinfo 2>/dev/null | grep renderer",
              "vulkaninfo 2>/dev/null | grep 'GPU id' | head -3"]),
        // Frame timing
        (&["frame", "time"], "check frame timing", "gaming",
            &["echo 'Use: MANGOHUD=1 MANGOHUD_DLSYM=1 <game> for frame time'",
              "echo 'Or: vkcube --present_mode 2 for Vulkan test'"]),
        // MangoHud stats
        (&["mangohud", "stats"], "show MangoHud stats configuration", "gaming",
            &["cat ~/.config/MangoHud/MangoHud.conf 2>/dev/null | head -30",
              "echo 'Usage: MANGOHUD=1 <game>'",
              "pacman -Q mangohud 2>/dev/null"]),
        (&["show", "mangohud"], "MangoHud info", "gaming",
            &["which mangohud", "cat ~/.config/MangoHud/MangoHud.conf 2>/dev/null"]),
        // vkBasalt
        (&["vkbasalt"], "check vkBasalt", "gaming",
            &["pacman -Q vkbasalt 2>/dev/null || echo 'Install: yay -S vkbasalt'",
              "cat ~/.config/vkBasalt/vkBasalt.conf 2>/dev/null | head -20",
              "echo 'Usage: ENABLE_VKBASALT=1 <game>'"]),
        (&["check", "vkbasalt"], "check vkBasalt status", "gaming",
            &["pacman -Qs vkbasalt", "ls /usr/share/vkBasalt 2>/dev/null"]),
        // Gaming performance
        (&["gaming", "performance"], "check gaming performance settings", "gaming",
            &["cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor | uniq",
              "gamemoded -s 2>/dev/null",
              "echo 'Use: gamemode + mangohud for best performance'"]),
        // GPU usage
        (&["gpu", "usage", "gaming"], "show GPU usage while gaming", "gaming",
            &["nvidia-smi 2>/dev/null | head -20 || radeontop 2>/dev/null",
              "echo 'Use: MANGOHUD=1 for in-game GPU stats'"]),
        (&["show", "gpu", "usage"], "show GPU usage", "gaming",
            &["nvidia-smi 2>/dev/null || radeontop -d- -l1 2>/dev/null || intel_gpu_top 2>/dev/null"]),
        // Esync/Fsync
        (&["esync", "fsync"], "esync vs fsync info", "gaming",
            &["cat /proc/sys/fs/file-max",
              "ulimit -Hn",
              "echo 'Fsync is newer, better. Requires kernel 5.16+'",
              "echo 'Set WINEFSYNC=1 or WINEESYNC=1'"]),
        (&["esync"], "check esync support", "gaming",
            &["ulimit -Hn", "echo 'Need 524288+ for esync'",
              "cat /etc/security/limits.conf | grep nofile"]),
        (&["fsync"], "check fsync support", "gaming",
            &["uname -r", "echo 'Fsync requires kernel 5.16+'",
              "echo 'Enable: WINEFSYNC=1'"]),
        // Shader cache
        (&["shader", "cache"], "show shader cache", "gaming",
            &["du -sh ~/.local/share/Steam/steamapps/shadercache 2>/dev/null",
              "ls ~/.cache/mesa_shader_cache 2>/dev/null | head -10",
              "ls ~/.nv/GLCache 2>/dev/null | head -10"]),
        (&["show", "shader", "cache"], "show shader cache location", "gaming",
            &["echo 'Steam: ~/.local/share/Steam/steamapps/shadercache'",
              "echo 'Mesa: ~/.cache/mesa_shader_cache'",
              "echo 'NVIDIA: ~/.nv/GLCache'"]),
        // NVIDIA Prime
        (&["nvidia", "prime"], "NVIDIA Prime status", "gaming",
            &["prime-run --help 2>/dev/null || echo 'prime-run not available'",
              "cat /etc/prime-discrete 2>/dev/null",
              "echo 'Use: prime-run <game> for NVIDIA GPU'"]),
        // AMD gaming
        (&["amd", "gpu", "gaming"], "AMD GPU gaming setup", "gaming",
            &["pacman -Q mesa vulkan-radeon amdvlk 2>/dev/null",
              "echo 'Enable ACO: RADV_PERFTEST=aco'",
              "vulkaninfo 2>/dev/null | grep -i amd | head -5"]),
        // ProtonDB
        (&["protondb", "compatib"], "ProtonDB compatibility info", "gaming",
            &["echo 'Check: https://www.protondb.com for game compatibility'",
              "echo 'Use Proton-GE for better compatibility'"]),
        // Gaming latency
        (&["gaming", "latency"], "check gaming latency", "gaming",
            &["echo 'Low latency tips:'",
              "echo '1. Use gamemode'",
              "echo '2. Disable compositor'",
              "echo '3. Set CPU governor to performance'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

/// Emulation queries
fn match_emulation(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[GamingPattern] = &[
        // RetroArch
        (&["retroarch", "core"], "list RetroArch cores", "gaming",
            &["ls ~/.config/retroarch/cores 2>/dev/null",
              "ls /usr/lib/libretro 2>/dev/null | head -20"]),
        (&["retroarch", "install"], "check RetroArch installation", "gaming",
            &["which retroarch 2>/dev/null", "pacman -Qs retroarch"]),
        // PCSX2
        (&["pcsx2"], "check PCSX2 installation", "gaming",
            &["which pcsx2 2>/dev/null", "pacman -Qs pcsx2",
              "flatpak list 2>/dev/null | grep -i pcsx2"]),
        // Dolphin
        (&["dolphin", "emulator"], "check Dolphin emulator", "gaming",
            &["which dolphin-emu 2>/dev/null", "pacman -Qs dolphin-emu",
              "flatpak list 2>/dev/null | grep -i dolphin"]),
        // RPCS3
        (&["rpcs3"], "check RPCS3 installation", "gaming",
            &["which rpcs3 2>/dev/null", "flatpak list 2>/dev/null | grep -i rpcs3"]),
        // yuzu/Ryujinx (Switch)
        (&["switch", "emulator"], "check Switch emulators", "gaming",
            &["which yuzu 2>/dev/null || which ryujinx 2>/dev/null",
              "flatpak list 2>/dev/null | grep -iE 'yuzu|ryujinx'"]),
        // BIOS files
        (&["emulator", "bios"], "check emulator BIOS files", "gaming",
            &["ls ~/.config/retroarch/system 2>/dev/null",
              "ls ~/.config/PCSX2/bios 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(make_understanding(interpreted, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steam() {
        assert!(match_patterns("steam installation").is_some());
        assert!(match_patterns("steam games").is_some());
        assert!(match_patterns("steam logs").is_some());
    }

    #[test]
    fn test_wine_proton() {
        assert!(match_patterns("wine version").is_some());
        assert!(match_patterns("proton version").is_some());
        assert!(match_patterns("dxvk version").is_some());
    }

    #[test]
    fn test_controllers() {
        assert!(match_patterns("controller detect").is_some());
        assert!(match_patterns("xbox controller").is_some());
        assert!(match_patterns("test controller").is_some());
    }

    #[test]
    fn test_gaming_graphics() {
        assert!(match_patterns("vulkan support").is_some());
        assert!(match_patterns("opengl version").is_some());
        assert!(match_patterns("mangohud").is_some());
    }

    #[test]
    fn test_emulation() {
        assert!(match_patterns("retroarch cores").is_some());
        assert!(match_patterns("dolphin emulator").is_some());
    }
}
