#!/usr/bin/env bash

# the video node will have one incoming tunnel and two outgoing tunnels
# the video-node under test should be alone in a region.
#
# This script assumes appliances are called `edge-connect-$host-$id` (for
# example `edge-connect-gen1-0`) where the `$host` denotes the physical host
# for it to run on and`$id` is a serial number. 
#
# This allows one or more edge-connects to run on the same physical machine
# The first edge-connect on each physical machine will be used when configuring
# the generator input. The generator input will send a CBR stream to a
# multicast stream that all other inputs on that physical machine will listen
# for.

bitrate="$((30 * 1000000))"

input_extra_args=()
fanout=1

while [[ $# -gt 0 ]]; do
    case $1 in
        --thumbnail)
            if [[ "$2" == "none" || "$2" == "core" || "$2" == "edge" ]]; then
                input_extra_args+=("--thumbnail=$2")
                shift 2
            else
                echo "Error: --thumbnail must be one of: none, core, edge"
                exit 1
            fi
            ;;
        --fanout)
            fanout="$2"
            shift 2
            ;;
        --bitrate)
            bitrate=$(numfmt --from=auto "${2}")
            shift 2
            ;;
        -v | --verbose)
            shift
            set -x
            ;;
        -*)
            echo "Unknown option $1"
            exit 1
            ;;
        *)
            break
            ;;
    esac
done

num_outputs="${1?missing argument: Number of outputs}"

mapfile -t input_appliances < <(edgectl appliance list | awk '$3 == "edgeConnect" && $1 ~ /input/ { print $1 }' | sort)
mapfile -t output_appliances < <(edgectl appliance list | awk '$3 == "edgeConnect" && $1 ~ /output/ { print $1 }' | sort)

cat >&2 <<EOF
Input appliances:
${input_appliances[@]}
Output appliances:
${output_appliances[@]}
EOF

# Figure out which physical machines our edge-connect runs on
# This is done by selecting one appliance per unique base machine name
# The base name is everything before the last dash (e.g., edge-connect-input-edge-192 from edge-connect-input-edge-192-000)
# We keep the full appliance name including the suffix
#
# It might be possible to make this better by using the hostname field
declare -A seen_machines
generator_appliances=()
for appliance in "${input_appliances[@]}"; do
    # Extract the base machine name (everything before the last dash)
    base_machine="${appliance%-*}"
    # If we haven't seen this base machine yet, add this appliance as the generator
    if [[ -z "${seen_machines[$base_machine]}" ]]; then
        seen_machines[$base_machine]=1
        generator_appliances+=("$appliance")
    fi
done

inputs="$(edgectl input list | awk '/^ID/ { next } { print $2 }')"
outputs="$(edgectl output list | awk '/^ID/ { next } { print $2 }')"

echo >&2 "Setting up generators"

for appliance in "${generator_appliances[@]}"; do
    if ! grep -qw "Perftest-generator-$appliance" <<<"$inputs"; then
        edgectl input create "Perftest-generator-$appliance" \
            --appliance "$appliance" \
            --mode generator \
            --bitrate "$bitrate" \
            --thumbnail none
    fi
    if ! grep -qw "Perftest-generator-$appliance" <<<"$outputs"; then
        edgectl output create "Perftest-generator-$appliance" \
            --input generator \
            --appliance "$appliance" \
            --interface lo \
            --mode rtp \
            --source 127.0.0.1 \
            --dest 224.0.0.44:4444 
    fi
done

for i in $(seq "$num_outputs"); do
    input="Perftest-$i-UDP_input"
    input_appliance="${input_appliances[$((i % ${#input_appliances[@]}))]}"
    if ! grep -qw "$input" <<<"$inputs"; then
        echo >&2 "Creating input $input on $input_appliance"
        edgectl input create "$input"  --appliance "$input_appliance" --interface lo --mode rtp --port 4444 --multicast 224.0.0.44 "${input_extra_args[@]}"
    fi
    for n in $(seq "$fanout"); do
        output_appliance="${output_appliances[$(( ((i-1)*fanout+n-1) % ${#output_appliances[@]}))]}"
        output="Perftest-$i-$n-UDP_output"
        if ! grep -qw "$output" <<<"$outputs"; then
            echo >&2 "Creating output $output on $output_appliance"
            edgectl output create "$output" --appliance "$output_appliance" --interface lo --mode udp --dest "127.0.0.1:1234" --input "$input"
        fi
    done
done
