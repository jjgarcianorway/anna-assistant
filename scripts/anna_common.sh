#!/usr/bin/env bash
# ============================================================================
# Anna Common Library - Bash Edition
# ============================================================================
#
# Unified messaging and utility functions for Anna Assistant shell scripts.
# This provides the same conversational interface as the Rust anna_common lib.
#
# Usage:
#   source scripts/anna_common.sh
#   anna_say narrative "Let me help you with that..."
#   anna_say ok "All done!"
#   anna_box narrative "Welcome to Anna!" "I'm here to help."

# ============================================================================
# Terminal Detection
# ============================================================================

# Detect terminal capabilities
IS_TTY=false
TERM_WIDTH=80
SUPPORTS_COLOR=false
SUPPORTS_UNICODE=false

detect_terminal() {
    # Check if stdout is a TTY
    if [[ -t 1 ]]; then
        IS_TTY=true
        TERM_WIDTH=$(tput cols 2>/dev/null || echo 80)
    fi

    # Check NO_COLOR environment variable
    if [[ -z "${NO_COLOR:-}" ]]; then
        # Check color support
        if [[ -n "${TERM:-}" ]] && command -v tput &>/dev/null; then
            if [[ $(tput colors 2>/dev/null || echo 0) -ge 8 ]]; then
                SUPPORTS_COLOR=true
            fi
        fi
    fi

    # Check Unicode support
    if [[ "${LANG:-}" =~ UTF-8 ]] || [[ "${LC_ALL:-}" =~ UTF-8 ]]; then
        SUPPORTS_UNICODE=true
    fi
}

# Run detection at source time
detect_terminal

# ============================================================================
# Color Palette (Pastel for dark terminals)
# ============================================================================

if [[ "$SUPPORTS_COLOR" == "true" ]]; then
    C_CYAN='\033[38;5;87m'       # Headers, narrative
    C_GREEN='\033[38;5;120m'     # Success
    C_YELLOW='\033[38;5;228m'    # Warnings
    C_RED='\033[38;5;210m'       # Errors
    C_BLUE='\033[38;5;111m'      # Info
    C_GRAY='\033[38;5;245m'      # Secondary
    C_BOLD='\033[1m'             # Bold
    NC='\033[0m'                 # Reset
else
    C_CYAN=''
    C_GREEN=''
    C_YELLOW=''
    C_RED=''
    C_BLUE=''
    C_GRAY=''
    C_BOLD=''
    NC=''
fi

# ============================================================================
# Emoji/Icons
# ============================================================================

if [[ "$SUPPORTS_UNICODE" == "true" ]]; then
    EMOJI_INFO="⚙️ "
    EMOJI_OK="✅"
    EMOJI_WARN="🟡"
    EMOJI_ERROR="❌"
    EMOJI_NARRATIVE="🤖"
else
    EMOJI_INFO="[i]"
    EMOJI_OK="[✓]"
    EMOJI_WARN="[!]"
    EMOJI_ERROR="[X]"
    EMOJI_NARRATIVE="[Anna]"
fi

# ============================================================================
# Timestamp Formatting
# ============================================================================

format_timestamp() {
    # Detect locale
    local locale="${LC_ALL:-${LC_TIME:-${LANG:-en_US.UTF-8}}}"
    local lang="${locale%%_*}"

    # Format based on language
    case "$lang" in
        en)
            date '+%b %d %Y %I:%M %p'
            ;;
        nb|no)
            date '+%d.%m.%Y %H:%M'
            ;;
        de)
            date '+%d.%m.%Y %H:%M'
            ;;
        fr|es)
            date '+%d/%m/%Y %H:%M'
            ;;
        ja)
            date '+%Y年%m月%d日 %H:%M'
            ;;
        zh)
            date '+%Y-%m-%d %H:%M'
            ;;
        *)
            date '+%H:%M'
            ;;
    esac
}

# ============================================================================
# Main Output Function - anna_say
# ============================================================================

# anna_say <type> <message>
#
# Types: info, ok, warn, error, narrative
#
# Examples:
#   anna_say info "Checking dependencies..."
#   anna_say ok "Installation complete!"
#   anna_say narrative "Let me help you with that."
anna_say() {
    local msg_type="$1"
    shift
    local message="$*"

    # Select color and emoji based on type
    local color emoji prefix=""
    case "$msg_type" in
        info)
            color="$C_BLUE"
            emoji="$EMOJI_INFO"
            ;;
        ok)
            color="$C_GREEN"
            emoji="$EMOJI_OK"
            ;;
        warn)
            color="$C_YELLOW"
            emoji="$EMOJI_WARN"
            ;;
        error)
            color="$C_RED"
            emoji="$EMOJI_ERROR"
            ;;
        narrative)
            color="$C_CYAN"
            emoji="$EMOJI_NARRATIVE"
            prefix="${C_BOLD}Anna:${NC}"
            ;;
        *)
            color=""
            emoji=""
            ;;
    esac

    # Build output
    local timestamp
    timestamp="$(format_timestamp)"

    # Print with color and formatting
    if [[ "$msg_type" == "narrative" ]]; then
        echo -e "${C_GRAY}[${timestamp}]${NC} ${color}${emoji}${NC} ${prefix} ${color}${message}${NC}"
    else
        echo -e "${C_GRAY}[${timestamp}]${NC} ${color}${emoji}${NC} ${color}${message}${NC}"
    fi

    # Also log to install log if it exists
    if [[ -w "/var/log/anna/install.log" ]] 2>/dev/null; then
        echo "[${timestamp}] [${msg_type^^}] ${message}" >> /var/log/anna/install.log
    fi
}

