#!/usr/bin/env bash

edge_url="${1?missing argument: edge URL}"
install_name="${edge_url}"
install_name="${install_name#https://}"
install_name="${install_name#http://}"
install_name="${install_name#/}"

cookie_jar="$HOME/.config/edge-cli/$install_name.cookie"

if [ -f "$cookie_jar" ]; then
    now="$(date +%s)"
    # parse the curl cookie file, reference: https://curl.se/docs/http-cookies.html
    expires="$(awk "/edgetoken/"' { print $5 }' "$cookie_jar")"
    if [ -z "$expires" ]; then # no cookies stored
        rm "$cookie_jar"
    elif (( "$expires" < "$now" )); then
        echo >&2 "Cookie has expired, refreshing"
        rm "$cookie_jar"
    fi
fi

if ! [ -f "$cookie_jar" ]; then
    mkdir -p "$(dirname "$cookie_jar")"
    tmpdir=$(mktemp -d edgecli.XXXXXXXX)
    trap 'rm -rf "$tmpdir"' EXIT 
    login_status=$(curl "$edge_url/api/login/" \
        --silent \
        -X POST \
        -H 'Accept: application/json' \
        -H 'Content-Type: application/json' \
        --cookie-jar "$cookie_jar" \
        -w "%{http_code}\n" -o "$tmpdir/login_resp" \
        --data-raw '{"username":"'"${EDGE_USER-admin}"'","password":"'"${EDGE_PASSWORD?The EDGE_PASSWORD environment variable is not set, cannot authenticate.}"'"}'
    )
    case "$login_status" in
        401) echo >&2 "Invalid username or password"; exit 1 ;;
        403) echo >&2 "Failed to log in:"; cat >&2 "$tmpdir/login_resp"; echo >&2 ; exit 1 ;;
        200) : ;;
        *) echo >&2 "Unsupported status code $login_status"; exit 1 ;;
    esac
fi

echo "$cookie_jar"
