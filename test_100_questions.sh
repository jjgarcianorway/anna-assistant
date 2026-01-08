#!/bin/bash
# Test Anna with 100 diverse questions
# Categories: System Info, Hardware, Storage, Network, Services, Packages, Security, Performance, Config, Misc

ANNACTL="/home/lhoqvso/anna-assistant/target/release/annactl"
RESULTS_DIR="/home/lhoqvso/anna-assistant/test_results"
mkdir -p "$RESULTS_DIR"

# Function to ask Anna and save result
ask_anna() {
    local num=$1
    local question="$2"
    local category="$3"

    echo "[$num/100] $category: $question"

    # Run anna with timeout
    timeout 120 $ANNACTL "$question" > "$RESULTS_DIR/q${num}.txt" 2>&1
    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo "  ✓ Answered"
    elif [ $exit_code -eq 124 ]; then
        echo "  ✗ Timeout"
        echo "TIMEOUT" > "$RESULTS_DIR/q${num}.txt"
    else
        echo "  ✗ Error ($exit_code)"
    fi

    # Small delay to not overwhelm
    sleep 1
}

echo "Starting 100-question test at $(date)"
echo "Results will be saved to $RESULTS_DIR"
echo ""

# System Info (1-15)
ask_anna 1 "what kernel version am I running?" "SysInfo"
ask_anna 2 "what is my hostname?" "SysInfo"
ask_anna 3 "how long has the system been running?" "SysInfo"
ask_anna 4 "what CPU do I have?" "SysInfo"
ask_anna 5 "how many CPU cores do I have?" "SysInfo"
ask_anna 6 "what is my OS version?" "SysInfo"
ask_anna 7 "what shell am I using?" "SysInfo"
ask_anna 8 "what is my username?" "SysInfo"
ask_anna 9 "what desktop environment am I running?" "SysInfo"
ask_anna 10 "am I using wayland or x11?" "SysInfo"
ask_anna 11 "what display manager is active?" "SysInfo"
ask_anna 12 "what bootloader am I using?" "SysInfo"
ask_anna 13 "what is the system architecture?" "SysInfo"
ask_anna 14 "what timezone am I in?" "SysInfo"
ask_anna 15 "what is the current system load?" "SysInfo"

# Hardware (16-25)
ask_anna 16 "what GPU do I have?" "Hardware"
ask_anna 17 "how much RAM do I have?" "Hardware"
ask_anna 18 "what network interfaces do I have?" "Hardware"
ask_anna 19 "what USB devices are connected?" "Hardware"
ask_anna 20 "what PCI devices are installed?" "Hardware"
ask_anna 21 "what audio devices do I have?" "Hardware"
ask_anna 22 "what storage devices do I have?" "Hardware"
ask_anna 23 "what is my screen resolution?" "Hardware"
ask_anna 24 "what bluetooth devices are available?" "Hardware"
ask_anna 25 "what NVIDIA driver version am I using?" "Hardware"

# Storage (26-40)
ask_anna 26 "how much disk space do I have?" "Storage"
ask_anna 27 "what filesystem is my root partition?" "Storage"
ask_anna 28 "show mounted filesystems" "Storage"
ask_anna 29 "what is the largest directory in home?" "Storage"
ask_anna 30 "how much space is /var using?" "Storage"
ask_anna 31 "are there any full partitions?" "Storage"
ask_anna 32 "what swap do I have configured?" "Storage"
ask_anna 33 "show disk partitions" "Storage"
ask_anna 34 "what is my /boot partition size?" "Storage"
ask_anna 35 "is my disk SSD or HDD?" "Storage"
ask_anna 36 "what is using the most disk space?" "Storage"
ask_anna 37 "how much cache can I clean?" "Storage"
ask_anna 38 "show btrfs subvolumes if any" "Storage"
ask_anna 39 "what is my disk health status?" "Storage"
ask_anna 40 "how many inodes are free?" "Storage"

