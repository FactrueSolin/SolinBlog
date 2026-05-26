#!/usr/bin/env bash
set -euo pipefail

TEST_NAME="homepage-cache"
TEST_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "${TEST_DIR}/.." && pwd)"
PYTHON_BIN="${PYTHON:-}"
CARGO_BIN="${CARGO:-}"
TEST_TMP=""
SERVER_PID=""
BASE_URL=""

log() {
    printf '[homepage-cache] %s\n' "$*"
}

pass() {
    printf '[homepage-cache][PASS] %s\n' "$*"
}

fail() {
    printf '[homepage-cache][FAIL] %s\n' "$*" >&2
    if [[ -n "${TEST_TMP}" && -f "${TEST_TMP}/server.log" ]]; then
        printf '[homepage-cache][server-log]\n' >&2
        tail -n 80 "${TEST_TMP}/server.log" >&2 || true
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
    if [[ -n "${CARGO_BIN}" ]]; then
        command -v "${CARGO_BIN}" >/dev/null 2>&1 || fail "missing required command: ${CARGO_BIN}"
    elif command -v cargo >/dev/null 2>&1; then
        CARGO_BIN="cargo"
    elif command -v cargo.exe >/dev/null 2>&1; then
        CARGO_BIN="cargo.exe"
    else
        fail "missing required command: cargo"
    fi
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

cleanup() {
    if [[ -n "${SERVER_PID}" ]]; then
        kill "${SERVER_PID}" >/dev/null 2>&1 || true
        wait "${SERVER_PID}" >/dev/null 2>&1 || true
    fi
    if [[ -n "${TEST_TMP}" && -d "${TEST_TMP}" ]]; then
        rm -rf "${TEST_TMP}"
    fi
}

write_fixture() {
    "${PYTHON_BIN}" - "${TEST_TMP}" <<'PY'
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
(root / "front").mkdir(parents=True, exist_ok=True)
(root / "public").mkdir(parents=True, exist_ok=True)
(root / "data" / "old-page").mkdir(parents=True, exist_ok=True)
(root / "data" / "new-page").mkdir(parents=True, exist_ok=True)

(root / "front" / "header.html").write_text("<header>SolinBlog Test</header>", encoding="utf-8")
(root / "front" / "index.html").write_text(
    "<!doctype html><html><head><title>{{site_title}}</title></head><body>{{site_header}}<p>{{site_subtitle}}</p><main>{{page_list}}</main>{{beian_number}}</body></html>",
    encoding="utf-8",
)
(root / "front" / "404.html").write_text(
    "<!doctype html><html><head><title>404</title></head><body>{{site_header}}<main>not found</main></body></html>",
    encoding="utf-8",
)
(root / "public" / "cache-ok.txt").write_text("public asset ok", encoding="utf-8")

index = {
    "pages": {
        "old-page": {
            "page_id": "old-page",
            "seo": {
                "title": "Index Oldest",
                "seo_title": "index-oldest",
                "description": "older page from index",
                "keywords": ["old"],
                "extra": {},
            },
            "original_id": None,
        },
        "new-page": {
            "page_id": "new-page",
            "seo": {
                "title": "Index Newest",
                "seo_title": "index-newest",
                "description": "newer page from index",
                "keywords": ["new"],
                "extra": {},
            },
            "original_id": None,
        },
    }
}
(root / "data" / "index.json").write_text(json.dumps(index, ensure_ascii=False, indent=2), encoding="utf-8")

old_meta = {
    "seo": {
        "title": "Meta Trap Old",
        "seo_title": "meta-trap-old",
        "description": "old page from meta",
        "keywords": ["old-meta"],
        "extra": {},
    },
    "page_uid": "oldpageuid000001",
    "created_at": 100,
    "updated_at": 100,
    "view_count": 0,
    "extra": {},
}
new_meta = {
    "seo": {
        "title": "Meta Trap New",
        "seo_title": "meta-trap-new",
        "description": "new page from meta",
        "keywords": ["new-meta"],
        "extra": {},
    },
    "page_uid": "newpageuid000001",
    "created_at": 200,
    "updated_at": 300,
    "view_count": 0,
    "extra": {},
}
(root / "data" / "old-page" / "meta.json").write_text(json.dumps(old_meta, ensure_ascii=False, indent=2), encoding="utf-8")
(root / "data" / "old-page" / "index.html").write_text("<html><body>old</body></html>", encoding="utf-8")
(root / "data" / "new-page" / "meta.json").write_text(json.dumps(new_meta, ensure_ascii=False, indent=2), encoding="utf-8")
(root / "data" / "new-page" / "index.html").write_text("<html><body>new</body></html>", encoding="utf-8")
PY
}

rewrite_index_for_cache_invalidation() {
    "${PYTHON_BIN}" - "${TEST_TMP}/data/index.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
index = json.loads(path.read_text(encoding="utf-8"))
index["pages"]["new-page"]["seo"]["title"] = "After Invalidate"
index["pages"]["new-page"]["updated_at"] = 999
path.write_text(json.dumps(index, ensure_ascii=False, indent=2), encoding="utf-8")
PY
}

request() {
    local output_file="$1"
    local url="$2"
    shift 2
    curl -sS -D "${output_file}.headers" -o "${output_file}" -w '%{http_code}' "${url}" "$@" || fail "curl failed: ${url}"
}

assert_status() {
    local actual="$1"
    local expected="$2"
    local label="$3"
    [[ "${actual}" == "${expected}" ]] || fail "${label}: expected HTTP ${expected}, got ${actual}"
    pass "${label}: HTTP ${expected}"
}

assert_contains() {
    local file="$1"
    local needle="$2"
    local label="$3"
    grep -Fq -- "${needle}" "${file}" || fail "${label}: missing '${needle}'"
    pass "${label}: found '${needle}'"
}

assert_not_contains() {
    local file="$1"
    local needle="$2"
    local label="$3"
    if grep -Fq -- "${needle}" "${file}"; then
        fail "${label}: unexpected '${needle}'"
    fi
    pass "${label}: did not find '${needle}'"
}

assert_header_contains() {
    local file="$1"
    local needle="$2"
    local label="$3"
    grep -Fqi -- "${needle}" "${file}.headers" || fail "${label}: missing header '${needle}'"
    pass "${label}: found header '${needle}'"
}

assert_header_absent() {
    local file="$1"
    local needle="$2"
    local label="$3"
    if grep -Fqi -- "${needle}" "${file}.headers"; then
        fail "${label}: unexpected header '${needle}'"
    fi
    pass "${label}: header '${needle}' absent"
}

assert_order() {
    local file="$1"
    local first="$2"
    local second="$3"
    local label="$4"
    "${PYTHON_BIN}" - "$file" "$first" "$second" <<'PY' || fail "${label}: '${first}' was not before '${second}'"
import sys
text = open(sys.argv[1], encoding="utf-8").read()
first = text.find(sys.argv[2])
second = text.find(sys.argv[3])
if first < 0 or second < 0 or first >= second:
    raise SystemExit(1)
PY
    pass "${label}: '${first}' before '${second}'"
}

assert_legacy_index_backfilled() {
    "${PYTHON_BIN}" - "${TEST_TMP}/data/index.json" <<'PY' || fail "legacy index.json was not backfilled from meta.json"
import json
import pathlib
import sys

index = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
old_page = index["pages"]["old-page"]
new_page = index["pages"]["new-page"]
if old_page.get("created_at") != 100 or old_page.get("updated_at") != 100:
    raise SystemExit(1)
if new_page.get("created_at") != 200 or new_page.get("updated_at") != 300:
    raise SystemExit(1)
PY
    pass "legacy index.json is backfilled from meta.json"
}

start_server() {
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
        cd "${TEST_TMP}"
        WEB_HOST="127.0.0.1" \
            WEB_PORT="${port}" \
            TOKEN="homepage-cache-token" \
            SITE_URL="${BASE_URL}" \
            "${server_bin}"
    ) >"${TEST_TMP}/server.log" 2>&1 &
    SERVER_PID="$!"
    trap cleanup EXIT

    for _ in $(seq 1 80); do
        local status
        status="$(curl -sS -o /dev/null -w '%{http_code}' "${BASE_URL}/" || true)"
        if [[ "${status}" == "200" ]]; then
            pass "server is ready"
            return 0
        fi
        sleep 0.25
    done
    fail "server did not become ready"
}

