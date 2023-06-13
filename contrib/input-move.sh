#!/bin/bash

src_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$src_url")"

dest_url="${1?missing argument: destination URL}"
input_id="${2?missing argument: input id}"

dest_cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$dest_url")"

echo >&2 "src: Fetching input"
input=$(curl "$src_url/api/input/$input_id" \
    --silent \
    --get \
    --cookie "$cookie_jar")

echo >&2 "src: Fetching group"
group=$(curl "$src_url/api/group/$(jq --raw-output .owner <<<"$input")" \
    --silent \
    --get \
    --cookie "$cookie_jar")

echo >&2 "dest: Find group"
dest_group=$(curl "$dest_url/api/group/" \
    --silent \
    --get \
    --data-urlencode 'q={"filter":{"searchName":"'"$(jq --raw-output .name <<<"$group")"'"}}' \
    --cookie "$dest_cookie_jar")

num_ports=$(jq --raw-output '.ports | length' <<<"$input")
if [ "$num_ports" != "1" ]; then
    echo >&2 "Currently only supports 1 port per input"
    exit 1
fi

echo >&2 "src: Finding a physicalPort"
src_port=$(curl "$src_url/api/port/$(jq --raw-output '.ports[0].physicalPort' <<<"$input")" \
    --silent \
    --get \
    --cookie "$cookie_jar")

port_appliance=$(jq --raw-output .appliance.name <<<"$src_port")
port_name=$(jq --raw-output .name <<<"$src_port")

echo >&2 "dest: Finding the destination appliance"
dest_appliance=$(curl "$dest_url/api/appliance/" \
    --silent \
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

new_input=$(jq \
    --arg dest_group "$(jq --raw-output '.items[0].id' <<<"$dest_group")" \
    --arg dest_port "$dest_port_id" \
    '.
    | del(.id)
    | del(.health)
    | del(.metrics)
    | del(.tsInfo)
    | del(.previewUrl)
    | del(.alarms)
    | del(.channelGroup)
    | del(.createdAt)
    | del(.updatedAt)
    | del(.misconfigured)
    | del(.downstreamAppliances)
    | del(.numOutputs)
    | del(.numSharedGruops)
    | del(.channelIds)
    | del(.channelId)
    | del(.appliances)
    | .ports = (.ports | map(.
        | del(.id)
    ))
    | .owner = $dest_group
    | .ports[0].physicalPort = $dest_port
    ' <<<"$input"
)

echo >&2 "dest: Creating the input"
curl "$dest_url/api/input/" \
    --silent \
    -X POST \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    --cookie "$dest_cookie_jar" \
    --data "$new_input" | jq .
