#!/usr/bin/env bash

# This script checks the status of all inputs and outputs and reports
# if any status is not "ok". Exits with non-zero code if issues found.

set -euo pipefail

exit_code=0
unhealthy_inputs=()
unhealthy_outputs=()

input_status=$(edge input list | awk 'NR>1 {$1=""; print $2, substr($0, index($0,$3))}')
if [[ -n "$input_status" ]]; then
    while IFS=' ' read -r name health_rest; do
        if [[ ! "$health_rest" =~ ✓ ]]; then
            # Extract just the status message part (after the indicator)
            clean_status=$(echo "$health_rest" | sed 's/\x1b\[[0-9;]*m//g' | sed 's/^[✗✓] *//')
            unhealthy_inputs+=("$name|$clean_status")
            exit_code=1
        fi
    done <<< "$input_status"
fi

output_status=$(edge output list | awk 'NR>1 {$1=""; print $2, substr($0, index($0,$3))}')
if [[ -n "$output_status" ]]; then
    while IFS=' ' read -r name health_rest; do
        if [[ ! "$health_rest" =~ ✓ ]]; then
            # Extract just the status message part (after the indicator)
            clean_status=$(echo "$health_rest" | sed 's/\x1b\[[0-9;]*m//g' | sed 's/^[✗✓] *//')
            unhealthy_outputs+=("$name|$clean_status")
            exit_code=1
        fi
    done <<< "$output_status"
fi

if [[ ${#unhealthy_inputs[@]} -gt 0 ]]; then
    echo "Found unhealthy inputs:"
    for item in "${unhealthy_inputs[@]}"; do
        name="${item%%|*}"
        status="${item#*|}"
        printf "  %-40s %s\n" "$name" "$status"
    done
fi

if [[ ${#unhealthy_outputs[@]} -gt 0 ]]; then
    echo "Found unhealthy outputs:"
    for item in "${unhealthy_outputs[@]}"; do
        name="${item%%|*}"
        status="${item#*|}"
        printf "  %-40s %s\n" "$name" "$status"
    done
fi

if [[ $exit_code -eq 0 ]]; then
    echo "✅ All OK"
else
    echo "❌ Found inputs/outputs with non-ok status"
fi

exit $exit_code