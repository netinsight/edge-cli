#!/bin/bash

set -euo pipefail

# appliance delete va-01 va-02

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
install_name="${edge_url}"
install_name="${install_name#https://}"
install_name="${install_name#http://}"
install_name="${install_name#/}"

cookie_jar="$HOME/.config/edge-cli/$install_name.cookie"
if ! [ -f "$cookie_jar" ]; then
    mkdir -p "$(dirname "$cookie_jar")"
    curl "$edge_url/api/login/" \
        -X POST \
        -H 'Accept: application/json' \
        -H 'Content-Type: application/json' \
        --cookie-jar "$cookie_jar" \
        --data-raw '{"username":"admin","password":"password"}'
fi

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
