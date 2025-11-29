#!/bin/bash
# Anna Installer v3.0.0
#
# This installer is versioned INDEPENDENTLY from Anna itself.
# Installer version: 3.x.x
# Anna version: fetched from GitHub releases
#
# Behavior:
#   - Detects installed version (if any)
#   - Fetches latest Anna version from GitHub
#   - Compares and shows planned action
#   - Non-interactive default: update if newer, skip if same/older
#   - Interactive mode (-i): prompt for confirmation
#   - Never clobbers config/data unless --reset is passed
#   - Logs all actions to /var/log/anna/install.log
#
# v3.0.0 (Autoprovision):
#   - Anna now self-provisions LLM models at first run
#   - Installer just ensures Ollama is available
#   - annad benchmarks and selects best Junior/Senior models automatically
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/.../install.sh | bash
#   curl -sSL ... | bash -s -- -i          # Interactive mode
#   curl -sSL ... | bash -s -- --reset     # Full reinstall (wipes config)
#
# Exit codes:
#   0 = Success or no action needed
#   1 = Error
#   2 = User declined

# Note: Using set -u (undefined vars) but NOT set -e (exit on error)
# because we handle errors explicitly and set -e can cause unexpected exits
set -uo pipefail

# ============================================================
# CONFIGURATION
# ============================================================

# Installer version (independent from Anna version)
INSTALLER_VERSION="3.1.0"
GITHUB_REPO="jjgarcianorway/anna-assistant"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/anna"
DATA_DIR="/var/lib/anna"
LOG_DIR="/var/log/anna"
RUN_DIR="/run/anna"
PROBES_DIR="/usr/share/anna/probes"

# ============================================================
# COLORS AND FORMATTING (ASCII-only, sysadmin style)
# ============================================================

BOLD=$'\033[1m'
DIM=$'\033[2m'
RED=$'\033[31m'
GREEN=$'\033[32m'
YELLOW=$'\033[33m'
BLUE=$'\033[34m'
CYAN=$'\033[36m'
NC=$'\033[0m'

# ============================================================
# VARIABLES
# ============================================================

TMP_DIR=""
INTERACTIVE=false
RESET_MODE=false
INSTALLED_VERSION=""
LATEST_VERSION=""
SUDO=""
JUNIOR_MODEL=""
SENIOR_MODEL=""

# ============================================================
# LOGGING
# ============================================================

log_to_file() {
    local msg="$1"
    local log_file="/var/log/anna/install.log"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    # Create log dir if missing (best effort)
    if [[ -w "$(dirname "$log_file")" ]] || sudo mkdir -p "$(dirname "$log_file")" 2>/dev/null; then
        echo "[$timestamp] $msg" | sudo tee -a "$log_file" >/dev/null 2>/dev/null || true
    fi
}

print_line() {
    printf "%s\n" "------------------------------------------------------------"
}

print_header() {
    printf "\n${BOLD}%s${NC}\n" "$1"
    print_line
}

log_info() {
    printf "  ${BLUE}*${NC}  %s\n" "$1"
}

log_ok() {
    printf "  ${GREEN}+${NC}  %s\n" "$1"
}

log_warn() {
    printf "  ${YELLOW}~${NC}  %s\n" "$1"
}

log_error() {
    printf "  ${RED}!${NC}  %s\n" "$1"
}

# ============================================================
# UTILITY FUNCTIONS
# ============================================================

cleanup() {
    if [[ -n "$TMP_DIR" ]] && [[ -d "$TMP_DIR" ]]; then
        rm -rf "$TMP_DIR"
    fi
}

check_sudo() {
    if command -v sudo &>/dev/null; then
        SUDO="sudo"
    elif [[ $EUID -eq 0 ]]; then
        SUDO=""
    else
        log_error "sudo not found and not running as root"
        exit 1
    fi
}

request_sudo() {
    if [[ -n "$SUDO" ]]; then
        printf "\n  Sudo required for installation to ${INSTALL_DIR}\n"
        $SUDO -v || {
            log_error "Failed to obtain sudo"
            exit 1
        }
    fi
}

# Compare semantic versions: returns 0 if $1 < $2, 1 if $1 == $2, 2 if $1 > $2
version_compare() {
    local v1="$1"
    local v2="$2"

    # Strip leading 'v' if present
    v1="${v1#v}"
    v2="${v2#v}"

    if [[ "$v1" == "$v2" ]]; then
        echo 1
        return
    fi

    # Compare using sort -V
    local sorted
    sorted=$(printf '%s\n%s' "$v1" "$v2" | sort -V | head -1)
    if [[ "$sorted" == "$v1" ]]; then
        echo 0  # v1 < v2
    else
        echo 2  # v1 > v2
    fi
}

# ============================================================
# VERSION DETECTION
# ============================================================

