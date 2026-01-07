# Minimal, safe color helpers (no dependencies). Handles "no color" TTYs.
if [[ -t 1 ]] && command -v tput >/dev/null 2>&1; then
  # shellcheck disable=SC2034
  bold=$(tput bold); dim=$(tput dim); reset=$(tput sgr0)
  # shellcheck disable=SC2034
  black=$(tput setaf 0); red=$(tput setaf 1); green=$(tput setaf 2)
  # shellcheck disable=SC2034
  yellow=$(tput setaf 3); blue=$(tput setaf 4); magenta=$(tput setaf 5); cyan=$(tput setaf 6)
else
  bold=""; dim=""; reset=""; black=""; red=""; green=""; yellow=""; blue=""; magenta=""; cyan=""
fi
