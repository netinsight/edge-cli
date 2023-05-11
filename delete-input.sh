#!/bin/bash

set -euo pipefail

# input delete RTP_Input_01
input="${1?missing argument: input}"
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

inputs="$(curl "$edge_url/api/input/" \
    --silent \
    --get \
    --data-urlencode 'q={"filter":{"searchName":"'"$input"'"}}' \
    --cookie "$cookie_jar")"

input_ids=$(jq --raw-output '.items | map(.id) | join(",")' <<<"$inputs")

deleted=$(curl "$edge_url/api/input" \
    -X DELETE \
    --silent \
    --get \
    --data-urlencode "ids=$input_ids" \
    --cookie "$cookie_jar")

echo "Delete inputs:"
jq --raw-output '.names | join("\n")' <<<"$deleted"
