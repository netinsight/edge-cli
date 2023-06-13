#!/bin/bash

set -euo pipefail

src_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$src_url")"

dest_url="${1?missing argument: destination URL}"
output_id="${2?missing argument: input id}"
curl_args=(--silent)

set -x

dest_cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$dest_url")"

output=$(curl "${curl_args[@]}" "$src_url/api/output/$output_id" \
    --get \
    --cookie "$cookie_jar")

group=$(curl "${curl_args[@]}" "$src_url/api/group/$(jq --raw-output .group <<<"$output")" \
    --get \
    --cookie "$cookie_jar")

dest_group=$(curl "${curl_args[@]}" "$dest_url/api/group/" \
    --get \
    --data-urlencode 'q={"filter":{"searchName":"'"$(jq --raw-output .name <<<"$group")"'"}}' \
    --cookie "$dest_cookie_jar")

num_ports=$(jq --raw-output '.ports | length' <<<"$output")
if [ "$num_ports" != "1" ]; then
    echo >&2 "Currently only supports 1 port per input"
    exit 1
fi
input=$(curl "${curl_args[@]}" "$src_url/api/input/$(jq --raw-output .input <<<"$output")" \
    --get \
    --cookie "$cookie_jar")

input_name=$(jq --raw-output .name <<<"$input")
dest_input=$(curl "${curl_args[@]}" "$dest_url/api/input/" \
    --get \
    --data-urlencode 'q={"filter":{"searchName":"'"$input_name"'"}}' \
    --cookie "$dest_cookie_jar")

dest_input_id=$(jq --raw-output \
    --arg name "$input_name" \
    '.items | map(select(.name == $name)) | .[0]
    | .id' <<<"$dest_input")

if [ "$(jq '.ports[0] | has("region")' <<<"$output")" == "true" ]; then
    echo >&2 "This is a regional output"
    dest_region=$(curl "${curl_args[@]}" "$dest_url/api/region/" \
        --get \
        --data-urlencode 'q={"filter":{"searchName":"'"$(jq --raw-output '.ports[0].region.name' <<<"$output")"'"}}' \
        --cookie "$dest_cookie_jar")

    new_port=$(jq \
        --arg input_id "$(jq --raw-output .id <<<"$output")" \
        --arg region_id "$(jq --raw-output '.items[0].id' <<<"$dest_region")" \
        '{
            purpose: "output",
            regionId: $region_id,
            inputId: $input_id,
        }' <<<"$output"
    )

    dest_port=$(curl "${curl_args[@]}" "$dest_url/api/allocatePort" \
        -X POST \
        -H 'Accept: application/json' \
        -H 'Content-Type: application/json' \
        --cookie "$dest_cookie_jar" \
        --data "$new_port")

    dest_port_id=$(jq --raw-output .physicalPort.id <<<"$dest_port")
    allocated_port_id=$(jq --raw-output .id <<<"$dest_port")
    port_number=$(jq --raw-output .portNumber <<<"$dest_port")
else
    src_port=$(curl "${curl_args[@]}" "$src_url/api/port/$(jq --raw-output '.ports[0].physicalPort' <<<"$output")" \
        --get \
        --cookie "$cookie_jar")

    port_appliance=$(jq --raw-output .appliance.name <<<"$src_port")
    port_name=$(jq --raw-output .name <<<"$src_port")

    dest_appliance=$(curl "${curl_args[@]}" "$dest_url/api/appliance/" \
        --get \
        --data-urlencode 'q={"filter":{"searchName":"'"${port_appliance}"'"}}' \
        --cookie "$dest_cookie_jar")

    dest_port_id=$(jq --raw-output \
        --arg name "$port_appliance" \
        --arg port_name "$port_name" \
        '.items | map(select(.name == $name)) | .[0]
            | .physicalPorts | map(select(.name == $port_name))
            | .[0].id' \
        <<<"$dest_appliance")
    echo >&2 "Normal outputs are no longer supported!"
    exit 1
fi

echo >&2 "Will create output $(jq .name <<<"$output")"

new_output=$(jq \
    --arg dest_group "$(jq --raw-output '.items[0].id' <<<"$dest_group")" \
    --arg dest_port "$dest_port_id" \
    --arg dest_input "$dest_input_id" \
    --arg allocated_port_id "${allocated_port_id--}" \
    --arg port_number "$port_number" \
    '.
    | del(.id)
    | del(.health)
    | del(.metrics)
    | del(.appliances)
    | del(.upstreamAppliances)
    | del(.channelIds)
    | del(.alarms)
    | del(.createdAt)
    | del(.updatedAt)
    | del(.misconfigured)
    | .ports = (.ports | map(.
        | del(.id)
        | del(.region)
        | .localPort = ($port_number | tonumber)
        | if $allocated_port_id == "" then . else .allocatedPortId = $allocated_port_id end
    ))
    | .group = $dest_group
    | .ports[0].physicalPort = $dest_port
    | .input = $dest_input
    ' <<<"$output"
)

output=$(curl "${curl_args[@]}" "$dest_url/api/output/" \
    -X POST \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    --cookie "$dest_cookie_jar" \
    --data "$new_output")

while [ "$(jq --raw-output .type <<<"$output")" == "internalServerError" ]; do
    echo >&2 "Please reroute input $input_name and press enter to continue"
    read -r
    output=$(curl "${curl_args[@]}" "$dest_url/api/output/" \
        -X POST \
        -H 'Accept: application/json' \
        -H 'Content-Type: application/json' \
        --cookie "$dest_cookie_jar" \
        --data "$new_output")
done

jq . <<<"$output"
