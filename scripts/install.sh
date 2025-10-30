#!/usr/bin/env bash
set -euo pipefail

# Anna Assistant Installer - Sprint 3 (v0.9.2a-final)
# Self-healing, idempotent installation with auto-repair
# Runs as normal user, escalates only when needed

VERSION="0.9.2a-final"
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
BIN_DIR="$INSTALL_PREFIX/bin"
SYSTEMD_DIR="/etc/systemd/system"
TMPFILES_DIR="/usr/lib/tmpfiles.d"
CONFIG_DIR="/etc/anna"
POLKIT_DIR="/usr/share/polkit-1/actions"
COMPLETION_DIR="/usr/share/bash-completion/completions"
STATE_DIR="/var/lib/anna"
ANNA_GROUP="anna"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_fixed() {
    echo -e "${CYAN}[FIXED]${NC} $1"
}

log_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

# Check if we have sudo/pkexec available
needs_elevation() {
    return $(test "$EUID" -ne 0)
}

run_elevated() {
    if needs_elevation; then
        if command -v sudo &>/dev/null; then
            sudo "$@"
        elif command -v pkexec &>/dev/null; then
            pkexec "$@"
        else
            log_error "Need elevation but sudo/pkexec not available"
            return 1
        fi
    else
        "$@"
    fi
}

