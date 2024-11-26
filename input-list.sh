#!/usr/bin/env bash

set -euo pipefail

edge_url="${EDGE_URL?:missing environment variable: EDGE_URL}"
cookie_jar="$("$(dirname -- "${BASH_SOURCE[0]}")/login.sh" "$edge_url")"

output_format=short

while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            case "$2" in
                short|wide)
                    output_format="$2"
                ;;
                *)
                    echo >&2 "$2 is not a valid output method"
                    exit 1
            esac
            shift 2
        ;;
    esac
done

parse_admin_status() {
    if [ "$1" == 1 ]; then
        echo -n on
    elif [ "$1" == 0 ]; then
        echo -n off
    else
        echo -n unknown
    fi
}

parse_thumbnail_mode() {
    if [ "$1" == 0 ]; then
        echo -n none
    elif [ "$1" == 2 ]; then
        echo -n core
    else
        echo -n unknown
    fi
}

declare -A groups
# Only the wide format needs the group name
if [ "$output_format" == "wide" ]; then
    while read -r group_id group_name ; do
        groups["$group_id"]="$group_name"
    done < <("$(dirname -- "${BASH_SOURCE[0]}")/group-list.sh")
fi

input_tsv() {
    if [ "$output_format" == "short" ]; then
        printf "%s\t%s\t%s\n" "ID" "Name" "Health"
    elif [ "$output_format" == "wide" ]; then
        printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n" \
            "ID" "Name" "Group" "Enabled" "Buffer" "Preview" "Thumbnails" "TR 101 290" "can subscribe" "Appliances" "Health"
    fi
    while IFS=$'\t' read -r \
        id \
        name \
        admin_status \
        owner \
        health_state \
        health_title \
        buffer_size \
        preview_mode \
        thumbnail_mode \
        tr101290 \
        can_subscribe \
        appliances
    do
        if [ "$health_state" == "allOk" ]; then
            health="\e[32m✓\e[0m"
        else
            health="\e[31m✗\e[0m $health_title"
        fi
        if [ "$output_format" == "short" ]; then
            printf "%s\t%s\t$health\n" "$id" "$name"
        elif [ "$output_format" == "wide" ]; then
            printf "%s\t%s\t%s\t%s\t%sms\t%s\t%s\t%s\t%s\t%s\t$health\n" \
                "$id" \
                "$name" \
                "${groups[$owner]}" \
                "$(parse_admin_status "$admin_status")" \
                "$buffer_size" \
                "$preview_mode" \
                "$(parse_thumbnail_mode "$thumbnail_mode")" \
                "$tr101290" \
                "$can_subscribe" \
                "$appliances"
        fi
    done < <(curl "$edge_url/api/input/" \
        --silent \
        --get \
        --cookie "$cookie_jar" \
        --data 'q={"limit":5000}' \
        | jq --raw-output '.items | map([
                .id,
                .name,
                .adminStatus,
                .owner,
                .health.state,
                ( if .health.title == "" then "-" else .health.title end), # read with empty fields and IFS=\t is not supported
                .bufferSize,
                .previewSettings.mode,
                .thumbnailMode,
                .tr101290Enabled,
                .canSubscribe,
                (.appliances | map(.name) | join(", "))
            ])[] | @tsv')
}

input_tsv | column -t -s $'\t'
