#!/usr/bin/env bash
# ╔═══════════════════════════════════════════════════════════════════════╗
# ║                    Anna Assistant Installer                           ║
# ║              The Most Beautiful Installer in History                  ║
# ╚═══════════════════════════════════════════════════════════════════════╝
set -Eeuo pipefail

# ═══════════════════════════════════════════════════════════════════════════
# 🎨 BEAUTIFUL TERMINAL COLORS
# ═══════════════════════════════════════════════════════════════════════════
if [[ -t 1 ]]; then
  # Pastel colors for dark terminals
  RESET='\033[0m'
  BOLD='\033[1m'
  DIM='\033[2m'

  # Main colors
  CYAN='\033[96m'      # Bright cyan for headers
  GREEN='\033[92m'     # Success
  YELLOW='\033[93m'    # Warning
  RED='\033[91m'       # Error
  BLUE='\033[94m'      # Info
  MAGENTA='\033[95m'   # Accent
  GRAY='\033[90m'      # Dimmed text
  WHITE='\033[97m'     # Bright white
else
  RESET='' BOLD='' DIM='' CYAN='' GREEN='' YELLOW='' RED='' BLUE='' MAGENTA='' GRAY='' WHITE=''
fi

# ═══════════════════════════════════════════════════════════════════════════
# 🎯 BEAUTIFUL OUTPUT FUNCTIONS
# ═══════════════════════════════════════════════════════════════════════════

clear_screen() {
  clear
  echo ""
}

print_header() {
  echo -e "${CYAN}${BOLD}"
  echo "╔═══════════════════════════════════════════════════════════════════════╗"
  echo "║                                                                       ║"
  echo "║                        🤖  Anna Assistant                             ║"
  echo "║                   Event-Driven Intelligence                           ║"
  echo "║                                                                       ║"
  echo "╚═══════════════════════════════════════════════════════════════════════╝"
  echo -e "${RESET}"
  echo ""
}

print_section() {
  local title="$1"
  echo ""
  echo -e "${CYAN}${BOLD}━━━ $title ${RESET}"
  echo ""
}

print_step() {
  local emoji="$1"
  local text="$2"
  echo -e "${BLUE}  $emoji  ${WHITE}$text${RESET}"
}

print_substep() {
  local text="$1"
  echo -e "${GRAY}     ↳ $text${RESET}"
}

print_success() {
  local text="$1"
  echo -e "${GREEN}  ✓  $text${RESET}"
}

print_warning() {
  local text="$1"
  echo -e "${YELLOW}  ⚠  $text${RESET}"
}

print_error() {
  local text="$1"
  echo -e "${RED}  ✗  $text${RESET}"
}

print_info() {
  local text="$1"
  echo -e "${CYAN}  ℹ  $text${RESET}"
}

spinner() {
  local pid=$1
  local message=$2
  local spinstr='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'

  echo -ne "${BLUE}  "
  while kill -0 $pid 2>/dev/null; do
    for i in $(seq 0 9); do
      echo -ne "\r${BLUE}  ${spinstr:$i:1}  ${WHITE}$message...${RESET}"
      sleep 0.1
    done
  done
  echo -ne "\r${GREEN}  ✓  ${WHITE}$message${RESET}\n"
}

progress_bar() {
  local current=$1
  local total=$2
  local width=50
  local percentage=$((current * 100 / total))
  local filled=$((width * current / total))
  local empty=$((width - filled))

  printf "\r${BLUE}  ["
  printf "${GREEN}%0.s█" $(seq 1 $filled)
  printf "${GRAY}%0.s░" $(seq 1 $empty)
  printf "${BLUE}] ${WHITE}%3d%%${RESET}" $percentage
}

# ═══════════════════════════════════════════════════════════════════════════
# 📦 CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════
OWNER="jjgarcianorway"
REPO="anna-assistant"
BIN_DIR="/usr/local/bin"
SERVICE="annad"
TMPDIR="$(mktemp -d)"

cleanup() { rm -rf "$TMPDIR"; }
trap cleanup EXIT

# ═══════════════════════════════════════════════════════════════════════════
# 🔧 DEPENDENCY MANAGEMENT
# ═══════════════════════════════════════════════════════════════════════════

