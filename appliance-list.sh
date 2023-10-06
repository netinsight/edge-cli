#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

appliances="$(curl "$edge_url/api/appliance/" \
    --silent \
    --get \
    --cookie "$cookie_jar")"

(
    printf "Name\tid\tstate\n" &&
    jq --raw-output '.items | map([.name, .id, .health.state])[] | @tsv' <<<"$appliances"
) | column -t