print_banner() {
    echo -e "${BLUE}"
    cat <<'EOF'
    ╔═══════════════════════════════════════╗
    ║                                       ║
    ║      ANNA ASSISTANT v0.9.2a-final     ║
    ║     Self-Healing System Assistant     ║
    ║   Sprint 3: Runtime Self-Healing      ║
    ║                                       ║
    ╚═══════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

check_environment() {
    if [[ $EUID -eq 0 ]] && [[ -z "${SUDO_USER:-}" ]]; then
        log_warn "Running as root directly (not via sudo)"
        log_warn "User-specific setup may not work correctly"
        log_info "Recommended: Run as normal user, will escalate as needed"
    elif [[ $EUID -ne 0 ]]; then
        log_info "Running as user $(whoami), will request elevation when needed"
    fi
}

check_requirements() {
    log_info "Checking system requirements..."

    # Check OS
    if [[ ! -f /etc/arch-release ]] && [[ ! -f /etc/os-release ]]; then
        log_warn "Could not detect Linux distribution. Proceeding anyway..."
    fi

    # Check for required tools
    local missing=()
    for tool in cargo rustc systemctl; do
        if ! command -v "$tool" &>/dev/null; then
            missing+=("$tool")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required tools: ${missing[*]}"
        log_info "Install with: sudo pacman -S rust cargo systemd (Arch)"
        log_info "           or: sudo apt install cargo rustc systemd (Debian/Ubuntu)"
        exit 1
    fi

    log_success "All requirements satisfied"
}

compile_binaries() {
    log_info "Compiling Anna (this may take a few minutes)..."

    # Always compile as the actual user (not root)
    if [[ -n "${SUDO_USER:-}" ]] && [[ $EUID -eq 0 ]]; then
        sudo -u "$SUDO_USER" cargo build --release
    else
        cargo build --release
    fi

    if [ $? -ne 0 ]; then
        log_error "Compilation failed"
        exit 1
    fi

    if [[ ! -f target/release/annad ]] || [[ ! -f target/release/annactl ]]; then
        log_error "Binaries not found after compilation"
        exit 1
    fi

    log_success "Compilation complete"
}

create_anna_group() {
    log_info "Setting up anna group..."

    if getent group "$ANNA_GROUP" > /dev/null 2>&1; then
        log_success "Group '$ANNA_GROUP' already exists"
    else
        if run_elevated groupadd "$ANNA_GROUP"; then
            log_fixed "Created group '$ANNA_GROUP'"
        else
            log_error "Failed to create group '$ANNA_GROUP'"
            exit 1
        fi
    fi
}

add_user_to_group() {
    log_info "Adding user to anna group..."

    # Determine the actual user (not root)
    local target_user="${SUDO_USER:-$USER}"

    if [[ -z "$target_user" ]] || [[ "$target_user" == "root" ]]; then
        log_warn "Could not determine non-root user to add to group"
        log_warn "Manual step: Run 'sudo usermod -aG anna YOUR_USERNAME'"
        return
    fi

    if id -nG "$target_user" 2>/dev/null | grep -qw "$ANNA_GROUP"; then
        log_success "User '$target_user' already in group '$ANNA_GROUP'"
    else
        if run_elevated usermod -aG "$ANNA_GROUP" "$target_user"; then
            log_fixed "Added '$target_user' to group '$ANNA_GROUP'"
            log_warn "NOTE: Group membership requires logout/login or 'newgrp anna' to take effect"
            export GROUP_MEMBERSHIP_CHANGED=1
        else
            log_error "Failed to add user to group"
            exit 1
        fi
    fi
}

install_binaries() {
    log_info "Installing binaries to $BIN_DIR..."

    if run_elevated install -m 755 target/release/annad "$BIN_DIR/annad" && \
       run_elevated install -m 755 target/release/annactl "$BIN_DIR/annactl"; then
        log_success "Binaries installed"
    else
        log_error "Failed to install binaries"
        exit 1
    fi
}

install_systemd_service() {
    log_info "Installing systemd service..."

    # Install service unit
    local service_file=""
    if [[ -f packaging/arch/annad.service ]]; then
        service_file="packaging/arch/annad.service"
    elif [[ -f etc/systemd/annad.service ]]; then
        service_file="etc/systemd/annad.service"
    else
        log_error "Service file not found"
        exit 1
    fi

    if run_elevated cp "$service_file" "$SYSTEMD_DIR/annad.service"; then
        log_success "Service unit installed"
    else
        log_error "Failed to install service unit"
        exit 1
    fi

    # Install tmpfiles configuration
    if [[ -f packaging/arch/annad.tmpfiles.conf ]]; then
        run_elevated mkdir -p "$TMPFILES_DIR"
        if run_elevated cp packaging/arch/annad.tmpfiles.conf "$TMPFILES_DIR/annad.conf"; then
            run_elevated systemd-tmpfiles --create "$TMPFILES_DIR/annad.conf" 2>/dev/null || true
            log_success "Tmpfiles configuration installed"
        fi
    fi

    run_elevated systemctl daemon-reload
    log_success "Systemd configuration reloaded"
}

install_polkit_policy() {
    log_info "Installing polkit policy..."

    # Check if polkit is available
    if ! command -v pkexec &>/dev/null && ! [[ -d /usr/share/polkit-1 ]]; then
        log_skip "Polkit not installed - autonomy features will be unavailable"
        log_info "To enable autonomy: sudo pacman -S polkit (or equivalent)"
        return
    fi

    if [[ ! -d "$POLKIT_DIR" ]]; then
        run_elevated mkdir -p "$POLKIT_DIR"
    fi

    if [[ -f polkit/com.anna.policy ]]; then
        if run_elevated cp polkit/com.anna.policy "$POLKIT_DIR/com.anna.policy"; then
            log_success "Polkit policy installed"
        else
            log_warn "Failed to install polkit policy"
        fi
    else
        log_warn "Polkit policy file not found, skipping"
    fi
}

install_bash_completion() {
    log_info "Installing bash completion..."

    if [[ ! -d "$COMPLETION_DIR" ]]; then
        log_skip "Bash completion directory not found"
        return
    fi

    if [[ -f completion/annactl.bash ]]; then
        if run_elevated cp completion/annactl.bash "$COMPLETION_DIR/annactl"; then
            log_success "Bash completion installed"
        else
            log_warn "Failed to install bash completion"
        fi
    else
        log_skip "Bash completion file not found"
    fi
}

setup_directories() {
    log_info "Setting up directories with correct permissions..."

    # Get anna group ID
    local anna_gid=$(getent group "$ANNA_GROUP" | cut -d: -f3)

    # Config directory: 0750 root:anna
    run_elevated mkdir -p "$CONFIG_DIR"
    run_elevated mkdir -p "$CONFIG_DIR/policies.d"
    run_elevated chown -R root:"$ANNA_GROUP" "$CONFIG_DIR" 2>/dev/null || true
    run_elevated chmod 0750 "$CONFIG_DIR"
    run_elevated chmod 0750 "$CONFIG_DIR/policies.d"
    log_success "Config directory: $CONFIG_DIR (0750 root:anna)"

    # State directory: 0750 root:anna
    run_elevated mkdir -p "$STATE_DIR/state"
    run_elevated mkdir -p "$STATE_DIR/events"
    run_elevated mkdir -p "$STATE_DIR/users"
    run_elevated chown -R root:"$ANNA_GROUP" "$STATE_DIR" 2>/dev/null || true
    run_elevated chmod -R 0750 "$STATE_DIR"
    log_success "State directory: $STATE_DIR (0750 root:anna)"

    # Create per-user audit logs
    local target_user="${SUDO_USER:-$USER}"
    if [[ -n "$target_user" ]] && [[ "$target_user" != "root" ]]; then
        local user_id=$(id -u "$target_user" 2>/dev/null || echo "")
        if [[ -n "$user_id" ]]; then
            run_elevated mkdir -p "$STATE_DIR/users/$user_id"
            run_elevated touch "$STATE_DIR/users/$user_id/audit.log"
            run_elevated chown root:"$ANNA_GROUP" "$STATE_DIR/users/$user_id/audit.log"
            run_elevated chmod 0640 "$STATE_DIR/users/$user_id/audit.log"
            log_success "User audit log created for UID $user_id"
        fi
    fi

    # Runtime directory: 0770 root:anna
    # (handled by systemd RuntimeDirectory, but create for immediate use)
    run_elevated mkdir -p /run/anna
    run_elevated chown root:"$ANNA_GROUP" /run/anna 2>/dev/null || true
    run_elevated chmod 0770 /run/anna
    log_success "Runtime directory: /run/anna (0770 root:anna)"

    log_success "All directories configured"
}

setup_config() {
    log_info "Setting up configuration..."

    if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
        if [[ -f config/default.toml ]]; then
            run_elevated cp config/default.toml "$CONFIG_DIR/config.toml"
            run_elevated chown root:"$ANNA_GROUP" "$CONFIG_DIR/config.toml"
            run_elevated chmod 0640 "$CONFIG_DIR/config.toml"
            log_success "Default configuration created"
        else
            log_warn "Default config not found, daemon will create it"
        fi
    else
        log_success "Configuration already exists, preserving"
    fi

    # Install example policies (if not already present)
    if [[ -d docs/policies.d ]] && ls docs/policies.d/*.yml >/dev/null 2>&1; then
        local policies_installed=0
        local policies_skipped=0

        for policy_file in docs/policies.d/*.yml; do
            local policy_name=$(basename "$policy_file")
            if [[ -f "$CONFIG_DIR/policies.d/$policy_name" ]]; then
                policies_skipped=$((policies_skipped + 1))
            else
                if run_elevated cp "$policy_file" "$CONFIG_DIR/policies.d/$policy_name"; then
                    run_elevated chown root:"$ANNA_GROUP" "$CONFIG_DIR/policies.d/$policy_name"
                    run_elevated chmod 0640 "$CONFIG_DIR/policies.d/$policy_name"
                    policies_installed=$((policies_installed + 1))
                fi
            fi
        done

        if [[ $policies_installed -gt 0 ]]; then
            log_success "Installed $policies_installed example policies"
        fi
        if [[ $policies_skipped -gt 0 ]]; then
            log_skip "$policies_skipped policies already present"
        fi
    fi
}

create_user_paths() {
    log_info "Creating user paths..."

    # User paths (if we know the user)
    if [[ -n "${SUDO_USER:-}" ]]; then
        local user_home=$(eval echo "~$SUDO_USER")
        sudo -u "$SUDO_USER" mkdir -p "$user_home/.config/anna"
        sudo -u "$SUDO_USER" mkdir -p "$user_home/.local/share/anna/events"
        log_success "User paths created for $SUDO_USER"
    else
        log_info "User paths will be created on first use"
    fi
}

enable_service() {
    log_info "Enabling and starting annad service..."

    # Enable the service
    run_elevated systemctl enable annad.service 2>/dev/null || log_warn "Service enable failed or already enabled"

    # Start or restart the service
    if run_elevated systemctl is-active --quiet annad.service; then
        log_info "Service already running, restarting..."
        run_elevated systemctl restart annad.service
    else
        run_elevated systemctl start annad.service
    fi

    # Wait for socket to appear
    log_info "Waiting for socket creation..."
    for i in {1..10}; do
        if [[ -S /run/anna/annad.sock ]]; then
            log_success "Service started successfully"
            return 0
        fi
        sleep 0.5
    done

    log_error "Service started but socket not available"
    log_info "Check status with: sudo systemctl status annad"
    log_info "View logs with: sudo journalctl -u annad --since -5m"
    return 1
}

run_doctor_bootstrap() {
    log_info "Running doctor repair bootstrap..."

    local target_user="${SUDO_USER:-$USER}"

    # Try to run doctor repair twice (first fixes, second verifies)
    log_info "First repair pass..."
    if [[ -n "$target_user" ]] && [[ "$target_user" != "root" ]]; then
        if id -nG "$target_user" 2>/dev/null | grep -qw "$ANNA_GROUP"; then
            # User already in group
            sudo -u "$target_user" annactl doctor --autofix 2>/dev/null || log_warn "Doctor repair had issues"
        else
            # Use sg to add group context temporarily
            sudo -u "$target_user" sg anna -c "annactl doctor --autofix" 2>/dev/null || log_warn "Doctor repair had issues"
        fi
    else
        annactl doctor --autofix 2>/dev/null || log_warn "Doctor repair had issues"
    fi

    sleep 1

    log_info "Second repair pass (verification)..."
    if [[ -n "$target_user" ]] && [[ "$target_user" != "root" ]]; then
        if id -nG "$target_user" 2>/dev/null | grep -qw "$ANNA_GROUP"; then
            sudo -u "$target_user" annactl doctor --autofix 2>/dev/null || true
        else
            sudo -u "$target_user" sg anna -c "annactl doctor --autofix" 2>/dev/null || true
        fi
    else
        annactl doctor --autofix 2>/dev/null || true
    fi

    log_success "Doctor bootstrap complete"
}

run_sanity_checks() {
    log_info "Running sanity checks..."

    local target_user="${SUDO_USER:-$USER}"

    # Check 1: Policy reload
    log_info "Reloading policies..."
    if [[ -n "$target_user" ]] && [[ "$target_user" != "root" ]]; then
        if id -nG "$target_user" 2>/dev/null | grep -qw "$ANNA_GROUP"; then
            local policy_output=$(sudo -u "$target_user" annactl policy reload 2>/dev/null | grep -o '[0-9]* policies loaded' || echo "0 policies loaded")
        else
            local policy_output=$(sudo -u "$target_user" sg anna -c "annactl policy reload" 2>/dev/null | grep -o '[0-9]* policies loaded' || echo "0 policies loaded")
        fi
    else
        local policy_output=$(annactl policy reload 2>/dev/null | grep -o '[0-9]* policies loaded' || echo "0 policies loaded")
    fi
    log_success "Policy reload: $policy_output"

    # Check 2: Show recent events
    log_info "Checking bootstrap events..."
    if [[ -n "$target_user" ]] && [[ "$target_user" != "root" ]]; then
        if id -nG "$target_user" 2>/dev/null | grep -qw "$ANNA_GROUP"; then
            local event_count=$(sudo -u "$target_user" annactl events list 2>/dev/null | grep -cE "SystemStartup|Custom.*DoctorBootstrap|ConfigChange" || echo "0")
        else
            local event_count=$(sudo -u "$target_user" sg anna -c "annactl events list" 2>/dev/null | grep -cE "SystemStartup|Custom.*DoctorBootstrap|ConfigChange" || echo "0")
        fi
    else
        local event_count=$(annactl events list 2>/dev/null | grep -cE "SystemStartup|Custom.*DoctorBootstrap|ConfigChange" || echo "0")
    fi

    if [[ "$event_count" -ge 3 ]]; then
        log_success "Bootstrap events: $event_count found"
    else
        log_warn "Bootstrap events: only $event_count found (expected 3)"
    fi

    log_success "Sanity checks complete"
}

post_install_validation() {
    log_info "Running post-install validation..."

    # Wait a moment for socket to be ready
    sleep 1

    local validation_failed=false
    local target_user="${SUDO_USER:-root}"

    # Check if socket exists
    if [[ ! -S /run/anna/annad.sock ]]; then
        log_error "Socket not found at /run/anna/annad.sock"
        validation_failed=true
    else
        log_success "Socket exists"
    fi

    # Check socket permissions
    if [[ -S /run/anna/annad.sock ]]; then
        local sock_perms=$(stat -c "%a" /run/anna/annad.sock 2>/dev/null)
        if [[ "$sock_perms" != "660" ]] && [[ "$sock_perms" != "666" ]]; then
            log_warn "Socket permissions are $sock_perms (expected 660 or 666)"
        else
            log_success "Socket permissions correct ($sock_perms)"
        fi
    fi

    # Test commands
    log_info "Testing annactl commands..."

    # Determine how to run annactl (with correct group context)
    local annactl_cmd
    if [[ "$target_user" != "root" ]] && ! id -nG "$target_user" | grep -qw "$ANNA_GROUP"; then
        log_warn "User '$target_user' not yet in anna group (requires logout/login)"
        log_info "Using 'sg anna' for validation..."
        annactl_cmd="sg anna -c"
    else
        annactl_cmd=""
    fi

    # Test ping
    if [[ -n "$annactl_cmd" ]]; then
        if sudo -u "$target_user" $annactl_cmd "annactl ping" &>/dev/null; then
            log_success "annactl ping: OK"
        else
            log_error "annactl ping: FAILED"
            validation_failed=true
        fi
    else
        if annactl ping &>/dev/null; then
            log_success "annactl ping: OK"
        else
            log_error "annactl ping: FAILED"
            validation_failed=true
        fi
    fi

    # Test status
    if [[ -n "$annactl_cmd" ]]; then
        if sudo -u "$target_user" $annactl_cmd "annactl status" &>/dev/null; then
            log_success "annactl status: OK"
        else
            log_error "annactl status: FAILED"
            validation_failed=true
        fi
    else
        if annactl status &>/dev/null; then
            log_success "annactl status: OK"
        else
            log_error "annactl status: FAILED"
            validation_failed=true
        fi
    fi

    if $validation_failed; then
        log_error "Post-install validation failed"
        log_info "Troubleshooting:"
        log_info "  1. Check service: sudo systemctl status annad"
        log_info "  2. View logs: sudo journalctl -u annad --since -5m"
        log_info "  3. Check socket: ls -lh /run/anna/annad.sock"
        return 1
    else
        log_success "All validation checks passed"
        return 0
    fi
}

print_completion() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                       ║${NC}"
    echo -e "${GREEN}║   INSTALLATION COMPLETE!              ║${NC}"
    echo -e "${GREEN}║                                       ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════╝${NC}"
    echo ""
    echo "Quick start:"
    echo "  annactl status              - Check daemon status"
    echo "  annactl doctor              - Run diagnostics"
    echo "  annactl config list         - List configuration"
    echo "  annactl policy list         - List policies"
    echo "  annactl events show         - Show recent events"
    echo "  annactl learning stats      - Learning statistics"
    echo ""
    echo "Service management:"
    echo "  sudo systemctl status annad"
    echo "  sudo systemctl restart annad"
    echo "  sudo journalctl -u annad -f"
    echo ""

    if [[ -n "${SUDO_USER:-}" ]]; then
        echo -e "${YELLOW}IMPORTANT:${NC} Group membership requires logout/login to take effect"
        echo "Temporary workaround: Run 'newgrp anna' in your shell"
        echo ""
    fi
}

main() {
    print_banner

    # Check if running from project root
    if [[ ! -f Cargo.toml ]]; then
        log_error "Must run from anna-assistant project root"
        exit 1
    fi

    check_environment
    check_requirements
    compile_binaries
    create_anna_group
    add_user_to_group
    install_binaries
    install_systemd_service
    install_polkit_policy
    install_bash_completion
    setup_directories
    setup_config
    create_user_paths

    if ! enable_service; then
        log_error "Service startup failed"
        exit 1
    fi

    # Run doctor repair bootstrap (auto-fix any remaining issues)
    run_doctor_bootstrap

    # Run sanity checks (policy reload, events check)
    run_sanity_checks

    if ! post_install_validation; then
        log_warn "Validation had issues, but installation completed"
        log_info "Review the errors above and check service logs"
    fi

    print_completion
}

main "$@"
