#!/usr/bin/env bash
set -euo pipefail

TEST_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "${TEST_DIR}/.." && pwd)"
PYTHON_BIN="${PYTHON:-}"
TEST_TOKEN="${IMAGE_API_TEST_TOKEN:-image-api-test-token}"
IMAGE_API_TEST_TMP=""
SERVER_PID=""
BASE_URL=""
CARGO_BIN=""

log() {
    printf '[image-api][%s] %s\n' "${TEST_NAME:-test}" "$*"
}

pass() {
    printf '[image-api][%s][PASS] %s\n' "${TEST_NAME:-test}" "$*"
}

fail() {
    printf '[image-api][%s][FAIL] %s\n' "${TEST_NAME:-test}" "$*" >&2
    if [[ -n "${IMAGE_API_TEST_TMP}" && -f "${IMAGE_API_TEST_TMP}/server.log" ]]; then
        printf '[image-api][%s][server-log]\n' "${TEST_NAME:-test}" >&2
        tail -n 80 "${IMAGE_API_TEST_TMP}/server.log" >&2 || true
    fi
    exit 1
}

require_cmd() {
    command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

detect_python() {
    if [[ -n "${PYTHON_BIN}" ]]; then
        command -v "${PYTHON_BIN}" >/dev/null 2>&1 || fail "missing required command: ${PYTHON_BIN}"
    elif command -v python >/dev/null 2>&1; then
        PYTHON_BIN="python"
    elif command -v python3 >/dev/null 2>&1; then
        PYTHON_BIN="python3"
    elif command -v python.exe >/dev/null 2>&1; then
        PYTHON_BIN="python.exe"
    else
        fail "missing required command: python"
    fi
}

detect_cargo() {
    if [[ -n "${CARGO:-}" ]]; then
        CARGO_BIN="${CARGO}"
    elif command -v cargo >/dev/null 2>&1; then
        CARGO_BIN="cargo"
    elif command -v cargo.exe >/dev/null 2>&1; then
        CARGO_BIN="cargo.exe"
    else
        fail "missing required command: cargo"
    fi
}

json_value() {
    local file="$1"
    local path="$2"
    "${PYTHON_BIN}" - "$file" "$path" <<'PY'
import json
import sys

file_path, field_path = sys.argv[1], sys.argv[2]
with open(file_path, "r", encoding="utf-8") as fh:
    value = json.load(fh)
for part in field_path.split("."):
    if isinstance(value, list):
        value = value[int(part)]
    else:
        value = value[part]
if isinstance(value, bool):
    print("true" if value else "false")
elif value is None:
    print("null")
else:
    print(value)
PY
}

json_len() {
    local file="$1"
    local path="$2"
    "${PYTHON_BIN}" - "$file" "$path" <<'PY'
import json
import sys

file_path, field_path = sys.argv[1], sys.argv[2]
with open(file_path, "r", encoding="utf-8") as fh:
    value = json.load(fh)
for part in field_path.split("."):
    if isinstance(value, list):
        value = value[int(part)]
    else:
        value = value[part]
print(len(value))
PY
}

assert_status() {
    local actual="$1"
    local expected="$2"
    local label="$3"
    [[ "${actual}" == "${expected}" ]] || fail "${label}: expected HTTP ${expected}, got ${actual}"
    pass "${label}: HTTP ${expected}"
}

assert_json_eq() {
    local file="$1"
    local path="$2"
    local expected="$3"
    local label="$4"
    local actual
    actual="$(json_value "$file" "$path")"
    [[ "${actual}" == "${expected}" ]] || fail "${label}: expected ${path}=${expected}, got ${actual}"
    pass "${label}: ${path}=${expected}"
}

assert_json_nonempty() {
    local file="$1"
    local path="$2"
    local label="$3"
    local actual
    actual="$(json_value "$file" "$path")"
    [[ -n "${actual}" && "${actual}" != "null" ]] || fail "${label}: expected non-empty ${path}"
    pass "${label}: ${path} is non-empty"
}

request() {
    local output_file="$1"
    local method="$2"
    local url="$3"
    shift 3
    curl -sS -o "${output_file}" -w '%{http_code}' -X "${method}" "${url}" "$@" || fail "curl failed: ${method} ${url}"
}

make_test_files() {
    local dir="$1"
    "${PYTHON_BIN}" - "$dir" <<'PY'
import base64
import pathlib
import sys

out = pathlib.Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)
png = base64.b64decode(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
)
out.joinpath("one.png").write_bytes(png)
out.joinpath("two.png").write_bytes(png)
out.joinpath("not-image.txt").write_text("not an image", encoding="utf-8")
out.joinpath("too-large.bin").write_bytes(b"x" * (1024 * 1024 + 1))
out.joinpath("alt-201.txt").write_text("a" * 201, encoding="utf-8")
out.joinpath("description-1001.txt").write_text("d" * 1001, encoding="utf-8")
out.joinpath("q-101.txt").write_text("q" * 101, encoding="utf-8")
PY
}