# Convenience functions
anna_info() {
    anna_say info "$@"
}

anna_ok() {
    anna_say ok "$@"
}

anna_warn() {
    anna_say warn "$@"
}

anna_error() {
    anna_say error "$@"
}

anna_narrative() {
    anna_say narrative "$@"
}

# ============================================================================
# Decorative Box
# ============================================================================

# anna_box <type> <line1> [line2] [line3] ...
#
# Example:
#   anna_box narrative "Welcome to Anna!" "I'm here to help."
anna_box() {
    local box_type="$1"
    shift
    local lines=("$@")

    # Select color based on type
    local color
    case "$box_type" in
        info) color="$C_BLUE" ;;
        ok) color="$C_GREEN" ;;
        warn) color="$C_YELLOW" ;;
        error) color="$C_RED" ;;
        narrative) color="$C_CYAN" ;;
        *) color="" ;;
    esac

    # Box drawing characters
    local tl tr bl br h v
    if [[ "$SUPPORTS_UNICODE" == "true" ]]; then
        tl='╭'; tr='╮'; bl='╰'; br='╯'; h='─'; v='│'
    else
        tl='+'; tr='+'; bl='+'; br='+'; h='-'; v='|'
    fi

    # Calculate max line length
    local max_len=0
    for line in "${lines[@]}"; do
        local len=${#line}
        if (( len > max_len )); then
            max_len=$len
        fi
    done

    # Box width (add padding)
    local box_width=$((max_len + 4))

    # Top border
    echo -e "${color}${tl}$(printf '%*s' $((box_width - 2)) '' | tr ' ' "$h")${tr}${NC}"

    # Content lines (centered)
    for line in "${lines[@]}"; do
        local len=${#line}
        local padding=$(( (box_width - 2 - len) / 2 ))
        local padding_right=$(( box_width - 2 - len - padding ))
        echo -e "${color}${v}${NC}$(printf '%*s' "$padding" '')${line}$(printf '%*s' "$padding_right" '')${color}${v}${NC}"
    done

    # Bottom border
    echo -e "${color}${bl}$(printf '%*s' $((box_width - 2)) '' | tr ' ' "$h")${br}${NC}"
}

# ============================================================================
# Privilege Escalation Helper
# ============================================================================

# Check if running as root
is_root() {
    [[ $EUID -eq 0 ]]
}

# anna_privilege <reason> <command> [args...]
#
# Example:
#   anna_privilege "install system files" cp binary /usr/local/bin/
anna_privilege() {
    local reason="$1"
    shift
    local command="$1"
    shift
    local args=("$@")

    # If already root, just run the command
    if is_root; then
        "$command" "${args[@]}"
        return $?
    fi

    # Explain why we need privileges
    anna_narrative "I need administrator rights to ${reason}."

    # Check if confirmation is needed (read from config or default to yes)
    local confirm_privilege="${ANNA_CONFIRM_PRIVILEGE:-true}"

    if [[ "$confirm_privilege" == "true" ]]; then
        anna_info "May I proceed with sudo?"
        printf "  [Y/n] "
        read -r response
        response=$(echo "$response" | tr '[:upper:]' '[:lower:]')

        if [[ -n "$response" ]] && [[ "$response" != "y" ]] && [[ "$response" != "yes" ]]; then
            anna_error "User declined privilege escalation"
            return 1
        fi
    fi

    # Execute with sudo
    anna_info "Requesting administrator privileges..."

    if sudo "$command" "${args[@]}"; then
        anna_ok "Done!"
        return 0
    else
        anna_error "The command failed."
        return 1
    fi
}

# ============================================================================
# Duration Formatting
# ============================================================================

format_duration() {
    local seconds=$1

    if (( seconds < 60 )); then
        echo "${seconds}s"
    elif (( seconds < 3600 )); then
        local mins=$((seconds / 60))
        local secs=$((seconds % 60))
        echo "${mins}m ${secs}s"
    else
        local hours=$((seconds / 3600))
        local mins=$(( (seconds % 3600) / 60 ))
        echo "${hours}h ${mins}m"
    fi
}

# ============================================================================
# Export Functions (for sourcing)
# ============================================================================

export -f anna_say anna_info anna_ok anna_warn anna_error anna_narrative
export -f anna_box format_timestamp format_duration
export -f is_root anna_privilege
