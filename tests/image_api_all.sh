#!/usr/bin/env bash
set -euo pipefail

TEST_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

printf '[image-api] running auth tests\n'
bash "${TEST_DIR}/image_api_auth_test.sh"

printf '[image-api] running upload/public tests\n'
bash "${TEST_DIR}/image_api_upload_public_test.sh"

printf '[image-api] running metadata tests\n'
bash "${TEST_DIR}/image_api_metadata_test.sh"

printf '[image-api] running replace/delete tests\n'
bash "${TEST_DIR}/image_api_replace_delete_test.sh"

printf '[image-api][PASS] all image API shell tests passed\n'
