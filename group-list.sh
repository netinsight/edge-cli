#!/bin/bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

(
    printf "%s\t%s\n" "ID" "Name"
    while read -r id name ; do
        printf "%s\t%s\n" "$id" "$name"
    done < <(curl "$edge_url/api/group/" \
        --silent \
        --get \
        --cookie "$cookie_jar" \
        | jq --raw-output '.items | map([.id, .name])[] | @tsv')
) | column -t
