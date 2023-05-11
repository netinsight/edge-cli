#!/bin/bash

set -euo pipefail

# output delete RTP_Output_01
output="${1?missing argument: output}"
shift # positional argument name

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

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
