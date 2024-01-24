#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

reset=0
if [ "$1" == "--clear" ] || [ "$1" == "--reset" ]; then
    reset=1
    shift
fi

appliances="$(curl "$edge_url/api/appliance/" \
    --data-urlencode 'q={"filter":{"searchName":"'"${1?missing argument: appliance name}"'"}}' \
    --silent \
    --get \
    --cookie "$cookie_jar")"


if [ "$(jq .total <<<"$appliances")" -ne 1 ]; then
    echo >&2 "Found $(jq .total <<<"$appliances") appliances"
    exit 1
fi

appliance_id=$(jq --raw-output .items[0].id <<<"$appliances")

if [ "$reset" -eq 1 ]; then
    curl "$edge_url/api/appliance/${appliance_id}/config/clear" \
        -X POST  \
        --silent \
        --cookie "$cookie_jar"
    echo >&2 "Cleared appliance config"
else
    config="$(curl "$edge_url/api/appliance/${appliance_id}/config" \
        --silent \
        --get \
        --cookie "$cookie_jar")"
    jq . <<<"$config"
fi
