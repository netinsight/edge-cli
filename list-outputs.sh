#!/bin/bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

outputs="$(curl "$edge_url/api/output/" \
    --silent \
    --get \
    --cookie "$cookie_jar")"

(
    printf "Name\tid\n" &&
    jq --raw-output '.items | map([.name, .id])[] | @tsv' <<<"$outputs"
) | column -t