detect_installed_version() {
    INSTALLED_VERSION=""

    # Try annactl -V first (primary method)
    # Use timeout and redirect stdin to /dev/null to prevent any stdin reading
    if command -v annactl &>/dev/null; then
        local output
        output=$(timeout 5 annactl -V </dev/null 2>&1 | head -20) || true
        # Look for "annactl vX.Y.Z" (v0.14.4+) or "Anna Assistant vX.Y.Z" (legacy)
        if [[ "$output" =~ annactl[[:space:]]+v?([0-9]+\.[0-9]+\.[0-9]+) ]]; then
            INSTALLED_VERSION="${BASH_REMATCH[1]}"
        elif [[ "$output" =~ Anna[[:space:]]+(Assistant[[:space:]]+)?v?([0-9]+\.[0-9]+\.[0-9]+) ]]; then
            INSTALLED_VERSION="${BASH_REMATCH[2]}"
        fi
    fi

    # Fallback: check binary for embedded CARGO_PKG_VERSION
    # Note: Don't use generic strings search as it picks up library versions
    if [[ -z "$INSTALLED_VERSION" ]] && [[ -x "${INSTALL_DIR}/annactl" ]]; then
        local output
        # Look specifically for Anna version patterns in the binary
        output=$(strings "${INSTALL_DIR}/annactl" 2>/dev/null | grep -E 'Anna.*[0-9]+\.[0-9]+\.[0-9]+' | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1) || true
        if [[ -n "$output" ]]; then
            INSTALLED_VERSION="$output"
        fi
    fi
}

fetch_latest_version() {
    LATEST_VERSION=""

    local api_url="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
    local response

    if command -v curl &>/dev/null; then
        response=$(curl -fsSL "$api_url" 2>/dev/null || true)
    elif command -v wget &>/dev/null; then
        response=$(wget -qO- "$api_url" 2>/dev/null || true)
    fi

    if [[ -n "$response" ]]; then
        # Extract tag_name from JSON
        LATEST_VERSION=$(echo "$response" | grep -oP '"tag_name":\s*"v?\K[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)
    fi

    # Fail if we can't fetch the version - don't use stale hardcoded version
    if [[ -z "$LATEST_VERSION" ]]; then
        log_error "Could not fetch latest version from GitHub"
        log_error "Check your internet connection or try again later"
        exit 1
    fi
}

detect_arch() {
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64)
            ARCH_NAME="x86_64-unknown-linux-gnu"
            ;;
        aarch64)
            ARCH_NAME="aarch64-unknown-linux-gnu"
            ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac
}

# ============================================================
# DISPLAY FUNCTIONS
# ============================================================

print_banner() {
    printf "\n${BOLD}Anna Assistant Installer${NC}\n"
    print_line
    printf "  Installer version: %s\n" "$INSTALLER_VERSION"
    printf "  Architecture: %s\n" "$ARCH_NAME"
    print_line
}

print_version_summary() {
    print_header "VERSION INFORMATION"

    if [[ -z "$INSTALLED_VERSION" ]]; then
        printf "  Installed version : ${DIM}(none)${NC}\n"
    else
        printf "  Installed version : v%s\n" "$INSTALLED_VERSION"
    fi

    printf "  Available version : v%s\n" "$LATEST_VERSION"
    printf "\n"
}

print_planned_action() {
    print_header "PLANNED ACTION"

    if [[ -z "$INSTALLED_VERSION" ]]; then
        printf "  ${GREEN}Fresh install${NC} - Anna will be installed\n"
        PLANNED_ACTION="install"
    else
        local cmp
        cmp=$(version_compare "$INSTALLED_VERSION" "$LATEST_VERSION")

        case "$cmp" in
            0)  # Installed < Latest
                printf "  ${GREEN}Update available${NC} - v%s -> v%s\n" "$INSTALLED_VERSION" "$LATEST_VERSION"
                PLANNED_ACTION="update"
                ;;
            1)  # Installed == Latest
                printf "  ${CYAN}Same version${NC} - v%s already installed\n" "$INSTALLED_VERSION"
                PLANNED_ACTION="reinstall"
                ;;
            2)  # Installed > Latest
                printf "  ${YELLOW}Downgrade${NC} - v%s -> v%s (not recommended)\n" "$INSTALLED_VERSION" "$LATEST_VERSION"
                PLANNED_ACTION="downgrade"
                ;;
        esac
    fi

    printf "\n  Target paths:\n"
    printf "    Binaries: %s\n" "$INSTALL_DIR"
    printf "    Config:   %s\n" "$CONFIG_DIR"
    printf "    Data:     %s\n" "$DATA_DIR"
    printf "    Logs:     %s\n" "$LOG_DIR"
    print_line
}

# ============================================================
# USER INTERACTION
# ============================================================

confirm_action() {
    local action="$1"

    if [[ "$INTERACTIVE" == "false" ]]; then
        # Non-interactive defaults
        case "$action" in
            install|update|reinstall)
                return 0  # Proceed - always allow install/update/reinstall
                ;;
            downgrade)
                log_warn "Downgrade not allowed in non-interactive mode"
                return 1  # Skip
                ;;
        esac
    fi

    # Interactive mode
    local prompt=""
    local default=""

    case "$action" in
        install)
            prompt="Proceed with installation? (Y/n)"
            default="y"
            ;;
        update)
            prompt="Proceed with update? (Y/n)"
            default="y"
            ;;
        reinstall)
            prompt="Reinstall same version? (y/N)"
            default="n"
            ;;
        downgrade)
            prompt="Downgrade to older version? (y/N)"
            default="n"
            ;;
    esac

    printf "\n  %s " "$prompt"
    read -r answer
    answer="${answer:-$default}"

    case "${answer,,}" in
        y|yes)
            return 0
            ;;
        *)
            log_info "Installation cancelled by user"
            return 1
            ;;
    esac
}

# ============================================================
# INSTALLATION FUNCTIONS
# ============================================================

