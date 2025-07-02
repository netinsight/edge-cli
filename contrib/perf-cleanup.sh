#!/usr/bin/env bash

# This script deletes ALL inputs and outputs whose names start with "Perftest"
# Use with extreme caution - this action cannot be undone!

set -euo pipefail

# Get all inputs starting with Perftest
perftest_inputs=$(edge input list | awk 'NR>1 && $2 ~ /^Perftest/ {print $2}')
# Get all outputs starting with Perftest
perftest_outputs=$(edge output list | awk 'NR>1 && $2 ~ /^Perftest/ {print $2}')

input_count=$(echo "$perftest_inputs" | grep -c . 2>/dev/null || echo 0)
output_count=$(echo "$perftest_outputs" | grep -c . 2>/dev/null || echo 0)

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

# Function to delete an item with progress
delete_item() {
    local type=$1
    local name=$2
    edge "$type" delete "$name"
}

# Export function for parallel execution
export -f delete_item
export EDGE_URL EDGE_PASSWORD

echo "Deleting inputs..."
if [[ $input_count -gt 0 ]]; then
    echo "$perftest_inputs" | xargs -I {} -P 10 bash -c 'delete_item input "{}"'
fi

echo "Deleting outputs..."
if [[ $output_count -gt 0 ]]; then
    echo "$perftest_outputs" | xargs -I {} -P 10 bash -c 'delete_item output "{}"'
fi

echo
echo "Cleanup completed!"
echo "   - Deleted $input_count inputs"
echo "   - Deleted $output_count outputs"