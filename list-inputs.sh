#!/bin/bash

set -euo pipefail

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
    --cookie "$cookie_jar")"

(
    printf "Name\tid\n" &&
    jq --raw-output '.items | map([.name, .id])[] | @tsv' <<<"$inputs"
) | column -t