download_binaries() {
    log_info "Downloading binaries..."

    local base_url="https://github.com/${GITHUB_REPO}/releases/download/v${LATEST_VERSION}"
    TMP_DIR=$(mktemp -d)

    # Determine binary names based on architecture
    local annad_name annactl_name
    case "$ARCH" in
        x86_64|x86_64-unknown-linux-gnu)
            annad_name="annad-${LATEST_VERSION}-x86_64-unknown-linux-gnu"
            annactl_name="annactl-${LATEST_VERSION}-x86_64-unknown-linux-gnu"
            ;;
        aarch64|aarch64-unknown-linux-gnu)
            annad_name="annad-${LATEST_VERSION}-aarch64-unknown-linux-gnu"
            annactl_name="annactl-${LATEST_VERSION}-aarch64-unknown-linux-gnu"
            ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            return 1
            ;;
    esac

    # Download individual binaries and checksums
    log_info "Downloading annad..."
    if command -v curl &>/dev/null; then
        curl -fsSL "${base_url}/${annad_name}" -o "${TMP_DIR}/annad" || {
            log_error "Failed to download ${annad_name}"
            return 1
        }
        log_info "Downloading annactl..."
        curl -fsSL "${base_url}/${annactl_name}" -o "${TMP_DIR}/annactl" || {
            log_error "Failed to download ${annactl_name}"
            return 1
        }
        log_info "Downloading checksums..."
        curl -fsSL "${base_url}/SHA256SUMS" -o "${TMP_DIR}/SHA256SUMS" || {
            log_error "Failed to download checksums"
            return 1
        }
    else
        wget -q "${base_url}/${annad_name}" -O "${TMP_DIR}/annad" || return 1
        wget -q "${base_url}/${annactl_name}" -O "${TMP_DIR}/annactl" || return 1
        wget -q "${base_url}/SHA256SUMS" -O "${TMP_DIR}/SHA256SUMS" || return 1
    fi

    log_ok "Downloaded binaries"

    # Verify checksums
    log_info "Verifying checksums..."
    cd "$TMP_DIR"

    local annad_sum annactl_sum
    annad_sum=$(sha256sum "annad" | awk '{print $1}')
    annactl_sum=$(sha256sum "annactl" | awk '{print $1}')

    # Check annad checksum
    if grep -q "$annad_sum" SHA256SUMS; then
        log_ok "annad checksum verified"
    else
        log_error "annad checksum verification failed!"
        return 1
    fi

    # Check annactl checksum
    if grep -q "$annactl_sum" SHA256SUMS; then
        log_ok "annactl checksum verified"
    else
        log_error "annactl checksum verification failed!"
        return 1
    fi

    # Set execute permissions
    chmod 755 "${TMP_DIR}/annad" "${TMP_DIR}/annactl"
    log_ok "Binaries ready for installation"
}

install_binaries() {
    log_info "Installing binaries..."

    # Backup existing if updating
    if [[ -f "${INSTALL_DIR}/annad" ]]; then
        $SUDO cp "${INSTALL_DIR}/annad" "${INSTALL_DIR}/annad.bak" 2>/dev/null || true
    fi
    if [[ -f "${INSTALL_DIR}/annactl" ]]; then
        $SUDO cp "${INSTALL_DIR}/annactl" "${INSTALL_DIR}/annactl.bak" 2>/dev/null || true
    fi

    # Install binaries (atomic move)
    $SUDO mv "${TMP_DIR}/annad" "${INSTALL_DIR}/annad"
    $SUDO mv "${TMP_DIR}/annactl" "${INSTALL_DIR}/annactl"
    $SUDO chmod 755 "${INSTALL_DIR}/annad"
    $SUDO chmod 755 "${INSTALL_DIR}/annactl"

    log_ok "Installed binaries to ${INSTALL_DIR}"

    # Install probe definitions
    if [[ -d "${TMP_DIR}/probes" ]]; then
        log_info "Installing probe definitions..."
        $SUDO mkdir -p "$PROBES_DIR"
        $SUDO cp -r "${TMP_DIR}/probes/"* "$PROBES_DIR/" 2>/dev/null || true
        log_ok "Installed probe definitions to ${PROBES_DIR}"
    fi
}

# ============================================================
# OLLAMA AND MODEL INSTALLATION
# ============================================================

SELECTED_MODEL=""

# ============================================================
# GPU DETECTION v2.4.0
# ============================================================
# Comprehensive GPU detection:
# - Detects ALL GPUs via lspci
# - Checks driver status for each vendor
# - Shows clear summary
# - NEVER installs drivers (just reports)

# Arrays to store detected GPUs and their status
declare -a DETECTED_GPUS=()
declare -a GPU_STATUS=()
declare -a GPU_VRAM=()
BEST_VRAM_MB=0
BEST_GPU_TYPE=""

