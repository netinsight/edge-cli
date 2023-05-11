#!/bin/bash

set -euo pipefail

# appliance delete va-01 va-02

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

echo "Deleted appliances:"
for appliance in "$@"; do
    appliances="$(curl "$edge_url/api/appliance/" \
        --silent \
        --get \
        --data-urlencode 'q={"filter":{"searchName":"'"$appliance"'"}}' \
        --cookie "$cookie_jar")"

    ids=$(jq --raw-output '.items | map(.id) | join("\n")' <<<"$appliances")

    for id in $ids; do
        deleted=$(curl "$edge_url/api/appliance/$id" \
            -X DELETE \
            --silent \
            --get \
            --cookie "$cookie_jar")
        jq --raw-output '.name' <<<"$deleted"
    done
done
