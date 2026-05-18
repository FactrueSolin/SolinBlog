#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"
if command -v cargo >/dev/null 2>&1; then
    cargo check "$@"
elif command -v cargo.exe >/dev/null 2>&1; then
    cargo.exe check "$@"
else
    printf 'missing required command: cargo\n' >&2
    exit 1
fi
