#!/bin/bash

set -euo pipefail

# input create RTP_Input_01 --appliance vt-170 --interface eth0 --mode rtp --port 12345
name="${1?missing argument: name}"
shift # positional argument name

# default values
multicast=""
fec=false
thumbnail_mode=2

while [[ $# -gt 0 ]]; do
	case $1 in
		--appliance)
			appliance="$2"
			shift 2
			;;
		--interface)
			interface="$2"
			shift 2
			;;
		--mode)
			mode="$2"
			shift 2
			;;
		--port)
			port="$2"
			shift 2
			;;
		--multicast)
			multicast="$2"
			shift 2
			;;
		--fec)
			fec=1 # todo defautl fec to false
			shift
			;;
        --disable-thumbnails)
            thumbnail_mode=0
            shift
            ;;
		-*)
			echo "unknown option $1"
			exit 1
			;;
		*)
			echo "unknown argument $1"
			exit 1
			;;
	esac
done

: "${appliance?missing argument: appliance}"
: "${interface?missing argument: interface}"
: "${mode?missing argument: mode}"
: "${port?missing argument: port}"

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

appliance="$(curl "$edge_url/api/appliance/" \
    --silent \
    --get \
    --data-urlencode 'q={"filter":{"searchName":"'"$appliance"'"}}' \
    --cookie "$cookie_jar" | jq .items[0] )"

appliance_name="$(jq --raw-output .name <<<"$appliance")"
appliance_id="$(jq --raw-output .id <<<"$appliance")"
appliance_type="$(jq --raw-output .type <<<"$appliance")"

physical_port=$(jq --arg name "$interface" '.physicalPorts | map(select(.name == $name))' <<<"$appliance")
physical_port_id=$(jq --raw-output .[0].id <<<"$physical_port")

port_address=$(jq --raw-output .[0].addresses[0].address <<<"$physical_port")

logical_ports="$(curl "$edge_url/api/port/" \
    --silent \
    --get \
    --data-urlencode 'q={"filter":{"appliance":"'"$appliance_id"'"},"skip":0,"limit":150}' \
    --cookie "$cookie_jar" | jq .items )"

# TODO Do I really need to include the logical port id?
logical_port=$(jq --arg name "$interface" '. | map(select(.name == $name))[0]' <<<"$logical_ports")
logical_port_id=$(jq --raw-output .id <<<"$logical_port")

input_json=$(jq --null-input \
    --arg name "$name" \
    --arg port_address "$port_address" \
    --arg port "$port" \
    --arg port_mode "$mode" \
    --arg physical_port "$physical_port_id" \
	--arg port_id "$logical_port_id" \
    --arg multicast "$multicast" \
	--arg fec "${fec-}" \
    --arg appliance_type "$appliance_type" \
    --arg thumbnail_mode "$thumbnail_mode" \
    '{
        name: $name,
        tr101290Enabled: true,
        broadcastStandard: "dvb",
        thumbnailMode: ($thumbnail_mode | tonumber),
        videoPreviewMode: (if $thumbnail_mode == 2 then "on demand" else "off" end),
        adminStatus: 1,
        ports: [{
			id: $port_id,
            mode: $port_mode,
            physicalPort: $physical_port,
            address: $port_address,
            port: ($port | tonumber),
            copies: 1,
        }
		| if $multicast | length > 0 then . += {
            multicastAddress: $multicast,
        } else . end
		| if $fec | length > 0 then . += {
			fec: true,
		} else . end
        | if $appliance_type == "core" then . += {
            whitelistCidrBlock: "0.0.0.0/0",
        } else . end
        ],
        handoverMethod: "udp",
        bufferSize: 6000,
        maxBitrate: null,
    }')

jq . <<<"$input_json"

curl "$edge_url/api/input/" \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    --cookie "$cookie_jar" \
    --data "$input_json"

echo >&2 "created input $name on $appliance_name"
