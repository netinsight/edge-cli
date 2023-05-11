#!/bin/bash

set -euo pipefail

# input delete RTP_Input_01
input="${1?missing argument: input}"
shift # positional argument name

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

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
