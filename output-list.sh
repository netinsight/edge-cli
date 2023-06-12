#!/bin/bash

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

parse_redundancy() {
    if [ "$1" == 0 ]; then
        echo -n none
    elif [ "$1" == 1 ]; then
        echo -n failover
    elif [ "$1" == 2 ]; then
        echo -n active
    else
        echo -n unknown
    fi
}

parse_delay_mode() {
    if [ "$1" == 1 ]; then
        echo -n On Arrival
    elif [ "$1" == 2 ]; then
        echo -n On Origin
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

output_tsv() {
    if [ "$output_format" == "short" ]; then
        printf "%s\t%s\t%s\n" "ID" "Name" "Health"
    elif [ "$output_format" == "wide" ]; then
        printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n" \
            "ID" "Name" "Group" "Enabled" "Input" "Redudancy" "Delay" "Delay mode" "Appliance" "Health"
    fi
    while IFS=$'\t' read -r \
            id \
            name \
            admin_status \
            redundancy_mode \
            group \
            input \
            delay \
            delay_mode \
            health_state \
            health_title \
            appliance
        do
        if [ "$health_state" == "allOk" ]; then
            health="\e[32m✓\e[0m"
        else
            health="\e[31m✗\e[0m"
        fi
        if [ "$output_format" == "short" ]; then
            printf "%s\t%s\t$health %s\n" "$id" "$name" "$health_title"
        elif [ "$output_format" == "wide" ]; then
            # We don't resolve input for performance reasons
            printf "%s\t%s\t%s\t%s\t%s\t%s\t%sms\t%s\t%s\t$health %s\n" \
                "$id" \
                "$name" \
                "${groups[$group]}" \
                "$(parse_admin_status "$admin_status")" \
                "$input" \
                "$(parse_redundancy "$redundancy_mode")" \
                "$delay" \
                "$(parse_delay_mode "$delay_mode")" \
                "$appliance" \
                "$health_title"
        fi
    done < <(curl "$edge_url/api/output/" \
        --silent \
        --get \
        --cookie "$cookie_jar" \
        | jq --raw-output '.items | map([
                .id,
                .name,
                .adminStatus,
                .redundancyMode,
                .group,
                .input,
                .delay,
                .delayMode,
                .health.state,
                .health.title,
                .appliances[].name
            ])[] | @tsv')
}

output_tsv | column -t -s $'\t'
