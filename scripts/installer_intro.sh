#!/usr/bin/env bash
set -euo pipefail

# Minimal color helpers (auto-disable if not a TTY)
if [[ -t 1 ]]; then
  b=$'\033[1m'; dim=$'\033[2m'; blue=$'\033[34m'; green=$'\033[32m'; reset=$'\033[0m'
else
  b=""; dim=""; blue=""; green=""; reset=""
fi

logo() {
cat <<'ASCII'
aaaaaaaaaaaaa  nnnn  nnnnnnnnnn  nnnn  nnnnnnnnnn    aaaaaaaaaaaaa
a::::::::::::a n:::n n:::::::::n n:::n n:::::::::n  a::::::::::::a
a::::aaaa::::::a:::nn::::::::::n:::nn::::::::::na  a:::::aaaa::::::a
a::::a    a:::::a:::n n:::nnn:::n:::n n:::nnn:::na  a::::a    a:::::a
a::::a    a:::::a:::n  n:::n  n:::n:::n  n:::n na  a::::a    a:::::a
a:::::aaaa::::::a:::n  n:::n  n:::n:::n  n:::n na  a:::::aaaa::::::a
 a::::::::::aa:::a:::n  n:::n  n:::n:::n  n:::n n   a::::::::::aa:::a
  aaaaaaaaaa  aaaa aaa   aaa   aaaa aaa   aaaa a     aaaaaaaaaa  aaaa
ASCII
}

hr(){ printf "%s\n" "${dim}-----------------------------------------------${reset}"; }

intro() {
  clear || true
  logo
  hr
  printf "${b}Hello! I’m ${blue}Anna${reset}${b}, your local, proactive system assistant.${reset}\n"
  printf "I run as a small daemon (${b}annad${reset}) with a CLI helper (${b}annactl${reset}).\n"
  printf "I’ll learn your system and suggest safe, reversible improvements.\n"
  hr
  printf "• Nothing leaves your machine. I respect your privacy.\n"
  printf "• You can install with defaults now and tweak later with ${b}annactl${reset}.\n"
  hr
  printf "${green}Press ENTER to continue or Ctrl+C to abort.${reset}\n"
  read -r _
}

intro
