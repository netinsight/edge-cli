#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

appliances="$(curl "$edge_url/api/region/" \
    --silent \
    --get \
    --cookie "$cookie_jar")"

 (
     printf "Name\tid\tis default\ttype\n" &&
     jq --raw-output '.items | map([.name, .id, .default_region, .external])[] | @tsv' <<<"$appliances"
 ) | column -t -s $'\t'

# {
#     "id": "69920e49-31c4-476a-b6bb-023305262007",
#     "name": "default",
#     "default_region": true,
#     "external": 0
# }
