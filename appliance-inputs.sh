#!/bin/bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

id="${1?missing argument: input ID}"

inputs="$(curl "$edge_url/api/appliance/$id/inputs" \
    --silent \
    --get \
    --cookie "$cookie_jar")"

(
    printf "ID\tName\n" &&
    jq --raw-output '.items | map([.inputId, .inputName])[] | @tsv' <<<"$inputs"
) | column -t
