#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

name="${1?missing argument: name}"

search=$(curl "$edge_url/api/appliance/" \
    --data-urlencode "q={\"filter\":{\"searchName\":\"$name\"}}" \
    --silent \
    --get \
    --cookie "$cookie_jar")

if [ "$(jq .total <<<"$search")" == 0 ]; then
    echo >&2 "No such appliance: $name"
    exit 1
fi
appliance=$(jq '.items[0]' <<<"$search")

id=$(jq --raw-output .id <<<"$appliance")

appliance=$(curl "$edge_url/api/appliance/$id" \
    --silent \
    --get \
    --cookie "$cookie_jar")

cat <<EOF
ID:                 $(jq --raw-output .id <<<"$appliance")
Name:               $(jq --raw-output .name <<<"$appliance")
Hostname:           $(jq --raw-output .hostname <<<"$appliance")
Serial:             $(jq --raw-output .serial <<<"$appliance")
Type:               $(jq --raw-output .type <<<"$appliance")
Contact:            $(jq --raw-output .contact <<<"$appliance")
Versions:
    Control:
        Software:   $(jq --raw-output .version.controlSoftwareVersion <<<"$appliance")
        Image:      $(jq --raw-output .version.controlImageVersion <<<"$appliance")
    Data:
        Software:   $(jq --raw-output .version.dataSoftwareVersion <<<"$appliance")
        Image:      $(jq --raw-output .version.dataImageVersion <<<"$appliance")
Region:             $(jq --raw-output .region.name <<<"$appliance")
Health:             $(jq --raw-output .health.state <<<"$appliance") - $(jq --raw-output .health.title <<<"$appliance")
EOF

for port_id in $(seq 0 "$(jq --raw-output '.physicalPorts | length - 1' <<<"$appliance")"); do
    addresses=$(jq --raw-output ".physicalPorts[$port_id].addresses | map(([.address, .publicAddress]) | map(select(.)) | join(\" public: \")) | join(\", \")" <<<"$appliance")
    name=$(jq --raw-output ".physicalPorts[$port_id].name" <<<"$appliance")
    port_type=$(jq --raw-output ".physicalPorts[$port_id].portType" <<<"$appliance")
    printf "Interface:          %-12s %-6s %s\n" "$name" "$port_type" "$addresses"
done
