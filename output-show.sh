#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

id="${1?missing argument: output ID}"

parse_admin_status() {
    if [ "$1" == 1 ]; then
        echo -n on
    elif [ "$1" == 0 ]; then
        echo -n off
    else
        echo -n unknown
    fi
}

parse_redundancy() {
    if [ "$1" == 0 ]; then
        echo -n none
    elif [ "$1" == 1 ]; then
        echo -n failover
    elif [ "$1" == 2 ]; then
        echo -n active
    else
        echo -n unknown
    fi
}

parse_delay_mode() {
    if [ "$1" == 1 ]; then
        echo -n On Arrival
    elif [ "$1" == 2 ]; then
        echo -n On Origin
    else
        echo -n unknown
    fi
}

format_ports() {
    output="$1"
    while IFS=$'\t' read -r  \
        copies \
        physical_port \
        mode \
        port \
        address \
        ttl \
        source_address
    do
        port_info=$(curl "$edge_url/api/port/$physical_port" \
            --silent \
            --get \
            --cookie "$cookie_jar")
        cat <<EOF
  - Mode:                   $mode
    Source interface:       $(jq --raw-output .name <<<"$port_info")
    Source address:         $source_address
    Destination Address:    $address:$port
    TTL:                    $ttl
    Copies:                 $copies
EOF
    done < <(jq --raw-output '. | map([
        .copies,
        .physicalPort,
        .mode,
        .port,
        .address,
        .ttl,
        .sourceAddress
    ])[] | @tsv' <<<"$output")
}

output=$(curl "$edge_url/api/output/$id" \
    --silent \
    --get \
    --cookie "$cookie_jar")

group=$(curl "$edge_url/api/group/$(jq --raw-output .group <<<"$output")" \
    --silent \
    --get \
    --cookie "$cookie_jar")

input=$(curl "$edge_url/api/input/$(jq --raw-output .input <<<"$output")" \
    --silent \
    --get \
    --cookie "$cookie_jar")

if [ "$(jq --raw-output .health.state <<<"$output")" == "allOk" ]; then
    health="\e[32m✓\e[0m"
else
    health="\e[31m✗\e[0m $(jq --raw-output .health.title <<<"$output")"
fi

cat <<EOF
ID:             $(jq --raw-output .id <<<"$output")
Name:           $(jq --raw-output .name <<<"$output")
Input:          $(jq --raw-output .name <<<"$input")
Admin status:   $(parse_admin_status "$(jq --raw-output .adminStatus <<<"$output")")
Redudancy:      $(parse_redundancy "$(jq --raw-output .redundancyMode <<<"$output")")
Group:          $(jq --raw-output .name <<<"$group")
Delay:          $(jq --raw-output .delay <<<"$output")ms
Delay mode:     $(parse_delay_mode "$(jq --raw-output .delayMode <<<"$output")")
Ports:
$(format_ports "$(jq --raw-output .ports <<<"$output")")
Alarms:         $(jq --raw-output .alarms <<<"$output")
Appliances:     $(jq --raw-output '.appliances | map(.name) | join(", ")' <<<"$output")
Misconfigured:  $(jq --raw-output .misconfigured <<<"$output")
Created:        $(jq --raw-output .createdAt <<<"$output")
Updated:        $(jq --raw-output .updatedAt <<<"$output")
Health:         $(echo -e "$health")
EOF
