#!/bin/bash

set -euo pipefail

# output delete RTP_Output_01
output="${1?missing argument: output}"
shift # positional argument name

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

outputs="$(curl "$edge_url/api/output/" \
    --silent \
    --get \
    --data-urlencode 'q={"filter":{"searchName":"'"$output"'"}}' \
    --cookie "$cookie_jar")"

output_ids=$(jq --raw-output '.items | map(.id) | join(",")' <<<"$outputs")

deleted=$(curl "$edge_url/api/output" \
    -X DELETE \
    --silent \
    --get \
    --data-urlencode "ids=$output_ids" \
    --cookie "$cookie_jar")

echo "Delete outputs:"
jq --raw-output '.names | join("\n")' <<<"$deleted"
