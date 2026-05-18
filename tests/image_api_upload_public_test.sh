#!/usr/bin/env bash
set -euo pipefail

TEST_NAME="upload-public"
source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/image_api_common.sh"

start_test_server

upload_body="${IMAGE_API_TEST_TMP}/upload-valid.json"
upload_valid_image "${upload_body}" "${IMAGE_API_TEST_TMP}/files/one.png" "public alt" "public description"
assert_json_nonempty "${upload_body}" "data.image_id" "upload returns image_id"
assert_json_nonempty "${upload_body}" "data.url" "upload returns public url"
assert_json_eq "${upload_body}" "data.meta.alt" "public alt" "upload preserves alt"
assert_json_eq "${upload_body}" "data.meta.description" "public description" "upload preserves description"

asset_path="$(url_path "${upload_body}" "data.url")"
asset_headers="${IMAGE_API_TEST_TMP}/asset.headers"
asset_body="${IMAGE_API_TEST_TMP}/asset.png"
status="$(curl -sS -D "${asset_headers}" -o "${asset_body}" -w '%{http_code}' "${BASE_URL}${asset_path}" || fail "curl failed: public asset read")"
assert_status "${status}" "200" "public asset read without token"
grep -qi '^content-type: image/png' "${asset_headers}" || fail "public asset read: expected content-type image/png"
grep -qi '^x-content-type-options: nosniff' "${asset_headers}" || fail "public asset read: expected nosniff header"
grep -qi '^cache-control: public, max-age=31536000, immutable' "${asset_headers}" || fail "public asset read: expected immutable cache header"
pass "public asset read: headers are correct"

etag="$(grep -i '^etag:' "${asset_headers}" | sed -E 's/^[Ee][Tt][Aa][Gg]:[[:space:]]*//; s/\r$//')"
[[ -n "${etag}" ]] || fail "public asset cache: missing ETag"
status="$(curl -sS -o "${IMAGE_API_TEST_TMP}/asset-304.body" -w '%{http_code}' -H "If-None-Match: ${etag}" "${BASE_URL}${asset_path}" || fail "curl failed: public asset conditional read")"
assert_status "${status}" "304" "public asset supports If-None-Match"

body="${IMAGE_API_TEST_TMP}/upload-missing-file.json"
status="$(request "${body}" POST "${BASE_URL}/api/images" -H "Authorization: Bearer ${TEST_TOKEN}" -F "alt=no file")"
assert_status "${status}" "400" "upload rejects missing file field"
assert_json_eq "${body}" "error.code" "invalid_request" "upload rejects missing file field"

body="${IMAGE_API_TEST_TMP}/upload-invalid-type.json"
status="$(request "${body}" POST "${BASE_URL}/api/images" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -F "file=@${IMAGE_API_TEST_TMP}/files/not-image.txt;type=text/plain")"
assert_status "${status}" "415" "upload rejects non-image payload"
assert_json_eq "${body}" "error.code" "unsupported_media_type" "upload rejects non-image payload"

body="${IMAGE_API_TEST_TMP}/upload-long-alt.json"
long_alt="$(cat "${IMAGE_API_TEST_TMP}/files/alt-201.txt")"
status="$(request "${body}" POST "${BASE_URL}/api/images" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -F "file=@${IMAGE_API_TEST_TMP}/files/one.png;type=image/png" \
    -F "alt=${long_alt}")"
assert_status "${status}" "400" "upload rejects overlong alt"
assert_json_eq "${body}" "error.code" "invalid_request" "upload rejects overlong alt"

body="${IMAGE_API_TEST_TMP}/upload-too-large.json"
status="$(request "${body}" POST "${BASE_URL}/api/images" \
    -H "Authorization: Bearer ${TEST_TOKEN}" \
    -F "file=@${IMAGE_API_TEST_TMP}/files/too-large.bin;type=image/png")"
assert_status "${status}" "413" "upload rejects oversized body"

log "upload and public read tests completed"
