#!/bin/bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

id="${1?missing argument: input ID}"

parse_thumbnail_mode() {
    if [ "$1" == 0 ]; then
        echo -n none
    elif [ "$1" == 2 ]; then
        echo -n core
    else
        echo -n unknown
    fi
}

parse_admin_status() {
    if [ "$1" == 1 ]; then
        echo -n on
    elif [ "$1" == 0 ]; then
        echo -n off
    else
        echo -n unknown
    fi
}

format_ports() {
    port="$1"
    while IFS=$'\t' read -r  \
        copies \
        physical_port \
        mode
    do
        port_info=$(curl "$edge_url/api/port/$physical_port" \
            --silent \
            --get \
            --cookie "$cookie_jar")
        cat <<EOF
  - Mode:                   $mode
    Source interface:       $(jq --raw-output .name <<<"$port_info")
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
    ])[] | @tsv' <<<"$port")
}

input=$(curl "$edge_url/api/input/$id" \
    --silent \
    --get \
    --cookie "$cookie_jar")

group=$(curl "$edge_url/api/group/$(jq --raw-output .owner <<<"$input")" \
    --silent \
    --get \
    --cookie "$cookie_jar")

if [ "$(jq --raw-output .health.state <<<"$input")" == "allOk" ]; then
    health="\e[32m✓\e[0m"
else
    health="\e[31m✗\e[0m $(jq --raw-output .health.title <<<"$input")"
fi

cat <<EOF
ID:             $(jq --raw-output .id <<<"$input")
Name:           $(jq --raw-output .name <<<"$input")
Admin status:   $(parse_admin_status "$(jq --raw-output .adminStatus <<<"$input")")
Owner:          $(jq --raw-output .name <<<"$group")
Buffer:         $(jq --raw-output .bufferSize <<<"$input")ms
Preview:        $(jq --raw-output .previewSettings.mode <<<"$input")
Thumbnail mode: $(parse_thumbnail_mode "$(jq --raw-output .thumbnailMode <<<"$input")")
TR 101 290:     $(jq --raw-output .tr101290Enabled <<<"$input")
Can subscribe:  $(jq --raw-output .canSubscribe <<<"$input")
Appliances:     $(jq --raw-output '(.appliances | map(.name) | join(", "))' <<<"$input")
Ports:
$(format_ports "$(jq --raw-output .ports <<<"$input")")
Created:        $(jq --raw-output .createdAt <<<"$input") 
Updated:        $(jq --raw-output .updatedAt <<<"$input") 
Health:         $(echo -e "$health")
EOF
