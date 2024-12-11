#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"

info=$(curl "$edge_url/api/build-info" \
    --silent \
    --get)

product_id=$(jq --raw-output .product <<<"$info")
case "$product_id" in
    "d0b70d6d-8db6-4524-8d4f-7ee716af241a")
        product=Connect iT
    ;;
    "705be4c2-0e82-4356-9ce2-6484c116796f")
        product=Edge
    ;;
    *)
        product=$product_id
    ;;
esac

cat <<EOF
Release:            $(jq --raw-output .release <<<"$info")
Build time:         $(jq --raw-output .buildTime <<<"$info")
Pipeline:           $(jq --raw-output .pipeline <<<"$info")
Commit:             $(jq --raw-output .commit <<<"$info")
Product:            $product
EOF
