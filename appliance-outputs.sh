#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

id="${1?missing argument: output ID}"

outputs="$(curl "$edge_url/api/output/" \
    --silent \
    --get \
    --data-urlencode 'q={"filter":{"appliance":"'"$id"'","searchName":""}}' \
    --cookie "$cookie_jar")"

(
    printf "ID\tName\n" &&
    jq --raw-output '.items | map([.id, .name])[] | @tsv' <<<"$outputs"
) | column -t
