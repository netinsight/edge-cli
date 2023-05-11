#!/bin/bash

edge_url="${1?missing argument: edge URL}"
install_name="${edge_url}"
install_name="${install_name#https://}"
install_name="${install_name#http://}"
install_name="${install_name#/}"

cookie_jar="$HOME/.config/edge-cli/$install_name.cookie"

if [ -f "$cookie_jar" ]; then
    now="$(date +%s)"
    # parse the curl cookie file, reference: https://curl.se/docs/http-cookies.html
    expires="$(awk "/$install_name/"' { print $5 }' "$cookie_jar")"
    if (( "$expires" < "$now" )); then
        echo >&2 "Cookie has expired, refreshing"
        rm "$cookie_jar"
    fi
fi

if ! [ -f "$cookie_jar" ]; then
    mkdir -p "$(dirname "$cookie_jar")"
    curl "$edge_url/api/login/" \
        --silent \
        -X POST \
        -H 'Accept: application/json' \
        -H 'Content-Type: application/json' \
        --cookie-jar "$cookie_jar" \
        --data-raw '{"username":"'"${EDGE_USER-admin}"'","password":"'"${EDGE_PASSWORD?The EDGE_PASSWORD environment variable is not set, cannot authenticate.}"'"}' \
    > /dev/null
fi

echo "$cookie_jar"
