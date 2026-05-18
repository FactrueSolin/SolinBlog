#!/usr/bin/env bash
set -euo pipefail

TEST_NAME="auth"
source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/image_api_common.sh"

start_test_server

body="${IMAGE_API_TEST_TMP}/auth-list-no-token.json"
status="$(request "${body}" GET "${BASE_URL}/api/images")"
assert_status "${status}" "401" "list rejects missing bearer token"
assert_json_eq "${body}" "success" "false" "list rejects missing bearer token"
assert_json_eq "${body}" "error.code" "unauthorized" "list rejects missing bearer token"

body="${IMAGE_API_TEST_TMP}/auth-list-wrong-token.json"
status="$(request "${body}" GET "${BASE_URL}/api/images" -H "Authorization: Bearer wrong-token")"
assert_status "${status}" "401" "list rejects wrong bearer token"
assert_json_eq "${body}" "error.code" "unauthorized" "list rejects wrong bearer token"

body="${IMAGE_API_TEST_TMP}/auth-upload-no-token.json"
status="$(request "${body}" POST "${BASE_URL}/api/images" -F "file=@${IMAGE_API_TEST_TMP}/files/one.png;type=image/png")"
assert_status "${status}" "401" "upload rejects missing bearer token"
assert_json_eq "${body}" "error.code" "unauthorized" "upload rejects missing bearer token"

body="${IMAGE_API_TEST_TMP}/auth-injected-token.json"
status="$(request "${body}" GET "${BASE_URL}/api/images" -H "Authorization: Bearer ' OR '1'='1")"
assert_status "${status}" "401" "list rejects injection-like bearer token"
assert_json_eq "${body}" "error.code" "unauthorized" "list rejects injection-like bearer token"

body="${IMAGE_API_TEST_TMP}/auth-public-missing.txt"
status="$(request "${body}" GET "${BASE_URL}/images/img_missing/missing.png")"
assert_status "${status}" "404" "public image read stays unauthenticated but returns 404 for missing asset"

log "auth tests completed"
