#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

name="${1?missing argument: name}"

group_search=$(curl "$edge_url/api/group/" \
    --data-urlencode "q={\"filter\":{\"searchName\":\"$name\"}}" \
    --silent \
    --get \
    --cookie "$cookie_jar")

if [ "$(jq .total <<<"$group_search")" == 0 ]; then
    echo >&2 "No such group: $name"
    exit 1
fi
group=$(jq '.items[0]' <<<"$group_search")

id=$(jq --raw-output .id <<<"$group")

core_secret=$(curl "$edge_url/api/group/$id/core-secret" \
    --silent \
    --get \
    --cookie "$cookie_jar")

cat <<EOF
ID:                 $(jq --raw-output .id <<<"$group")
Name:               $(jq --raw-output .name <<<"$group")
Appliance secret:   $(jq --raw-output .applianceSecret <<<"$group")
Core secret:        $(jq --raw-output .secret <<<"$core_secret")
Appliances:         $(jq --raw-output .applianceCount <<<"$group")
Users:              $(jq --raw-output .userCount <<<"$group")
EOF
