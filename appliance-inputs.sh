#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

id="${1?missing argument: input ID}"

inputs="$(curl "$edge_url/api/input/" \
    --silent \
    --get \
    --data-urlencode 'q={"filter":{"applianceId":"'"$id"'"}}' \
    --cookie "$cookie_jar")"

(
    printf "ID\tName\n" &&
    jq --raw-output '.items | map([.id, .name])[] | @tsv' <<<"$inputs"
) | column -t
