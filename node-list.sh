#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

node="$(curl "$edge_url/api/k8s/node/" \
    --silent \
    --get \
    --cookie "$cookie_jar")"

(
    printf "Name\tStatus\tInternalIP\tExternalIP\tHostname\tRoles\tKubeletVersion\tRegionName\tRegionType\n" &&
    jq --raw-output '.items[] | [.name, .status, .internalIP, (if .externalIP == "" or .externalIP == null then "none" else .externalIP end), .hostname, (if (.roles | length) == 0 then "none" else (.roles | join(",")) end), .kubeletVersion, .region.name, (if .region.external == 0 then "0-core" elif .region.external == 1 then "1-external" else .region.external end)] | @tsv' <<<"$node"
) | column -t