random_port() {
    "${PYTHON_BIN}" - <<'PY'
import socket

sock = socket.socket()
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
}

cleanup_server() {
    if [[ -n "${SERVER_PID}" ]]; then
        kill "${SERVER_PID}" >/dev/null 2>&1 || true
        wait "${SERVER_PID}" >/dev/null 2>&1 || true
    fi
    if [[ -n "${IMAGE_API_TEST_TMP}" && -d "${IMAGE_API_TEST_TMP}" ]]; then
        rm -rf "${IMAGE_API_TEST_TMP}"
    fi
}

start_test_server() {
    require_cmd curl
    detect_python
    detect_cargo

    IMAGE_API_TEST_TMP="$(mktemp -d)"
    make_test_files "${IMAGE_API_TEST_TMP}/files"

    log "building server binary"
    (cd "${ROOT_DIR}" && "${CARGO_BIN}" build --quiet --bin server) || fail "cargo build --bin server failed"

    local server_bin="${ROOT_DIR}/target/debug/server"
    if [[ ! -x "${server_bin}" && -f "${server_bin}.exe" ]]; then
        server_bin="${server_bin}.exe"
    fi
    [[ -x "${server_bin}" || -f "${server_bin}" ]] || fail "server binary not found: ${server_bin}"

    local port
    port="$(random_port)"
    BASE_URL="http://127.0.0.1:${port}"
    log "starting isolated server at ${BASE_URL}"
    (
        cd "${IMAGE_API_TEST_TMP}"
        WEB_HOST="127.0.0.1" \
            WEB_PORT="${port}" \
            TOKEN="${TEST_TOKEN}" \
            SITE_URL="${BASE_URL}" \
            IMAGE_MAX_UPLOAD_MB="1" \
            "${server_bin}"
    ) >"${IMAGE_API_TEST_TMP}/server.log" 2>&1 &
    SERVER_PID="$!"
    trap cleanup_server EXIT

    for _ in $(seq 1 80); do
        local status
        status="$(curl -sS -o /dev/null -w '%{http_code}' "${BASE_URL}/api/images" || true)"
        if [[ "${status}" == "401" ]]; then
            pass "server is ready"
            return 0
        fi
        sleep 0.25
    done
    fail "server did not become ready"
}

upload_valid_image() {
    local output_file="$1"
    local image_file="${2:-${IMAGE_API_TEST_TMP}/files/one.png}"
    local alt="${3:-test alt}"
    local description="${4:-test description}"
    local status
    status="$(request "${output_file}" POST "${BASE_URL}/api/images" \
        -H "Authorization: Bearer ${TEST_TOKEN}" \
        -F "file=@${image_file};type=image/png" \
        -F "alt=${alt}" \
        -F "description=${description}")"
    assert_status "${status}" "201" "upload valid image"
    assert_json_eq "${output_file}" "success" "true" "upload valid image"
}

url_path() {
    local file="$1"
    local path="$2"
    local raw
    raw="$(json_value "$file" "$path")"
    "${PYTHON_BIN}" - "$raw" <<'PY'
import sys
from urllib.parse import urlparse

value = sys.argv[1]
parsed = urlparse(value)
print(parsed.path if parsed.scheme else value)
PY
}
