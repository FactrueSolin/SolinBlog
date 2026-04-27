script_dir := justfile_directory() / "just"

default:
    @just --list

check:
    @"{{script_dir}}/check.sh"

build:
    @"{{script_dir}}/build-release.sh"

run:
    @"{{script_dir}}/run.sh"

deploy:
    @"{{script_dir}}/deploy-macos.sh"

undeploy:
    @"{{script_dir}}/undeploy-macos.sh"

status:
    @"{{script_dir}}/status-macos.sh"

logs:
    @"{{script_dir}}/logs-macos.sh"

help:
    @sed -n '1,240p' "{{script_dir}}/just.md"
