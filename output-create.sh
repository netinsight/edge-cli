#!/usr/bin/env bash

set -euo pipefail

# output create UDP_Output_01 --appliance EC1 --interface eth0 --mode udp --dest 127.0.0.1:1234 --input RTP_Input_01

name="${1?missing argument: name}"
shift # positional argument name
fec=""
fec_rows=5
fec_cols=5

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
		--source)
            source_addr="$2"
			shift 2
			;;
		--dest)
            IFS=':' read -r dest_addr dest_port <<<"$2"
			shift 2
			;;
		--input)
			input="$2"
			shift 2
			;;
        --fec)
            case $2 in
                1d|1D)
                    fec='1D'
                    ;;
                2d|2D)
                    fec="2D"
                    ;;
                *)
                    echo >&2 "Invalid FEC mode $2, expected 1D or 2D"
                    exit 1
                    ;;
            esac
            shift 2
            ;;
        --fec-rows)
            fec_rows="$2"
            shift 2
            ;;
        --fec-cols)
            fec_cols="$2"
            shift 2
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
: "${dest_addr?missing or incomplete argument: destination address}"
: "${dest_port?missing or incomplete argument: destination port}"
: "${input?missing argument: input name}"

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

appliance="$(curl "$edge_url/api/appliance/" \
    --silent \
    --get \
    --data-urlencode 'q={"filter":{"searchName":"'"$appliance"'"}}' \
    --cookie "$cookie_jar" | jq .items[0] )"

appliance_name="$(jq --raw-output .name <<<"$appliance")"

physical_port=$(jq --arg name "$interface" '.physicalPorts | map(select(.name == $name))' <<<"$appliance")
physical_port_id=$(jq --raw-output .[0].id <<<"$physical_port")

inputs="$(curl "$edge_url/api/input/" \
    --silent \
    --get \
    --data-urlencode 'q={"filter":{"searchName":"'"$input"'"},"skip":0,"limit":150}' \
    --cookie "$cookie_jar")"

input=$(jq .items[0] <<<"$inputs")

input_id=$(jq --raw-output .id <<<"$input")


output_json=$(jq --null-input \
    --arg name "$name" \
    --arg port_mode "$mode" \
    --arg physical_port "$physical_port_id" \
    --arg dest_addr "$dest_addr" \
    --arg dest_port "$dest_port" \
    --arg input "$input_id" \
    --arg source_addr "${source_addr-}" \
    --arg fec "$fec" \
    --arg fec_rows "$fec_rows" \
    --arg fec_cols "$fec_cols" \
    '{
        name: $name,
        delay: 1000,
        delayMode: 2,
        adminStatus: 1,
        ports: [
        {
            mode: $port_mode,
            physicalPort: $physical_port,
            copies: 1,
            address: $dest_addr,
            port: ($dest_port | tonumber),
            ttl: 64,
        }
        | if $source_addr | length > 0 then . += {
          sourceAddress: $source_addr
        } else . end
        | if $fec | length > 0 then . += {
          fec: $fec,
          fecRows: ($fec_rows | tonumber),
          fecCols: ($fec_cols | tonumber),
        } else . end
        | if $port_mode == "rist" then . += {
          profile: "simple",
        } else . end
        ],
        redundancyMode: 0,
        input: $input,
}
')

jq . <<<"$output_json"

curl "$edge_url/api/output/" \
    --fail-with-body \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    --cookie "$cookie_jar" \
    --data "$output_json"

echo >&2 "created output $name on $appliance_name"
