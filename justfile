set windows-shell := ["pwsh", "-NoLogo", "-Command"]

script_dir := justfile_directory() / "just"
test_dir := justfile_directory() / "tests"

default:
    @just --list

check:
    @bash -lc 'bash just/check.sh'

test-image-api:
    @bash -lc 'bash tests/image_api_all.sh'

build:
    @bash -lc 'bash just/build-release.sh'

run:
    @bash -lc 'bash just/run.sh'

deploy:
    @bash -lc 'bash just/deploy-macos.sh'

undeploy:
    @bash -lc 'bash just/undeploy-macos.sh'

status:
    @bash -lc 'bash just/status-macos.sh'

logs:
    @bash -lc 'bash just/logs-macos.sh'

help:
    @sed -n '1,240p' "{{script_dir}}/just.md"