check_homepage_behavior() {
    local body="${TEST_TMP}/home.html"
    local status
    status="$(request "${body}" "${BASE_URL}/")"
    assert_status "${status}" "200" "homepage renders with legacy index.json"
    assert_contains "${body}" "Index Newest" "homepage uses index entry title"
    assert_contains "${body}" "Index Oldest" "homepage includes older index entry"
    assert_order "${body}" "Index Newest" "Index Oldest" "homepage sorts by backfilled meta updated_at"
    assert_not_contains "${body}" "Meta Trap" "homepage ignores per-page meta.json content"
    assert_legacy_index_backfilled

    sleep 1.1
    rewrite_index_for_cache_invalidation
    local updated_body="${TEST_TMP}/home-updated.html"
    status="$(request "${updated_body}" "${BASE_URL}/")"
    assert_status "${status}" "200" "homepage renders after index change"
    assert_contains "${updated_body}" "After Invalidate" "homepage cache invalidates on index.json key change"
}

check_public_cache_headers() {
    local ok_body="${TEST_TMP}/public-ok.txt"
    local missing_body="${TEST_TMP}/public-missing.html"
    local traversal_body="${TEST_TMP}/public-traversal.html"
    local status

    status="$(request "${ok_body}" "${BASE_URL}/public/cache-ok.txt")"
    assert_status "${status}" "200" "/public existing asset"
    assert_header_contains "${ok_body}" "content-type:" "/public existing asset content type"
    assert_header_contains "${ok_body}" "cache-control: public, max-age=86400" "/public existing asset cache control"

    status="$(request "${missing_body}" "${BASE_URL}/public/missing.txt")"
    assert_status "${status}" "404" "/public missing asset"
    assert_header_absent "${missing_body}" "cache-control: public, max-age=86400" "/public missing asset cache control"

    status="$(request "${traversal_body}" "${BASE_URL}/public/%2e%2e/data/index.json")"
    assert_status "${status}" "404" "/public traversal attempt"
    assert_header_absent "${traversal_body}" "cache-control: public, max-age=86400" "/public traversal cache control"
}

