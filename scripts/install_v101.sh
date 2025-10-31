#!/usr/bin/env bash
# Anna v0.10.1 Installer - 6-Phase Flow
# Pure telemetry observer with robust capability system
#
# Exit codes:
#   0  - Success
#   10 - Preflight failed
#   11 - Autofix failed
#   12 - Postflight degraded
#   20 - Permissions error
#   21 - Disk space insufficient
#   30 - Build failed

set -euo pipefail

VERSION="0.10.1"
QUIET=${QUIET:-0}
BUILD_FROM_SOURCE=${BUILD_FROM_SOURCE:-0}

# Colors (only if terminal supports it)
if [[ -t 1 ]] && command -v tput &>/dev/null; then
    BOLD=$(tput bold)
    GREEN=$(tput setaf 2)
    YELLOW=$(tput setaf 3)
    RED=$(tput setaf 1)
    BLUE=$(tput setaf 4)
    RESET=$(tput sgr0)
else
    BOLD="" GREEN="" YELLOW="" RED="" BLUE="" RESET=""
fi

log_phase() {
    echo ""
    echo "${BOLD}${BLUE}┌─ Phase $1: $2${RESET}"
    echo "${BOLD}${BLUE}│${RESET}"
}

log_step() {
    echo "${BOLD}${BLUE}│${RESET}  $1"
}

log_success() {
    echo "${GREEN}✓${RESET} $1"
}

log_warn() {
    echo "${YELLOW}⚠${RESET} $1"
}

log_error() {
    echo "${RED}✗${RESET} $1"
}

# =============================================================================
# Phase 1: Detection & Preflight
# =============================================================================
phase1_detection() {
    log_phase 1 "Detection & Preflight"

    # Check if running as root
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run with sudo"
        exit 20
    fi

    log_step "Running preflight checks..."

    # Build annactl first if needed (for doctor pre)
    if [[ ! -f "target/release/annactl" ]]; then
        log_step "Building annactl for preflight checks..."
        cargo build --release --bin annactl &>/dev/null || {
            log_error "Failed to build annactl"
            exit 30
        }
    fi

    # Run preflight checks
    if [[ $QUIET -eq 1 ]]; then
        if ! ./target/release/annactl doctor pre; then
            log_error "Preflight checks failed"
            log_step "Retrying with verbose output..."
            ./target/release/annactl doctor pre --verbose || exit 10
        fi
    else
        ./target/release/annactl doctor pre --verbose || exit 10
    fi

    log_success "Preflight checks passed"

    # Detect existing installation
    if [[ -f "/usr/local/bin/annad" ]]; then
        local current_version
        current_version=$(/usr/local/bin/annad --version 2>/dev/null | awk '{print $2}' || echo "unknown")
        log_step "Detected existing installation: v$current_version"
        log_step "Will upgrade to v$VERSION"
        UPGRADE=1
    else
        log_step "Fresh installation of v$VERSION"
        UPGRADE=0
    fi

    echo "${BOLD}${BLUE}╰────────────────────────────────────────────────────────${RESET}"
}

# =============================================================================
# Phase 2: Preparation
# =============================================================================
phase2_preparation() {
    log_phase 2 "Preparation"

    # Check for --build flag
    if [[ "${1:-}" == "--build" ]]; then
        BUILD_FROM_SOURCE=1
    fi

    # Try to use prebuilt binaries (future feature)
    if [[ $BUILD_FROM_SOURCE -eq 0 ]]; then
        log_step "Checking for prebuilt binaries..."
        log_warn "Prebuilt binaries not available yet"
        BUILD_FROM_SOURCE=1
    fi

    if [[ $BUILD_FROM_SOURCE -eq 1 ]]; then
        log_step "Building from source..."
        if ! cargo build --release; then
            log_error "Build failed"
            exit 30
        fi
        log_success "Build completed"
    fi

    # Create backup if upgrading
    if [[ ${UPGRADE:-0} -eq 1 ]]; then
        local backup_dir="/var/lib/anna/backups/pre-$VERSION-$(date +%s)"
        log_step "Creating backup: $backup_dir"
        mkdir -p "$backup_dir"
        cp -p /usr/local/bin/annad "$backup_dir/" 2>/dev/null || true
        cp -p /usr/local/bin/annactl "$backup_dir/" 2>/dev/null || true
        cp -p /etc/systemd/system/annad.service "$backup_dir/" 2>/dev/null || true
        log_success "Backup created"
    fi

    echo "${BOLD}${BLUE}╰────────────────────────────────────────────────────────${RESET}"
}

