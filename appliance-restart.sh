#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

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
appliance_name=$(jq --raw-output .items[0].name <<<"$appliances")

echo >&2 "Restarting appliance ${appliance_name}"

curl "$edge_url/api/appliance/${appliance_id}/restart" \
    -X POST  \
    --silent \
    --cookie "$cookie_jar"

echo >&2 "Appliance ${appliance_name} restarted"
