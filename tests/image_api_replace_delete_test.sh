#!/usr/bin/env bash
set -euo pipefail

TEST_NAME="replace-delete"
source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/image_api_common.sh"

start_test_server

upload_body="${IMAGE_API_TEST_TMP}/replace-upload.json"
upload_valid_image "${upload_body}" "${IMAGE_API_TEST_TMP}/files/one.png" "before replace" "before replace description"
image_id="$(json_value "${upload_body}" "data.image_id")"
old_path="$(url_path "${upload_body}" "data.url")"

body="${IMAGE_API_TEST_TMP}/replace-invalid-type.json"
status="$(request "${body}" PUT "${BASE_URL}/api/images/${image_id}" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -F "file=@${IMAGE_API_TEST_TMP}/files/not-image.txt;type=text/plain")"
assert_status "${status}" "415" "replace rejects non-image payload"
assert_json_eq "${body}" "error.code" "unsupported_media_type" "replace rejects non-image payload"
status="$(curl -sS -o "${IMAGE_API_TEST_TMP}/old-after-failed-replace.png" -w '%{http_code}' "${BASE_URL}${old_path}" || fail "curl failed: old asset after failed replace")"
assert_status "${status}" "200" "failed replace keeps old public URL working"

body="${IMAGE_API_TEST_TMP}/replace-valid.json"
status="$(request "${body}" PUT "${BASE_URL}/api/images/${image_id}" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -F "file=@${IMAGE_API_TEST_TMP}/files/two.png;type=image/png" \
    -F "alt=after replace" \
    -F "description=after replace description")"
assert_status "${status}" "200" "replace image succeeds"
assert_json_eq "${body}" "data.image_id" "${image_id}" "replace image keeps image_id"
assert_json_eq "${body}" "data.alt" "after replace" "replace image updates alt"
new_path="$(url_path "${body}" "data.url")"
[[ "${new_path}" != "${old_path}" ]] || fail "replace image succeeds: expected URL filename to change"
pass "replace image succeeds: public URL changed"

status="$(curl -sS -o "${IMAGE_API_TEST_TMP}/new-after-replace.png" -w '%{http_code}' "${BASE_URL}${new_path}" || fail "curl failed: new asset after replace")"
assert_status "${status}" "200" "replace makes new public URL readable"
status="$(curl -sS -o "${IMAGE_API_TEST_TMP}/old-after-replace.txt" -w '%{http_code}' "${BASE_URL}${old_path}" || fail "curl failed: old asset after replace")"
assert_status "${status}" "404" "replace invalidates old filename URL"

body="${IMAGE_API_TEST_TMP}/replace-missing-file.json"
status="$(request "${body}" PUT "${BASE_URL}/api/images/${image_id}" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -F "alt=no file")"
assert_status "${status}" "400" "replace rejects missing file field"
assert_json_eq "${body}" "error.code" "invalid_request" "replace rejects missing file field"

body="${IMAGE_API_TEST_TMP}/replace-not-found.json"
status="$(request "${body}" PUT "${BASE_URL}/api/images/img_missing" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -F "file=@${IMAGE_API_TEST_TMP}/files/one.png;type=image/png")"
assert_status "${status}" "404" "replace rejects missing image id"
assert_json_eq "${body}" "error.code" "image_not_found" "replace rejects missing image id"

body="${IMAGE_API_TEST_TMP}/delete-valid.json"
status="$(request "${body}" DELETE "${BASE_URL}/api/images/${image_id}" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "200" "delete image succeeds"
assert_json_eq "${body}" "data.deleted" "true" "delete image succeeds"
assert_json_eq "${body}" "data.image_id" "${image_id}" "delete image succeeds"

body="${IMAGE_API_TEST_TMP}/delete-detail-after.json"
status="$(request "${body}" GET "${BASE_URL}/api/images/${image_id}" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "404" "deleted image metadata is gone"
assert_json_eq "${body}" "error.code" "image_not_found" "deleted image metadata is gone"
status="$(curl -sS -o "${IMAGE_API_TEST_TMP}/deleted-public.txt" -w '%{http_code}' "${BASE_URL}${new_path}" || fail "curl failed: deleted asset read")"
assert_status "${status}" "404" "deleted public URL is gone"

body="${IMAGE_API_TEST_TMP}/delete-again.json"
status="$(request "${body}" DELETE "${BASE_URL}/api/images/${image_id}" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "404" "delete is non-idempotent"
assert_json_eq "${body}" "error.code" "image_not_found" "delete is non-idempotent"

body="${IMAGE_API_TEST_TMP}/delete-path-traversal.json"
status="$(request "${body}" DELETE "${BASE_URL}/api/images/img_..%5C..%5Cetc%5Cpasswd" -H "Authorization: Bearer ${TEST_TOKEN}")"
assert_status "${status}" "404" "delete rejects path traversal-like id"
assert_json_eq "${body}" "error.code" "image_not_found" "delete rejects path traversal-like id"

log "replace and delete tests completed"
