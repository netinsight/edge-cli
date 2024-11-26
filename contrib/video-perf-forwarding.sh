#!/usr/bin/env bash

# This script sets up a performance test where one device generates streams and
# sends them via RTP to the device under test. The device under test then sends
# that stream via RTP to the output device which does TR101290 analysis.
#
# This test does not involve a video node in the traditional sense because the
# stream isn't routed via a core node.

bitrate="$((20 * 1000000))"

while [[ $# -gt 0 ]]; do
    case $1 in
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

set -euo pipefail

generator_appliance=$(edge appliance list | awk '$3 == "edgeConnect" && $1 ~ /input/ { print $1 }' | sort | head)
output_appliance=$(edge appliance list | awk '$3 == "edgeConnect" && $1 ~ /output/ { print $1 }' | sort | head)
test_appliance=edge-194

generator_interface=$(edge appliance show "$generator_appliance" | awk '/^Interface.*public/ { print $2 }')
output_interface=$(edge appliance show "$output_appliance" | awk '/^Interface.*public/ { print $2 }')
output_ip=$(edge appliance show "$output_appliance" | awk '/^Interface.*public/ { print $6 }') # the public ip
test_interface=$(edge appliance show "$test_appliance" | awk '/^Interface.*public/ { print $2 }')
test_ip=$(edge appliance show "$test_appliance" | awk '/^Interface.*public/ { print $6 }') # the public ip

cat >&2 <<EOF
Generator appliance:    ${generator_appliance}
Output appliance:       ${output_appliance}
Test appliance:         ${test_appliance}
EOF

inputs="$(edge input list | awk '/^ID/ { next } { print $2 }')"
outputs="$(edge output list | awk '/^ID/ { next } { print $2 }')"

echo >&2 "Setting up generators"

if ! grep -qw "Perftest-generator-$generator_appliance" <<<"$inputs"; then
    edge input create "Perftest-generator-$generator_appliance" \
        --appliance "$generator_appliance" \
        --interface lo \
        --mode generator \
        --bitrate "$bitrate" \
        --disable-thumbnails
fi
if ! grep -qw "Perftest-generator-$generator_appliance" <<<"$outputs"; then
    edge output create "Perftest-generator-$generator_appliance" \
        --input "Perftest-generator-$generator_appliance" \
        --appliance "$generator_appliance" \
        --interface lo \
        --mode rtp \
        --source 127.0.0.1 \
        --dest 224.0.0.44:4444 
fi

for i in $(seq "$num_outputs"); do
    gen_in="Perftest-$i-s1-RTP_gen_input"
    if ! grep -qw "$gen_in" <<<"$inputs"; then
        edge input create "$gen_in" --appliance "$generator_appliance" --interface lo --mode rtp --port 4444 --multicast 224.0.0.44 --disable-thumbnails
    fi

    gen_out="Perftest-$i-s2-RTP_gen_output"
    if ! grep -qw "$gen_out" <<<"$outputs"; then
        edge output create "$gen_out" --appliance "$generator_appliance" --interface "$generator_interface" --mode rtp --dest "$test_ip:$((4000 + i*6))" --input "$gen_in"
    fi

    input="Perftest-$i-s3-RTP_input"
    if ! grep -qw "$input" <<<"$inputs"; then
        edge input create "$input" --appliance "$test_appliance" --interface "$test_interface" --mode rtp --port "$((4000 + i*6))" --thumbnails edge
    fi

    output="Perftest-$i-s4-RTP_output"
    if ! grep -qw "$output" <<<"$outputs"; then
        edge output create "$output" --appliance "$test_appliance" --interface "$test_interface" --mode rtp --dest "$output_ip:$((4000 + i*6))" --input "$input"
    fi

    tr101290_input="Perftest-$i-s5-RTP_input_tr101290"
    if ! grep -qw "$tr101290_input" <<<"$inputs"; then
        edge input create "$tr101290_input" --appliance "$output_appliance" --interface "$output_interface" --mode rtp --port "$((4000 + i*6))" --disable-thumbnails
    fi
done
