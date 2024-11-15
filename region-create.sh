#!/usr/bin/env bash

set -euo pipefail

name="${1?missing argument: name}"
shift

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

data_json=$( jq --null-input \
    --arg name "$name" \
    '{
        name: $name,
        external: 2,
    }')

curl "$edge_url/api/region/" \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    --cookie "$cookie_jar" \
    --data "$data_json"
