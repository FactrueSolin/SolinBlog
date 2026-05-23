#!/usr/bin/env bash
set -euo pipefail

TEST_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "${TEST_DIR}/.." && pwd)"

log() {
    printf '[mcp-image-tools] %s\n' "$*"
}

pass() {
    printf '[mcp-image-tools][PASS] %s\n' "$*"
}

fail() {
    printf '[mcp-image-tools][FAIL] %s\n' "$*" >&2
    exit 1
}

require_cmd() {
    command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

detect_python() {
    if [[ -n "${PYTHON:-}" ]]; then
        PYTHON_BIN="${PYTHON}"
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

grep_required() {
    local pattern="$1"
    local file="$2"
    local label="$3"
    grep -Eq "$pattern" "$file" || fail "$label: missing pattern '$pattern' in $file"
    pass "$label"
}

grep_forbidden() {
    local pattern="$1"
    local file="$2"
    local label="$3"
    if grep -Eq "$pattern" "$file"; then
        fail "$label: forbidden pattern '$pattern' found in $file"
    fi
    pass "$label"
}

assert_struct_fields_exact() {
    local struct_name="$1"
    local expected_csv="$2"
    local file="$3"
    local label="$4"

    "$PYTHON_BIN" - "$struct_name" "$expected_csv" "$file" <<'PY'
import re
import sys

struct_name, expected_csv, file_path = sys.argv[1:]
expected = [field for field in expected_csv.split(",") if field]
source = open(file_path, "r", encoding="utf-8").read()
match = re.search(r"pub struct " + re.escape(struct_name) + r"\s*\{(?P<body>.*?)\n\}", source, re.S)
if not match:
    print(f"struct not found: {struct_name}", file=sys.stderr)
    sys.exit(1)
fields = re.findall(r"pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:", match.group("body"))
if fields != expected:
    print(f"{struct_name} fields mismatch", file=sys.stderr)
    print(f"expected: {expected}", file=sys.stderr)
    print(f"actual:   {fields}", file=sys.stderr)
    sys.exit(1)
PY
    pass "$label"
}

assert_struct_block_forbidden() {
    local struct_name="$1"
    local forbidden_csv="$2"
    local file="$3"
    local label="$4"

    "$PYTHON_BIN" - "$struct_name" "$forbidden_csv" "$file" <<'PY'
import re
import sys

struct_name, forbidden_csv, file_path = sys.argv[1:]
forbidden = {field for field in forbidden_csv.split(",") if field}
source = open(file_path, "r", encoding="utf-8").read()
match = re.search(r"pub struct " + re.escape(struct_name) + r"\s*\{(?P<body>.*?)\n\}", source, re.S)
if not match:
    print(f"struct not found: {struct_name}", file=sys.stderr)
    sys.exit(1)
fields = set(re.findall(r"pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:", match.group("body")))
violations = sorted(fields & forbidden)
if violations:
    print(f"{struct_name} contains forbidden request fields: {violations}", file=sys.stderr)
    sys.exit(1)
PY
    pass "$label"
}

require_cmd grep
detect_python

DTO_FILE="${ROOT_DIR}/src/mcp/dto.rs"
TOOLS_FILE="${ROOT_DIR}/src/mcp/tools.rs"
SERVER_FILE="${ROOT_DIR}/src/bin/server.rs"

[[ -f "$DTO_FILE" ]] || fail "missing DTO file: $DTO_FILE"
[[ -f "$TOOLS_FILE" ]] || fail "missing MCP tools file: $TOOLS_FILE"
[[ -f "$SERVER_FILE" ]] || fail "missing server file: $SERVER_FILE"

log "checking MCP image request schema boundaries"
assert_struct_fields_exact "ListImagesRequest" "limit,offset,q" "$DTO_FILE" "list_images request exposes only limit/offset/q"
assert_struct_fields_exact "GetImageRequest" "image_id" "$DTO_FILE" "get_image request exposes only image_id"
assert_struct_fields_exact "UpdateImageRequest" "image_id,alt,description" "$DTO_FILE" "update_image request exposes only image_id/alt/description"
assert_struct_fields_exact "DeleteImageRequest" "image_id" "$DTO_FILE" "delete_image request exposes only image_id"

FORBIDDEN_INPUT_FIELDS="filename,relative_path,content_type,size_bytes,width,height,sha256,created_at,updated_at,file,bytes,base64,url"
assert_struct_block_forbidden "ListImagesRequest" "$FORBIDDEN_INPUT_FIELDS" "$DTO_FILE" "list_images rejects upload/replace-style input fields"
assert_struct_block_forbidden "GetImageRequest" "$FORBIDDEN_INPUT_FIELDS" "$DTO_FILE" "get_image rejects upload/replace-style input fields"
assert_struct_block_forbidden "UpdateImageRequest" "$FORBIDDEN_INPUT_FIELDS" "$DTO_FILE" "update_image rejects upload/replace-style input fields"
assert_struct_block_forbidden "DeleteImageRequest" "$FORBIDDEN_INPUT_FIELDS" "$DTO_FILE" "delete_image rejects upload/replace-style input fields"

log "checking MCP image tool implementation boundaries"
grep_required "async fn list_images" "$TOOLS_FILE" "list_images tool exists"
grep_required "async fn get_image" "$TOOLS_FILE" "get_image tool exists"
grep_required "async fn update_image" "$TOOLS_FILE" "update_image tool exists"
grep_required "async fn delete_image" "$TOOLS_FILE" "delete_image tool exists"
grep_required "ImageMetaPatch" "$TOOLS_FILE" "update_image builds ImageMetaPatch"
grep_required "alt: params\.alt" "$TOOLS_FILE" "update_image patch uses params.alt"
grep_required "description: params\.description" "$TOOLS_FILE" "update_image patch uses params.description"
grep_required "\.update_image_meta\(&params\.image_id, patch\)" "$TOOLS_FILE" "update_image calls update_image_meta"
grep_required "success: false" "$TOOLS_FILE" "image tools return success=false for business failures"
grep_required "error: Some\(mcp_image_error\(err\)\)" "$TOOLS_FILE" "image tools use structured MCP image errors"
grep_required "fn mcp_image_error\(error: ImageHostError\) -> McpImageError" "$TOOLS_FILE" "structured MCP image error mapper exists"
grep_required "ImageStore" "$SERVER_FILE" "MCP server is wired to ImageStore"

log "checking unsupported upload/replace constraints"
grep_forbidden "async fn (upload|replace)_image" "$TOOLS_FILE" "MCP does not expose upload_image or replace_image tools"
grep_forbidden "struct (Upload|Replace)ImageRequest" "$DTO_FILE" "MCP does not define upload/replace request DTOs"
assert_struct_block_forbidden "ListImagesRequest" "file,bytes,base64,url" "$DTO_FILE" "list_images request does not expose upload/import fields"
assert_struct_block_forbidden "GetImageRequest" "file,bytes,base64,url" "$DTO_FILE" "get_image request does not expose upload/import fields"
assert_struct_block_forbidden "UpdateImageRequest" "file,bytes,base64,url" "$DTO_FILE" "update_image request does not expose upload/import fields"
assert_struct_block_forbidden "DeleteImageRequest" "file,bytes,base64,url" "$DTO_FILE" "delete_image request does not expose upload/import fields"
grep_required "do not support uploading images, replacing images, remote URL import, or base64 image input" "$TOOLS_FILE" "MCP instructions document unsupported upload/replace inputs"

log "MCP image tool tests completed"
