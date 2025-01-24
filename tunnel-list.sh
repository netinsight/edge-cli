#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

tunnels="$(curl "$edge_url/api/tunnel/" \
    --silent \
    --get \
    --cookie "$cookie_jar")"

(
    printf "%s\t%s\t%s\t%s\t%s\n" "ID" "Type" "Client" "Server" "Inputs" &&
        jq --raw-output '.items | map([
                .id,
                ( if .type == 1 then "external" elif .type == 2 then "internal" elif .type == 3 then "inter-region" else .type end),
                .clientName,
                .serverName,
                (.inputs | length)
            ])[] | @tsv' <<<"$tunnels"
) | column -t