detect_all_gpus() {
    # Reset globals
    DETECTED_GPUS=()
    GPU_STATUS=()
    GPU_VRAM=()
    BEST_VRAM_MB=0
    BEST_GPU_TYPE=""

    if ! command -v lspci &>/dev/null; then
        # No lspci - try nvidia-smi/rocm-smi directly
        check_nvidia_driver
        check_amd_driver
        return
    fi

    # Detect NVIDIA GPUs (VGA and 3D controllers)
    while IFS= read -r line; do
        if [[ -n "$line" ]]; then
            local gpu_name
            gpu_name=$(echo "$line" | sed 's/.*: //')
            DETECTED_GPUS+=("NVIDIA: $gpu_name")
            GPU_STATUS+=("checking")
            GPU_VRAM+=("0")
        fi
    done < <(lspci 2>/dev/null | grep -iE "(VGA|3D).*NVIDIA" || true)

    # Detect AMD GPUs (discrete and integrated)
    # NOTE: Use word boundaries (\b) to avoid false matches like "compatible" containing "ati"
    while IFS= read -r line; do
        if [[ -n "$line" ]]; then
            local gpu_name
            gpu_name=$(echo "$line" | sed 's/.*: //')
            # Check if it's an integrated GPU (APU)
            if echo "$gpu_name" | grep -qiE "Renoir|Cezanne|Barcelo|Rembrandt|Phoenix|Raphael|Hawk|Strix"; then
                DETECTED_GPUS+=("AMD iGPU: $gpu_name")
            else
                DETECTED_GPUS+=("AMD: $gpu_name")
            fi
            GPU_STATUS+=("checking")
            GPU_VRAM+=("0")
        fi
    done < <(lspci 2>/dev/null | grep -iE "(VGA|3D).*(\bAMD\b|\bATI\b|Radeon)" || true)

    # Detect Intel GPUs (integrated)
    while IFS= read -r line; do
        if [[ -n "$line" ]]; then
            local gpu_name
            gpu_name=$(echo "$line" | sed 's/.*: //')
            DETECTED_GPUS+=("Intel iGPU: $gpu_name")
            GPU_STATUS+=("no-ollama-support")
            GPU_VRAM+=("0")
        fi
    done < <(lspci 2>/dev/null | grep -iE "(VGA|3D).*Intel" || true)

    # Now check driver status for each GPU
    check_nvidia_driver
    check_amd_driver
}

check_nvidia_driver() {
    local nvidia_vram=0

    if command -v nvidia-smi &>/dev/null; then
        local gpu_info
        gpu_info=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits 2>/dev/null | head -1) || true
        if [[ -n "$gpu_info" ]] && [[ "$gpu_info" =~ ^[0-9]+$ ]]; then
            nvidia_vram="$gpu_info"
        fi
    fi

    # Update status for all NVIDIA GPUs
    for i in "${!DETECTED_GPUS[@]}"; do
        if [[ "${DETECTED_GPUS[$i]}" == NVIDIA:* ]]; then
            if [[ "$nvidia_vram" -gt 0 ]]; then
                GPU_STATUS[$i]="driver-ok"
                GPU_VRAM[$i]="$nvidia_vram"
                if [[ "$nvidia_vram" -gt "$BEST_VRAM_MB" ]]; then
                    BEST_VRAM_MB="$nvidia_vram"
                    BEST_GPU_TYPE="nvidia"
                fi
            else
                GPU_STATUS[$i]="no-driver"
            fi
        fi
    done

    # If no lspci but nvidia-smi works, add the GPU
    if [[ "${#DETECTED_GPUS[@]}" -eq 0 ]] && [[ "$nvidia_vram" -gt 0 ]]; then
        DETECTED_GPUS+=("NVIDIA GPU (detected via nvidia-smi)")
        GPU_STATUS+=("driver-ok")
        GPU_VRAM+=("$nvidia_vram")
        BEST_VRAM_MB="$nvidia_vram"
        BEST_GPU_TYPE="nvidia"
    fi
}

check_amd_driver() {
    local amd_vram=0

    if command -v rocm-smi &>/dev/null; then
        local vram_bytes
        vram_bytes=$(rocm-smi --showmeminfo vram 2>/dev/null | grep -oE '[0-9]+' | head -1) || true
        if [[ -n "$vram_bytes" ]] && [[ "$vram_bytes" -gt 0 ]]; then
            # rocm-smi reports in bytes, convert to MB
            amd_vram=$((vram_bytes / 1024 / 1024))
        fi
    fi

    # Update status for all AMD GPUs (discrete only, not iGPU)
    for i in "${!DETECTED_GPUS[@]}"; do
        if [[ "${DETECTED_GPUS[$i]}" == AMD:* ]] && [[ "${DETECTED_GPUS[$i]}" != *iGPU* ]]; then
            if [[ "$amd_vram" -gt 0 ]]; then
                GPU_STATUS[$i]="driver-ok"
                GPU_VRAM[$i]="$amd_vram"
                if [[ "$amd_vram" -gt "$BEST_VRAM_MB" ]]; then
                    BEST_VRAM_MB="$amd_vram"
                    BEST_GPU_TYPE="amd"
                fi
            else
                GPU_STATUS[$i]="no-driver"
            fi
        elif [[ "${DETECTED_GPUS[$i]}" == *"AMD iGPU"* ]]; then
            # iGPUs use shared system RAM, not dedicated VRAM
            GPU_STATUS[$i]="igpu-shared-ram"
        fi
    done
}