# Network (41-50)
ask_anna 41 "what is my IP address?" "Network"
ask_anna 42 "what DNS servers am I using?" "Network"
ask_anna 43 "am I connected to the internet?" "Network"
ask_anna 44 "what is my default gateway?" "Network"
ask_anna 45 "show open network ports?" "Network"
ask_anna 46 "what wifi network am I connected to?" "Network"
ask_anna 47 "what is my MAC address?" "Network"
ask_anna 48 "what firewall rules are active?" "Network"
ask_anna 49 "is NetworkManager running?" "Network"
ask_anna 50 "what is my public IP address?" "Network"

# Services (51-65)
ask_anna 51 "what services are running?" "Services"
ask_anna 52 "are there any failed services?" "Services"
ask_anna 53 "is sshd running?" "Services"
ask_anna 54 "is docker installed and running?" "Services"
ask_anna 55 "what services start at boot?" "Services"
ask_anna 56 "is ollama service running?" "Services"
ask_anna 57 "what user services are running?" "Services"
ask_anna 58 "show systemd timers" "Services"
ask_anna 59 "when was the system last booted?" "Services"
ask_anna 60 "what is using port 11434?" "Services"
ask_anna 61 "is cups printing service installed?" "Services"
ask_anna 62 "what is the status of NetworkManager?" "Services"
ask_anna 63 "are there any masked services?" "Services"
ask_anna 64 "show recent systemd logs" "Services"
ask_anna 65 "what services failed in the last hour?" "Services"

# Packages (66-80)
ask_anna 66 "how many packages are installed?" "Packages"
ask_anna 67 "is neovim installed?" "Packages"
ask_anna 68 "when was the last system update?" "Packages"
ask_anna 69 "what packages need updates?" "Packages"
ask_anna 70 "is yay or paru installed?" "Packages"
ask_anna 71 "what version of python is installed?" "Packages"
ask_anna 72 "is firefox installed?" "Packages"
ask_anna 73 "show recently installed packages" "Packages"
ask_anna 74 "what orphan packages exist?" "Packages"
ask_anna 75 "is rust/cargo installed?" "Packages"
ask_anna 76 "what packages depend on openssl?" "Packages"
ask_anna 77 "is git installed and what version?" "Packages"
ask_anna 78 "show AUR packages installed" "Packages"
ask_anna 79 "what kernel packages are installed?" "Packages"
ask_anna 80 "is flatpak installed?" "Packages"

# Security & Performance (81-90)
ask_anna 81 "what processes are using the most CPU?" "Perf"
ask_anna 82 "what processes are using the most memory?" "Perf"
ask_anna 83 "are there any zombie processes?" "Perf"
ask_anna 84 "what is the current memory usage?" "Perf"
ask_anna 85 "what users are logged in?" "Security"
ask_anna 86 "show recent login attempts" "Security"
ask_anna 87 "what sudo rules are configured?" "Security"
ask_anna 88 "are there any world-writable files in /etc?" "Security"
ask_anna 89 "is the firewall enabled?" "Security"
ask_anna 90 "what gpg keys are in my keyring?" "Security"

# Configuration & Misc (91-100)
ask_anna 91 "what is in my fstab?" "Config"
ask_anna 92 "show my pacman mirrors" "Config"
ask_anna 93 "what locale is configured?" "Config"
ask_anna 94 "is reflector installed for mirror updates?" "Config"
ask_anna 95 "what graphics driver is loaded?" "Config"
ask_anna 96 "show environment variables" "Config"
ask_anna 97 "what is my default editor?" "Config"
ask_anna 98 "is paccache configured for cleanup?" "Config"
ask_anna 99 "what session type is active?" "Config"
ask_anna 100 "summarize my system" "Config"

echo ""
echo "Test completed at $(date)"
echo "Results saved to $RESULTS_DIR"

# Count successes
success=$(ls -1 "$RESULTS_DIR"/*.txt 2>/dev/null | xargs grep -L "TIMEOUT\|Error\|error" 2>/dev/null | wc -l)
echo "Successful answers: $success/100"
