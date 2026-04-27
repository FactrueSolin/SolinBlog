#!/usr/bin/env bash
set -euo pipefail

LOG_DIR="${SOLINBLOG_LOG_DIR:-${HOME}/Library/Logs/SolinBlog}"

mkdir -p "${LOG_DIR}"
touch "${LOG_DIR}/server.out.log" "${LOG_DIR}/server.err.log"
tail -n 120 -f "${LOG_DIR}/server.out.log" "${LOG_DIR}/server.err.log"
