#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

id="${1?missing argument: input ID}"

outputs="$(curl "$edge_url/api/appliance/$id/outputs" \
    --silent \
    --get \
    --cookie "$cookie_jar")"

(
    printf "ID\tName\n" &&
    jq --raw-output '.items | map([.outputId, .outputName])[] | @tsv' <<<"$outputs"
) | column -t