print_gpu_detection() {
    print_header "GPU DETECTION"

    if [[ "${#DETECTED_GPUS[@]}" -eq 0 ]]; then
        log_info "No discrete GPU detected"
        log_info "Ollama will use CPU mode"
        printf "\n"
        return
    fi

    # Print each GPU with status
    for i in "${!DETECTED_GPUS[@]}"; do
        local gpu="${DETECTED_GPUS[$i]}"
        local status="${GPU_STATUS[$i]}"
        local vram="${GPU_VRAM[$i]}"

        case "$status" in
            driver-ok)
                if [[ "$vram" -gt 0 ]]; then
                    log_ok "$gpu"
                    log_info "    VRAM: ${vram}MB | Driver: OK"
                else
                    log_ok "$gpu"
                    log_info "    Driver: OK"
                fi
                ;;
            no-driver)
                log_warn "$gpu"
                log_info "    Driver: NOT FOUND"
                ;;
            no-ollama-support)
                log_info "$gpu"
                log_info "    Status: No Ollama GPU support"
                ;;
            igpu-shared-ram)
                log_info "$gpu"
                log_info "    Status: Integrated (uses system RAM)"
                ;;
            *)
                log_info "$gpu"
                log_info "    Status: Unknown"
                ;;
        esac
    done

    printf "\n"

    # Explain what Ollama will use
    if [[ "$BEST_VRAM_MB" -gt 0 ]]; then
        log_ok "Ollama will use: ${BEST_GPU_TYPE^^} GPU (${BEST_VRAM_MB}MB VRAM)"
    else
        # Check if we have GPUs but no working drivers
        local has_nvidia=false
        local has_amd=false
        for i in "${!DETECTED_GPUS[@]}"; do
            if [[ "${GPU_STATUS[$i]}" == "no-driver" ]]; then
                if [[ "${DETECTED_GPUS[$i]}" == NVIDIA:* ]]; then
                    has_nvidia=true
                elif [[ "${DETECTED_GPUS[$i]}" == AMD:* ]]; then
                    has_amd=true
                fi
            fi
        done

        if $has_nvidia || $has_amd; then
            log_warn "GPU detected but drivers not working"
            log_info "Ollama will use: CPU mode"
            printf "\n"
            log_info "To enable GPU acceleration, install drivers:"
            if $has_nvidia; then
                log_info "  NVIDIA: Install nvidia-driver package"
            fi
            if $has_amd; then
                log_info "  AMD: Install ROCm (rocm-smi)"
            fi
        else
            log_info "Ollama will use: CPU mode"
        fi
    fi

    printf "\n"
}

# Legacy function for compatibility - returns VRAM for model selection
detect_gpu() {
    # Run full detection if not already done
    if [[ "${#DETECTED_GPUS[@]}" -eq 0 ]]; then
        detect_all_gpus
    fi
    echo "$BEST_VRAM_MB"
}

select_model() {
    local vram_mb
    vram_mb=$(detect_gpu)

    # First, check if a good model is already installed
    local installed_models
    installed_models=$(ollama list 2>/dev/null || echo "")

    # Prioritized list of known-good models for Anna (best to acceptable)
    # Criteria: JSON reliability, SPEED (critical for agent loops), reasoning ability
    #
    # Note: Llama 3 8B is ~3x faster than Qwen 2 7B at similar quality!
    # For agent communication loops, speed matters as much as quality.
    #
    # Large models (14B+) - excellent quality, need 16GB+ VRAM
    # MoE models like qwen3:30b-a3b have many params but fewer active (efficient)
    # 2025 update: Qwen3 is prioritized for JSON/agent tasks
    local large_models=(
        "qwen3:30b-a3b"
        "qwen3:32b"
        "qwen3:14b"
        "qwen2.5:72b"
        "qwen2.5:32b"
        "qwen2.5:14b"
        "llama3.1:70b"
        "mixtral:8x7b"
        "deepseek-coder:33b"
        "gemma3:27b"
    )

    # Medium models (7-8B) - the sweet spot for most users
    # 2025 update: Qwen3 8B prioritized for JSON reliability + agent tasks
    # These run on 6-12GB VRAM or 16GB system RAM (4-bit quantized)
    local medium_models=(
        "qwen3:8b"
        "qwen3:4b"
        "qwen2.5:7b"
        "llama3.1:8b"
        "llama3:8b"
        "gemma3:9b"
        "gemma2:9b"
        "mistral:7b-instruct"
        "mistral:7b"
        "deepseek-coder:6.7b"
        "codellama:7b"
        "phi3:medium"
    )

    # Small models (1-4B) - for low VRAM/CPU, still good JSON output
    # 2025 update: Qwen3 tiny models excellent for routing/junior tasks
    # Can run on <6GB VRAM or 8GB system RAM
    local small_models=(
        "qwen3:1.7b"
        "qwen3:0.6b"
        "qwen3:4b"
        "llama3.2:3b"
        "qwen2.5:3b"
        "gemma3:1b"
        "gemma2:2b"
        "phi3:mini"
    )

    # Check for already-installed large models first
    for model in "${large_models[@]}"; do
        local model_name="${model%%:*}"
        local model_tag="${model##*:}"
        if echo "$installed_models" | grep -qE "^${model_name}:${model_tag}[[:space:]]"; then
            SELECTED_MODEL="$model"
            log_ok "Using already installed ${model} (large model - excellent quality)"
            return
        fi
    done

    # Check for medium models
    for model in "${medium_models[@]}"; do
        local model_name="${model%%:*}"
        local model_tag="${model##*:}"
        if echo "$installed_models" | grep -qE "^${model_name}:${model_tag}[[:space:]]"; then
            SELECTED_MODEL="$model"
            log_ok "Using already installed ${model} (medium model - good quality)"
            return
        fi
    done

    # Check for small models (only if we don't have much VRAM)
    if [[ "$vram_mb" -lt 8000 ]]; then
        for model in "${small_models[@]}"; do
            local model_name="${model%%:*}"
            local model_tag="${model##*:}"
            if echo "$installed_models" | grep -qE "^${model_name}:${model_tag}[[:space:]]"; then
                SELECTED_MODEL="$model"
                log_ok "Using already installed ${model} (small model)"
                return
            fi
        done
    fi

    # No suitable model installed, select based on VRAM and download
    # 2025 update: Qwen3 preferred for JSON/agent tasks (better structured output)
    #
    # VRAM requirements (4-bit quantized):
    #   - 0.6-1.7B: <4GB VRAM or CPU-only
    #   - 4B: 4-6GB VRAM
    #   - 8B: 6-12GB VRAM
    #   - 14B: 12-16GB VRAM
    #   - 32B: 24GB+ VRAM
    #   - 72B: 48GB+ VRAM (datacenter)
    if [[ "$vram_mb" -ge 48000 ]]; then
        # 48GB+ VRAM: Datacenter - use 72B
        SELECTED_MODEL="qwen2.5:72b"
        log_ok "Datacenter GPU (${vram_mb}MB VRAM) - will download qwen2.5:72b"
    elif [[ "$vram_mb" -ge 24000 ]]; then
        # 24-48GB VRAM: Use 32B or MoE 30B
        SELECTED_MODEL="qwen3:32b"
        log_ok "High-end GPU (${vram_mb}MB VRAM) - will download qwen3:32b"
    elif [[ "$vram_mb" -ge 16000 ]]; then
        # 16-24GB VRAM: 14B runs comfortably (RTX 3090/4090 class)
        SELECTED_MODEL="qwen3:14b"
        log_ok "Strong GPU (${vram_mb}MB VRAM) - will download qwen3:14b"
    elif [[ "$vram_mb" -ge 6000 ]]; then
        # 6-16GB VRAM: Sweet spot - Qwen3 8B excellent for agents
        SELECTED_MODEL="qwen3:8b"
        log_ok "Mid-range GPU (${vram_mb}MB VRAM) - will download qwen3:8b (great for agents)"
    elif [[ "$vram_mb" -ge 4000 ]]; then
        # 4-6GB VRAM: Qwen3 4B runs well
        SELECTED_MODEL="qwen3:4b"
        log_ok "Low-mid GPU (${vram_mb}MB VRAM) - will download qwen3:4b"
    else
        # CPU only or very low VRAM: Use 1.7B model (fast + good JSON)
        SELECTED_MODEL="qwen3:1.7b"
        if [[ "$vram_mb" -eq 0 ]]; then
            log_warn "No GPU detected - will download qwen3:1.7b (CPU mode, fast)"
        else
            log_ok "Low GPU memory (${vram_mb}MB) - will download qwen3:1.7b"
        fi
    fi
}