check_source_guards() {
    "${PYTHON_BIN}" - "${ROOT_DIR}" <<'PY'
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1])
store = (root / "src" / "store.rs").read_text(encoding="utf-8")
web_core = (root / "src" / "web_core.rs").read_text(encoding="utf-8")

def fail(message: str) -> None:
    print(f"[homepage-cache][FAIL] {message}", file=sys.stderr)
    raise SystemExit(1)

def body(name: str, text: str) -> str:
    marker = f"fn {name}"
    start = text.find(marker)
    if start < 0:
        fail(f"missing function {name}")
    brace = text.find("{", start)
    if brace < 0:
        fail(f"missing body for {name}")
    depth = 0
    for index in range(brace, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[brace:index + 1]
    fail(f"unterminated function {name}")

entry_match = re.search(r"pub struct PageIndexEntry \{(?P<body>.*?)\n\}", store, re.S)
if not entry_match:
    fail("PageIndexEntry struct not found")
entry_body = entry_match.group("body")
for field in ("created_at", "updated_at"):
    if f"pub {field}: i64" not in entry_body:
        fail(f"PageIndexEntry missing {field}: i64")
    before = entry_body[:entry_body.find(f"pub {field}: i64")]
    previous_lines = [line.strip() for line in before.splitlines()[-2:]]
    if "#[serde(default)]" not in previous_lines:
        fail(f"PageIndexEntry {field} must use #[serde(default)]")

for fn_name in (
    "save_page_with_markdown",
    "update_page_meta",
    "update_page_html",
    "update_page_markdown",
    "rebuild_index",
):
    fn_body = body(fn_name, store)
    if "PageIndexEntry" not in fn_body:
        fail(f"{fn_name} does not update PageIndexEntry")
    for field in ("created_at", "updated_at"):
        if field not in fn_body:
            fail(f"{fn_name} does not maintain {field}")

increment_body = body("increment_view_count", store)
if "save_index" in increment_body or "index.pages" in increment_body:
    fail("increment_view_count must not refresh index.json or homepage cache")

render_body = body("render_index_html", web_core)
if "list_page_entries" not in render_body:
    fail("render_index_html must use index entries")
for forbidden in ("get_page_meta", "load_page", "meta.json"):
    if forbidden in render_body:
        fail(f"render_index_html still references {forbidden}")
for required in ("updated_at", "created_at", "page_id"):
    if required not in render_body:
        fail(f"render_index_html missing sort/use field {required}")

print("[homepage-cache][PASS] source guards for index fields and homepage meta reads passed")
PY
}

main() {
    require_cmd curl
    require_cmd grep
    detect_python
    detect_cargo
    TEST_TMP="$(mktemp -d)"
    write_fixture
    start_server
    check_homepage_behavior
    check_public_cache_headers
    check_source_guards
    pass "homepage cache and public cache tests passed"
}

main "$@"