ensure_pkg() {
  local pkg="$1"
  if ! command -v "$pkg" >/dev/null 2>&1; then
    print_step "📦" "Installing $pkg"
    if command -v pacman >/dev/null 2>&1; then
      sudo pacman -Sy --noconfirm "$pkg" >/dev/null 2>&1
    elif command -v apt >/dev/null 2>&1; then
      sudo apt update >/dev/null 2>&1 && sudo apt install -y "$pkg" >/dev/null 2>&1
    elif command -v dnf >/dev/null 2>&1; then
      sudo dnf install -y "$pkg" >/dev/null 2>&1
    elif command -v zypper >/dev/null 2>&1; then
      sudo zypper install -y "$pkg" >/dev/null 2>&1
    else
      print_error "Unsupported package manager; install '$pkg' manually"
      exit 1
    fi
    print_success "$pkg installed"
  fi
}

# ═══════════════════════════════════════════════════════════════════════════
# 🌐 RELEASE SELECTION
# ═══════════════════════════════════════════════════════════════════════════

select_release() {
  local api="https://api.github.com/repos/$OWNER/$REPO/releases?per_page=15"
  local max_retries=3
  local retry_delay=10
  local min_version="v1.0.0-rc.15"  # Minimum acceptable version

  print_step "🔍" "Finding latest release with assets (min: $min_version)" >&2

  for attempt in $(seq 1 $max_retries); do
    # Get all releases with asset counts
    local releases_json
    releases_json=$(curl -fsSL "$api" 2>/dev/null)

    if [[ -z "$releases_json" || "$releases_json" == "null" ]]; then
      if [[ $attempt -lt $max_retries ]]; then
        print_warning "Failed to fetch releases (attempt $attempt/$max_retries)" >&2
        print_substep "Retrying in ${retry_delay}s..." >&2
        sleep $retry_delay
        continue
      else
        print_error "Failed to fetch releases from GitHub after $max_retries attempts" >&2
        return 1
      fi
    fi

    # Get all tags with assets, filter by minimum version, then sort
    local latest_tag
    latest_tag=$(echo "$releases_json" | \
      jq -r '.[] | select(.draft==false) | select(.assets[] | .name=="anna-linux-x86_64.tar.gz") | .tag_name' | \
      while read tag; do
        # Compare versions (strip 'v' prefix for comparison)
        if version_gte "${tag#v}" "${min_version#v}"; then
          echo "$tag"
        fi
      done | \
      sort -Vr | head -n1)

    if [[ -z "$latest_tag" || "$latest_tag" == "null" ]]; then
      if [[ $attempt -lt $max_retries ]]; then
        print_warning "No releases >= $min_version with assets found yet (attempt $attempt/$max_retries)" >&2
        print_info "GitHub Actions may still be building..." >&2
        print_substep "Retrying in ${retry_delay}s..." >&2
        sleep $retry_delay
        continue
      else
        print_error "No releases >= $min_version found with uploaded assets after $max_retries attempts" >&2
        print_info "Please wait for GitHub Actions to complete, then try again" >&2
        print_info "Check: https://github.com/$OWNER/$REPO/actions" >&2
        return 1
      fi
    fi

    # Success!
    print_substep "Latest version: ${MAGENTA}$latest_tag${RESET}" >&2
    print_success "Assets available for $latest_tag" >&2
    echo "$latest_tag"
    return 0
  done

  return 1
}

# Version comparison function (semver with -rc.N support)
version_gte() {
  local ver1="$1"
  local ver2="$2"

  # Split version and rc suffix
  local v1_base="${ver1%%-rc.*}"
  local v1_rc="${ver1##*-rc.}"
  [[ "$v1_rc" == "$ver1" ]] && v1_rc=""

  local v2_base="${ver2%%-rc.*}"
  local v2_rc="${ver2##*-rc.}"
  [[ "$v2_rc" == "$ver2" ]] && v2_rc=""

  # Compare base versions
  if [[ "$v1_base" != "$v2_base" ]]; then
    printf '%s\n' "$v1_base" "$v2_base" | sort -Vr | head -n1 | grep -q "^$v1_base$"
    return $?
  fi

  # If base versions equal, compare rc numbers
  if [[ -z "$v1_rc" && -z "$v2_rc" ]]; then
    return 0  # Both stable
  elif [[ -z "$v1_rc" ]]; then
    return 0  # v1 is stable, v2 is rc -> v1 >= v2
  elif [[ -z "$v2_rc" ]]; then
    return 1  # v1 is rc, v2 is stable -> v1 < v2
  else
    # Both are rc, compare rc numbers
    [[ "$v1_rc" -ge "$v2_rc" ]]
    return $?
  fi
}

