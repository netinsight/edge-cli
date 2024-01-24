#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

echo "Deleted region:"
for region in "$@"; do
    regions="$(curl "$edge_url/api/region/" \
        --silent \
        --get \
        --data-urlencode 'q={"filter":{"name":"'"$region"'"}}' \
        --cookie "$cookie_jar")"

    ids=$(jq --raw-output '.items | map(.id) | join("\n")' <<<"$regions")

    for id in $ids; do
        deleted=$(curl "$edge_url/api/region/$id" \
            -X DELETE \
            --silent \
            --get \
            --cookie "$cookie_jar")
        jq --raw-output '.name' <<<"$deleted"
    done
done
