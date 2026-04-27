#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: deploy only supports macOS launchd." >&2
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

LABEL="${SOLINBLOG_LAUNCHD_LABEL:-com.factrue.solinblog}"
PLIST_DIR="${SOLINBLOG_LAUNCHD_DIR:-${HOME}/Library/LaunchAgents}"
PLIST_PATH="${PLIST_DIR}/${LABEL}.plist"
LOG_DIR="${SOLINBLOG_LOG_DIR:-${HOME}/Library/Logs/SolinBlog}"
BINARY_PATH="${ROOT_DIR}/target/release/server"
LAUNCHD_DOMAIN="gui/$(id -u)"

xml_escape() {
  local value="$1"
  value="${value//&/&amp;}"
  value="${value//</&lt;}"
  value="${value//>/&gt;}"
  value="${value//\"/&quot;}"
  value="${value//\'/&apos;}"
  printf '%s' "${value}"
}

mkdir -p "${PLIST_DIR}" "${LOG_DIR}"

cd "${ROOT_DIR}"
cargo build --release --bin server

cat > "${PLIST_PATH}" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$(xml_escape "${LABEL}")</string>

  <key>ProgramArguments</key>
  <array>
    <string>$(xml_escape "${BINARY_PATH}")</string>
  </array>

  <key>WorkingDirectory</key>
  <string>$(xml_escape "${ROOT_DIR}")</string>

  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>
    <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
    <key>RUST_BACKTRACE</key>
    <string>1</string>
  </dict>

  <key>RunAtLoad</key>
  <true/>

  <key>KeepAlive</key>
  <true/>

  <key>StandardOutPath</key>
  <string>$(xml_escape "${LOG_DIR}/server.out.log")</string>

  <key>StandardErrorPath</key>
  <string>$(xml_escape "${LOG_DIR}/server.err.log")</string>
</dict>
</plist>
PLIST

plutil -lint "${PLIST_PATH}" >/dev/null

if [[ "${SOLINBLOG_DEPLOY_DRY_RUN:-}" == "1" ]]; then
  echo "dry-run deploy ${LABEL}"
  echo "plist: ${PLIST_PATH}"
  echo "root: ${ROOT_DIR}"
  echo "logs: ${LOG_DIR}/server.out.log ${LOG_DIR}/server.err.log"
  exit 0
fi

if launchctl print "${LAUNCHD_DOMAIN}/${LABEL}" >/dev/null 2>&1; then
  launchctl bootout "${LAUNCHD_DOMAIN}/${LABEL}" >/dev/null 2>&1 || true
fi

if ! launchctl bootstrap "${LAUNCHD_DOMAIN}" "${PLIST_PATH}" 2>/dev/null; then
  launchctl load -w "${PLIST_PATH}"
fi

launchctl enable "${LAUNCHD_DOMAIN}/${LABEL}" >/dev/null 2>&1 || true
launchctl kickstart -k "${LAUNCHD_DOMAIN}/${LABEL}" >/dev/null 2>&1 || true

echo "deployed ${LABEL}"
echo "plist: ${PLIST_PATH}"
echo "root: ${ROOT_DIR}"
echo "logs: ${LOG_DIR}/server.out.log ${LOG_DIR}/server.err.log"