install_ollama() {
    # Detect all GPUs FIRST and show summary
    detect_all_gpus
    print_gpu_detection

    print_header "OLLAMA SETUP"

    # Check if Ollama is installed
    if command -v ollama &>/dev/null; then
        local ollama_version
        ollama_version=$(ollama --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1) || ollama_version="unknown"
        log_ok "Ollama v${ollama_version} already installed"
    else
        log_info "Installing Ollama..."

        # Use official Ollama installer
        if command -v curl &>/dev/null; then
            curl -fsSL https://ollama.com/install.sh | $SUDO sh || {
                log_error "Failed to install Ollama"
                log_warn "Install manually: https://ollama.com/download"
                return 1
            }
        else
            wget -qO- https://ollama.com/install.sh | $SUDO sh || {
                log_error "Failed to install Ollama"
                return 1
            }
        fi

        log_ok "Ollama installed successfully"

        # Start Ollama service
        if command -v systemctl &>/dev/null; then
            $SUDO systemctl enable ollama 2>/dev/null || true
            $SUDO systemctl start ollama 2>/dev/null || true
            sleep 2  # Give Ollama time to start
        fi
    fi

    # Select appropriate model based on detected GPU (uses BEST_VRAM_MB from detect_all_gpus)
    select_model

    # Determine junior/senior models based on hardware
    # 2025: Use Qwen3 for better JSON/agent support
    local senior_model="$SELECTED_MODEL"
    local junior_model="qwen3:1.7b"  # Default: fast 1.7B for junior (great for routing)

    # Adjust junior based on senior model size
    case "$senior_model" in
        *72b*|*70b*)
            # Very large senior -> use 8B junior for balance
            junior_model="qwen3:8b"
            ;;
        *32b*|*30b*)
            # Large senior -> use 4B junior
            junior_model="qwen3:4b"
            ;;
        *14b*)
            # Medium-large senior -> use 4B junior
            junior_model="qwen3:4b"
            ;;
        *8b*|*9b*)
            # Medium senior -> use 1.7B junior (fast)
            junior_model="qwen3:1.7b"
            ;;
        *)
            # Small senior -> junior same as senior (single model mode)
            junior_model="$senior_model"
            ;;
    esac

    # Export for use in write_config
    JUNIOR_MODEL="$junior_model"
    SENIOR_MODEL="$senior_model"

    # Show role-specific model selection
    print_header "MODEL SELECTION"
    log_info "Anna uses TWO models for optimal performance:"
    log_info "  Junior (LLM-A): Fast model for quick probe execution"
    log_info "  Senior (LLM-B): Smarter model for reasoning & synthesis"
    log_info ""
    log_ok "Selected models for your hardware:"
    log_ok "  Junior: ${junior_model}"
    log_ok "  Senior: ${senior_model}"
    log_info ""

    # Pull BOTH models
    print_header "DOWNLOADING MODELS"

    # Pull junior model
    pull_model_if_needed "$junior_model" "junior"

    # Pull senior model (if different from junior)
    if [[ "$senior_model" != "$junior_model" ]]; then
        pull_model_if_needed "$senior_model" "senior"
    else
        log_ok "Senior model same as junior (${senior_model})"
    fi
}