# ═══════════════════════════════════════════════════════════════════════════
# 📥 DOWNLOAD AND VERIFY
# ═══════════════════════════════════════════════════════════════════════════

download_and_verify_tarball() {
  local tag="$1"
  local api="https://api.github.com/repos/$OWNER/$REPO/releases/tags/$tag"
  local tmp="$TMPDIR/anna"
  mkdir -p "$tmp"

  print_step "📡" "Downloading release $tag"

  local assets
  assets=$(curl -fsSL "$api")

  local tar_url checksum_url
  tar_url=$(echo "$assets" | jq -r '.assets[] | select(.name=="anna-linux-x86_64.tar.gz") | .browser_download_url')
  checksum_url=$(echo "$assets" | jq -r '.assets[] | select(.name=="anna-linux-x86_64.tar.gz.sha256") | .browser_download_url')

  if [[ -z "$tar_url" || "$tar_url" == "null" ]]; then
    print_error "Assets not found for $tag" >&2
    print_error "This release may not have been built by GitHub Actions" >&2
    print_info "Only install releases that have been built and published properly" >&2
    print_info "Check: https://github.com/$OWNER/$REPO/releases/tag/$tag" >&2
    return 1
  fi

  if [[ -z "$checksum_url" || "$checksum_url" == "null" ]]; then
    print_error "Checksum file not found for $tag" >&2
    print_error "Cannot verify integrity without checksum" >&2
    return 1
  fi

  # Download with progress
  print_substep "Downloading tarball..."
  curl -fsSL "$tar_url" -o "$tmp/anna-linux-x86_64.tar.gz" 2>/dev/null &
  spinner $! "Downloading tarball"

  print_substep "Downloading checksum..."
  curl -fsSL "$checksum_url" -o "$tmp/anna-linux-x86_64.tar.gz.sha256" 2>/dev/null &
  spinner $! "Downloading checksum"

  print_step "🔐" "Verifying integrity"
  if (cd "$tmp" && sha256sum -c anna-linux-x86_64.tar.gz.sha256 >/dev/null 2>&1); then
    print_success "Checksum verified"
  else
    print_error "Checksum verification failed"
    return 1
  fi

  print_step "📦" "Extracting binaries"
  tar -xzf "$tmp/anna-linux-x86_64.tar.gz" -C "$tmp" 2>/dev/null

  if [[ ! -f "$tmp/annad" || ! -f "$tmp/annactl" ]]; then
    print_error "Binaries not found in tarball"
    return 1
  fi

  sudo install -m 0755 "$tmp/annad" "$BIN_DIR/annad"
  sudo install -m 0755 "$tmp/annactl" "$BIN_DIR/annactl"

  print_success "Binaries installed to $BIN_DIR"
  echo "$tag" > "$TMPDIR/installed_tag"
  return 0
}

# ═══════════════════════════════════════════════════════════════════════════
# ⚙️  SYSTEM CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════

configure_systemd() {
  print_step "⚙️ " "Configuring system"

  # Create anna user
  if ! id anna &>/dev/null; then
    print_substep "Creating anna user..."
    sudo useradd -r -s /usr/bin/nologin anna 2>/dev/null
    print_success "User 'anna' created"
  else
    print_substep "User 'anna' already exists"
  fi

  # Create directories
  print_substep "Setting up directories..."
  sudo mkdir -p /var/lib/anna /run/anna /var/log/anna /etc/anna
  sudo chown anna:anna /var/lib/anna /run/anna /var/log/anna
  sudo chmod 0770 /var/lib/anna /run/anna /var/log/anna

  # Add user to anna group
  if ! groups | grep -q anna; then
    print_substep "Adding $USER to 'anna' group..."
    sudo usermod -aG anna "$USER"
    print_warning "You'll need to logout/login for group membership to take effect"
  fi

  # Install systemd service
  if [[ ! -f /etc/systemd/system/annad.service ]]; then
    print_substep "Installing systemd service..."
    sudo tee /etc/systemd/system/annad.service > /dev/null <<'EOF'
[Unit]
Description=Anna Assistant Daemon
Documentation=https://github.com/jjgarcianorway/anna-assistant
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/annad
Restart=on-failure
RestartSec=10s

RuntimeDirectory=anna
RuntimeDirectoryMode=0750

User=anna
Group=anna
UMask=0000

NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/anna /var/log/anna

MemoryMax=100M
TasksMax=100

[Install]
WantedBy=multi-user.target
EOF
  fi

  print_substep "Starting daemon..."
  sudo systemctl daemon-reload
  sudo systemctl enable --now annad >/dev/null 2>&1
  sudo systemctl restart annad >/dev/null 2>&1

  print_success "Daemon configured and started"
}

