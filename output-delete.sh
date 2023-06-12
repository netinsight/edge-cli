#!/bin/bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

# Delete by ID instead of name
if [ "$#" -gt 0 ] && [ "$1" == "--id" ]; then
    shift
    output_ids=( "$@" )

    ids=$(tr ' ' ',' <<<"${output_ids[*]}")
    deleted=$(curl "$edge_url/api/output" \
        -X DELETE \
        --silent \
        --get \
        --data-urlencode "ids=${ids}" \
        --cookie "$cookie_jar"
    )

    echo "Delete outputs:"
    jq --raw-output '.names | join("\n")' <<<"$deleted"

else
    # output delete RTP_Output_01
    output="${1?missing argument: output}"
    shift # positional argument name

    outputs="$(curl "$edge_url/api/output/" \
        --silent \
        --get \
        --data-urlencode 'q={"filter":{"searchName":"'"$output"'"}}' \
        --cookie "$cookie_jar")"

    jq --raw-output '.items | map(.id)[]' <<<"$outputs" \
        | xargs --max-args 50 "$0" --id
fi
