#!/usr/bin/env bash
# Anna Assistant - Bash Style Toolkit
# Provides consistent terminal formatting for installer and uninstaller scripts
# Respects NO_COLOR, CLICOLOR, and terminal capabilities

# Terminal capability flags (set by detect_caps)
ST_COLOR=0
ST_EMOJI=0
ST_UTF8=0
ST_WIDTH=80

# ANSI color codes (pastel palette for dark terminals)
readonly C_CYAN='\033[38;5;87m'     # primary
readonly C_GREEN='\033[38;5;120m'   # ok
readonly C_YELLOW='\033[38;5;228m'  # warn
readonly C_RED='\033[38;5;210m'     # err
readonly C_MAGENTA='\033[38;5;213m' # accent
readonly C_GRAY_250='\033[38;5;250m' # fg default
readonly C_GRAY_240='\033[38;5;240m' # dim
readonly C_BOLD='\033[1m'
readonly C_RESET='\033[0m'

# Detect terminal capabilities
# Sets: ST_COLOR, ST_EMOJI, ST_UTF8, ST_WIDTH
detect_caps() {
    # Check if stdout is a TTY
    if [ ! -t 1 ]; then
        ST_COLOR=0
        ST_EMOJI=0
        ST_UTF8=0
        ST_WIDTH=80
        return
    fi

    # Check for NO_COLOR or CLICOLOR
    if [ -n "$NO_COLOR" ] || [ "$CLICOLOR" = "0" ]; then
        ST_COLOR=0
    else
        # Check TERM variable
        local term="${TERM:-dumb}"
        if [[ "$term" != "dumb" && "$term" != "unknown" ]]; then
            ST_COLOR=1
        fi
    fi

    # Check for UTF-8 support
    local lang="${LANG:-}${LC_ALL:-}"
    if [[ "$lang" =~ UTF-8|utf-8|utf8 ]]; then
        ST_UTF8=1
    fi

    # Emoji support: UTF-8 + xterm/tmux/screen
    if [ "$ST_UTF8" = "1" ]; then
        if [[ "$TERM" =~ xterm|tmux|screen|alacritty|kitty ]]; then
            ST_EMOJI=1
        fi
    fi

    # Get terminal width
    if command -v tput >/dev/null 2>&1; then
        ST_WIDTH=$(tput cols 2>/dev/null || echo 80)
    elif [ -n "$COLUMNS" ]; then
        ST_WIDTH=$COLUMNS
    fi

    # Clamp width to [60, 120]
    if [ "$ST_WIDTH" -lt 60 ]; then
        ST_WIDTH=60
    elif [ "$ST_WIDTH" -gt 120 ]; then
        ST_WIDTH=120
    fi
}

# Header box (top-level title)
# Usage: st_header "Anna Installer"
st_header() {
    local title="$1"
    local tl tr h

    if [ "$ST_UTF8" = "1" ]; then
        tl="╭─"
        tr="╮"
        h="─"
    else
        tl="+-"
        tr="+"
        h="-"
    fi

    local title_len=${#title}
    local padding_total=$((ST_WIDTH - title_len - 6))
    if [ "$padding_total" -lt 0 ]; then
        padding_total=0
    fi

    local padding
    padding=$(printf "%${padding_total}s" "" | tr ' ' "$h")

    local line="${tl} ${title} ${padding}${tr}"

    if [ "$ST_COLOR" = "1" ]; then
        echo -e "${C_CYAN}${line}${C_RESET}"
    else
        echo "$line"
    fi
}

# Section divider
# Usage: st_section "Step 1"
st_section() {
    local title="$1"

    if [ -z "$title" ]; then
        # Just a separator line
        local h
        if [ "$ST_UTF8" = "1" ]; then
            h="─"
        else
            h="-"
        fi

        local line
        line=$(printf "%40s" "" | tr ' ' "$h")

        if [ "$ST_COLOR" = "1" ]; then
            echo -e "${C_GRAY_240}${line}${C_RESET}"
        else
            echo "$line"
        fi
    else
        # Section title
        if [ "$ST_COLOR" = "1" ]; then
            echo -e "${C_CYAN}${title}${C_RESET}"
        else
            echo "$title"
        fi
    fi
}

# Status line with symbol
# Usage: st_status ok "Installation complete"
#        st_status warn "Missing optional dependency"
#        st_status err "Failed to start daemon"
st_status() {
    local level="$1"
    local msg="$2"
    local symbol color

    case "$level" in
        ok)
            if [ "$ST_EMOJI" = "1" ]; then
                symbol="✓"
            else
                symbol="OK"
            fi
            color="$C_GREEN"
            ;;
        warn)
            if [ "$ST_EMOJI" = "1" ]; then
                symbol="⚠"
            else
                symbol="!"
            fi
            color="$C_YELLOW"
            ;;
        err)
            if [ "$ST_EMOJI" = "1" ]; then
                symbol="✗"
            else
                symbol="X"
            fi
            color="$C_RED"
            ;;
        info)
            if [ "$ST_EMOJI" = "1" ]; then
                symbol="▶"
            else
                symbol=">"
            fi
            color="$C_CYAN"
            ;;
        *)
            symbol="•"
            color="$C_GRAY_250"
            ;;
    esac

    if [ "$ST_COLOR" = "1" ]; then
        echo -e "${color}${symbol}${C_RESET} ${msg}"
    else
        echo "${symbol} ${msg}"
    fi
}

# Key-value line with alignment
# Usage: st_kv "CPU" "AMD Ryzen 9"
st_kv() {
    local key="$1"
    local val="$2"
    local key_width=12

    if [ "$ST_COLOR" = "1" ]; then
        printf "${C_GRAY_250}%-${key_width}s${C_RESET}%s\n" "$key:" "$val"
    else
        printf "%-${key_width}s%s\n" "$key:" "$val"
    fi
}

# Hint line (dimmed)
# Usage: st_hint "Try: sudo systemctl status annad"
st_hint() {
    local msg="$1"

    if [ "$ST_COLOR" = "1" ]; then
        echo -e "${C_GRAY_240}${msg}${C_RESET}"
    else
        echo "$msg"
    fi
}

# Code line
# Usage: st_code "annactl status"
st_code() {
    local cmd="$1"

    if [ "$ST_COLOR" = "1" ]; then
        echo -e "${C_GRAY_240}\$${C_RESET} ${cmd}"
    else
        echo "\$ $cmd"
    fi
}

# Progress indicator
# Usage: st_progress 2 5 "Installing binaries"
st_progress() {
    local current="$1"
    local total="$2"
    local label="$3"

    local progress="[${current}/${total}]"

    if [ "$ST_COLOR" = "1" ]; then
        echo -e "${C_GRAY_240}${progress}${C_RESET} ${label}"
    else
        echo "${progress} ${label}"
    fi
}

# Bullet point
# Usage: st_bullet "Item one"
st_bullet() {
    local msg="$1"
    local bullet

    if [ "$ST_EMOJI" = "1" ]; then
        bullet="•"
    else
        bullet="-"
    fi

    echo "  ${bullet} ${msg}"
}

# Initialize capabilities on source
detect_caps
