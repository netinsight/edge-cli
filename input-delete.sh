#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

# Delete by ID instead of name
if [ "$#" -gt 0 ] && [ "$1" == "--id" ]; then
    shift
    input_ids=( "$@" )

    ids=$(tr ' ' ',' <<<"${input_ids[*]}")
    deleted=$(curl "$edge_url/api/input" \
        -X DELETE \
        --silent \
        --get \
        --data-urlencode "ids=${ids}" \
        --cookie "$cookie_jar"
    )

    echo "Delete inputs:"
    jq --raw-output '.names | join("\n")' <<<"$deleted"

else
    # input delete RTP_Input_01
    input="${1?missing argument: input}"
    shift # positional argument name

    inputs="$(curl "$edge_url/api/input/" \
        --silent \
        --get \
        --data-urlencode 'q={"filter":{"searchName":"'"$input"'"}}' \
        --cookie "$cookie_jar")"

    jq --raw-output '.items | map(.id)[]' <<<"$inputs" \
        | xargs --max-args 50 "$0" --id
fi