# Helper: Pull model if not already available
pull_model_if_needed() {
    local model="$1"
    local role="$2"

    local model_name="${model%%:*}"
    local model_tag="${model##*:}"

    log_info "Checking ${role} model: ${model}..."

    if ollama list 2>/dev/null | grep -E "^${model_name}:${model_tag}[[:space:]]" >/dev/null 2>&1; then
        log_ok "${role^} model ${model} already available"
    else
        log_info "Pulling ${role} model ${model} (this may take a few minutes)..."

        if ollama pull "$model" 2>&1; then
            log_ok "${role^} model ${model} downloaded successfully"
        else
            log_error "Failed to pull ${role} model ${model}"
            log_warn "You can manually pull later: ollama pull ${model}"
        fi
    fi
}

create_user_and_dirs() {
    log_info "Creating user and directories..."

    # v2.1.0: Detect runtime user - who will run annactl
    # This is typically the user who invoked the installer (via sudo)
    local runtime_user="${SUDO_USER:-root}"
    local runtime_group
    if [[ -n "${SUDO_USER:-}" ]]; then
        runtime_group=$(id -gn "$SUDO_USER" 2>/dev/null || echo "$SUDO_USER")
    else
        runtime_group="root"
    fi
    log_info "Runtime user detected: ${runtime_user}:${runtime_group}"

    # Create anna system user if not exists (for daemon)
    if ! id "anna" &>/dev/null; then
        $SUDO useradd -r -s /bin/false -d "$DATA_DIR" anna 2>/dev/null || true
        log_ok "Created system user 'anna'"
    else
        log_ok "System user 'anna' exists"
    fi

    # Create directories (never wipe existing)
    $SUDO mkdir -p "$DATA_DIR" "$LOG_DIR" "$RUN_DIR" "$CONFIG_DIR" "$PROBES_DIR"

    # v0.11.0: Knowledge store directory
    $SUDO mkdir -p "${DATA_DIR}/knowledge"

    # v1.1.0: XP tracking and stats directories
    $SUDO mkdir -p "${DATA_DIR}/xp"
    $SUDO mkdir -p "${DATA_DIR}/knowledge/stats"

    # v2.0.0: LLM autoprovision directories
    $SUDO mkdir -p "${DATA_DIR}/llm"
    $SUDO mkdir -p "${DATA_DIR}/llm/benchmarks"

    # v2.1.0: Set permissions - CRITICAL for XP and telemetry to work
    # Config: readable by all
    $SUDO chmod 755 "$CONFIG_DIR"

    # Data directory: writable by runtime user (for annactl XP tracking)
    # and readable/writable by root (for annad daemon)
    $SUDO chmod 755 "$DATA_DIR"
    $SUDO chown -R "${runtime_user}:${runtime_group}" "$DATA_DIR"
    # Make data accessible to root daemon as well
    $SUDO chmod -R g+rw "$DATA_DIR"

    # Log directory: writable by runtime user (for telemetry)
    $SUDO chmod 755 "$LOG_DIR"
    $SUDO chown -R "${runtime_user}:${runtime_group}" "$LOG_DIR"
    $SUDO chmod -R g+rw "$LOG_DIR"

    # Run directory: root only (for daemon socket)
    $SUDO chown root:root "$RUN_DIR"
    $SUDO chmod 755 "$RUN_DIR"

    log_ok "Created directories with correct permissions"
    log_ok "  Data: ${DATA_DIR} (owner: ${runtime_user})"
    log_ok "  Logs: ${LOG_DIR} (owner: ${runtime_user})"
    log_ok "  XP: ${DATA_DIR}/xp"
    log_ok "  Knowledge: ${DATA_DIR}/knowledge"
}

install_systemd_service() {
    log_info "Installing systemd service..."

    local service_file="/etc/systemd/system/annad.service"

    # Always update the service file to ensure correct restart behavior
    # (Unlike config, service file has no user customizations to preserve)
    $SUDO tee "$service_file" > /dev/null << 'EOF'
[Unit]
Description=Anna Daemon - Evidence Oracle
Documentation=https://github.com/jjgarcianorway/anna-assistant
After=network.target

[Service]
Type=simple
# Run as root to access nvidia-smi and system hardware
ExecStart=/usr/local/bin/annad
WorkingDirectory=/var/lib/anna
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
    $SUDO systemctl daemon-reload
    log_ok "Installed systemd service"
}

write_config() {
    local config_file="${CONFIG_DIR}/config.toml"

    # Use models selected by install_ollama (set as global vars)
    # 2025 defaults: Qwen3 for better JSON/agent support
    local junior_model="${JUNIOR_MODEL:-qwen3:1.7b}"
    local senior_model="${SENIOR_MODEL:-qwen3:8b}"

    # Check if existing config needs migration (no junior/senior models)
    if [[ -f "$config_file" ]] && [[ "$RESET_MODE" == "false" ]]; then
        if grep -q "junior_model" "$config_file" 2>/dev/null; then
            log_ok "Config exists with role-specific models (preserving)"
            return
        else
            log_warn "Config exists but needs migration to role-specific models"
            log_info "Updating config with junior_model and senior_model..."
            # Fall through to write new config
        fi
    fi

    log_info "Writing configuration..."
    log_info "  Junior model (fast): ${junior_model}"
    log_info "  Senior model (smart): ${senior_model}"

    $SUDO tee "$config_file" > /dev/null << EOF
# Anna v${LATEST_VERSION} Configuration
# This file was auto-generated. Feel free to customize.

[core]
mode = "normal"

[llm]
# Role-specific models for optimized resource usage
# Junior (LLM-A): Fast model for probe execution
junior_model = "${junior_model}"
# Senior (LLM-B): Smarter model for reasoning and synthesis
senior_model = "${senior_model}"
# Legacy/fallback (used if junior/senior not set)
preferred_model = "${senior_model}"
fallback_model = "qwen3:1.7b"
selection_mode = "auto"

[update]
enabled = true
interval_seconds = 600
channel = "main"

[log]
level = "debug"
daemon_enabled = true
requests_enabled = true
llm_enabled = true
EOF

    log_ok "Configuration written to ${config_file}"
}