# =============================================================================
# Phase 3: Installation
# =============================================================================
phase3_installation() {
    log_phase 3 "Installation"

    log_step "Installing binaries..."
    install -m 0755 target/release/annad /usr/local/bin/annad
    install -m 0755 target/release/annactl /usr/local/bin/annactl
    log_success "Binaries installed"

    log_step "Installing configuration..."
    mkdir -p /etc/anna /usr/lib/anna
    install -m 0644 etc/CAPABILITIES.toml /usr/lib/anna/CAPABILITIES.toml

    # Install modules.yaml if it doesn't exist (preserve user config)
    if [[ ! -f /etc/anna/modules.yaml ]]; then
        install -m 0644 etc/modules.yaml /etc/anna/modules.yaml
        log_success "Configuration installed"
    else
        log_step "Preserving existing modules.yaml"
    fi

    log_step "Running system setup (doctor apply)..."
    if [[ $QUIET -eq 1 ]]; then
        if ! /usr/local/bin/annad --doctor-apply --quiet; then
            log_warn "System setup had issues, retrying verbose..."
            /usr/local/bin/annad --doctor-apply --verbose || exit 11
        fi
    else
        /usr/local/bin/annad --doctor-apply --verbose || exit 11
    fi
    log_success "System setup complete"

    echo "${BOLD}${BLUE}╰────────────────────────────────────────────────────────${RESET}"
}

# =============================================================================
# Phase 4: Service Activation
# =============================================================================
phase4_activation() {
    log_phase 4 "Service Activation"

    log_step "Enabling annad.service..."
    systemctl daemon-reload
    systemctl enable annad.service
    log_success "Service enabled"

    log_step "Starting annad.service..."
    if systemctl is-active --quiet annad.service; then
        systemctl restart annad.service
    else
        systemctl start annad.service
    fi

    # Wait for service to start
    sleep 2

    if systemctl is-active --quiet annad.service; then
        log_success "Service started"
    else
        log_error "Service failed to start"
        log_step "Check logs: journalctl -u annad -n 50"
        exit 11
    fi

    echo "${BOLD}${BLUE}╰────────────────────────────────────────────────────────${RESET}"
}

# =============================================================================
# Phase 5: Verification
# =============================================================================
phase5_verification() {
    log_phase 5 "Verification"

    log_step "Running postflight checks..."

    if [[ $QUIET -eq 1 ]]; then
        if ! /usr/local/bin/annactl doctor post; then
            log_warn "Postflight had warnings, retrying verbose..."
            /usr/local/bin/annactl doctor post --verbose || {
                local exit_code=$?
                if [[ $exit_code -eq 12 ]]; then
                    log_warn "Installation complete with degraded capabilities"
                    return 0
                else
                    exit $exit_code
                fi
            }
        fi
    else
        /usr/local/bin/annactl doctor post --verbose || {
            local exit_code=$?
            if [[ $exit_code -eq 12 ]]; then
                log_warn "Installation complete with degraded capabilities"
                return 0
            else
                exit $exit_code
            fi
        }
    fi

    log_success "Postflight checks passed"

    echo "${BOLD}${BLUE}╰────────────────────────────────────────────────────────${RESET}"
}

# =============================================================================
# Phase 6: Summary
# =============================================================================
phase6_summary() {
    log_phase 6 "Summary"

    echo ""
    echo "  ${BOLD}${GREEN}╭─────────────────────────────────────────────╮${RESET}"
    echo "  ${BOLD}${GREEN}│                                             │${RESET}"
    echo "  ${BOLD}${GREEN}│      Anna v$VERSION is ready to serve!      │${RESET}"
    echo "  ${BOLD}${GREEN}│                                             │${RESET}"
    echo "  ${BOLD}${GREEN}╰─────────────────────────────────────────────╯${RESET}"
    echo ""

    log_step "${BOLD}Next steps:${RESET}"
    echo ""
    echo "    annactl status      - Check daemon health"
    echo "    annactl capabilities - View module status"
    echo "    annactl sensors     - View system telemetry"
    echo "    annactl radar       - View persona analysis"
    echo ""
    echo "  ${BOLD}Logs:${RESET} journalctl -u annad -f"
    echo "  ${BOLD}Docs:${RESET} /usr/lib/anna/CAPABILITIES.toml"
    echo ""
}

# =============================================================================
# Main
# =============================================================================
main() {
    echo ""
    echo "${BOLD}Anna v$VERSION Installer${RESET}"
    echo "Pure Telemetry Observer"
    echo ""

    phase1_detection
    phase2_preparation "${@:-}"
    phase3_installation
    phase4_activation
    phase5_verification
    phase6_summary

    log_success "${BOLD}Installation complete!${RESET}"
    echo ""
}

main "$@"
