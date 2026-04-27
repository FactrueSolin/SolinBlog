#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: undeploy only supports macOS launchd." >&2
  exit 1
fi

LABEL="${SOLINBLOG_LAUNCHD_LABEL:-com.factrue.solinblog}"
PLIST_DIR="${SOLINBLOG_LAUNCHD_DIR:-${HOME}/Library/LaunchAgents}"
PLIST_PATH="${PLIST_DIR}/${LABEL}.plist"
LAUNCHD_DOMAIN="gui/$(id -u)"

if launchctl print "${LAUNCHD_DOMAIN}/${LABEL}" >/dev/null 2>&1; then
  launchctl bootout "${LAUNCHD_DOMAIN}/${LABEL}" >/dev/null 2>&1 || true
fi

if [[ -f "${PLIST_PATH}" ]]; then
  rm -f "${PLIST_PATH}"
fi

echo "undeployed ${LABEL}"
