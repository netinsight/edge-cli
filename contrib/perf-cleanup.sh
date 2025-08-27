#!/usr/bin/env bash

# This script deletes ALL inputs and outputs whose names start with "Perftest"
# Use with caution - this action cannot be undone!

set -euo pipefail

# Get all inputs starting with Perftest
perftest_inputs=$(edgectl input list | awk 'NR>1 && $2 ~ /^Perftest/ {print $2}')
# Get all outputs starting with Perftest
perftest_outputs=$(edgectl output list | awk 'NR>1 && $2 ~ /^Perftest/ {print $2}')

input_count=$(wc -l <<<"$perftest_inputs")
output_count=$(wc -l <<<"$perftest_outputs")

# Handle empty strings properly
if [[ -z "$perftest_inputs" ]]; then
    input_count=0
fi
if [[ -z "$perftest_outputs" ]]; then
    output_count=0
fi

if [[ $input_count -eq 0 && $output_count -eq 0 ]]; then
    echo "No inputs or outputs found."
    exit 0
fi

echo "Found:"
echo "  - $input_count inputs starting with 'Perftest'"
echo "  - $output_count outputs starting with 'Perftest'"
echo
echo "⚠️  WARNING: This will delete $input_count inputs and $output_count outputs. This action cannot be undone!"
echo
echo "Are you sure you want to continue? (y/N): "
read -r response

if [[ ! "$response" =~ ^[Yy]$ ]]; then
    echo "Operation cancelled."
    exit 0
fi

echo

if [[ $input_count -eq 0 ]]; then
    echo "No inputs to delete."
else
    echo "Deleting inputs..."
    edgectl input delete Perftest
fi

if [[ $output_count -eq 0 ]]; then
    echo "No outputs to delete."
else
    echo "Deleting outputs..."
    edgectl output delete Perftest
fi

echo
echo "Cleanup completed!"
echo "   - Deleted $input_count inputs"
echo "   - Deleted $output_count outputs"
