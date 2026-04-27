#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: status only supports macOS launchd." >&2
  exit 1
fi

LABEL="${SOLINBLOG_LAUNCHD_LABEL:-com.factrue.solinblog}"
LAUNCHD_DOMAIN="gui/$(id -u)"

if launchctl print "${LAUNCHD_DOMAIN}/${LABEL}" >/dev/null 2>&1; then
  launchctl print "${LAUNCHD_DOMAIN}/${LABEL}"
else
  echo "${LABEL} is not loaded"
  exit 1
fi
