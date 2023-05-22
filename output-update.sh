#!/bin/bash

set -euo pipefail

# output update RTP_Output_01 --delay 200ms
output="${1?missing argument: output}"
shift

while [[ $# -gt 0 ]]; do
    case $1 in
        --delay)
            if ! [[ "$2" =~ ^([0-9]+)ms$ ]]; then
                echo >&2 "Invalid delay format '$2', expected: 200ms"
                exit 1
            fi
            delay_ms="${BASH_REMATCH[1]}"
            shift 2
            ;;
        *)
            echo >&2 "Unknown argument: $1"
            exit 1
            ;;
    esac
done

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

outputs="$(curl "$edge_url/api/output/" \
    --silent \
    --get \
    --data-urlencode 'q={"filter":{"searchName":"'"$output"'"}}' \
    --cookie "$cookie_jar")"

if [[ $(jq --raw-output .total <<< "$outputs") -gt 1 ]]; then
    echo >&2 "Found more than one output while searching for '$output', found:"
    jq --raw-output '.items | map("- " + .name)[]' <<<"$outputs" >&2
    exit 1
fi

if [[ $(jq --raw-output .total <<< "$outputs") -eq 0 ]]; then
    echo >&2 "Could not find any outputs called '$output'"
    exit 1
fi

output_data=$(jq --raw-output '.items[0]' <<<"$outputs")
output_id=$(jq --raw-output .id <<<"$output_data")
output_update_data=$(jq '{
    name: .name,
    input: .input,
    ports: .ports,
    delay: .delay,
    delayMode: .delayMode,
    adminStatus: .adminStatus,
    redundancyMode: .redundancyMode,
}' <<<"$output_data")

if [ -n "$delay_ms" ]; then
    output_update_data=$(jq --arg delay "$delay_ms" \
        '. += { delay: ($delay | tonumber) }' <<<"$output_update_data")
fi

tmpfile=$(mktemp edge_output.XXXXXX)
trap 'rm -f "$tmpfile"' EXIT

http_status=$(curl "$edge_url/api/output/$output_id" \
    --silent \
    --write "%{http_code}" \
    --output "$tmpfile" \
    -X POST \
    --data "$output_update_data" \
    --header "content-type: application/json" \
    --cookie "$cookie_jar")

if (( "$http_status" >= 400 )); then
    echo >&2 "Update request failed with status $http_status:"
    jq --raw-output '.title, .detail' < "$tmpfile" >&2
    exit 1
fi
