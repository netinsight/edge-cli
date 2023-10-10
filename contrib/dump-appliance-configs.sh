#!/usr/bin/env bash

cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/../login.sh" "$EDGE_URL")"

out=${1?missing argument: output directory}
mkdir -p "$out"
for appliance in $(edge appliance list | awk 'NR == 1 { next } { print $2 }'); do
    echo "Fetching appliance config for appliance $appliance" >&2
    curl --silent "$EDGE_URL/api/appliance/$appliance/config" --get --cookie "$cookie_jar" > "$out/$appliance.json"
done
