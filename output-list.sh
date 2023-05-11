#!/bin/bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"


while read -r id name health_state health_title ; do
    if [ "$health_state" == "allOk" ]; then
        health="\e[32m✓\e[0m"
    else
        health="\e[31m✗\e[0m"
    fi
    printf "%s %s $health: %s\n" "$id" "$name" "$health_title"
done < <(curl "$edge_url/api/output/" \
    --silent \
    --get \
    --cookie "$cookie_jar" \
    | jq --raw-output '.items | map([.id, .name, .health.state, .health.title])[] | @tsv')