# ═══════════════════════════════════════════════════════════════════════════
# 🏥 HEALTH VERIFICATION
# ═══════════════════════════════════════════════════════════════════════════

wait_for_daemon() {
  print_step "🏥" "Waiting for daemon to be ready"

  for i in {1..20}; do
    if timeout 2 "$BIN_DIR/annactl" version >/dev/null 2>&1; then
      print_success "Daemon is responding"
      return 0
    fi
    sleep 0.5
  done

  print_error "Daemon not responding"
  print_info "Check logs: journalctl -u annad -n 30"
  return 1
}

verify_versions() {
  local expected="$1"
  local actual

  print_step "🔍" "Verifying installation"

  actual=$("$BIN_DIR/annactl" -V 2>/dev/null | grep -oP 'v?[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?' | head -1)

  print_substep "Expected: ${MAGENTA}$expected${RESET}"
  print_substep "Installed: ${MAGENTA}$actual${RESET}"

  if [[ "$actual" == "$expected" || "$actual" == "${expected#v}" ]]; then
    print_success "Version verification passed"
    return 0
  else
    print_warning "Version mismatch (expected $expected, got $actual)"
    print_info "This may indicate a build issue - please report"
    return 0  # Don't fail, just warn
  fi
}

verify_daemon_ready() {
  print_section "🩺 Final Verification"

  print_step "🏥" "Verifying Anna's health"

  # Quick check that daemon is responding
  if "$BIN_DIR/annactl" version >/dev/null 2>&1; then
    print_success "Anna is healthy and responding"
  else
    print_error "Daemon not responding properly"
    print_info "You can run 'annactl doctor check' to diagnose issues"
    return 1
  fi
}

# ═══════════════════════════════════════════════════════════════════════════
# 🎉 MAIN INSTALLATION FLOW
# ═══════════════════════════════════════════════════════════════════════════

main() {
  clear_screen
  print_header

  print_section "🚀 Installation Starting"

  # Check dependencies
  print_step "🔧" "Checking dependencies"
  for dep in curl jq sudo systemctl; do
    ensure_pkg "$dep"
  done

  # Select and download release
  print_section "📦 Fetching Latest Release"

  TAG=$(select_release)
  if [[ -z "$TAG" || "$TAG" == "null" ]]; then
    print_error "No releases found"
    exit 1
  fi

  echo ""
  download_and_verify_tarball "$TAG" || exit 1

  # Configure system
  echo ""
  print_section "⚙️  System Configuration"
  configure_systemd

  # Wait for daemon
  echo ""
  wait_for_daemon || exit 2

  # Verify installation
  echo ""
  INSTALLED_TAG=$(cat "$TMPDIR/installed_tag" 2>/dev/null || echo "$TAG")
  verify_versions "$INSTALLED_TAG"

  # Verify daemon is ready
  echo ""
  verify_daemon_ready || exit 2

  # Success!
  echo ""
  echo ""
  echo -e "${GREEN}${BOLD}"
  echo "╔═══════════════════════════════════════════════════════════════════════╗"
  echo "║                                                                       ║"
  echo "║                   ✨  Installation Complete! ✨                       ║"
  echo "║                                                                       ║"
  echo "║              Anna is now running and ready to assist                  ║"
  echo "║                                                                       ║"
  echo "╚═══════════════════════════════════════════════════════════════════════╝"
  echo -e "${RESET}"
  echo ""
  echo -e "${CYAN}  Next Steps:${RESET}"
  echo -e "    ${WHITE}annactl status${RESET}   ${GRAY}# Check Anna's current state${RESET}"
  echo -e "    ${WHITE}annactl report${RESET}   ${GRAY}# Get system health report${RESET}"
  echo -e "    ${WHITE}annactl --help${RESET}   ${GRAY}# See all available commands${RESET}"
  echo ""
  echo -e "${GRAY}  Documentation: docs/V1.0-QUICKSTART.md${RESET}"
  echo ""
}

main "$@"