verify_installation() {
    log_info "Verifying installation..."

    local errors=0

    # Check binaries
    if [[ -x "${INSTALL_DIR}/annad" ]]; then
        log_ok "annad binary OK"
    else
        log_error "annad binary missing or not executable"
        ((errors++))
    fi

    if [[ -x "${INSTALL_DIR}/annactl" ]]; then
        # Quick version check using fast -V flag (v0.14.4+)
        local version
        version=$("${INSTALL_DIR}/annactl" -V 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1) || version=""
        if [[ "$version" == "$LATEST_VERSION" ]]; then
            log_ok "annactl v${version} OK"
        elif [[ -n "$version" ]]; then
            log_ok "annactl v${version} installed"
        else
            log_ok "annactl binary OK"
        fi
    else
        log_error "annactl binary missing or not executable"
        ((errors++)) || true
    fi

    # Check config
    if [[ -f "${CONFIG_DIR}/config.toml" ]]; then
        log_ok "Configuration OK"
    else
        log_warn "Configuration file missing"
    fi

    # Check for stray binaries that could shadow the installed ones
    local stray_paths=("$HOME/.local/bin" "$HOME/bin" "$HOME/.cargo/bin")
    for dir in "${stray_paths[@]}"; do
        if [[ -x "${dir}/annactl" ]] || [[ -x "${dir}/annad" ]]; then
            log_warn "Found anna binaries in ${dir} - these may shadow /usr/local/bin"
            log_warn "Run: rm -f ${dir}/annactl ${dir}/annad"
        fi
    done

    # Verify PATH order
    local which_annactl
    which_annactl=$(command -v annactl 2>/dev/null || true)
    if [[ -n "$which_annactl" && "$which_annactl" != "${INSTALL_DIR}/annactl" ]]; then
        log_warn "PATH issue: 'annactl' resolves to ${which_annactl}"
        log_warn "Expected: ${INSTALL_DIR}/annactl"
    fi

    return $errors
}

# ============================================================
# MAIN
# ============================================================

main() {
    trap cleanup EXIT

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -i|--interactive)
                INTERACTIVE=true
                shift
                ;;
            --reset)
                RESET_MODE=true
                shift
                ;;
            -h|--help)
                printf "Usage: install.sh [-i|--interactive] [--reset]\n"
                printf "  -i, --interactive  Prompt before actions\n"
                printf "  --reset            Full reinstall (wipes config)\n"
                exit 0
                ;;
            *)
                shift
                ;;
        esac
    done

    # Detect architecture
    detect_arch

    # Print banner
    print_banner

    # Detect versions
    detect_installed_version
    fetch_latest_version

    # Show version summary
    print_version_summary

    # Determine and show planned action
    print_planned_action

    # Confirm action
    if ! confirm_action "$PLANNED_ACTION"; then
        log_to_file "Installer: action=${PLANNED_ACTION} result=declined installed=${INSTALLED_VERSION:-none} target=${LATEST_VERSION}"
        exit 0
    fi

    # Request sudo
    check_sudo
    request_sudo

    # Log start
    log_to_file "Installer: action=${PLANNED_ACTION} starting installed=${INSTALLED_VERSION:-none} target=${LATEST_VERSION}"

    print_header "INSTALLING"

    # Download
    if ! download_binaries; then
        log_to_file "Installer: action=${PLANNED_ACTION} result=download_failed"
        exit 1
    fi

    # Install Ollama and select model (before config so model is known)
    install_ollama

    # Install Anna
    create_user_and_dirs
    install_binaries
    install_systemd_service
    write_config

    # Verify
    print_header "VERIFICATION"
    if verify_installation; then
        log_to_file "Installer: action=${PLANNED_ACTION} result=success version=${LATEST_VERSION}"
    else
        log_to_file "Installer: action=${PLANNED_ACTION} result=partial_success version=${LATEST_VERSION}"
    fi

    # Start daemon
    print_header "STARTING DAEMON"
    log_info "Starting annad service..."
    if $SUDO systemctl restart annad 2>/dev/null; then
        sleep 1
        if $SUDO systemctl is-active --quiet annad; then
            log_ok "annad is running"
        else
            log_warn "annad failed to start, check: journalctl -u annad"
        fi
    else
        log_warn "Could not start annad service"
    fi

    # Enable at boot
    if ! $SUDO systemctl is-enabled --quiet annad 2>/dev/null; then
        $SUDO systemctl enable annad 2>/dev/null && log_ok "Enabled annad at boot" || true
    fi

    # Final message
    print_header "COMPLETE"
    printf "  Anna v%s installed and running.\n" "$LATEST_VERSION"
    printf "\n"
    printf "  ${YELLOW}*${NC}  First run: Anna will benchmark available models\n"
    printf "     and select the best Junior/Senior combination.\n"
    printf "     This happens automatically - just ask your first question!\n"
    printf "\n"
    printf "  Check status:    ${CYAN}annactl status${NC}\n"
    printf "  Ask a question:  ${CYAN}annactl \"How many CPU cores?\"${NC}\n"
    printf "\n"
    print_line
}

main "$@"
