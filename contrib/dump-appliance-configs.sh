#!/usr/bin/env bash

# Usage:
# ./$0 appliance-config-1
# # Do some change
# ./$0 appliance-config-2
# diff -qr appliance-config-1 appliance-config-2

out=${1?missing argument: output directory}

cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/../login.sh" "$EDGE_URL")"

mkdir -p "$out"
for appliance in $(edgectl appliance list | awk 'NR == 1 { next } { print $2 }'); do
    echo "Fetching appliance config for appliance $appliance" >&2
    curl --silent "$EDGE_URL/api/appliance/$appliance/config" --get --cookie "$cookie_jar" | jq --sort-keys . > "$out/$appliance.json"
done
