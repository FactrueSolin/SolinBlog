#!/usr/bin/env bash
set -euo pipefail

TEST_NAME="metadata"
source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/image_api_common.sh"

start_test_server

first="${IMAGE_API_TEST_TMP}/meta-upload-first.json"
second="${IMAGE_API_TEST_TMP}/meta-upload-second.json"
upload_valid_image "${first}" "${IMAGE_API_TEST_TMP}/files/one.png" "alpha alt" "first description"
upload_valid_image "${second}" "${IMAGE_API_TEST_TMP}/files/two.png" "bravo alt" "second description"
image_id="$(json_value "${first}" "data.image_id")"

body="${IMAGE_API_TEST_TMP}/meta-list.json"
status="$(request "${body}" GET "${BASE_URL}/api/images?limit=100&offset=0" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "200" "list images succeeds"
assert_json_eq "${body}" "success" "true" "list images succeeds"
items_len="$(json_len "${body}" "data.items")"
[[ "${items_len}" -ge 2 ]] || fail "list images succeeds: expected at least 2 items, got ${items_len}"
pass "list images succeeds: returned ${items_len} items"

body="${IMAGE_API_TEST_TMP}/meta-search.json"
status="$(request "${body}" GET "${BASE_URL}/api/images?q=alpha" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "200" "search images by alt succeeds"
assert_json_eq "${body}" "data.items.0.image_id" "${image_id}" "search images by alt succeeds"

body="${IMAGE_API_TEST_TMP}/meta-detail.json"
status="$(request "${body}" GET "${BASE_URL}/api/images/${image_id}" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "200" "get image metadata succeeds"
assert_json_eq "${body}" "data.image_id" "${image_id}" "get image metadata succeeds"

body="${IMAGE_API_TEST_TMP}/meta-list-limit-zero.json"
status="$(request "${body}" GET "${BASE_URL}/api/images?limit=0" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "400" "list rejects limit below range"
assert_json_eq "${body}" "error.code" "invalid_request" "list rejects limit below range"

body="${IMAGE_API_TEST_TMP}/meta-list-limit-too-large.json"
status="$(request "${body}" GET "${BASE_URL}/api/images?limit=101" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "400" "list rejects limit above range"
assert_json_eq "${body}" "error.code" "invalid_request" "list rejects limit above range"

body="${IMAGE_API_TEST_TMP}/meta-list-q-too-long.json"
long_q="$(cat "${IMAGE_API_TEST_TMP}/files/q-101.txt")"
status="$(request "${body}" GET "${BASE_URL}/api/images?q=${long_q}" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "400" "list rejects overlong injection-like query"
assert_json_eq "${body}" "error.code" "invalid_request" "list rejects overlong injection-like query"

body="${IMAGE_API_TEST_TMP}/meta-not-found.json"
status="$(request "${body}" GET "${BASE_URL}/api/images/img_..%5C..%5Cetc%5Cpasswd" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "404" "get image rejects path traversal-like id"
assert_json_eq "${body}" "error.code" "image_not_found" "get image rejects path traversal-like id"

body="${IMAGE_API_TEST_TMP}/meta-patch-valid.json"
status="$(request "${body}" PATCH "${BASE_URL}/api/images/${image_id}" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -H "Content-Type: application/json" \
    --data '{"alt":"updated alt","description":"updated description"}')"
assert_status "${status}" "200" "patch metadata succeeds"
assert_json_eq "${body}" "data.alt" "updated alt" "patch metadata succeeds"
assert_json_eq "${body}" "data.description" "updated description" "patch metadata succeeds"

body="${IMAGE_API_TEST_TMP}/meta-patch-empty.json"
status="$(request "${body}" PATCH "${BASE_URL}/api/images/${image_id}" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -H "Content-Type: application/json" \
    --data '{}')"
assert_status "${status}" "400" "patch rejects empty object"
assert_json_eq "${body}" "error.code" "invalid_request" "patch rejects empty object"

body="${IMAGE_API_TEST_TMP}/meta-patch-forbidden-only.json"
status="$(request "${body}" PATCH "${BASE_URL}/api/images/${image_id}" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -H "Content-Type: application/json" \
    --data '{"filename":"../../evil.png"}')"
assert_status "${status}" "400" "patch rejects forbidden-field-only payload"
assert_json_eq "${body}" "error.code" "invalid_request" "patch rejects forbidden-field-only payload"

body="${IMAGE_API_TEST_TMP}/meta-patch-long-description.json"
long_description="$(cat "${IMAGE_API_TEST_TMP}/files/description-1001.txt")"
status="$(request "${body}" PATCH "${BASE_URL}/api/images/${image_id}" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -H "Content-Type: application/json" \
    --data "{\"description\":\"${long_description}\"}")"
assert_status "${status}" "400" "patch rejects overlong description"
assert_json_eq "${body}" "error.code" "invalid_request" "patch rejects overlong description"

body="${IMAGE_API_TEST_TMP}/meta-patch-invalid-json.txt"
status="$(request "${body}" PATCH "${BASE_URL}/api/images/${image_id}" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -H "Content-Type: application/json" \
    --data '{not-json')"
assert_status "${status}" "400" "patch rejects invalid JSON"

log "metadata tests completed"
